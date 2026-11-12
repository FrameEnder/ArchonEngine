//! World bots' stage 3: what makes them more like players (docs/src/fork/world-bots.md): minions, weapons drawn and
//! sheathed, mounts in the open world, crafting in place, Player Search and chat.
//!
//! Every number here is from retail's other players in Bob's Chronofoil captures (124 captures: 404 player-hours in
//! the six city zones, 179 elsewhere outside duties; numbers only), read by the diagnostics repository's
//! `notes/world-bots/` scripts and the ones named where they're used.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use kawari::{
    common::CharacterMode,
    ipc::zone::{
        ActionEffect, ActionType, ActorControlCategory, LiveEventType, ServerZoneIpcData,
        ServerZoneIpcSegment, TargetEffect,
    },
};
use parking_lot::Mutex;
use serde_json::Value;

use crate::{
    FromServer,
    server::{
        actor::{NetworkedActor, set_character_mode},
        instance::Instance,
        network::{DestinationNetwork, NetworkState},
        npc_behavior::BEHAVIOR_TICK_SECONDS,
    },
};

use kawari::common::{ObjectId, ObjectTypeId, ObjectTypeKind};

/// The six city zones (Limsa Lominsa's, Ul'dah's and Gridania's two each), where retail's numbers differ from the open
/// world's.
pub fn is_city(territory: u16) -> bool {
    (128..=133).contains(&territory)
}

/// A value drawn from retail's 10th, 25th, 50th, 75th and 90th percentiles (straight lines between them; below the 10th
/// down to `low`, above the 90th up to twice it.)
pub fn from_percentiles(table: [f32; 5], low: f32) -> f32 {
    let points = [
        (0.0, low),
        (0.1, table[0]),
        (0.25, table[1]),
        (0.5, table[2]),
        (0.75, table[3]),
        (0.9, table[4]),
        (1.0, table[4] * 2.0),
    ];
    let roll = fastrand::f32();
    points
        .windows(2)
        .find(|pair| roll <= pair[1].0)
        .map(|pair| {
            let ((p0, s0), (p1, s1)) = (pair[0], pair[1]);
            s0 + (s1 - s0) * (roll - p0) / (p1 - p0)
        })
        .unwrap_or(table[4])
}

/// Whether something that happens `per_hour` times an hour happens this tick.
fn this_tick(per_hour: f32) -> bool {
    fastrand::f32() < per_hour * BEHAVIOR_TICK_SECONDS / 3600.0
}

// ----------------------------------------------------------------------------------------------------------------
// Minions and mounts: which ones.

/// Retail's minions and mounts as players came into view (`resources/data/world_bot_companions.json`: Companion and
/// Mount rows with how many had each; 497 minions among 8,850, 137 mounts among 473.)
pub struct Companions {
    pub minions: Vec<(u16, u16)>,
    pub mounts: Vec<(u16, u16)>,
}

pub fn companions() -> &'static Companions {
    static COMPANIONS: std::sync::OnceLock<Companions> = std::sync::OnceLock::new();
    COMPANIONS.get_or_init(|| {
        let value: Value = serde_json::from_str(include_str!(
            "../../../../resources/data/world_bot_companions.json"
        ))
        .unwrap_or_default();
        let list = |key: &str| -> Vec<(u16, u16)> {
            value
                .get(key)
                .and_then(Value::as_array)
                .map(|rows| {
                    rows.iter()
                        .filter_map(|row| {
                            let row = row.as_array()?;
                            Some((
                                row.first()?.as_u64()? as u16,
                                row.get(1)?.as_u64()?.min(u16::MAX as u64) as u16,
                            ))
                        })
                        .collect()
                })
                .unwrap_or_default()
        };
        Companions {
            minions: list("minions"),
            mounts: list("mounts"),
        }
    })
}

/// One of `choices` by weight.
pub fn weighted<T: Copy>(choices: &[(T, u16)]) -> Option<T> {
    let total: u32 = choices.iter().map(|(_, weight)| *weight as u32).sum();
    if total == 0 {
        return None;
    }
    let mut roll = fastrand::u32(..total);
    for (choice, weight) in choices {
        if roll < *weight as u32 {
            return Some(*choice);
        }
        roll -= *weight as u32;
    }
    None
}

/// More than half of retail's players had a minion out as they came into view: 6,627 of 12,295 in the cities, 2,223
/// of 3,878 elsewhere.
pub const MINION_SHARE: f32 = 0.55;

/// A minion for a new bot, or none (0), as many as retail's had one.
pub fn random_minion() -> u16 {
    if fastrand::f32() < MINION_SHARE {
        weighted(&companions().minions).unwrap_or(0)
    } else {
        0
    }
}

// ----------------------------------------------------------------------------------------------------------------
// While it stands: minions called and sent away, weapons drawn and sheathed.

/// ActorControl 270 (MinionSpawnControl) with a minion: 266 in the cities (0.66 a player-hour), 103 elsewhere (0.58);
/// with none, sending it away: 6 in all.
const MINION_SUMMONS_PER_HOUR: f32 = 0.63;
const MINION_DISMISSALS_PER_HOUR: f32 = 0.01;

/// ActorControl 0 (ToggleWeapon) `[1, 1]`, drawing it: 641 in the cities (1.6 a player-hour), 469 elsewhere (2.6), and
/// how long it stayed out before `[0, 1]` (seconds, 10th to 90th percentiles: 620 and 509 of them.)
const WEAPON_DRAWS_PER_HOUR_CITY: f32 = 1.6;
const WEAPON_DRAWS_PER_HOUR_ELSEWHERE: f32 = 2.6;
const DRAWN_FOR_CITY: [f32; 5] = [1.2, 1.6, 3.3, 6.0, 19.8];
const DRAWN_FOR_ELSEWHERE: [f32; 5] = [1.4, 2.9, 7.3, 21.3, 81.0];

/// What a standing bot's minion and weapon are doing.
#[derive(Debug, Clone, Default)]
pub struct Standing {
    /// Its weapon is out until then.
    pub drawn_until: Option<Instant>,
}

/// A standing bot's tick (not sitting, crafting or riding): now and then a minion called (a new one) or sent away, its
/// weapon drawn and, a while later, sheathed, as often as retail's players.
pub fn stand(
    standing: &mut Standing,
    minion: &mut u16,
    territory: u16,
    actor: ObjectId,
    instance: &mut Instance,
    network: &Arc<Mutex<NetworkState>>,
    now: Instant,
) {
    if standing.drawn_until.is_some_and(|until| now >= until) {
        standing.drawn_until = None;
        send_control(actor, instance, network, sheathe());
    } else if standing.drawn_until.is_none() {
        let (rate, table) = if is_city(territory) {
            (WEAPON_DRAWS_PER_HOUR_CITY, DRAWN_FOR_CITY)
        } else {
            (WEAPON_DRAWS_PER_HOUR_ELSEWHERE, DRAWN_FOR_ELSEWHERE)
        };
        if this_tick(rate) {
            standing.drawn_until =
                Some(now + Duration::from_secs_f32(from_percentiles(table, 0.8)));
            send_control(
                actor,
                instance,
                network,
                ActorControlCategory::ToggleWeapon {
                    shown: true,
                    unk_flag: 1,
                    unk2: 0,
                },
            );
        }
    }

    let new_minion = if this_tick(MINION_SUMMONS_PER_HOUR) {
        weighted(&companions().minions)
    } else if *minion != 0 && this_tick(MINION_DISMISSALS_PER_HOUR) {
        Some(0)
    } else {
        None
    };
    if let Some(new_minion) = new_minion {
        set_minion(minion, new_minion, actor, instance, network);
    }
}

/// Puts its weapon away now if it's out (before it sits, crafts or mounts.)
pub fn put_weapon_away(
    standing: &mut Standing,
    actor: ObjectId,
    instance: &mut Instance,
    network: &Arc<Mutex<NetworkState>>,
) {
    if standing.drawn_until.take().is_some() {
        send_control(actor, instance, network, sheathe());
    }
}

/// ToggleWeapon `[0, 1]`: sheathed (also what retail sent as a player mounted and got off.)
pub fn sheathe() -> ActorControlCategory {
    ActorControlCategory::ToggleWeapon {
        shown: false,
        unk_flag: 1,
        unk2: 0,
    }
}

/// Its minion: called (another one), or sent away (0): MinionSpawnControl from it, and its spawn for those who come.
pub fn set_minion(
    minion: &mut u16,
    to: u16,
    actor: ObjectId,
    instance: &mut Instance,
    network: &Arc<Mutex<NetworkState>>,
) {
    *minion = to;
    if let Some(NetworkedActor::Npc { spawn, .. }) = instance.find_actor_mut(actor) {
        spawn.common.active_minion = to;
    }
    send_control(
        actor,
        instance,
        network,
        ActorControlCategory::MinionSpawnControl {
            minion_id: to as u32,
        },
    );
}

/// An ActorControl from `actor` to everyone who sees it.
pub fn send_control(
    actor: ObjectId,
    instance: &mut Instance,
    network: &Arc<Mutex<NetworkState>>,
    category: ActorControlCategory,
) {
    network.lock().send_in_range_instance(
        actor,
        instance,
        FromServer::ActorControl(actor, category),
        DestinationNetwork::ZoneClients,
    );
}

// ----------------------------------------------------------------------------------------------------------------
// Mounts: a long walk in a zone that allows them is ridden, as retail's players rode.

/// How retail's other players mounted, as others saw it (Bob's Chronofoil captures, the open-world zones): ActorCast of
/// the mount (action type 13) with a bar of 0.98 s, its ActionEffect as the bar ended, then 0.58 s later, all at once,
/// ToggleWeapon `[0, 1]`, SetMode 4 and the Mount packet. Getting off: SetMode 1 with PlayDismountAnimation (ActorControl
/// 930, where it stands packed), and ToggleWeapon `[0, 1]` 0.6 s later.
const MOUNT_CAST: f32 = 0.98;
const MOUNT_ARRIVES: Duration = Duration::from_millis(580);
/// How fast it rides: retail's mounted players' moves were 11.7 yalms a second at the median (28,354 of them, 5.2-19.9
/// from the 10th to the 90th percentile), and the commonest speed byte 120.
pub const RIDE_SPEED: f32 = 12.0;
/// Walks at least this long are ridden where mounts are allowed (a guess: retail's captures don't show where a player
/// set off from.) 473 of 3,878 retail players outside the cities (12%) were riding as they came into view; none in a
/// city, where mounts aren't allowed.
pub const RIDES_FROM: f32 = 40.0;

/// A bot riding, or getting on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Riding {
    /// Its cast is under way: the ActionEffect at `effect_at`, on the mount at `up_at`.
    Mounting {
        mount: u16,
        effect_at: Instant,
        up_at: Instant,
        effect_sent: bool,
    },
    Riding {
        mount: u16,
    },
}

impl Riding {
    pub fn mount(&self) -> u16 {
        match self {
            Riding::Mounting { mount, .. } | Riding::Riding { mount } => *mount,
        }
    }
}

/// Gets on a mount drawn from retail's: its cast shown now if it's seen, else just on it.
pub fn mount_up(
    shown: Option<(ObjectId, &mut Instance)>,
    facing: f32,
    network: &Arc<Mutex<NetworkState>>,
    now: Instant,
) -> Option<Riding> {
    let mount = weighted(&companions().mounts)?;
    let Some((actor, instance)) = shown else {
        return Some(Riding::Riding { mount });
    };
    let Some(position) = instance.find_actor(actor).map(|actor| actor.position()) else {
        return Some(Riding::Riding { mount });
    };
    let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorCast {
        spell_id: mount,
        action_type: ActionType::Mount,
        omen_delay: 0,
        action_id: mount as u32,
        cast_time: MOUNT_CAST,
        target: actor,
        rotation: facing,
        interruptible: false,
        ballista_entity_id: ObjectId::default(),
        position,
    });
    network.lock().send_in_range_instance(
        actor,
        instance,
        FromServer::PacketSegment(ipc, actor),
        DestinationNetwork::ZoneClients,
    );
    let effect_at = now + Duration::from_secs_f32(MOUNT_CAST);
    Some(Riding::Mounting {
        mount,
        effect_at,
        up_at: effect_at + MOUNT_ARRIVES,
        effect_sent: false,
    })
}

/// A tick of getting on: the ActionEffect as the cast ends, then on the mount. Returns whether it's still getting on
/// (it doesn't move till it's up.)
pub fn mounting(
    riding: &mut Riding,
    actor: ObjectId,
    instance: &mut Instance,
    network: &Arc<Mutex<NetworkState>>,
    facing: f32,
    now: Instant,
) -> bool {
    let Riding::Mounting {
        mount,
        effect_at,
        up_at,
        effect_sent,
    } = riding
    else {
        return false;
    };
    let mount = *mount;
    if !*effect_sent && now >= *effect_at {
        *effect_sent = true;
        let mut network = network.lock();
        let ipc = ActionEffect {
            animation_target_id: own(actor),
            targets: vec![own(actor)],
            action_id: mount as u32,
            animation_lock: 0.6,
            rotation: facing,
            spell_id: mount,
            effects: [TargetEffect::default(); 8],
            action_type: ActionType::Mount,
            global_sequence: network.global_action_sequence,
            ..Default::default()
        }
        .package();
        network.global_action_sequence += 1;
        if let Some(ipc) = ipc {
            network.send_in_range_instance(
                actor,
                instance,
                FromServer::PacketSegment(ipc, actor),
                DestinationNetwork::ZoneClients,
            );
        }
    }
    if now < *up_at {
        return true;
    }
    *riding = Riding::Riding { mount };
    on_the_mount(actor, instance, network, mount);
    false
}

/// An actor as an action's target.
fn own(actor: ObjectId) -> ObjectTypeId {
    ObjectTypeId {
        object_id: actor,
        object_type: ObjectTypeKind::None,
    }
}

/// On it, as retail's players were seen: ToggleWeapon `[0, 1]`, SetMode 4 and the Mount packet; its spawn too.
fn on_the_mount(
    actor: ObjectId,
    instance: &mut Instance,
    network: &Arc<Mutex<NetworkState>>,
    mount: u16,
) {
    if let Some(NetworkedActor::Npc { spawn, .. }) = instance.find_actor_mut(actor) {
        spawn.common.current_mount = mount;
    }
    send_control(actor, instance, network, sheathe());
    set_character_mode(
        instance,
        &mut network.lock(),
        actor,
        CharacterMode::Mounted,
        0,
        true,
    );
    network.lock().send_in_range_instance(
        actor,
        instance,
        FromServer::PacketSegment(
            crate::server::action::mount_segment(mount as u32, None),
            actor,
        ),
        DestinationNetwork::ZoneClients,
    );
}

/// Off its mount where it stands, as retail's players got off: SetMode 1, PlayDismountAnimation, ToggleWeapon `[0, 1]`.
pub fn dismount(actor: ObjectId, instance: &mut Instance, network: &Arc<Mutex<NetworkState>>) {
    let Some(NetworkedActor::Npc { spawn, .. }) = instance.find_actor_mut(actor) else {
        return;
    };
    spawn.common.current_mount = 0;
    let (unk1, unk2, unk3) =
        kawari::common::dismount_landing(&spawn.common.position, spawn.common.rotation);
    set_character_mode(
        instance,
        &mut network.lock(),
        actor,
        CharacterMode::Normal,
        0,
        true,
    );
    send_control(
        actor,
        instance,
        network,
        ActorControlCategory::PlayDismountAnimation { unk1, unk2, unk3 },
    );
    send_control(actor, instance, network, sheathe());
}

// ----------------------------------------------------------------------------------------------------------------
// Crafting in place: a crafter in town now and then crafts where it stands, as retail's players were seen to.

/// How retail's other players' synthesis looked to others (Bob's Chronofoil captures: 142 sessions, 669 syntheses,
/// SetMode 5 and LiveEvent, ActorControl 300, from their actors; `craft.py`-style counts):
///
/// * the session: SetMode 5; at its end LiveEvent 14 and 40, then SetMode 1 a second later (122 of 141);
/// * each synthesis: 38 (the main tool's model) and 39 (the off hand: nothing, 756 of 979), then 8 (the session's
///   first) or 9 (`[3, …, 1]`), the condition 10 `[2]` 0.7 s later, its steps (12, the CraftAction's two animations)
///   the first 0.5-3.1 s after that and then every 2.5-5.2 s, each followed by a condition 2.0 s later about half the
///   time; 2.2 s after the last step HQ (482 of 597) as 16 then 19 `[item + 1,000,000, how many]`, 17 0.5 s later and
///   11 1.8 s after that, or NQ as 15 then 18 `[item, how many]` and 11 1.5 s later; the next synthesis 0.8-10.6 s
///   after the 11.
const CRAFT_STEP_ANIMATIONS: [((u32, u32), u16); 4] = [
    ((240, 247), 1374),
    ((239, 246), 716),
    ((241, 248), 691),
    ((242, 249), 271),
];
/// Steps in a synthesis (retail's, without quick syntheses: 10th to 90th percentiles.)
const CRAFT_STEPS: [f32; 5] = [1.0, 1.0, 3.0, 8.0, 15.0];
/// Seconds between steps, and from the 11 to the next synthesis.
const CRAFT_STEP_GAP: [f32; 5] = [2.5, 2.5, 2.8, 3.9, 5.2];
const CRAFT_NEXT_GAP: [f32; 5] = [0.8, 1.0, 1.3, 3.7, 10.6];
/// A session's length (seconds; 142 sessions, the ones seen from start to end.)
const CRAFT_SESSION: [f32; 5] = [12.6, 25.5, 60.3, 121.7, 308.0];
/// The condition after a step: Normal (2) most often, Good (3), Excellent (4), Poor (5).
const CRAFT_CONDITIONS: [(u32, u16); 4] = [(2, 1166), (3, 543), (4, 61), (5, 50)];
const CRAFT_HQ: f32 = 0.81;
/// StartCraft's second argument (a retail value: `events/crafting.rs`.)
const START_CRAFT_UNKNOWN: u32 = 136_153;
/// HQ items' IDs as the client shows them.
const HQ_ID_OFFSET: u32 = 1_000_000;

/// A crafter at home in a city crafts where it stands instead of going to another spot this often: 116 of retail's
/// 12,295 city players came into view crafting, about one crafter in eight (classes 8-15 being 1% each.)
pub const CRAFTS_INSTEAD: f32 = 0.12;

/// A crafter's recipes near its level (`GameData::crafter_recipes`): (item, how many, can be HQ.)
pub fn recipes_near(
    recipes: &std::collections::HashMap<u8, Vec<(u8, u32, u8, bool)>>,
    classjob: u8,
    level: u8,
) -> Vec<(u32, u8, bool)> {
    recipes
        .get(&classjob)
        .map(|all| {
            all.iter()
                .filter(|(at, ..)| *at <= level && *at + 10 > level)
                .map(|(_, item, amount, hq)| (*item, *amount, *hq))
                .collect()
        })
        .unwrap_or_default()
}

/// A bot crafting in place.
#[derive(Debug, Clone)]
pub struct Crafting {
    /// When the session ends (after the synthesis under way.)
    pub until: Instant,
    queue: std::collections::VecDeque<(Instant, LiveEventType)>,
    syntheses: u32,
    /// Its session's end has been sent: up at `until`.
    ending: bool,
}

/// One synthesis as retail's looked, from `at`: what's sent and when.
pub fn synthesis(
    at: Instant,
    first: bool,
    (item, amount, can_hq): (u32, u8, bool),
    main_tool: u64,
) -> Vec<(Instant, LiveEventType)> {
    let unknown = |event, param1, param2, param3| LiveEventType::Unknown {
        event,
        param1,
        param2,
        param3,
    };
    let mut events = vec![
        (
            at,
            LiveEventType::SetMainHand {
                model_id: main_tool as u32,
                unk1: (main_tool >> 32) as u32,
                unk2: 0,
            },
        ),
        (
            at,
            LiveEventType::SetOffHand {
                model_id: 0,
                unk1: 0,
                unk2: 0,
            },
        ),
        (
            at,
            if first {
                LiveEventType::StartCraft {
                    unk1: 3,
                    unk2: START_CRAFT_UNKNOWN,
                    unk3: 1,
                }
            } else {
                unknown(9, 3, START_CRAFT_UNKNOWN, 1)
            },
        ),
    ];
    let mut t = at + Duration::from_millis(700);
    events.push((t, unknown(10, 2, 0, 0)));
    t += Duration::from_secs_f32(0.5 + fastrand::f32() * 2.6);
    let steps = from_percentiles(CRAFT_STEPS, 1.0).round().max(1.0) as u32;
    for step in 0..steps {
        if step > 0 {
            t += Duration::from_secs_f32(from_percentiles(CRAFT_STEP_GAP, 2.4));
        }
        let (start, end) = weighted(&CRAFT_STEP_ANIMATIONS).unwrap_or((240, 247));
        events.push((
            t,
            LiveEventType::PlayAnimation {
                animation_start: start,
                animation_end: end,
            },
        ));
        if step + 1 < steps && fastrand::bool() {
            events.push((
                t + Duration::from_secs(2),
                unknown(10, weighted(&CRAFT_CONDITIONS).unwrap_or(2), 0, 0),
            ));
        }
    }
    t += Duration::from_millis(2200);
    if can_hq && fastrand::f32() < CRAFT_HQ {
        events.push((t, unknown(16, 0, 0, 0)));
        events.push((t, unknown(19, item + HQ_ID_OFFSET, amount as u32, 0)));
        events.push((t + Duration::from_millis(500), unknown(17, 0, 0, 0)));
        events.push((t + Duration::from_millis(2300), unknown(11, 0, 0, 0)));
    } else {
        events.push((t, LiveEventType::EndCraft));
        events.push((t, unknown(18, item, amount as u32, 0)));
        events.push((t + Duration::from_millis(1500), unknown(11, 0, 0, 0)));
    }
    events
}

/// Starts crafting where it stands: SetMode 5 if it's seen, and its first synthesis.
pub fn start_crafting(
    shown: Option<(ObjectId, &mut Instance)>,
    network: &Arc<Mutex<NetworkState>>,
    recipe: (u32, u8, bool),
    main_tool: u64,
    now: Instant,
) -> Crafting {
    if let Some((actor, instance)) = shown {
        set_character_mode(
            instance,
            &mut network.lock(),
            actor,
            CharacterMode::Crafting,
            0,
            true,
        );
    }
    Crafting {
        until: now + Duration::from_secs_f32(from_percentiles(CRAFT_SESSION, 8.0)),
        queue: synthesis(now, true, recipe, main_tool).into(),
        syntheses: 1,
        ending: false,
    }
}

/// A tick of its crafting: what's due sent (if it's seen), the next synthesis planned, or the session ended (14 and
/// 40, then SetMode 1 a second later). Returns whether it's still crafting.
pub fn crafting(
    craft: &mut Crafting,
    mut shown: Option<(ObjectId, &mut Instance)>,
    network: &Arc<Mutex<NetworkState>>,
    recipe: Option<(u32, u8, bool)>,
    main_tool: u64,
    now: Instant,
) -> bool {
    while let Some((at, _)) = craft.queue.front()
        && *at <= now
    {
        let (_, event) = craft.queue.pop_front().unwrap();
        if let Some((actor, instance)) = shown.as_mut() {
            send_control(
                *actor,
                instance,
                network,
                ActorControlCategory::LiveEvent { event },
            );
        }
    }
    if !craft.queue.is_empty() {
        return true;
    }
    if craft.ending {
        if now < craft.until {
            return true;
        }
        if let Some((actor, instance)) = shown {
            set_character_mode(
                instance,
                &mut network.lock(),
                actor,
                CharacterMode::Normal,
                0,
                true,
            );
        }
        return false;
    }
    if now < craft.until
        && let Some(recipe) = recipe
    {
        let next = now + Duration::from_secs_f32(from_percentiles(CRAFT_NEXT_GAP, 0.6));
        craft.queue = synthesis(next, false, recipe, main_tool).into();
        craft.syntheses += 1;
        return true;
    }
    // Done: 14 and 40, then up a second later.
    craft.ending = true;
    craft.until = now + Duration::from_secs(1);
    if let Some((actor, instance)) = shown {
        for event in [14, 40] {
            send_control(
                actor,
                instance,
                network,
                ActorControlCategory::LiveEvent {
                    event: LiveEventType::Unknown {
                        event,
                        param1: 0,
                        param2: 0,
                        param3: 0,
                    },
                },
            );
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retail_s_minions_and_mounts_are_read() {
        let companions = companions();
        assert_eq!(companions.minions.len(), 497);
        assert_eq!(companions.mounts.len(), 137);
        let all: u32 = companions.minions.iter().map(|(_, n)| *n as u32).sum();
        assert_eq!(all, 8850);
    }

    /// Stage 3: a synthesis as retail's players' looked to others: the tools, the start, the condition, its steps,
    /// then HQ (16, 19 with the HQ item, 17, 11) or NQ (15, 18 with the item, 11).
    #[test]
    fn a_synthesis_as_retail_s() {
        let now = Instant::now();
        for _ in 0..50 {
            let events = synthesis(now, true, (5056, 3, true), 0x0001_0002_0003);
            let kinds: Vec<u32> = events
                .iter()
                .map(|(_, event)| match event {
                    LiveEventType::SetMainHand { .. } => 38,
                    LiveEventType::SetOffHand { .. } => 39,
                    LiveEventType::StartCraft { .. } => 8,
                    LiveEventType::PlayAnimation { .. } => 12,
                    LiveEventType::EndCraft => 15,
                    LiveEventType::Unknown { event, .. } => *event,
                    _ => 0,
                })
                .collect();
            assert_eq!(kinds[..4], [38, 39, 8, 10]);
            assert!(kinds.contains(&12));
            assert_eq!(*kinds.last().unwrap(), 11);
            let hq = kinds.contains(&16);
            assert_eq!(kinds.contains(&19), hq);
            assert_eq!(kinds.contains(&18), !hq);
            // In time order (a condition 2 s after a step comes before the next, 2.4 s or more after it.)
            assert!(events.windows(2).all(|pair| pair[0].0 <= pair[1].0));
            if hq {
                assert!(events.iter().any(|(_, event)| matches!(
                    event,
                    LiveEventType::Unknown {
                        event: 19,
                        param1: 1_005_056,
                        param2: 3,
                        ..
                    }
                )));
            }
        }
    }

    /// Stage 3: crafting in place, seen: SetMode 5, the syntheses' LiveEvents as they come due, then 14 and 40 and
    /// SetMode 1 a second later, as retail's players' sessions ended.
    #[test]
    fn a_crafter_crafts_where_it_stands() {
        use crate::server::test_world::{TestWorld, control_from};
        let mut world = TestWorld::new();
        let player = world.player(1, [0.0, 0.0, 0.0]);
        let actor = world.player_npc(2, player, [1.0, 0.0, 0.0]);
        let network = world.network.clone();
        let now = Instant::now();
        world.sent(player);
        let recipe = (5056, 1, true);
        let mut craft =
            start_crafting(Some((actor, &mut world.instance)), &network, recipe, 7, now);
        assert!(world.sent(player).iter().any(|message| control_from(
            message,
            actor,
            |category| matches!(
                category,
                ActorControlCategory::SetMode {
                    mode: CharacterMode::Crafting,
                    ..
                }
            )
        )));
        // Over in a minute or so: everything due goes out, then the end.
        craft.until = now + Duration::from_secs(1);
        let mut live = 0;
        let mut t = now;
        let mut ended = false;
        while t < now + Duration::from_secs(300) {
            if !crafting(
                &mut craft,
                Some((actor, &mut world.instance)),
                &network,
                Some(recipe),
                7,
                t,
            ) {
                ended = true;
                break;
            }
            live += world
                .sent(player)
                .iter()
                .filter(|message| {
                    control_from(message, actor, |category| {
                        matches!(category, ActorControlCategory::LiveEvent { .. })
                    })
                })
                .count();
            t += Duration::from_millis(500);
        }
        assert!(ended);
        assert!(live >= 8, "{live} LiveEvents");
        assert_eq!(craft.syntheses, 1);
        assert!(world.sent(player).iter().any(|message| control_from(
            message,
            actor,
            |category| matches!(
                category,
                ActorControlCategory::SetMode {
                    mode: CharacterMode::Normal,
                    ..
                }
            )
        )));
    }

    #[test]
    fn percentiles_stay_within_their_ends() {
        for _ in 0..1000 {
            let drawn = from_percentiles(DRAWN_FOR_CITY, 0.8);
            assert!((0.8..=DRAWN_FOR_CITY[4] * 2.0).contains(&drawn));
        }
    }
}
