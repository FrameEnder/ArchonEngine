//! Party NPCs: retail's Duty Support members, the story's adventurers who go through a duty with you. See
//! `docs/src/fork/party-npcs.md`.
//!
//! They are party bots underneath (`party_bot::PartyBot`, with `party_npc` set), so they follow, pick targets, die
//! and get up as bots do, but each is a recorded member rather than a made-up one:
//!
//! * spawned as retail spawns it: its BNpcBase and BNpcName, owned by the player, of sub-kind NpcPartyMember, at the
//!   duty's level with retail's HP, its look and gear from the game data (BNpcBase's BNpcCustomize and NpcEquip);
//! * listed in the client's party list the way retail lists Duty Support (ActorControlSelf 257, 276 and 1525 for each
//!   slot, then the member's StatusEffectList), not with a PartyList;
//! * fighting with its own kit, at its own pace (`Kit`), the actions and timings retail's member used;
//! * talking: its lines during fights, and the duty's story lines when the director says (`queue_yell`);
//! * gone when the duty is cleared, as retail took them away.
//!
//! Every number here is from Bob's Chronofoil capture of 2026-10-23 (700cc294, visit 0.3: Sastasha with Duty Support,
//! the player a level 17 Conjurer), `questkit chronofoil --battle 0.3` and its packets. Only the members of that run
//! are known; any other member the client asks for is filled by a party bot of the role it lacks.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use kawari::{
    common::{ObjectId, ObjectTypeId, ObjectTypeKind, Position},
    config::get_config,
    ipc::zone::{
        ActorControlCategory, BattleNpcSubKind, CharacterDataFlag, CommonSpawn, DisplayFlag,
        ObjectKind, ServerZoneIpcData, ServerZoneIpcSegment, SpawnNpc, StatusEffect,
    },
};
use parking_lot::Mutex;

use crate::{
    FromServer, GameData,
    server::{
        actor::{NetworkedActor, NpcState},
        effect::send_effects_list,
        instance::Instance,
        network::{DestinationNetwork, NetworkState},
        party_bot::{self, BotRole, PartyBot},
    },
};

/// One step of a member's burst: its action, how long after the member's previous action it goes, whether it's on
/// the member's GCD, and whether it's used on the member itself.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Step {
    pub action: u32,
    pub after: f32,
    pub gcd: bool,
    pub on_self: bool,
}

const fn gcd(action: u32, after: f32) -> Step {
    Step {
        action,
        after,
        gcd: true,
        on_self: false,
    }
}

const fn ability(action: u32, after: f32) -> Step {
    Step {
        action,
        after,
        gcd: false,
        on_self: false,
    }
}

const fn on_self(action: u32, after: f32) -> Step {
    Step {
        action,
        after,
        gcd: false,
        on_self: true,
    }
}

/// How a member fights: its GCD actions in turn (`filler`) every `gcd` seconds, and every `burst_every` seconds its
/// burst, step by step. The tank's ranged pull and area combo go through the bots' tank logic (`PartyBot`'s
/// `pull_action` and `aoe_action`); `opener` is a member's first action in each fight.
#[derive(Debug, Clone, Copy)]
pub struct Kit {
    pub gcd: f32,
    pub filler: &'static [u32],
    pub burst_every: f32,
    /// How long into a fight (from its first GCD action) the first burst comes: retail's came 9 s in (the tank, the
    /// Lancer) and 13-19 s (the Thaumaturge), in each fight.
    pub first_burst: f32,
    pub burst: &'static [Step],
    pub opener: Option<u32>,
    pub pull: Option<u32>,
    pub aoe: Option<u32>,
    /// The tank's provoke (an ability, on an enemy it takes off someone else) and its cooldown.
    pub provoke: Option<(u32, f32)>,
    /// A defensive ability on itself with this many enemies on the party, and its cooldown.
    pub defensive: Option<(u32, usize, f32)>,
}

/// A Duty Support member, as retail spawned it and as it fought.
#[derive(Debug, Clone, Copy)]
pub struct Member {
    /// What the client asks for in its Duty Support registration (DawnQuestMember.)
    pub dawn_quest_member: u8,
    pub base_id: u32,
    pub name_id: u32,
    /// The spawn's `pet_id`, as retail sent it.
    pub pet_id: u32,
    pub classjob: u8,
    pub level: u8,
    pub hp: u32,
    /// ActorControlSelf 276's third value and 1525's second for its party slot.
    pub slot_member: u32,
    pub slot_resident: u32,
    /// How much of an enemy's hit on a player it takes. In the capture the same enemies' attacks did a third to the
    /// Marauder of what they did to Bob, a level 17 Conjurer (the median of 12 attacks seen on both: Sastasha's
    /// pirates' 16 against 49, 14/45, 10/32, 15/47..., all 0.27-0.44), and about the same to the Lancer and the
    /// Thaumaturge (33/49, 44/45, 54/45, 9-10/11).
    pub damage_taken: f32,
    /// Statuses it spawned with (the tank's 3124.)
    pub statuses: &'static [u16],
    pub auto_attack: u32,
    /// How close it fights, centre to centre: the members' median distances from what they hit were 3.0 (tank), 3.4
    /// (lancer) and 8.1 yalms (thaumaturge.)
    pub attack_range: f32,
    pub kit: Kit,
    /// Its lines in fights (NpcYell), the first one at its first fight if `opens_with_first`.
    pub chatter: &'static [u32],
    pub opens_with_first: bool,
}

/// Sastasha's three: a Marauder, a Lancer and a Thaumaturge (the run's player was the healer). Party slots in the order
/// the client chose them (36, 38, 37 in its registration: slot 0 the Marauder, 1 the Thaumaturge, 2 the Lancer, each
/// slot's ActorControlSelf 276 and 1525 as listed here.)
pub const MEMBERS: [Member; 3] = [
    Member {
        dawn_quest_member: 36,
        base_id: 14646,
        name_id: 11326,
        pet_id: 1,
        classjob: 3,
        level: 15,
        hp: 411,
        slot_member: 10000,
        slot_resident: 1042776,
        damage_taken: 0.31,
        statuses: &[3124],
        auto_attack: 17058,
        attack_range: 3.0,
        kit: Kit {
            // Two GCD actions in turn every 2.67 s; every 20 s an ability (26230, with 29303 on itself straight
            // after), then its two stronger ones twice at 2.13 s, then 29766 on itself.
            gcd: 2.67,
            filler: &[29299, 29300],
            burst_every: 20.0,
            first_burst: 9.0,
            burst: &[
                ability(26230, 1.07),
                on_self(29303, 0.18),
                gcd(29764, 1.61),
                gcd(29765, 2.13),
                gcd(29764, 2.13),
                gcd(29765, 2.13),
                on_self(29766, 1.07),
            ],
            opener: None,
            // Its first throw at a pack (21 times) and its area combo (34 times, up to 8 hit.)
            pull: Some(29291),
            aoe: Some(29294),
            // 25083 on an enemy turning on someone else, 30 s at least between them; 25082 on itself in big pulls,
            // 108 and 128 s apart.
            provoke: Some((25083, 30.0)),
            defensive: Some((25082, 3, 100.0)),
        },
        // 13951 at its first fight (46 s in), then 13961, 13952 twice; in the Tam-Tara Deepcroft (Bob's capture of
        // 2026-10-24) 13953 three times and 13951.
        chatter: &[13951, 13952, 13961, 13953],
        opens_with_first: true,
    },
    Member {
        dawn_quest_member: 37,
        base_id: 14647,
        name_id: 11327,
        pet_id: 9,
        classjob: 4,
        level: 15,
        hp: 322,
        slot_member: 10001,
        slot_resident: 1042777,
        damage_taken: 1.0,
        statuses: &[],
        auto_attack: 17059,
        attack_range: 3.0,
        kit: Kit {
            // Two GCD actions in turn every 2.67 s; every 11 s an ability (26230), a GCD, 29311 on itself, its
            // strongest (29772, 275 to trash) and 29767 on itself.
            gcd: 2.67,
            filler: &[29305, 29306],
            burst_every: 11.0,
            first_burst: 9.0,
            burst: &[
                ability(26230, 1.07),
                gcd(29305, 1.78),
                on_self(29311, 1.07),
                gcd(29772, 1.73),
                on_self(29767, 1.07),
            ],
            // Each fight starts with 29304 (21 times), as the tank throws.
            opener: Some(29304),
            pull: None,
            aoe: None,
            provoke: None,
            defensive: None,
        },
        chatter: &[13962, 13964],
        opens_with_first: false,
    },
    Member {
        dawn_quest_member: 38,
        base_id: 14648,
        name_id: 11328,
        pet_id: 5,
        classjob: 7,
        level: 15,
        hp: 293,
        slot_member: 10002,
        slot_resident: 1042778,
        damage_taken: 1.0,
        statuses: &[],
        auto_attack: 0,
        attack_range: 8.0,
        kit: Kit {
            // Two spells in turn (1.7 s casts) every 3.07 s, and every third or so its strongest, 29312.
            gcd: 3.07,
            filler: &[29314, 29315],
            burst_every: 9.0,
            first_burst: 13.0,
            burst: &[gcd(29312, 3.07)],
            opener: None,
            pull: None,
            aoe: None,
            provoke: None,
            defensive: None,
        },
        chatter: &[13972, 13974],
        opens_with_first: false,
    },
];

/// The level the members come at in a duty (its ContentFinderCondition) and each one's HP there, by
/// `dawn_quest_member`: retail's Sastasha members were level 15 (Bob's capture of 2026-10-23), the Tam-Tara
/// Deepcroft's the same three at 16 with 443, 347 and 316 HP (his capture of 2026-10-24, 3ec702a8 visit 0.3.) A duty
/// not here takes the member's own `level` and `hp`.
pub const DUTY_LEVELS: [(u16, u8, [(u8, u32); 3]); 2] = [
    (4, 15, [(36, 411), (37, 322), (38, 293)]),
    (2, 16, [(36, 443), (37, 347), (38, 316)]),
];

/// A member's level and HP in the duty `content_finder_condition`.
pub fn level_and_hp(member: &Member, content_finder_condition: u16) -> (u8, u32) {
    DUTY_LEVELS
        .iter()
        .find(|(duty, _, _)| *duty == content_finder_condition)
        .and_then(|(_, level, hps)| {
            hps.iter()
                .find(|(id, _)| *id == member.dawn_quest_member)
                .map(|(_, hp)| (*level, *hp))
        })
        .unwrap_or((member.level, member.hp))
}

/// The recorded member the client means by `dawn_quest_member`, if it's one of them.
pub fn member(dawn_quest_member: u8) -> Option<&'static Member> {
    MEMBERS
        .iter()
        .find(|member| member.dawn_quest_member == dawn_quest_member)
}

/// The members the client chose in its Duty Support registration: one byte each in the word after the DawnContent
/// (`RegisterDutySupport`'s unk1), in party slot order, up to the first 0. Retail's run sent 0x00252624: 36, 38, 37.
/// The words after it aren't members: Bob's client sent 0x6FFF in the third (Kawari read it as members 255 and 111,
/// and a Player NPC joined for them: his bundle of 2026-10-23.)
pub fn chosen_members(words: [u32; 4]) -> Vec<u8> {
    words[0]
        .to_le_bytes()
        .into_iter()
        .take_while(|byte| *byte != 0)
        .collect()
}

/// Lines between fights' chatter: the members spoke every few minutes of fighting (10 lines in 30 minutes.)
const CHATTER_MIN: f32 = 150.0;
const CHATTER_MAX: f32 = 400.0;
/// The tank's first line came 2.7 s into the first fight.
const FIRST_LINE_AFTER: f32 = 2.7;
/// A fight this long after the last action is a new one (the opener again.)
const NEW_FIGHT_AFTER: Duration = Duration::from_secs(8);

/// Where a member is in its kit, and when it next talks.
#[derive(Debug, Clone)]
pub struct MemberState {
    pub member: &'static Member,
    /// Its party slot (the client's order.)
    pub slot: u8,
    filler: usize,
    burst: Option<usize>,
    burst_ready: Option<Instant>,
    last_action: Option<Instant>,
    last_gcd: Option<Instant>,
    opened: bool,
    provoke_ready: Option<Instant>,
    defensive_ready: Option<Instant>,
    next_line: Option<(Instant, u32)>,
    spoke_first: bool,
}

impl MemberState {
    pub fn new(member: &'static Member, slot: u8) -> Self {
        Self {
            member,
            slot,
            filler: 0,
            burst: None,
            burst_ready: None,
            last_action: None,
            last_gcd: None,
            opened: false,
            provoke_ready: None,
            defensive_ready: None,
            next_line: None,
            spoke_first: false,
        }
    }

    fn since(at: Option<Instant>, now: Instant) -> f32 {
        at.map(|at| now.duration_since(at).as_secs_f32())
            .unwrap_or(f32::INFINITY)
    }

    /// Something else used its GCD (the tank's pull or area combo.)
    pub fn used_gcd(&mut self, now: Instant) {
        if self.burst_ready.is_none() {
            self.burst_ready = Some(now + Duration::from_secs_f32(self.member.kit.first_burst));
        }
        self.last_gcd = Some(now);
        self.last_action = Some(now);
        self.opened = true;
    }

    /// Out of combat: the next fight starts with its opener again.
    pub fn out_of_combat(&mut self, now: Instant) {
        if Self::since(self.last_action, now) >= NEW_FIGHT_AFTER.as_secs_f32() {
            self.opened = false;
            self.burst = None;
            self.burst_ready = None;
        }
    }

    /// Its next action, if one is due: `(action, on itself)`. `in_reach` is whether its target is close enough for
    /// its attacks, `in_range` for its opener.
    pub fn choose(&mut self, now: Instant, in_reach: bool, in_range: bool) -> Option<(u32, bool)> {
        let kit = &self.member.kit;
        if !self.opened {
            if let Some(opener) = kit.opener {
                if !in_range {
                    return None;
                }
                self.opened = true;
                self.used_gcd(now);
                return Some((opener, false));
            }
            self.opened = true;
        }

        // A burst under way goes on step by step.
        if self.burst.is_none()
            && !kit.burst.is_empty()
            && in_reach
            && self.burst_ready.is_some_and(|ready| now >= ready)
            && Self::since(self.last_action, now) >= kit.burst[0].after
        {
            self.burst = Some(0);
            self.burst_ready = Some(now + Duration::from_secs_f32(kit.burst_every));
        }
        if let Some(index) = self.burst {
            let step = kit.burst[index];
            if Self::since(self.last_action, now) < step.after || !(in_reach || step.on_self) {
                return None;
            }
            self.burst = (index + 1 < kit.burst.len()).then_some(index + 1);
            self.last_action = Some(now);
            if step.gcd {
                self.last_gcd = Some(now);
            }
            return Some((step.action, step.on_self));
        }

        if in_reach && !kit.filler.is_empty() && Self::since(self.last_gcd, now) >= kit.gcd {
            let action = kit.filler[self.filler % kit.filler.len()];
            self.filler += 1;
            self.used_gcd(now);
            return Some((action, false));
        }
        None
    }

    /// The tank's provoke on an enemy it's taking off someone else, if it's ready.
    pub fn provoke(&mut self, now: Instant) -> Option<u32> {
        let (action, cooldown) = self.member.kit.provoke?;
        if self.provoke_ready.is_some_and(|ready| now < ready) {
            return None;
        }
        self.provoke_ready = Some(now + Duration::from_secs_f32(cooldown));
        self.last_action = Some(now);
        Some(action)
    }

    /// Its defensive ability on itself, with `enemies` fighting the party, if it's ready.
    pub fn defensive(&mut self, now: Instant, enemies: usize) -> Option<u32> {
        let (action, needs, cooldown) = self.member.kit.defensive?;
        if enemies < needs || self.defensive_ready.is_some_and(|ready| now < ready) {
            return None;
        }
        // Between GCD actions, as retail's came (1.07 s after the area combo.)
        if Self::since(self.last_action, now) < 1.0 {
            return None;
        }
        self.defensive_ready = Some(now + Duration::from_secs_f32(cooldown));
        self.last_action = Some(now);
        Some(action)
    }

    /// The line it says now, while the party fights, if it's time.
    pub fn chatter(&mut self, now: Instant) -> Option<u32> {
        let lines = self.member.chatter;
        if lines.is_empty() {
            return None;
        }
        let Some((at, line)) = self.next_line else {
            let (after, line) = if self.member.opens_with_first && !self.spoke_first {
                (FIRST_LINE_AFTER, lines[0])
            } else {
                (
                    CHATTER_MIN + fastrand::f32() * (CHATTER_MAX - CHATTER_MIN),
                    lines[fastrand::usize(..lines.len())],
                )
            };
            self.next_line = Some((now + Duration::from_secs_f32(after), line));
            return None;
        };
        if now < at {
            return None;
        }
        self.spoke_first = true;
        let after = CHATTER_MIN + fastrand::f32() * (CHATTER_MAX - CHATTER_MIN);
        self.next_line = Some((
            now + Duration::from_secs_f32(after),
            lines[fastrand::usize(..lines.len())],
        ));
        Some(line)
    }
}

/// The party role a director names a speaker by: "tank", "healer", "melee" or "ranged" (the ClassJob sheet's Role.)
fn role_name(role_column: u8) -> &'static str {
    match role_column {
        1 => "tank",
        2 => "melee",
        3 => "ranged",
        4 => "healer",
        _ => "",
    }
}

/// A story line the director queued for whoever plays `role` in the party (`director:party_npc_yell`).
#[derive(Debug, Clone)]
pub struct QueuedYell {
    pub at: Instant,
    pub role: String,
    pub yell: u32,
}

/// Queues a story line for the party NPC playing `role`, `delay` seconds from now. Nobody of that role: it's dropped.
pub fn queue_yell(instance: &mut Instance, role: &str, yell: u32, delay: f32) {
    instance.party_npc_yells.push(QueuedYell {
        at: Instant::now() + Duration::from_secs_f32(delay.max(0.0)),
        role: role.to_lowercase(),
        yell,
    });
}

/// Has a party NPC yell a line (NpcYell): its speech bubble and the line in chat, under its BNpcName as retail sent it.
pub fn yell(
    network: &Arc<Mutex<NetworkState>>,
    instance: &Instance,
    actor_id: ObjectId,
    yell: u32,
) {
    let Some(state) = instance
        .party_bots
        .get(&actor_id)
        .and_then(|bot| bot.party_npc.as_ref())
    else {
        return;
    };
    let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::NpcYell {
        object_id: ObjectTypeId {
            object_id: actor_id,
            object_type: ObjectTypeKind::None,
        },
        name_id: state.member.name_id,
        npc_yell_id: yell,
        param1: 0,
        param2: 0,
        param3: 0,
        param4: 0,
    });
    network.lock().send_to_instance(
        ObjectId::default(),
        instance,
        FromServer::PacketSegment(ipc, actor_id),
        DestinationNetwork::ZoneClients,
    );
}

/// The story lines that are due: each goes to the living party NPC of its role.
pub fn update_yells(
    network: &Arc<Mutex<NetworkState>>,
    game_data: &Arc<Mutex<GameData>>,
    instance: &mut Instance,
    now: Instant,
) {
    if instance.party_npc_yells.is_empty() {
        return;
    }
    let (due, later): (Vec<QueuedYell>, Vec<QueuedYell>) =
        std::mem::take(&mut instance.party_npc_yells)
            .into_iter()
            .partition(|queued| queued.at <= now);
    instance.party_npc_yells = later;
    for queued in due {
        let speaker = instance
            .party_bots
            .iter()
            .filter_map(|(id, bot)| bot.party_npc.as_ref().map(|state| (*id, state)))
            .find(|(id, state)| {
                let role = game_data
                    .lock()
                    .classjob_role(state.member.classjob)
                    .map(|(role, _, _)| role)
                    .unwrap_or_default();
                role_name(role) == queued.role
                    && instance
                        .find_actor(*id)
                        .is_some_and(|actor| actor.get_common_spawn().health_points > 0)
            })
            .map(|(id, _)| id);
        if let Some(speaker) = speaker {
            yell(network, instance, speaker, queued.yell);
        }
    }
}

/// Spawns `member` for the player `leader` in their instance, in party slot `slot`. Returns its actor ID.
fn spawn_member(
    instance: &mut Instance,
    game_data: &mut GameData,
    leader: ObjectId,
    member: &'static Member,
    slot: u8,
) -> Option<ObjectId> {
    let (_, _, look, _, npc_equip, behavior) = game_data.find_bnpc(member.base_id)?;
    let gear = game_data
        .get_npc_equip(npc_equip as u32)
        .unwrap_or_default();
    let (role_column, _, _) = game_data.classjob_role(member.classjob)?;
    let (level, hp) = level_and_hp(member, instance.content_finder_condition_id);

    // Its stats: a player's of its class, race and level in the best gear it could wear then, with retail's HP.
    let wearable = game_data.wearable_restrictions(look.race, look.gender);
    let items: Vec<u32> = game_data
        .best_gear(member.classjob, level, &wearable)
        .iter()
        .map(|(_, id)| *id)
        .collect();
    let mut parameters = party_bot::bot_parameters(
        game_data,
        member.classjob,
        look.tribe as u8,
        level,
        &items,
        None,
    )?;
    parameters.hp = hp;

    let leader_actor = instance.find_actor(leader)?;
    let (position, facing) = (leader_actor.position().0, leader_actor.rotation());

    let mut status_effects = [StatusEffect::default(); 30];
    for (index, status) in member.statuses.iter().enumerate().take(30) {
        status_effects[index] = StatusEffect {
            effect_id: *status,
            ..Default::default()
        };
    }
    let spawn = SpawnNpc {
        common: CommonSpawn {
            base_id: member.base_id,
            name_id: member.name_id,
            pet_id: member.pet_id,
            handler_id: instance
                .find_actor(leader)
                .map(|actor| actor.get_common_spawn().handler_id)
                .unwrap_or_default(),
            owner_id: leader,
            max_health_points: hp,
            health_points: hp,
            // Retail's: 3,000 MP, behavior 634 (the BNpcBase's), display flags 0x40000.
            max_resource_points: 3000,
            resource_points: 3000,
            behavior,
            display_flags: DisplayFlag::UNK1,
            object_kind: ObjectKind::BattleNpc(BattleNpcSubKind::NpcPartyMember),
            battalion: 0,
            model_chara: 0,
            level,
            class_job: member.classjob,
            position: Position(position),
            rotation: facing,
            look,
            main_weapon_model: gear.main_weapon_model,
            sec_weapon_model: gear.sec_weapon_model,
            models: gear.models,
            glasses_ids: gear.glasses_ids,
            status_effects,
            ..Default::default()
        },
        // Retail sent 0x2 (HOSTILE) for them, which Kawari reads as an enemy's; left out, as for the bots shown as
        // party members. Its link range was 20, as other NPCs'.
        character_data_flags: CharacterDataFlag::empty(),
        normal_scaling: true,
        link_range: 20,
        ..Default::default()
    };

    let actor_id = Instance::generate_actor_id();
    let config = get_config();
    // Its timeline (`resources/timelines/*_<base>.json`) is its auto-attack, hitbox and what its actions did on retail.
    instance.insert_npc(actor_id, spawn, &config, game_data);
    if let Some(NetworkedActor::Npc {
        timeline,
        parameters: npc_parameters,
        state,
        ..
    }) = instance.find_actor_mut(actor_id)
    {
        timeline.autoattack_action_id = member.auto_attack;
        timeline.attack_range = member.attack_range;
        timeline.rotation = None;
        *npc_parameters = parameters;
        *state = NpcState::Follow;
    }

    let order = instance
        .party_bots
        .values()
        .filter(|bot| bot.leader == leader)
        .map(|bot| bot.order + 1)
        .max()
        .unwrap_or(0);
    let mut bot = PartyBot::new(
        leader,
        BotRole::from_classjob_role(role_column),
        member.attack_range,
        order,
    );
    bot.pull_action = member.kit.pull;
    bot.aoe_action = member.kit.aoe;
    bot.party_npc = Some(MemberState::new(member, slot));
    instance.party_bots.insert(actor_id, bot);
    tracing::info!(
        "Party NPC {} (BNpcBase {}, class {}, level {}) joined {leader} in slot {slot} as {actor_id}",
        member.name_id,
        member.base_id,
        member.classjob,
        level
    );
    Some(actor_id)
}

/// Spawns the recorded members a player who registered through Duty Support chose, in their instance. Returns the
/// party NPCs' actor IDs and the roles they play (ClassJob Role), so party bots can take the rest (members nobody has
/// recorded yet.)
pub fn fill_duty_support(
    instance: &mut Instance,
    game_data: &mut GameData,
    leader: ObjectId,
    chosen: &[u8],
) -> (Vec<ObjectId>, Vec<u8>) {
    let mut spawned = Vec::new();
    let mut roles = Vec::new();
    let mut unknown = Vec::new();
    for (slot, wanted) in chosen.iter().enumerate() {
        let Some(member) = member(*wanted) else {
            unknown.push(*wanted);
            continue;
        };
        if let Some(actor_id) = spawn_member(instance, game_data, leader, member, slot as u8) {
            spawned.push(actor_id);
            if let Some((role, _, _)) = game_data.classjob_role(member.classjob) {
                roles.push(role);
            }
        }
    }
    if !unknown.is_empty() {
        tracing::warn!(
            "Duty Support: no recording of members {unknown:?} yet; party bots take their roles"
        );
    }
    (spawned, roles)
}

/// Tells the leader's client about its party NPCs as retail did on entering (per slot, last first: ActorControlSelf
/// 257 with the slot and the member, 276 with the slot and its member number, 1525 with the slot and its ENpcResident),
/// then sends each member's StatusEffectList, from which the client's party list shows its class, level and HP.
pub fn announce(network: &Arc<Mutex<NetworkState>>, instance: &Instance, leader: ObjectId) {
    let mut members: Vec<(ObjectId, &MemberState)> = instance
        .party_bots
        .iter()
        .filter(|(_, bot)| bot.leader == leader)
        .filter_map(|(id, bot)| bot.party_npc.as_ref().map(|state| (*id, state)))
        .collect();
    if members.is_empty() {
        return;
    }
    members.sort_by_key(|(_, state)| std::cmp::Reverse(state.slot));
    let send = |category: ActorControlCategory| {
        network.lock().send_to_by_actor_id(
            leader,
            FromServer::ActorControlSelf(category),
            DestinationNetwork::ZoneClients,
        );
    };
    // Retail's first: no pet (slot 0, nobody, 1, 1.)
    send(ActorControlCategory::SetupPet {
        owner_id: leader,
        pet_id: 0,
        pet_actor_id: ObjectId::default(),
        unk2: 1,
        unk3: 1,
    });
    for (actor_id, state) in &members {
        send(ActorControlCategory::SetupPet {
            owner_id: leader,
            pet_id: state.slot as u32,
            pet_actor_id: *actor_id,
            unk2: 1,
            unk3: 2,
        });
        send(ActorControlCategory::Unknown {
            category: 276,
            param1: state.slot as u32,
            param2: 1,
            param3: state.member.slot_member,
            param4: 0,
            param5: 0,
        });
        send(ActorControlCategory::Unknown {
            category: 1525,
            param1: state.slot as u32,
            param2: state.member.slot_resident,
            param3: 0,
            param4: 0,
            param5: 0,
        });
    }
    for (actor_id, _) in &members {
        send_effects_list(network.clone(), instance, *actor_id);
    }
}

/// The duty was cleared: retail emptied the party slots (ActorControlSelf 257 with each slot and nobody, 1, 2) and
/// took the members away (DeleteActor, 0.3 s later.)
pub fn dismiss(network: &mut NetworkState, instance: &mut Instance) {
    let members: Vec<(ObjectId, ObjectId, u8)> = instance
        .party_bots
        .iter()
        .filter_map(|(id, bot)| {
            bot.party_npc
                .as_ref()
                .map(|state| (*id, bot.leader, state.slot))
        })
        .collect();
    for (actor_id, leader, slot) in members {
        network.send_to_by_actor_id(
            leader,
            FromServer::ActorControlSelf(ActorControlCategory::SetupPet {
                owner_id: leader,
                pet_id: slot as u32,
                pet_actor_id: ObjectId::default(),
                unk2: 1,
                unk3: 2,
            }),
            DestinationNetwork::ZoneClients,
        );
        instance.party_bots.remove(&actor_id);
        network.remove_actor(instance, actor_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retails_registration() {
        // Bob's capture: RegisterDutySupport's first word after the DawnContent was 0x00252624.
        assert_eq!(chosen_members([0x0025_2624, 0, 0, 0]), vec![36, 38, 37]);
        // Bob's Kawari client (his bundle of 2026-10-23): something else in the third word.
        assert_eq!(
            chosen_members([0x0025_2624, 0, 0x6FFF, 0]),
            vec![36, 38, 37]
        );
        assert_eq!(member(36).map(|m| m.base_id), Some(14646));
        assert!(member(39).is_none());
    }

    #[test]
    fn the_tank_fights_as_retails() {
        let tank = member(36).unwrap();
        let mut state = MemberState::new(tank, 0);
        let start = Instant::now();
        let mut used = Vec::new();
        // Ticks of half a second, as the server's.
        for tick in 0..60 {
            let now = start + Duration::from_millis(tick * 500);
            if let Some((action, _)) = state.choose(now, true, true) {
                used.push(action);
            }
        }
        // Its two GCD actions in turn, then its burst once it's had a GCD and its cooldown allows.
        assert_eq!(&used[..2], &[29299, 29300]);
        assert!(used.contains(&26230));
        let burst = used.iter().position(|a| *a == 26230).unwrap();
        assert_eq!(
            &used[burst..burst + 7],
            &[26230, 29303, 29764, 29765, 29764, 29765, 29766]
        );
    }

    #[test]
    fn the_lancer_opens_then_alternates() {
        let lancer = member(37).unwrap();
        let mut state = MemberState::new(lancer, 2);
        let now = Instant::now();
        assert_eq!(state.choose(now, false, true), Some((29304, false)));
        let later = now + Duration::from_secs(3);
        assert_eq!(
            state.choose(later, true, true).map(|(a, _)| a).is_some(),
            true
        );
    }

    #[test]
    fn the_first_line_comes_at_the_first_fight() {
        let tank = member(36).unwrap();
        let mut state = MemberState::new(tank, 0);
        let now = Instant::now();
        assert_eq!(state.chatter(now), None);
        assert_eq!(state.chatter(now + Duration::from_secs(3)), Some(13951));
    }

    #[test]
    fn each_duty_has_its_level() {
        // Sastasha (ContentFinderCondition 4) at 15, the Tam-Tara Deepcroft (2) at 16, as retail spawned them.
        let tank = member(36).unwrap();
        assert_eq!(level_and_hp(tank, 4), (15, 411));
        assert_eq!(level_and_hp(tank, 2), (16, 443));
        assert_eq!(level_and_hp(member(38).unwrap(), 2), (16, 316));
        assert_eq!(level_and_hp(tank, 9999), (tank.level, tank.hp));
    }
}
