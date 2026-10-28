//! Coffers and their loot: opening one, Need / Greed / Pass on each item, and who wins it, as retail does it (Bob's
//! Chronofoil capture of five Sastasha runs with Duty Finder parties, 2026-10-22: 15 coffers opened, 35 lots cast by
//! the other players, 18 items decided.) Party bots roll like those players did.
//!
//! Opening a coffer sends the party its loot (LootList, one LootItem per item) and OpenedTreasure. Each member then
//! chooses for each item (LootRoll from a player; the server answers with LootRollResult and tells everyone that they
//! cast their lot, LootMessage 7.) An item is decided once everyone has chosen, or when its 300 seconds are up: the
//! Need rolls if there are any, otherwise the Greed rolls (LootMessage 0 for each, 1 for the winner, LootResult.) With
//! no rolls, someone who didn't pass gets it with a roll of 0 (LootMessage 3, as Bob got two items on retail), and if
//! everyone passed it's lost (LootMessage 2.) The coffer fades out once all its items are decided.

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use parking_lot::Mutex;

use kawari::{
    common::ObjectId,
    ipc::zone::{ServerZoneIpcData, ServerZoneIpcSegment},
};

use crate::{
    FromServer, GameData, ToServer,
    server::{
        WorldServer,
        actor::NetworkedActor,
        instance::Instance,
        network::{DestinationNetwork, NetworkState},
    },
};

/// A player opened a coffer or rolled for an item.
pub fn handle_loot_messages(
    data: Arc<Mutex<WorldServer>>,
    network: Arc<Mutex<NetworkState>>,
    game_data: Arc<Mutex<GameData>>,
    msg: &ToServer,
) -> bool {
    match msg {
        ToServer::OpenTreasure(from_actor_id, treasure_id) => {
            let mut data = data.lock();
            if let Some(instance) = data.find_actor_instance_mut(*from_actor_id) {
                let mut network = network.lock();
                let mut game_data = game_data.lock();
                open(
                    &mut network,
                    &mut game_data,
                    instance,
                    *from_actor_id,
                    *treasure_id,
                );
            }
            true
        }
        ToServer::LootRoll(from_actor_id, treasure_id, slot, choice) => {
            let mut data = data.lock();
            if let Some(instance) = data.find_actor_instance_mut(*from_actor_id) {
                let mut network = network.lock();
                player_roll(
                    &mut network,
                    instance,
                    *from_actor_id,
                    *treasure_id,
                    *slot,
                    *choice,
                );
            }
            true
        }
        _ => false,
    }
}

/// How long a coffer's items can be rolled for (retail: 300 s, in every LootList.)
pub const CLAIM_TIME: Duration = Duration::from_secs(300);

/// A player's choice in `LootRoll` (retail: 2 greed and 5 pass seen; 1 need is the LootMessages' value for it.)
pub const CHOICE_NEED: u32 = 1;
pub const CHOICE_GREED: u32 = 2;
pub const CHOICE_PASS: u32 = 5;

/// LootRollResult's answers.
const RESULT_ROLLED: u8 = 1;
const RESULT_PASSED: u8 = 4;

/// LootMessage kinds (see `ServerZoneIpcData::LootMessage`.)
const LOOT_ROLL: u32 = 0;
const LOOT_WON: u32 = 1;
const LOOT_LOST: u32 = 2;
const LOOT_OBTAINED: u32 = 3;
const LOOT_CAST_LOT: u32 = 7;

/// A roll's kind in LootMessage 0 and 1.
const ROLL_NEED: u32 = 1;
const ROLL_GREED: u32 = 2;

/// LootResult's winner when everyone passed.
const NOBODY: ObjectId = ObjectId(0xE000_0000);

/// How long after a coffer opened the other players cast their lot for an item, in seconds (the 35 lots of the
/// capture.) A bot picks one of these.
const LOT_DELAYS: [f32; 35] = [
    2.0, 2.1, 2.3, 2.3, 2.6, 2.9, 3.1, 3.4, 3.4, 4.1, 4.4, 4.5, 4.6, 5.2, 5.4, 5.6, 5.9, 6.0, 6.2,
    6.5, 6.9, 7.5, 7.9, 9.1, 10.3, 12.6, 14.7, 63.3, 65.0, 73.0, 74.4, 80.5, 81.1, 212.1, 248.6,
];

/// One member's choice for an item.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Choice {
    Need(u8),
    Greed(u8),
    Pass,
}

/// One item of an opened coffer.
#[derive(Debug, Clone)]
struct LootSlot {
    item_id: u32,
    choices: HashMap<ObjectId, Choice>,
    decided: bool,
}

/// An opened coffer.
#[derive(Debug, Clone)]
pub struct Coffer {
    opened_at: Instant,
    slots: Vec<LootSlot>,
    /// Bots still to choose: (bot, slot, when.)
    bots_due: Vec<(ObjectId, usize, Instant)>,
    /// The first slot's LootItem serial.
    first_serial: u32,
}

/// Who can roll: the players here and their bots.
fn members(instance: &Instance) -> Vec<ObjectId> {
    let mut members: Vec<ObjectId> = instance
        .actors
        .iter()
        .filter(|(_, actor)| matches!(actor, NetworkedActor::Player { .. }))
        .map(|(id, _)| *id)
        .collect();
    members.extend(
        instance
            .party_bots
            .keys()
            .filter(|id| instance.find_actor(**id).is_some())
            .copied(),
    );
    members
}

fn players(instance: &Instance) -> Vec<ObjectId> {
    instance
        .actors
        .iter()
        .filter(|(_, actor)| matches!(actor, NetworkedActor::Player { .. }))
        .map(|(id, _)| *id)
        .collect()
}

/// Sends `data` to every player here, each as coming from themselves (retail's loot packets do.)
fn to_players(network: &mut NetworkState, instance: &Instance, data: ServerZoneIpcData) {
    for player in players(instance) {
        let ipc = ServerZoneIpcSegment::new(data.clone());
        network.send_to_by_actor_id(
            player,
            FromServer::PacketSegment(ipc, player),
            DestinationNetwork::ZoneClients,
        );
    }
}

fn loot_message(network: &mut NetworkState, instance: &Instance, kind: u32, params: [u32; 7]) {
    to_players(
        network,
        instance,
        ServerZoneIpcData::LootMessage { kind, params },
    );
}

fn roll_contents(instance: &Instance, base_id: u32) -> Vec<u32> {
    contents(&instance.loot_pools, base_id, |n| fastrand::usize(..n))
}

/// What a coffer holds: for its base ID, one of the duty's recorded openings picks how many items, and each slot one
/// of the items found in that slot across them (`director:treasure_loot`.) A coffer without a recording gets one item
/// from all of the duty's. `pick(n)` chooses one of n.
fn contents(
    pools: &HashMap<u32, Vec<Vec<u32>>>,
    base_id: u32,
    mut pick: impl FnMut(usize) -> usize,
) -> Vec<u32> {
    if let Some(openings) = pools.get(&base_id)
        && !openings.is_empty()
    {
        let count = openings[pick(openings.len())].len();
        let mut items = Vec::new();
        for slot in 0..count {
            let candidates: Vec<u32> = openings
                .iter()
                .filter_map(|opening| opening.get(slot).copied())
                .filter(|item| !items.contains(item))
                .collect();
            if !candidates.is_empty() {
                items.push(candidates[pick(candidates.len())]);
            }
        }
        return items;
    }
    let mut keys: Vec<&u32> = pools.keys().collect();
    keys.sort();
    let all: Vec<u32> = keys
        .into_iter()
        .flat_map(|key| pools[key].iter().flatten().copied())
        .collect();
    if all.is_empty() {
        Vec::new()
    } else {
        vec![all[pick(all.len())]]
    }
}

/// How an item was decided.
#[derive(Debug, PartialEq)]
enum Outcome {
    /// The rolls (by member), their kind (need or greed), and the winner with their roll.
    Rolled {
        rolls: Vec<(ObjectId, u8)>,
        kind: u32,
        winner: ObjectId,
        roll: u8,
    },
    /// No one rolled: this member, who didn't pass, gets it.
    Unrolled(ObjectId),
    /// Everyone passed.
    Lost,
}

/// The highest Need roll wins, or failing any the highest Greed roll (the first of a tie, by member); with neither,
/// one of the members who didn't pass gets it (`pick(n)` chooses), and if everyone passed it's lost.
fn outcome(
    choices: &HashMap<ObjectId, Choice>,
    everyone: &[ObjectId],
    pick: impl FnOnce(usize) -> usize,
) -> Outcome {
    let rolls_of = |need: bool| {
        let mut rolls: Vec<(ObjectId, u8)> = choices
            .iter()
            .filter_map(|(member, choice)| match (choice, need) {
                (Choice::Need(roll), true) | (Choice::Greed(roll), false) => Some((*member, *roll)),
                _ => None,
            })
            .collect();
        rolls.sort_by_key(|(member, _)| member.0);
        rolls
    };
    let needs = rolls_of(true);
    let (rolls, kind) = if needs.is_empty() {
        (rolls_of(false), ROLL_GREED)
    } else {
        (needs, ROLL_NEED)
    };
    if let Some(&(winner, roll)) = rolls.iter().rev().max_by_key(|(_, roll)| *roll) {
        return Outcome::Rolled {
            rolls,
            kind,
            winner,
            roll,
        };
    }
    let undecided: Vec<ObjectId> = everyone
        .iter()
        .filter(|member| !matches!(choices.get(member), Some(Choice::Pass)))
        .copied()
        .collect();
    if undecided.is_empty() {
        Outcome::Lost
    } else {
        Outcome::Unrolled(undecided[pick(undecided.len())])
    }
}

fn item_slot(
    coffer_id: ObjectId,
    coffer: &Coffer,
    slot: usize,
    unique: bool,
    opened: bool,
) -> ServerZoneIpcData {
    let left = CLAIM_TIME
        .saturating_sub(coffer.opened_at.elapsed())
        .as_secs_f32();
    ServerZoneIpcData::LootItem {
        treasure_id: coffer_id,
        item_id: coffer.slots[slot].item_id,
        unk1: 0,
        time_left: left,
        time_total: [CLAIM_TIME.as_secs_f32(); 2],
        unk2: 1,
        unk3: 0,
        slot: slot as u8,
        unique: unique as u8,
        unk4: 0,
        unk5: [1, 1],
        opened: opened as u8,
        unk6: 0,
        serial: coffer.first_serial + slot as u32,
        unk7: 0,
        bonus: [0; 4],
        unk8: 0,
    }
}

/// `opener` opens the coffer `treasure_id`: the first time, its loot goes to the party and everyone may roll; after
/// that, a player opening it again sees what's left (retail sent the items again, marked opened, with the time left.)
pub fn open(
    network: &mut NetworkState,
    game_data: &mut GameData,
    instance: &mut Instance,
    opener: ObjectId,
    treasure_id: ObjectId,
) {
    let Some(NetworkedActor::Treasure { treasure }) = instance.find_actor(treasure_id) else {
        return;
    };
    let base_id = treasure.base_id;

    if let Some(coffer) = instance.coffers.get(&treasure_id) {
        let mut segments = Vec::new();
        for (slot, loot) in coffer.slots.iter().enumerate() {
            if !loot.decided {
                let unique = game_data.item_is_unique(loot.item_id);
                segments.push(item_slot(treasure_id, coffer, slot, unique, true));
            }
        }
        for data in segments {
            let ipc = ServerZoneIpcSegment::new(data);
            network.send_to_by_actor_id(
                opener,
                FromServer::PacketSegment(ipc, opener),
                DestinationNetwork::ZoneClients,
            );
        }
        return;
    }

    let items = roll_contents(instance, base_id);
    let now = Instant::now();
    let first_serial = instance.loot_serial;
    instance.loot_serial += items.len() as u32;
    let mut coffer = Coffer {
        opened_at: now,
        slots: items
            .iter()
            .map(|item_id| LootSlot {
                item_id: *item_id,
                choices: HashMap::new(),
                decided: false,
            })
            .collect(),
        bots_due: Vec::new(),
        first_serial,
    };
    for bot in instance.party_bots.keys() {
        for slot in 0..items.len() {
            let delay = LOT_DELAYS[fastrand::usize(..LOT_DELAYS.len())];
            coffer
                .bots_due
                .push((*bot, slot, now + Duration::from_secs_f32(delay)));
        }
    }

    let mut listed = [0u32; 16];
    for (slot, item) in items.iter().take(16).enumerate() {
        listed[slot] = *item;
    }
    let claim = CLAIM_TIME.as_secs_f32();
    to_players(
        network,
        instance,
        ServerZoneIpcData::LootList {
            treasure_id,
            count: items.len() as u32,
            time_left: [claim; 3],
            items: listed,
            unk: 0,
        },
    );
    for slot in 0..items.len() {
        let unique = game_data.item_is_unique(items[slot]);
        to_players(
            network,
            instance,
            item_slot(treasure_id, &coffer, slot, unique, false),
        );
    }
    // OpenedTreasure comes from the coffer: the time three times, 1, the coffer.
    let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::OpenedTreasure {
        unk1: claim.to_bits(),
        unk2: claim.to_bits(),
        unk3: claim.to_bits(),
        unk4: 1,
        entity_id: treasure_id,
        unk6: 0,
    });
    network.send_to_instance(
        ObjectId::default(),
        instance,
        FromServer::PacketSegment(ipc, treasure_id),
        DestinationNetwork::ZoneClients,
    );

    let empty = items.is_empty();
    instance.coffers.insert(treasure_id, coffer);
    if empty {
        fade_out(network, instance, treasure_id);
    }
}

/// `member` chooses for one item. A player's choice is answered (LootRollResult); everyone hears they cast their lot.
fn choose(
    network: &mut NetworkState,
    instance: &mut Instance,
    member: ObjectId,
    treasure_id: ObjectId,
    slot: usize,
    choice: u32,
) {
    let is_player = matches!(
        instance.find_actor(member),
        Some(NetworkedActor::Player { .. })
    );
    let Some(coffer) = instance.coffers.get_mut(&treasure_id) else {
        return;
    };
    let Some(loot) = coffer.slots.get_mut(slot) else {
        return;
    };
    if loot.decided {
        return;
    }
    let roll = fastrand::u8(1..=99);
    // Greed Only (and Lootmaster, as Kawari plays it): a Need counts as a Greed (`duty_finder::LootRule`.)
    let choice = if choice == CHOICE_NEED && instance.loot_rule.greed_only() {
        CHOICE_GREED
    } else {
        choice
    };
    let made = match choice {
        CHOICE_NEED => Choice::Need(roll),
        CHOICE_GREED => Choice::Greed(roll),
        _ => Choice::Pass,
    };
    let first = !loot.choices.contains_key(&member);
    loot.choices.insert(member, made);
    let item_id = loot.item_id;

    if is_player {
        let (result, roll) = match made {
            Choice::Need(roll) | Choice::Greed(roll) => (RESULT_ROLLED, roll),
            Choice::Pass => (RESULT_PASSED, 0),
        };
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::LootRollResult {
            treasure_id,
            slot: slot as u32,
            result,
            roll,
            unk: 0,
        });
        network.send_to_by_actor_id(
            member,
            FromServer::PacketSegment(ipc, member),
            DestinationNetwork::ZoneClients,
        );
    }
    if first {
        loot_message(
            network,
            instance,
            LOOT_CAST_LOT,
            [member.0, item_id, 1, 0, 0, 0, 0],
        );
    }

    let everyone = members(instance);
    let all_chose = instance
        .coffers
        .get(&treasure_id)
        .and_then(|coffer| coffer.slots.get(slot))
        .is_some_and(|loot| {
            everyone
                .iter()
                .all(|member| loot.choices.contains_key(member))
        });
    if all_chose {
        decide(network, instance, treasure_id, slot);
    }
}

/// A player's LootRoll.
pub fn player_roll(
    network: &mut NetworkState,
    instance: &mut Instance,
    player: ObjectId,
    treasure_id: ObjectId,
    slot: u32,
    choice: u32,
) {
    choose(
        network,
        instance,
        player,
        treasure_id,
        slot as usize,
        choice,
    );
}

/// The item in `slot` is decided: the highest Need roll wins, or failing any the highest Greed roll; with none,
/// someone who didn't pass gets it, and if everyone passed it's lost.
fn decide(network: &mut NetworkState, instance: &mut Instance, treasure_id: ObjectId, slot: usize) {
    let everyone = members(instance);
    let Some(coffer) = instance.coffers.get_mut(&treasure_id) else {
        return;
    };
    let Some(loot) = coffer.slots.get_mut(slot) else {
        return;
    };
    if loot.decided {
        return;
    }
    loot.decided = true;
    let item_id = loot.item_id;
    let choices = loot.choices.clone();

    let winner = match outcome(&choices, &everyone, |n| fastrand::usize(..n)) {
        Outcome::Rolled {
            rolls,
            kind,
            winner,
            roll,
        } => {
            for (member, roll) in &rolls {
                loot_message(
                    network,
                    instance,
                    LOOT_ROLL,
                    [member.0, kind, *roll as u32, item_id, 1, 0, 0],
                );
            }
            loot_message(
                network,
                instance,
                LOOT_WON,
                [winner.0, kind, roll as u32, item_id, 1, 1, 0],
            );
            Some((winner, roll))
        }
        Outcome::Unrolled(winner) => {
            loot_message(
                network,
                instance,
                LOOT_OBTAINED,
                [winner.0, item_id, 1, 0, 0, 0, 0],
            );
            Some((winner, 0))
        }
        Outcome::Lost => {
            loot_message(network, instance, LOOT_LOST, [item_id, 1, 0, 0, 0, 0, 0]);
            None
        }
    };

    let (winner_id, roll) = winner.unwrap_or((NOBODY, 0));
    to_players(
        network,
        instance,
        ServerZoneIpcData::LootResult {
            treasure_id,
            slot: slot as u8,
            unk: [0; 3],
            winner: winner_id,
            roll: roll as u32,
        },
    );
    if let Some((winner, _)) = winner
        && matches!(
            instance.find_actor(winner),
            Some(NetworkedActor::Player { .. })
        )
    {
        network.send_to_by_actor_id(
            winner,
            FromServer::LootWon(item_id),
            DestinationNetwork::ZoneClients,
        );
    }

    let all_decided = instance
        .coffers
        .get(&treasure_id)
        .is_some_and(|coffer| coffer.slots.iter().all(|loot| loot.decided));
    if all_decided {
        fade_out(network, instance, treasure_id);
    }
}

fn fade_out(network: &mut NetworkState, instance: &Instance, treasure_id: ObjectId) {
    let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::TreasureFadeOut { unk1: 0, unk2: 0 });
    network.send_to_instance(
        ObjectId::default(),
        instance,
        FromServer::PacketSegment(ipc, treasure_id),
        DestinationNetwork::ZoneClients,
    );
}

/// Every tick: bots whose time has come choose, and items whose time is up are decided.
pub fn tick(network: &mut NetworkState, instance: &mut Instance) {
    if instance.coffers.is_empty() {
        return;
    }
    let now = Instant::now();
    let mut due = Vec::new();
    let mut expired = Vec::new();
    for (treasure_id, coffer) in &mut instance.coffers {
        coffer.bots_due.retain(|(bot, slot, at)| {
            if *at <= now {
                due.push((*treasure_id, *bot, *slot));
                false
            } else {
                true
            }
        });
        if coffer.opened_at.elapsed() >= CLAIM_TIME {
            for (slot, loot) in coffer.slots.iter().enumerate() {
                if !loot.decided {
                    expired.push((*treasure_id, slot));
                }
            }
        }
    }
    for (treasure_id, bot, slot) in due {
        if instance.find_actor(bot).is_none() {
            continue;
        }
        // Bots always pass, so the loot is the player's (Bob's call; retail's players passed 33 of their 35 lots in
        // the capture, greeded the other two.)
        let choice = CHOICE_PASS;
        choose(network, instance, bot, treasure_id, slot, choice);
    }
    for (treasure_id, slot) in expired {
        decide(network, instance, treasure_id, slot);
    }
}

/// The duty is over: whatever is still to be decided is decided now (retail decided the rest as the duty ended.)
pub fn decide_all(network: &mut NetworkState, instance: &mut Instance) {
    let pending: Vec<(ObjectId, usize)> = instance
        .coffers
        .iter()
        .flat_map(|(treasure_id, coffer)| {
            coffer
                .slots
                .iter()
                .enumerate()
                .filter(|(_, loot)| !loot.decided)
                .map(|(slot, _)| (*treasure_id, slot))
                .collect::<Vec<_>>()
        })
        .collect();
    for (treasure_id, slot) in pending {
        decide(network, instance, treasure_id, slot);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: ObjectId = ObjectId(1);
    const B: ObjectId = ObjectId(2);
    const C: ObjectId = ObjectId(3);

    #[test]
    fn need_beats_greed_and_the_highest_roll_wins() {
        let choices = HashMap::from([
            (A, Choice::Greed(90)),
            (B, Choice::Need(10)),
            (C, Choice::Need(40)),
        ]);
        assert_eq!(
            outcome(&choices, &[A, B, C], |_| 0),
            Outcome::Rolled {
                rolls: vec![(B, 10), (C, 40)],
                kind: ROLL_NEED,
                winner: C,
                roll: 40
            }
        );
        let choices = HashMap::from([
            (A, Choice::Greed(33)),
            (B, Choice::Pass),
            (C, Choice::Greed(44)),
        ]);
        assert_eq!(
            outcome(&choices, &[A, B, C], |_| 0),
            Outcome::Rolled {
                rolls: vec![(A, 33), (C, 44)],
                kind: ROLL_GREED,
                winner: C,
                roll: 44
            }
        );
    }

    #[test]
    fn unrolled_items_go_to_someone_who_didnt_pass_or_are_lost() {
        // Retail: the others passed and Bob hadn't chosen when the time ran out; he got it, roll 0.
        let choices = HashMap::from([(A, Choice::Pass), (B, Choice::Pass)]);
        assert_eq!(outcome(&choices, &[A, B, C], |_| 0), Outcome::Unrolled(C));
        let choices = HashMap::from([(A, Choice::Pass), (B, Choice::Pass), (C, Choice::Pass)]);
        assert_eq!(outcome(&choices, &[A, B, C], |_| 0), Outcome::Lost);
    }

    #[test]
    fn contents_follow_the_recorded_openings() {
        // Sastasha's first boss coffer: two items, a piece of gear and the wristlets or the earrings.
        let pools = HashMap::from([
            (
                94,
                vec![vec![3554, 4096], vec![2684, 4096], vec![2683, 4207]],
            ),
            (95, vec![vec![3034]]),
        ]);
        assert_eq!(contents(&pools, 94, |_| 0), vec![3554, 4096]);
        assert_eq!(contents(&pools, 94, |n| n - 1), vec![2683, 4207]);
        assert_eq!(contents(&pools, 95, |_| 0), vec![3034]);
        // A coffer never recorded holds one of the duty's items.
        assert_eq!(contents(&pools, 100, |n| n - 1).len(), 1);
        assert!(contents(&HashMap::new(), 94, |_| 0).is_empty());
    }

    #[test]
    fn lot_delays_are_retails_spread() {
        // Half of retail's lots came within 6 s of the coffer opening, all within the 300 s.
        let mut delays = LOT_DELAYS.to_vec();
        delays.sort_by(f32::total_cmp);
        assert_eq!(delays[delays.len() / 2], 6.0);
        assert!(delays.iter().all(|delay| *delay < CLAIM_TIME.as_secs_f32()));
    }
}
