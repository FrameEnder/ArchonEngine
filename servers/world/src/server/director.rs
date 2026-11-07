// TODO: Restore DirectorTrigger handling, I removed it after the refactor...

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use kawari::{
    common::{
        ContentDirectorEvent, DirectorEvent, DirectorTrigger, DutyOption, EOBJ_SHORTCUT,
        EventState, GimmickAccessorType, GimmickRectEvent, HandlerId, HandlerType,
        InstanceContentDirectorEvent, LeveDirectorEvent, LogMessageType, ObjectId, ObjectTypeId,
        ObjectTypeKind, Position, PublicContentType, TerritoryIntendedUse, WARP_DELAY, WarpType,
        timestamp_secs,
    },
    config::get_config,
    ipc::chat::ChatChannelType,
    ipc::zone::{
        ActorControlCategory, ActorControlSelf, ActorSetPos, ChatMessage, DisplayFlag,
        DutyFinderSetting, EventLogMessage, MapEffects, ObjectKind, PrepareZoning,
        ServerZoneIpcData, ServerZoneIpcSegment, SpawnObject, TreasureKind, WalkInEvent,
        WeatherChange,
    },
};
use mlua::{Function, LuaSerdeExt, UserData, UserDataMethods, Value};
use parking_lot::Mutex;

use crate::{
    ClientId, FromServer, GameData, QUEST_BATTLE_SCENE_FINISH, QUEST_BATTLE_SCENE_FINISH_FLAGS,
    ToServer,
    lua::KawariLua,
    server::{
        WorldServer,
        actor::NetworkedActor,
        effect::{gain_effect_instance, lose_effect_instance},
        instance::{Instance, QueuedTaskData},
        network::{DestinationNetwork, NetworkState},
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum LuaDirectorTask {
    HideEObj {
        base_id: u32,
    },
    ShowEObj {
        base_id: u32,
    },
    DeleteEObj {
        base_id: u32,
    },
    /// Spawn an event object of the zone's layout (moved to `position`), or, when the layout has none, one made from
    /// its EObj row alone, as retail spawned Haukke Manor's dropped keys (layout ID 0, the director's handler; Bob's
    /// Chronofoil captures of 2026-11-02.) `args2` is SpawnObject's, as retail sent it (EObj-specific.)
    SpawnEObj {
        base_id: u32,
        position: Option<Position>,
        args2: Option<u32>,
    },
    SendVariables,
    AbandonDuty {
        actor_id: ObjectId,
    },
    BeginEventAction {
        actor_id: ObjectId,
        target: ObjectId,
        action_id: u32,
    },
    FinishGimmickEvent {
        actor_id: ObjectId,
    },
    LogMessage {
        id: u32,
        params: Vec<u32>,
    },
    /// Spawn a BNpc by layout ID as this actor (`spawn_bnpc` returns it, so a script can tell apart several spawned
    /// from one layout ID, like the Copperbell Mines' three helpers sharing 9054923.)
    SpawnBattleNpc {
        id: u32,
        actor_id: ObjectId,
        /// Where, if not its layout's spot (and its facing, if given): the Thousand Maws of Toto-Rak's last boss put its
        /// adds (one layout ID for each kind) wherever it called them.
        at: Option<(Position, Option<f32>)>,
    },
    /// Spawn a BNpc (by layout ID) tethered to another (by layout ID): the line the client draws between them, a
    /// Channeling row, comes with its spawn and goes when it dies (retail: Tam-Tara's shields to Galvanth.)
    SpawnTetheredBattleNpc {
        id: u32,
        to: u32,
        channeling: u16,
    },
    /// Remove a BNpc (by layout ID) without it dying, e.g. the background fighters before a quest battle's cutscene.
    DespawnBattleNpc {
        layout_id: u32,
    },
    GainEffect {
        actor_id: ObjectId,
        id: u16,
        param: u16,
        duration: f32,
    },
    /// The status `id` on an actor ends now (`effect::lose_effect_instance`): the Navel's Granite Gaol (292) as the
    /// gaol holding them breaks.
    LoseEffect {
        actor_id: ObjectId,
        id: u16,
    },
    /// A BNpc (actor ID or layout ID) kept `distance` yalms in front of another, facing as it faces, wherever that one
    /// goes (`Instance::attached_npcs`): the Navel's Heart of the Mountain at Titan's chest.
    AttachBattleNpc {
        who: u32,
        to: u32,
        distance: f32,
    },
    SetBGM {
        id: u32,
    },
    SealBossWall {
        actor_id: ObjectId,
        id: u32,
        place_name: u32,
        time_until: u32,
    },
    SpawnBoss {
        bnpc_id: u32,
        wall_id: u32,
        line_id: u32,
        place_name: u32,
        /// The layout's range (an EventRange or MapRange's layout ID, 0 for none) the room's floor is: see
        /// `DirectorBoss::arena`.
        arena: u32,
    },
    /// A boss arena whose boss is spawned some other way (or later): sealed `wall_id` 15 s after the boss or one of
    /// `triggers` (layout IDs) goes for someone, unsealed when the boss dies. The Tam-Tara Deepcroft's rooms seal as
    /// their pack is pulled and open when the boss that pack calls dies.
    BossRoom {
        bnpc_id: u32,
        wall_id: u32,
        line_id: u32,
        place_name: u32,
        triggers: Vec<u32>,
        arena: u32,
    },
    /// Whether a BNpc takes damage (`director:set_invulnerable`.)
    SetInvulnerable {
        layout_id: u32,
        invulnerable: bool,
    },
    /// What a coffer held on retail, one list of items per recorded opening (`server/loot.rs`.)
    TreasureLoot {
        base_id: u32,
        openings: Vec<Vec<u32>>,
    },
    /// What a coffer may hold besides what was recorded: the wiki's list for it (`director:treasure_pool`.)
    TreasurePool {
        base_id: u32,
        items: Vec<u32>,
    },
    SpawnTreasure {
        id: u32,
    },
    VariantVoteRoute {
        npc_route: u32,
    },
    /// Play a cutscene (scene 3 with its ID), with the scene's parameters after the ID: `DEFAULT_CUTSCENE_PARAMS`
    /// unless the script gives its own.
    PlayCutscene {
        cutscene_id: u32,
        params: Vec<u32>,
    },
    UpdateShortcut {
        poprange_id: u32,
    },
    /// Send this player through a shortcut: to `pop_range_id`, or (without one) to where the duty's shortcut
    /// points now (`update_shortcut`).
    UseShortcut {
        actor_id: ObjectId,
        pop_range_id: Option<u32>,
    },
    CompleteDuty,
    /// Win a quest battle, which also moves its quest to this sequence. "Duty Complete" shows straight away; unless
    /// `stay`, the players are then sent back, otherwise they stay until `LeaveQuestBattle`.
    CompleteQuestBattle {
        sequence: u8,
        stay: bool,
        /// The scene played before they're sent back, see `QUEST_BATTLE_SCENE_FINISH`.
        finish_scene: u16,
        /// Its scene flags, see `QUEST_BATTLE_SCENE_FINISH_FLAGS`.
        finish_flags: u32,
        /// Quest variables (index 0-5, value) set in the same journal update as the sequence (Lurkers in the Grotto's
        /// ITEM0, UI8BH, as retail sets it on winning.)
        variables: Vec<(u8, u8)>,
        /// Whether "Duty Complete" and its fanfare (`QUEST_BATTLE_SCENE_COMPLETE`) play before the finish scene.
        /// Retail skips them when the battle ends on a timer into a cutscene (Lord of the Inferno's: scene 7 at once.)
        fanfare: bool,
    },
    /// Play one of the director's scenes for every player (e.g. a cutscene halfway through a quest battle), with
    /// these scene flags. The script's `onSceneFinished(director, scene)` is called when a player finishes it.
    PlayScene {
        scene: u16,
        flags: u32,
    },
    /// The fight is over: every NPC stops fighting (`npc_behavior::end_fight`.)
    EndFight,
    /// Send the players of a won quest battle back (see `CompleteQuestBattle`'s `stay`.)
    LeaveQuestBattle,
    /// Move every player to a position in the same instance through a loading screen, as retail does after To Guard
    /// a Guardian's cutscene (PrepareZoning to the same territory, then ActorSetPos; the client then finishes zoning
    /// and gets ZoneIn). A cutscene that ends this way leaves the client waiting for it.
    WarpPlayers {
        position: Position,
        rotation: f32,
    },
    /// Lose a quest battle, which sends the players back to where they entered it.
    FailQuestBattle,
    /// Make a BNpc use an action on another BNpc (by layout ID), or on the player if there's no target.
    UseAction {
        layout_id: u32,
        action_id: u32,
        target: Option<u32>,
    },
    /// A BNpc says a line of quest text (by its key) in /say, e.g. an ally's battle chatter.
    Say {
        layout_id: u32,
        key: String,
    },
    /// Someone says the quest battle's quest's `number`th battle talk line (1-based, in key order), see
    /// `GameData::get_quest_battle_talk`. It comes from the BNpc at `layout_id`, under the name `name_id` (BNpcName) if
    /// given, for speakers that aren't there, like the masked mage.
    SayBattleTalk {
        layout_id: u32,
        name_id: Option<u32>,
        number: usize,
    },
    /// Shows the battle talk window (DirectorPopupMessage), as retail does in quest battles: `text_id` is the
    /// content's text (InstanceContentTextData; one outside the content's range crashes the client, so use recorded
    /// IDs), `name_id` the speaker's BNpcName and `icon` the portrait. It comes from the BNpc at `layout_id` if
    /// it's there. `kind` is the speaker's ObjectKind: 2 (BattleNpc, `name_id` a BNpcName) or 3 (EventNpc, `name_id`
    /// an ENpcResident); the client crashes on a name that isn't in the sheet `kind` points it to.
    BattleTalk {
        layout_id: u32,
        name_id: u32,
        text_id: u32,
        icon: u32,
        duration_ms: u32,
        kind: u8,
    },
    /// Plays an animation (ActionTimeline) on a BNpc, as retail's Sastasha captain does as he calls his men through the
    /// gate (980.)
    PlayActionTimeline {
        layout_id: u32,
        timeline_id: u32,
    },
    /// Has the party NPC (Duty Support member) of a role ("tank", "healer", "melee", "ranged") yell a line from the
    /// NpcYell sheet, `delay` seconds from now: the duty's story lines, at the moments retail's members said them.
    /// `otherwise`: the role and line said instead when no party NPC plays `role` (the player does.)
    PartyNpcYell {
        role: String,
        npc_yell_id: u32,
        delay: f32,
        otherwise: Option<(String, u32)>,
    },
    /// Has an NPC yell a line from the NpcYell sheet: a speech bubble over them, and the line in chat.
    Yell {
        layout_id: u32,
        npc_yell_id: u32,
    },
    /// Change the director's sequence, which the client's script for it reacts to (e.g. `OnSequence001`.)
    SetSequence {
        sequence: u8,
    },
    /// Whether an aggressive BNpc notices whoever comes close (the default) or only fights when hit or sent after
    /// someone.
    SetNotice {
        layout_id: u32,
        notices: bool,
    },
    /// How close (in yalms) a BNpc gets to its target before attacking: melee (3) or a caster's range (15.)
    SetAttackRange {
        layout_id: u32,
        yalms: f32,
    },
    /// Keep a BNpc within `yalms` of every one of its anchors (BNpcs by layout ID; `None` is the player): it heads back
    /// to the middle of them when it's further from any, whatever it's fighting (a support near the tank and the
    /// player.) No anchors or no distance stops it.
    KeepNear {
        layout_id: u32,
        anchors: Vec<Option<u32>>,
        yalms: Option<f32>,
    },
    /// Have a BNpc stand behind the enemy another BNpc (the tank) is fighting, `yalms` beyond it: out of its frontal
    /// cones and cleaves, where a support belongs. `None` stops it.
    StandBehind {
        layout_id: u32,
        tank: u32,
        yalms: Option<f32>,
    },
    /// Make a BNpc hate another BNpc (by layout ID), or the player if there's no target, by this much enmity.
    SetHate {
        layout_id: u32,
        target: Option<u32>,
        amount: u32,
    },
    /// A map effect's state and timeline (DirectorMapEffect): `state` 1 for most; Brayflox's Longstop's puddles went
    /// to 4 with timeline 8 as they burst (Bob's Chronofoil capture 44df02fc of 2026-11-03.)
    MapEffect {
        index: u32,
        timeline_id: u32,
        state: u16,
    },
    /// The same, for the map effect on a layout object (`layout_id`, e.g. a SharedGroup of the duty's LGB): its index
    /// is where that ID stands in the InstanceContent's ContentDirectorManagedSG list (`GameData::get_map_effects`,
    /// the list retail's DirectorSetupMapEffects follows), or `otherwise` when it isn't in it.
    MapEffectAt {
        layout_id: u32,
        timeline_id: u32,
        state: u16,
        otherwise: Option<u32>,
    },
    /// Send an EObj (by EObj ID or layout ID) an ActorControl by its category and parameters, for the ones Kawari has
    /// no name for: the Copperbell Mines' blasts (413 with 8, 2 on the charge as it goes off, Bob's captures of
    /// 2026-10-25.)
    EObjControl {
        id: u32,
        category: u32,
        params: [u32; 4],
    },
    /// Fade a BNpc (actor ID or layout ID) out, as retail does before taking away one that leaves without dying
    /// (ActorControl `FadeOut`, 607); the script despawns it a moment later (`despawn_bnpc`.)
    FadeOutBattleNpc {
        who: u32,
    },
    /// A BNpc (actor ID or layout ID) gives up the fight and runs back to where it appeared, paying no attention to
    /// anyone (as an enemy leashed in the open world does; it keeps its health): Brotherly Love's beast as its friends
    /// fell (Bob's Chronofoil capture 5fcc970c.) With `to`, it runs there instead (its home from then on): Haukke
    /// Manor's eye to the last room's door. A cast it's in the middle of is cancelled (CancelCast 537, as retail
    /// cancelled the eye's 514.)
    RetreatBattleNpc {
        who: u32,
        to: Option<glam::Vec3A>,
    },
    /// Make a BNpc (actor ID or layout ID) targetable or not (ActorControl `SetTargetable`, 54, to everyone around):
    /// the Thousand Maws of Toto-Rak's room bosses wait in their webs untargetable until their room's guards are down.
    SetTargetable {
        who: u32,
        targetable: bool,
    },
    /// Send a BNpc (actor ID or layout ID) an ActorControl by its category and parameters, for the ones Kawari has no
    /// name for (as `EObjControl`): the Thousand Maws of Toto-Rak's last boss's 30 and 31 as its tail comes off.
    BattleNpcControl {
        who: u32,
        category: u32,
        params: [u32; 4],
    },
    /// A BNpc (actor ID or layout ID) dies where it stands, as if its health ran out (`actor::kill_actor`): the
    /// Thousand Maws of Toto-Rak's last boss's eggs, gone with their own blast.
    KillBattleNpc {
        who: u32,
    },
    /// The duty's objectives (ContentDirector 0x80000016 and 0x80000015): `done` is done, and `next` is the one now, if
    /// there's another.
    Objective {
        done: u32,
        next: Option<u32>,
    },
    /// The weather everyone in the instance sees now (WeatherId), fading in over `fade` seconds; `forced` as retail's
    /// second byte (1 for the Thousand Maws of Toto-Rak's last fight's, 0 as it went back.)
    SetWeather {
        id: u8,
        fade: f32,
        forced: bool,
    },
    /// Show the duty's exits (every event object in the zone whose gimmick is the duty exit, `Zone::exit_eobjs`.)
    ShowExits,
    /// Bring a BNpc spawned hidden (its display flags have INVISIBLE, as retail's spawn had) into view `delay` seconds
    /// from now: ActorControl `Appear` (a fade-in, and `action_timeline` played on it) then `SetTargetable`, as retail
    /// did for the Copperbell Mines' ambushing gigas (158, 0.6-0.9 s after their spawn; Bob's captures of 2026-10-25.)
    AppearBattleNpc {
        layout_id: u32,
        action_timeline: u32,
        delay: f32,
    },
    /// Put a BNpc (by actor ID, or layout ID) somewhere at once: an ActorSetPos from it to everyone, as retail moves
    /// its invisible helpers before they cast (the Copperbell Mines' first boss's three markers.)
    WarpBattleNpc {
        who: u32,
        position: Position,
        rotation: f32,
    },
    /// A BNpc (by actor ID, or layout ID) jumps somewhere: ActorControl `SnapNextMove` then an ActorMove there, as
    /// retail's Kottos leapt onto each marked spot in turn (`npc_behavior` picks up from where it lands.)
    JumpBattleNpc {
        who: u32,
        position: Position,
        /// Facing as it lands; `None` faces the way it leapt (retail's Kottos did, on all three of each round.)
        rotation: Option<f32>,
    },
    /// A BNpc (by actor ID, or layout ID) runs somewhere: one ActorMove there, which the client walks it along, as
    /// retail's last Copperbell boss went to the arena's middle for 28760. `speed` is the ActorMove's speed byte
    /// (`JUMP_MOVE_SPEED` unless given; retail's walk is 20, its run 60.)
    MoveBattleNpc {
        who: u32,
        position: Position,
        rotation: f32,
        speed: Option<u8>,
    },
    /// Make a BNpc (by actor ID) use an action on another actor (by actor ID), or on itself if there's none.
    ActorUseAction {
        actor_id: u32,
        action_id: u32,
        target: Option<u32>,
    },
    /// Draw a BNpc (by actor ID, or layout ID) at another ModelScale row (ActorControl `SetModelScale`), as retail grew
    /// the Copperbell Mines' second boss each round.
    SetModelScale {
        who: u32,
        model_scale_id: u32,
    },
    /// Let a helper go as its boss falls: ActorControl 417 with no one (0xE0000000) from it, as retail sent each of
    /// Kottos' and the last boss's helpers the moment their boss died.
    ReleaseBattleNpc {
        who: u32,
    },
    /// Hold a BNpc (by actor ID, or layout ID): `npc_behavior` leaves it alone, so it does only what the director has it
    /// do, as retail's boss helpers (`Instance::held_npcs`); `false` lets it go back to its own ways.
    /// Keep a BNpc (actor ID or layout ID) busy for `seconds`: it doesn't move, turn or act on its own (no
    /// auto-attacks), but its timeline goes on counting, as retail's Kottos did nothing of its own from 28543 to its
    /// third landing (`Instance::busy_npcs`.)
    BusyBattleNpc {
        who: u32,
        seconds: f32,
    },
    /// Keep a BNpc (actor ID or layout ID) where it stands for `seconds`: it doesn't move, but acts as ever (its
    /// timeline, its attacks), as retail's Lady Amandine stood 4.9 s beside her eye where she'd leapt, using her 28643
    /// there (`Instance::rooted_npcs`.)
    RootBattleNpc {
        who: u32,
        seconds: f32,
    },
    /// Starts a `from_pull` timeline's clock again from now for its points `after` this (an action's ID, as a point
    /// with `restarts` does, or any number the timeline's points use for a phase): a phase the director begins
    /// (Ifrit's at 80 % and 50 %, and after his Hellfire, in the Bowl of Embers.)
    RestartTimeline {
        who: u32,
        anchor: u32,
    },
    HoldBattleNpc {
        who: u32,
        held: bool,
    },
    /// A task for later (`director:delay`): it runs when its time comes, in a later tick.
    Later {
        at: Instant,
        task: Box<LuaDirectorTask>,
    },
}

/// A BNpc by actor ID, or failing that by layout ID.
fn resolve_bnpc(instance: &Instance, who: u32) -> Option<ObjectId> {
    match instance.find_actor(ObjectId(who)) {
        Some(NetworkedActor::Npc { .. }) => Some(ObjectId(who)),
        _ => instance.find_npc(who).map(|(id, _)| id),
    }
}

/// Retail's speed byte on the ActorMove that lands a jump (Kottos', Bob's Copperbell Mines captures of 2026-10-25.)
const JUMP_MOVE_SPEED: u8 = 90;
/// CancelCast's first parameter for a cast the director called off (`retreat_bnpc`): retail's for Haukke Manor's eye,
/// its 514 cut short as it fled (Bob's Chronofoil capture 5d9c7d7f, the Duty Support run.)
const RETREAT_CANCEL: u32 = 537;
/// The line retail's director logged as a player left a duty by its exit, with the InstanceContent's ID.
const LOG_DUTY_EXIT: u32 = 1534;

/// A BNpc to bring into view (`LuaDirectorTask::AppearBattleNpc`.)
#[derive(Debug, Clone, Copy)]
pub struct PendingAppearance {
    pub at: std::time::Instant,
    pub layout_id: u32,
    pub action_timeline: u32,
}

/// How long after its spawn retail brought an ambushing gigas into view: 0.58-0.94 s in six ambushes (Bob's Copperbell
/// Mines captures of 2026-10-25.)
pub const DEFAULT_APPEAR_DELAY: f32 = 0.75;

/// How far an attached BNpc may be from its place, and how far turned from its facing (radians), before it's moved
/// (`keep_attached`.)
const ATTACHED_SLACK: f32 = 0.1;
const ATTACHED_TURN_SLACK: f32 = 0.05;
/// The speed byte of retail's ActorMoves for the Navel's Heart of the Mountain as it went with Titan (Bob's Chronofoil
/// capture 650dbfd8.)
const ATTACHED_MOVE_SPEED: u8 = 60;

/// Puts every attached BNpc (`director:attach_bnpc`) in its place, `distance` yalms in front of the one it's attached
/// to and facing as it faces, with an ActorMove when it has moved or turned: retail moved the Navel's Heart of the
/// Mountain with Titan, 2.3 yalms before him the way he faced, as he walked and turned (Bob's Chronofoil captures
/// 650dbfd8, 0fd8bce0 and 48aaf6fe.) Either one gone or dead, it's no longer attached.
pub fn keep_attached(network: &Arc<Mutex<NetworkState>>, instance: &mut Instance) {
    if instance.attached_npcs.is_empty() {
        return;
    }
    let attached: Vec<(ObjectId, (ObjectId, f32))> = instance
        .attached_npcs
        .iter()
        .map(|(child, to)| (*child, *to))
        .collect();
    for (child, (parent, distance)) in attached {
        let alive = |id: ObjectId| {
            instance
                .find_actor(id)
                .is_some_and(|actor| actor.get_common_spawn().health_points > 0)
        };
        if !alive(child) || !alive(parent) {
            instance.attached_npcs.remove(&child);
            continue;
        }
        let Some(parent_actor) = instance.find_actor(parent) else {
            continue;
        };
        let rotation = parent_actor.rotation();
        let spot = parent_actor.position().0
            + glam::Vec3A::new(distance * rotation.sin(), 0.0, distance * rotation.cos());
        let Some(NetworkedActor::Npc {
            spawn,
            navmesh_path,
            navmesh_path_lerp,
            last_position,
            ..
        }) = instance.find_actor_mut(child)
        else {
            continue;
        };
        let turned = {
            let difference = (spawn.common.rotation - rotation).rem_euclid(std::f32::consts::TAU);
            difference.min(std::f32::consts::TAU - difference)
        };
        if (spawn.common.position.0 - spot).length() <= ATTACHED_SLACK
            && turned <= ATTACHED_TURN_SLACK
        {
            continue;
        }
        spawn.common.position = Position(spot);
        spawn.common.rotation = rotation;
        navmesh_path.clear();
        *navmesh_path_lerp = 0.0;
        *last_position = Some(spot);
        network.lock().send_in_range_instance(
            child,
            instance,
            FromServer::ActorMove(
                child,
                Position(spot),
                rotation,
                kawari::common::MoveAnimationType::empty(),
                kawari::common::MoveAnimationState::None,
                kawari::common::JumpState::NoneOrFalling,
                Some(ATTACHED_MOVE_SPEED),
            ),
            DestinationNetwork::ZoneClients,
        );
    }
}

/// Brings the BNpcs whose time has come into view: `Appear` and `SetTargetable` to everyone around, and INVISIBLE
/// dropped from their spawn, so whoever sees them spawn later sees them at once.
pub fn update_appearances(
    network: &mut NetworkState,
    instance: &mut Instance,
    now: std::time::Instant,
) {
    if instance.pending_appearances.is_empty() {
        return;
    }
    let (due, later): (Vec<PendingAppearance>, Vec<PendingAppearance>) =
        std::mem::take(&mut instance.pending_appearances)
            .into_iter()
            .partition(|pending| pending.at <= now);
    instance.pending_appearances = later;
    for pending in due {
        let Some((actor_id, _)) = instance.find_npc(pending.layout_id) else {
            tracing::warn!(
                "Failed to find bnpc {} to bring into view!",
                pending.layout_id
            );
            continue;
        };
        if let Some(NetworkedActor::Npc { spawn, .. }) = instance.find_actor_mut(actor_id) {
            spawn.common.display_flags.remove(DisplayFlag::INVISIBLE);
        }
        instance.untargetable_npcs.remove(&actor_id);
        network.send_ac_in_range_instance(
            instance,
            actor_id,
            ActorControlCategory::Appear {
                fade_in: true,
                action_timeline: pending.action_timeline,
            },
        );
        network.send_ac_in_range_instance(
            instance,
            actor_id,
            ActorControlCategory::SetTargetable { targetable: true },
        );
    }
}

/// One actor's numbers for a director's `onUpdate` (see `DirectorData::update`.)
#[derive(Debug, Clone, Copy)]
pub struct ActorInfo {
    pub hp: u32,
    pub max_hp: u32,
    pub position: glam::Vec3A,
    /// Who it's targeting: an NPC's layout ID, or the player.
    pub target_layout: Option<u32>,
    pub target_is_player: bool,
    /// Whether it's targeting anyone at all (a party NPC has no layout ID: its target shows as "party".)
    pub has_target: bool,
    /// Its actor ID, and its target's (0 for none): the Navel's Granite Gaol falls on whomever Titan's Rock Throw was
    /// aimed at, and his Weight of the Land under two of the party other than the one he fights.
    pub actor_id: u32,
    pub target_id: u32,
}

/// A director cutscene's scene parameters after its ID, unless the script gives its own: upstream's, as retail sent
/// for Denn's arrival in Sastasha (175, then 1, 38, 1, 0). Retail sent other ones for his death (176: 1, 18, 0, 0), and
/// with these the client went on to play his arrival's dialogue again after it (Bob's bundle of 2026-10-23.)
pub const DEFAULT_CUTSCENE_PARAMS: [u32; 4] = [1, 38, 1, 0];

// TODO: Maybe collapse into DirectorData?
#[derive(Default, Debug)]
pub struct LuaDirector {
    pub data: [u8; 10],
    pub tasks: Vec<LuaDirectorTask>,
    pub bosses: HashMap<u32, DirectorBoss>,
    /// From which task on (its index) the ones queued are delayed, and by how many seconds (`director:delay`.)
    pub delays: Vec<(usize, f32)>,
}

impl UserData for LuaDirector {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("hide_eobj", |_, this, base_id: u32| {
            this.tasks.push(LuaDirectorTask::HideEObj { base_id });
            Ok(())
        });
        methods.add_method_mut("show_eobj", |_, this, base_id: u32| {
            this.tasks.push(LuaDirectorTask::ShowEObj { base_id });
            Ok(())
        });
        methods.add_method_mut("delete_eobj", |_, this, base_id: u32| {
            this.tasks.push(LuaDirectorTask::DeleteEObj { base_id });
            Ok(())
        });
        methods.add_method_mut(
            "spawn_eobj",
            |lua, this, (base_id, position, args2): (u32, Value, Option<u32>)| {
                let position: Option<Position> = lua.from_value(position).ok();
                this.tasks.push(LuaDirectorTask::SpawnEObj {
                    base_id,
                    position,
                    args2,
                });
                Ok(())
            },
        );
        methods.add_method_mut("set_data", |_, this, (index, data): (u8, u8)| {
            this.data[index as usize] = data;
            Ok(())
        });
        methods.add_method("data", |_, this, index: u8| Ok(this.data[index as usize]));
        methods.add_method_mut(
            "event_action",
            |_, this, (action_id, actor_id, target): (u32, u32, u32)| {
                this.tasks.push(LuaDirectorTask::BeginEventAction {
                    actor_id: ObjectId(actor_id),
                    target: ObjectId(target),
                    action_id,
                });
                Ok(())
            },
        );
        methods.add_method_mut("finish_gimmick", |_, this, actor_id: u32| {
            this.tasks.push(LuaDirectorTask::FinishGimmickEvent {
                actor_id: ObjectId(actor_id),
            });
            Ok(())
        });
        methods.add_method_mut("log_message", |_, this, (id, params): (u32, Vec<u32>)| {
            this.tasks.push(LuaDirectorTask::LogMessage { id, params });
            Ok(())
        });
        methods.add_method_mut(
            "spawn_bnpc",
            |_, this, (id, x, y, z, rotation): (u32, Option<f32>, Option<f32>, Option<f32>, Option<f32>)| {
                let actor_id = ObjectId(fastrand::u32(1..0xE000_0000));
                let at = match (x, y, z) {
                    (Some(x), Some(y), Some(z)) => {
                        Some((Position(glam::Vec3A::new(x, y, z)), rotation))
                    }
                    _ => None,
                };
                this.tasks
                    .push(LuaDirectorTask::SpawnBattleNpc { id, actor_id, at });
                Ok(actor_id.0)
            },
        );
        methods.add_method_mut(
            "spawn_tethered_bnpc",
            |_, this, (id, to, channeling): (u32, u32, u16)| {
                this.tasks
                    .push(LuaDirectorTask::SpawnTetheredBattleNpc { id, to, channeling });
                Ok(())
            },
        );
        methods.add_method_mut("despawn_bnpc", |_, this, layout_id: u32| {
            this.tasks
                .push(LuaDirectorTask::DespawnBattleNpc { layout_id });
            Ok(())
        });
        methods.add_method_mut(
            "gain_effect",
            |_, this, (actor_id, id, param, duration): (u32, u16, u16, f32)| {
                this.tasks.push(LuaDirectorTask::GainEffect {
                    actor_id: ObjectId(actor_id),
                    id,
                    param,
                    duration,
                });
                Ok(())
            },
        );
        methods.add_method_mut("lose_effect", |_, this, (actor_id, id): (u32, u16)| {
            this.tasks.push(LuaDirectorTask::LoseEffect {
                actor_id: ObjectId(actor_id),
                id,
            });
            Ok(())
        });
        methods.add_method_mut(
            "attach_bnpc",
            |_, this, (who, to, distance): (u32, u32, f32)| {
                this.tasks
                    .push(LuaDirectorTask::AttachBattleNpc { who, to, distance });
                Ok(())
            },
        );
        methods.add_method_mut("set_bgm", |_, this, id: u32| {
            this.tasks.push(LuaDirectorTask::SetBGM { id });
            Ok(())
        });
        methods.add_method_mut(
            "spawn_boss",
            |_,
             this,
             (bnpc_id, wall_id, line_id, place_name, arena): (
                u32,
                u32,
                u32,
                u32,
                Option<u32>,
            )| {
                this.tasks.push(LuaDirectorTask::SpawnBoss {
                    bnpc_id,
                    wall_id,
                    line_id,
                    place_name,
                    arena: arena.unwrap_or(0),
                });
                Ok(())
            },
        );
        methods.add_method_mut(
            "boss_room",
            |_,
             this,
             (bnpc_id, wall_id, line_id, place_name, triggers, arena): (
                u32,
                u32,
                u32,
                u32,
                Option<Vec<u32>>,
                Option<u32>,
            )| {
                this.tasks.push(LuaDirectorTask::BossRoom {
                    bnpc_id,
                    wall_id,
                    line_id,
                    place_name,
                    triggers: triggers.unwrap_or_default(),
                    arena: arena.unwrap_or(0),
                });
                Ok(())
            },
        );
        // A boss that leaves its room without dying, as retail's dragon (BNpcBase 429) flew off from Brayflox's Longstop's
        // third boss room: it no longer holds the room, which opens (wall and line hidden, log 2014) if none of the
        // room's other bosses is left (Bob's Chronofoil capture 44df02fc of 2026-11-03: the room opened as it went.)
        methods.add_method_mut("leave_boss_room", |_, this, bnpc_id: u32| {
            if let Some(boss) = this.bosses.remove(&bnpc_id)
                && !this
                    .bosses
                    .values()
                    .any(|other| other.wall_id == boss.wall_id)
            {
                this.tasks.push(LuaDirectorTask::HideEObj {
                    base_id: boss.wall_id,
                });
                this.tasks.push(LuaDirectorTask::HideEObj {
                    base_id: boss.line_id,
                });
                this.tasks.push(LuaDirectorTask::LogMessage {
                    id: LogMessageType::BossRoomUnsealed as u32,
                    params: vec![boss.place_name],
                });
            }
            Ok(())
        });
        methods.add_method_mut(
            "set_invulnerable",
            |_, this, (layout_id, invulnerable): (u32, bool)| {
                this.tasks.push(LuaDirectorTask::SetInvulnerable {
                    layout_id,
                    invulnerable,
                });
                Ok(())
            },
        );
        methods.add_method_mut("spawn_treasure", |_, this, id: u32| {
            this.tasks.push(LuaDirectorTask::SpawnTreasure { id });
            Ok(())
        });
        methods.add_method_mut(
            "treasure_loot",
            |_, this, (base_id, openings): (u32, Vec<Vec<u32>>)| {
                this.tasks
                    .push(LuaDirectorTask::TreasureLoot { base_id, openings });
                Ok(())
            },
        );
        methods.add_method_mut(
            "treasure_pool",
            |_, this, (base_id, items): (u32, Vec<u32>)| {
                this.tasks
                    .push(LuaDirectorTask::TreasurePool { base_id, items });
                Ok(())
            },
        );
        methods.add_method_mut("variant_vote_route", |_, this, npc_route: u32| {
            this.tasks
                .push(LuaDirectorTask::VariantVoteRoute { npc_route });
            Ok(())
        });
        methods.add_method_mut(
            "play_cutscene",
            |_, this, (cutscene_id, params): (u32, Option<Vec<u32>>)| {
                this.tasks.push(LuaDirectorTask::PlayCutscene {
                    cutscene_id,
                    params: params.unwrap_or_else(|| DEFAULT_CUTSCENE_PARAMS.to_vec()),
                });
                Ok(())
            },
        );
        methods.add_method_mut("update_shortcut", |_, this, poprange_id: u32| {
            // Show the shortcut object because it's hidden by default
            this.tasks.push(LuaDirectorTask::ShowEObj {
                base_id: EOBJ_SHORTCUT,
            });
            this.tasks
                .push(LuaDirectorTask::UpdateShortcut { poprange_id });
            Ok(())
        });
        // Send a player somewhere in the duty, as a shortcut does (a gimmick of the script's own.)
        methods.add_method_mut(
            "warp_to_pop_range",
            |_, this, (actor_id, pop_range_id): (u32, u32)| {
                this.tasks.push(LuaDirectorTask::UseShortcut {
                    actor_id: ObjectId(actor_id),
                    pop_range_id: Some(pop_range_id),
                });
                Ok(())
            },
        );
        methods.add_method_mut("complete_duty", |_, this, _: ()| {
            // Show the exit objects
            this.tasks.push(LuaDirectorTask::ShowExits);
            this.tasks.push(LuaDirectorTask::CompleteDuty {});
            Ok(())
        });
        methods.add_method_mut(
            "complete_quest_battle",
            |_,
             this,
             (sequence, stay, finish_scene, finish_flags, variables, fanfare): (
                u8,
                Option<bool>,
                Option<u16>,
                Option<u32>,
                Option<mlua::Table>,
                Option<bool>,
            )| {
                // `variables` is a table of quest variable index (0-5) to value, e.g. { [1] = 0x10 }.
                let mut set = Vec::new();
                if let Some(variables) = variables {
                    for pair in variables.pairs::<u8, u8>() {
                        set.push(pair?);
                    }
                    set.sort();
                }
                this.tasks.push(LuaDirectorTask::CompleteQuestBattle {
                    sequence,
                    stay: stay.unwrap_or(false),
                    finish_scene: finish_scene.unwrap_or(QUEST_BATTLE_SCENE_FINISH),
                    finish_flags: finish_flags.unwrap_or(QUEST_BATTLE_SCENE_FINISH_FLAGS),
                    variables: set,
                    fanfare: fanfare.unwrap_or(true),
                });
                Ok(())
            },
        );
        methods.add_method_mut("play_scene", |_, this, (scene, flags): (u16, u32)| {
            this.tasks.push(LuaDirectorTask::PlayScene { scene, flags });
            Ok(())
        });
        methods.add_method_mut(
            "warp_players",
            |_, this, (x, y, z, rotation): (f32, f32, f32, f32)| {
                this.tasks.push(LuaDirectorTask::WarpPlayers {
                    position: Position(glam::Vec3A::new(x, y, z)),
                    rotation,
                });
                Ok(())
            },
        );
        methods.add_method_mut("leave_quest_battle", |_, this, _: ()| {
            this.tasks.push(LuaDirectorTask::LeaveQuestBattle);
            Ok(())
        });
        methods.add_method_mut(
            "use_action",
            |_, this, (layout_id, action_id, target): (u32, u32, Option<u32>)| {
                this.tasks.push(LuaDirectorTask::UseAction {
                    layout_id,
                    action_id,
                    target,
                });
                Ok(())
            },
        );
        methods.add_method_mut("end_fight", |_, this, _: ()| {
            this.tasks.push(LuaDirectorTask::EndFight);
            Ok(())
        });
        methods.add_method_mut("say", |_, this, (layout_id, key): (u32, String)| {
            this.tasks.push(LuaDirectorTask::Say { layout_id, key });
            Ok(())
        });
        methods.add_method_mut(
            "say_battle_talk",
            |_, this, (layout_id, number, name_id): (u32, usize, Option<u32>)| {
                this.tasks.push(LuaDirectorTask::SayBattleTalk {
                    layout_id,
                    name_id,
                    number,
                });
                Ok(())
            },
        );
        methods.add_method_mut(
            "battle_talk",
            |_,
             this,
             (layout_id, name_id, text_id, icon, duration_ms, kind): (
                u32,
                u32,
                u32,
                u32,
                Option<u32>,
                Option<u8>,
            )| {
                this.tasks.push(LuaDirectorTask::BattleTalk {
                    layout_id,
                    name_id,
                    text_id,
                    icon,
                    // Retail shows every line for 4 seconds.
                    duration_ms: duration_ms.unwrap_or(4000),
                    // A BattleNpc unless said otherwise.
                    kind: kind.unwrap_or(BATTLE_TALK_KIND_BATTLE_NPC),
                });
                Ok(())
            },
        );
        methods.add_method_mut(
            "play_action_timeline",
            |_, this, (layout_id, timeline_id): (u32, u32)| {
                this.tasks.push(LuaDirectorTask::PlayActionTimeline {
                    layout_id,
                    timeline_id,
                });
                Ok(())
            },
        );
        methods.add_method_mut(
            "party_npc_yell",
            |_,
             this,
             (role, npc_yell_id, delay, other_role, other_yell): (
                String,
                u32,
                Option<f32>,
                Option<String>,
                Option<u32>,
            )| {
                this.tasks.push(LuaDirectorTask::PartyNpcYell {
                    role,
                    npc_yell_id,
                    delay: delay.unwrap_or(0.0),
                    otherwise: other_role.zip(other_yell),
                });
                Ok(())
            },
        );
        methods.add_method_mut("yell", |_, this, (layout_id, npc_yell_id): (u32, u32)| {
            this.tasks.push(LuaDirectorTask::Yell {
                layout_id,
                npc_yell_id,
            });
            Ok(())
        });
        methods.add_method_mut("set_sequence", |_, this, sequence: u8| {
            this.tasks.push(LuaDirectorTask::SetSequence { sequence });
            Ok(())
        });
        methods.add_method_mut("fail_quest_battle", |_, this, _: ()| {
            this.tasks.push(LuaDirectorTask::FailQuestBattle);
            Ok(())
        });
        methods.add_method_mut(
            "set_hate",
            |_, this, (layout_id, target, amount): (u32, Option<u32>, Option<u32>)| {
                this.tasks.push(LuaDirectorTask::SetHate {
                    layout_id,
                    target,
                    amount: amount.unwrap_or(1),
                });
                Ok(())
            },
        );
        methods.add_method_mut(
            "set_notice",
            |_, this, (layout_id, notices): (u32, bool)| {
                this.tasks
                    .push(LuaDirectorTask::SetNotice { layout_id, notices });
                Ok(())
            },
        );
        methods.add_method_mut(
            "set_attack_range",
            |_, this, (layout_id, yalms): (u32, f32)| {
                this.tasks
                    .push(LuaDirectorTask::SetAttackRange { layout_id, yalms });
                Ok(())
            },
        );
        methods.add_method_mut(
            "keep_near",
            |_, this, (layout_id, anchors, yalms): (u32, mlua::Value, Option<f32>)| {
                // One anchor (a layout ID, or nil / "player" for the player) or a list of them.
                let one = |value: &mlua::Value| match value {
                    mlua::Value::Integer(layout) => Some(Some(*layout as u32)),
                    mlua::Value::Number(layout) => Some(Some(*layout as u32)),
                    mlua::Value::Nil => Some(None),
                    mlua::Value::String(text) if text.to_str().is_ok_and(|t| t == "player") => {
                        Some(None)
                    }
                    _ => None,
                };
                let anchors = match &anchors {
                    mlua::Value::Table(list) => list
                        .sequence_values::<mlua::Value>()
                        .filter_map(|value| value.ok())
                        .filter_map(|value| one(&value))
                        .collect(),
                    value => one(value).into_iter().collect(),
                };
                this.tasks.push(LuaDirectorTask::KeepNear {
                    layout_id,
                    anchors,
                    yalms,
                });
                Ok(())
            },
        );
        methods.add_method_mut(
            "stand_behind",
            |_, this, (layout_id, tank, yalms): (u32, u32, Option<f32>)| {
                this.tasks.push(LuaDirectorTask::StandBehind {
                    layout_id,
                    tank,
                    yalms,
                });
                Ok(())
            },
        );
        methods.add_method_mut(
            "map_effect",
            |_, this, (index, timeline_id, state): (u32, u32, Option<u16>)| {
                this.tasks.push(LuaDirectorTask::MapEffect {
                    index,
                    timeline_id,
                    state: state.unwrap_or(1),
                });
                Ok(())
            },
        );
        methods.add_method_mut(
            "map_effect_at",
            |_, this, (layout_id, timeline_id, state, otherwise): (u32, u32, Option<u16>, Option<u32>)| {
                this.tasks.push(LuaDirectorTask::MapEffectAt {
                    layout_id,
                    timeline_id,
                    state: state.unwrap_or(1),
                    otherwise,
                });
                Ok(())
            },
        );
        methods.add_method_mut("delay", |_, this, seconds: f32| {
            this.delays.push((this.tasks.len(), seconds.max(0.0)));
            Ok(())
        });
        methods.add_method_mut(
            "warp_bnpc",
            |_, this, (who, x, y, z, rotation): (u32, f32, f32, f32, Option<f32>)| {
                this.tasks.push(LuaDirectorTask::WarpBattleNpc {
                    who,
                    position: Position(glam::Vec3A::new(x, y, z)),
                    rotation: rotation.unwrap_or(0.0),
                });
                Ok(())
            },
        );
        methods.add_method_mut(
            "jump_bnpc",
            |_, this, (who, x, y, z, rotation): (u32, f32, f32, f32, Option<f32>)| {
                this.tasks.push(LuaDirectorTask::JumpBattleNpc {
                    who,
                    position: Position(glam::Vec3A::new(x, y, z)),
                    rotation,
                });
                Ok(())
            },
        );
        methods.add_method_mut("root_bnpc", |_, this, (who, seconds): (u32, f32)| {
            this.tasks
                .push(LuaDirectorTask::RootBattleNpc { who, seconds });
            Ok(())
        });
        methods.add_method_mut("busy_bnpc", |_, this, (who, seconds): (u32, f32)| {
            this.tasks
                .push(LuaDirectorTask::BusyBattleNpc { who, seconds });
            Ok(())
        });
        methods.add_method_mut("restart_timeline", |_, this, (who, anchor): (u32, u32)| {
            this.tasks
                .push(LuaDirectorTask::RestartTimeline { who, anchor });
            Ok(())
        });
        methods.add_method_mut(
            "set_model_scale",
            |_, this, (who, model_scale_id): (u32, u32)| {
                this.tasks.push(LuaDirectorTask::SetModelScale {
                    who,
                    model_scale_id,
                });
                Ok(())
            },
        );
        methods.add_method_mut("fade_out_bnpc", |_, this, who: u32| {
            this.tasks.push(LuaDirectorTask::FadeOutBattleNpc { who });
            Ok(())
        });
        methods.add_method_mut(
            "retreat_bnpc",
            |_, this, (who, x, y, z): (u32, Option<f32>, Option<f32>, Option<f32>)| {
                let to = match (x, y, z) {
                    (Some(x), Some(y), Some(z)) => Some(glam::Vec3A::new(x, y, z)),
                    _ => None,
                };
                this.tasks
                    .push(LuaDirectorTask::RetreatBattleNpc { who, to });
                Ok(())
            },
        );
        methods.add_method_mut(
            "set_targetable",
            |_, this, (who, targetable): (u32, bool)| {
                this.tasks
                    .push(LuaDirectorTask::SetTargetable { who, targetable });
                Ok(())
            },
        );
        methods.add_method_mut(
            "bnpc_control",
            |_, this, (who, category, params): (u32, u32, Option<Vec<u32>>)| {
                let mut four = [0; 4];
                for (slot, value) in four.iter_mut().zip(params.unwrap_or_default()) {
                    *slot = value;
                }
                this.tasks.push(LuaDirectorTask::BattleNpcControl {
                    who,
                    category,
                    params: four,
                });
                Ok(())
            },
        );
        methods.add_method_mut("kill_bnpc", |_, this, who: u32| {
            this.tasks.push(LuaDirectorTask::KillBattleNpc { who });
            Ok(())
        });
        methods.add_method_mut("objective", |_, this, (done, next): (u32, Option<u32>)| {
            this.tasks.push(LuaDirectorTask::Objective { done, next });
            Ok(())
        });
        methods.add_method_mut(
            "set_weather",
            |_, this, (id, fade, forced): (u8, Option<f32>, Option<bool>)| {
                this.tasks.push(LuaDirectorTask::SetWeather {
                    id,
                    fade: fade.unwrap_or(0.0),
                    forced: forced.unwrap_or(false),
                });
                Ok(())
            },
        );
        methods.add_method_mut("release_bnpc", |_, this, who: u32| {
            this.tasks.push(LuaDirectorTask::ReleaseBattleNpc { who });
            Ok(())
        });
        methods.add_method_mut("hold_bnpc", |_, this, (who, held): (u32, Option<bool>)| {
            this.tasks.push(LuaDirectorTask::HoldBattleNpc {
                who,
                held: held.unwrap_or(true),
            });
            Ok(())
        });
        methods.add_method_mut(
            "move_bnpc",
            |_,
             this,
             (who, x, y, z, rotation, speed): (
                u32,
                f32,
                f32,
                f32,
                Option<f32>,
                Option<u8>,
            )| {
                this.tasks.push(LuaDirectorTask::MoveBattleNpc {
                    who,
                    position: Position(glam::Vec3A::new(x, y, z)),
                    rotation: rotation.unwrap_or(0.0),
                    speed,
                });
                Ok(())
            },
        );
        methods.add_method_mut(
            "actor_use_action",
            |_, this, (actor_id, action_id, target): (u32, u32, Option<u32>)| {
                this.tasks.push(LuaDirectorTask::ActorUseAction {
                    actor_id,
                    action_id,
                    target,
                });
                Ok(())
            },
        );
        methods.add_method_mut(
            "appear_bnpc",
            |_, this, (layout_id, action_timeline, delay): (u32, Option<u32>, Option<f32>)| {
                this.tasks.push(LuaDirectorTask::AppearBattleNpc {
                    layout_id,
                    action_timeline: action_timeline.unwrap_or(0),
                    delay: delay.unwrap_or(DEFAULT_APPEAR_DELAY),
                });
                Ok(())
            },
        );
        methods.add_method_mut(
            "eobj_control",
            |_, this, (id, category, params): (u32, u32, Option<Vec<u32>>)| {
                let mut four = [0; 4];
                for (slot, value) in four.iter_mut().zip(params.unwrap_or_default()) {
                    *slot = value;
                }
                this.tasks.push(LuaDirectorTask::EObjControl {
                    id,
                    category,
                    params: four,
                });
                Ok(())
            },
        );
    }
}

/// A boss room's arena (`DirectorBoss::arena`) the zone has no range for keeps nobody in: said once, as it's set.
fn warn_if_no_arena(instance: &Instance, arena: u32) {
    if arena != 0
        && !instance
            .zone
            .map_ranges
            .iter()
            .any(|range| range.instance_id == arena)
    {
        tracing::warn!(
            "Boss room arena {arena} isn't one of the zone's ranges: the party isn't kept in it"
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DirectorBossState {
    /// The boss arena is open and nothing has happened yet.
    Open,
    /// The boss arena is closing soon, because the boss was aggravated.
    Aggro,
    /// The boss arena is closed.
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DirectorBoss {
    state: DirectorBossState,
    actor_id: ObjectId,
    wall_id: u32,
    line_id: u32,
    place_name: u32,
    /// Other BNpcs (layout IDs, 0 for none) whose pull seals the arena too.
    triggers: [u32; 8],
    /// The layout's range that is the room's floor (an EventRange's or MapRange's layout ID, 0 for none): from the
    /// pull until the boss is gone, party bots and party NPCs keep inside it (`dodge::arena`). Brayflox's Longstop's
    /// last boss's terrace is the cylinder 9059667 (20 yalms) in its gimmick layer; retail's party never went more
    /// than 19.4 yalms from its middle, Kawari's ran 27 out to get away from its puddles (Bob's bundle 20261104-204423.)
    arena: u32,
}

#[derive(Debug, Default)]
pub struct DirectorData {
    pub id: HandlerId,
    /// The director's sequence (DirectorVars' first byte.)
    pub flag: u8,
    pub data: [u8; 10],
    /// Lua state for this director.
    pub lua: KawariLua,
    pub tasks: Vec<LuaDirectorTask>,
    /// List of alive bosses and their data.
    pub bosses: HashMap<u32, DirectorBoss>,
    /// What the shortcut is currently pointing to.
    pub shortcut_poprange_id: Option<u32>,
    /// For quest battles, the quest it belongs to.
    pub quest_battle_quest_id: u32,
}

impl DirectorData {
    pub fn setup(&mut self) {
        let mut run_script = || {
            let mut lua_director = self.create_lua_director();
            let err = self.lua.0.scope(|scope| {
                let data = scope.create_userdata_ref_mut(&mut lua_director)?;

                let func: Function = self.lua.0.globals().get("onSetup")?;

                func.call::<()>(data)?;

                Ok(())
            });
            self.apply_lua_director(lua_director);
            err
        };
        if let Err(err) = run_script() {
            tracing::warn!("Syntax error during onSetup: {err:?}");
        }
    }

    /// Called every server tick for scripts that define `onUpdate`, with the health (in percent) of every living NPC
    /// by its layout ID, the player's, and (fourth) everyone's details: `info[layout] = { hp, max_hp, x, y, z, target, id,
    /// target_id }` (`target`: the layout ID it's after, or "player"; `id` and `target_id` the actor IDs), and
    /// `info.player` the same for the player, for scripts that need real numbers or distances (content/lib/Battle.lua.)
    pub fn update(
        &mut self,
        npc_health: &HashMap<u32, u32>,
        player_health: Option<u32>,
        info: &HashMap<String, ActorInfo>,
    ) {
        let Ok(func) = self.lua.0.globals().get::<Function>("onUpdate") else {
            return;
        };

        let mut lua_director = self.create_lua_director();
        let err = self.lua.0.scope(|scope| {
            let data = scope.create_userdata_ref_mut(&mut lua_director)?;
            let health = self
                .lua
                .0
                .create_table_from(npc_health.iter().map(|(k, v)| (*k, *v)))?;
            let details = self.lua.0.create_table()?;
            for (key, actor) in info {
                let entry = self.lua.0.create_table()?;
                entry.set("hp", actor.hp)?;
                entry.set("max_hp", actor.max_hp)?;
                entry.set("x", actor.position.x)?;
                entry.set("y", actor.position.y)?;
                entry.set("z", actor.position.z)?;
                if let Some(target) = actor.target_layout {
                    entry.set("target", target)?;
                } else if actor.target_is_player {
                    entry.set("target", "player")?;
                } else if actor.has_target {
                    entry.set("target", "party")?;
                }
                entry.set("id", actor.actor_id)?;
                if actor.has_target {
                    entry.set("target_id", actor.target_id)?;
                }
                match key.parse::<u32>() {
                    Ok(layout_id) => details.set(layout_id, entry)?,
                    Err(_) => details.set(key.as_str(), entry)?,
                }
            }

            func.call::<()>((data, health, player_health, details))?;

            Ok(())
        });
        self.apply_lua_director(lua_director);
        if let Err(err) = err {
            tracing::warn!("Syntax error during onUpdate: {err:?}");
        }
    }

    /// Called when the player is ready for the content to begin, after its opening cutscene.
    pub fn commence(&mut self) {
        let mut run_script = || {
            let mut lua_director = self.create_lua_director();
            let err = self.lua.0.scope(|scope| {
                let data = scope.create_userdata_ref_mut(&mut lua_director)?;

                let func: Function = self.lua.0.globals().get("onCommence")?;

                func.call::<()>(data)?;

                Ok(())
            });
            self.apply_lua_director(lua_director);
            err
        };
        if let Err(err) = run_script() {
            tracing::warn!("Syntax error during onCommence: {err:?}");
        }
    }

    /// The opening scene's flags, for a script that sets `OPENING_SCENE_FLAGS` (quest battles: retail's differ from
    /// one to the next.)
    /// Which way players face as they come in (and after a wipe), for a script that sets `ENTRANCE_ROTATION`: a duty
    /// whose InstanceContent has no entrance rect (the Bowl of Embers': retail turned players to face Ifrit, 1.571,
    /// where its entrance circle faces 0.)
    pub fn entrance_rotation(&self) -> Option<f32> {
        self.lua
            .0
            .globals()
            .get::<Option<f32>>("ENTRANCE_ROTATION")
            .ok()
            .flatten()
    }

    pub fn opening_scene_flags(&self) -> Option<u32> {
        self.lua
            .0
            .globals()
            .get::<Option<u32>>("OPENING_SCENE_FLAGS")
            .ok()
            .flatten()
    }

    /// Called when a player finishes one of the director's scenes, for scripts that define `onSceneFinished`.
    pub fn scene_finished(&mut self, scene: u16) {
        let Ok(func) = self.lua.0.globals().get::<Function>("onSceneFinished") else {
            return;
        };

        let mut lua_director = self.create_lua_director();
        let err = self.lua.0.scope(|scope| {
            let data = scope.create_userdata_ref_mut(&mut lua_director)?;
            func.call::<()>((data, scene))?;
            Ok(())
        });
        self.apply_lua_director(lua_director);
        if let Err(err) = err {
            tracing::warn!("Syntax error during onSceneFinished: {err:?}");
        }
    }

    pub fn gimmick_accessor(&mut self, actor_id: ObjectId, id: u32, params: &[i32]) {
        let mut run_script = || {
            let mut lua_director = self.create_lua_director();
            let err = self.lua.0.scope(|scope| {
                let data = scope.create_userdata_ref_mut(&mut lua_director)?;

                let func: Function = self.lua.0.globals().get("onGimmickAccessor")?;

                func.call::<()>((data, actor_id.0, id, params))?;

                Ok(())
            });
            self.apply_lua_director(lua_director);
            err
        };
        if let Err(err) = run_script() {
            tracing::warn!("Syntax error during onGimmickAccessor: {err:?}");

            // Finish the gimmick event so the client at least gets unstuck
            self.tasks
                .push(LuaDirectorTask::FinishGimmickEvent { actor_id });
        }
    }

    /// A player's talk with a ContentNpc (row `id`, event NPC `npc`) returned: the script's `onContentNpc(director,
    /// actor_id, id, npc, results)`, which finishes the event (`finish_gimmick`). Without one the event just finishes.
    pub fn content_npc(&mut self, actor_id: ObjectId, id: u32, npc: ObjectId, results: &[i32]) {
        let Ok(func) = self.lua.0.globals().get::<Function>("onContentNpc") else {
            self.tasks
                .push(LuaDirectorTask::FinishGimmickEvent { actor_id });
            return;
        };

        let mut lua_director = self.create_lua_director();
        let err = self.lua.0.scope(|scope| {
            let data = scope.create_userdata_ref_mut(&mut lua_director)?;
            func.call::<()>((data, actor_id.0, id, npc.0, results))?;
            Ok(())
        });
        self.apply_lua_director(lua_director);
        if let Err(err) = err {
            tracing::warn!("Syntax error during onContentNpc: {err:?}");
            self.tasks
                .push(LuaDirectorTask::FinishGimmickEvent { actor_id });
        }
    }

    /// A player talked to an event object of this battle's (`ToServer::QuestBattleTalk`): the script's
    /// `onObjectTalk(director, actor_id, object_id, base_id)`, which answers it (an event action, then
    /// `finish_gimmick`.) False if the script has none, so it gets EVENT_QIB_NOT_TALK instead.
    pub fn object_talk(&mut self, actor_id: ObjectId, object_id: ObjectId, base_id: u32) -> bool {
        let Ok(func) = self.lua.0.globals().get::<Function>("onObjectTalk") else {
            return false;
        };

        let mut lua_director = self.create_lua_director();
        let err = self.lua.0.scope(|scope| {
            let data = scope.create_userdata_ref_mut(&mut lua_director)?;
            func.call::<()>((data, actor_id.0, object_id.0, base_id))?;
            Ok(())
        });
        self.apply_lua_director(lua_director);
        if let Err(err) = err {
            tracing::warn!("Syntax error during onObjectTalk: {err:?}");
            self.tasks
                .push(LuaDirectorTask::FinishGimmickEvent { actor_id });
        }
        true
    }

    pub fn event_action_cast(&mut self, actor_id: ObjectId, target: ObjectId) {
        let mut run_script = || {
            let mut lua_director = self.create_lua_director();
            let err = self.lua.0.scope(|scope| {
                let data = scope.create_userdata_ref_mut(&mut lua_director)?;

                let func: Function = self.lua.0.globals().get("onEventActionCast")?;

                func.call::<()>((data, actor_id.0, target.0))?;

                Ok(())
            });
            self.apply_lua_director(lua_director);
            err
        };
        if let Err(err) = run_script() {
            tracing::warn!("Syntax error during onEventActionCast: {err:?}");
        }
    }

    /// Calls the script's `name(director)`, if it has it (`onPartyWipe`, `onDutyRecommence`: `server/wipe.rs`.)
    pub fn call_hook(&mut self, name: &str) {
        if self.lua.0.globals().get::<Function>(name).is_err() {
            return;
        }
        let mut run_script = || {
            let mut lua_director = self.create_lua_director();
            let err = self.lua.0.scope(|scope| {
                let data = scope.create_userdata_ref_mut(&mut lua_director)?;
                let func: Function = self.lua.0.globals().get(name)?;
                func.call::<()>(data)?;
                Ok(())
            });
            self.apply_lua_director(lua_director);
            err
        };
        if let Err(err) = run_script() {
            tracing::warn!("Syntax error during {name}: {err:?}");
        }
    }

    /// Called when a player in this instance dies, if the script has `onPlayerDeath`.
    pub fn on_player_death(&mut self) {
        if self
            .lua
            .0
            .globals()
            .get::<Function>("onPlayerDeath")
            .is_err()
        {
            return;
        }

        let mut run_script = || {
            let mut lua_director = self.create_lua_director();
            let err = self.lua.0.scope(|scope| {
                let data = scope.create_userdata_ref_mut(&mut lua_director)?;

                let func: Function = self.lua.0.globals().get("onPlayerDeath")?;

                func.call::<()>(data)?;

                Ok(())
            });
            self.apply_lua_director(lua_director);
            err
        };
        if let Err(err) = run_script() {
            tracing::warn!("Syntax error during onPlayerDeath: {err:?}");
        }
    }

    /// A BNpc (`layout_id`, as `actor_id`) began an action (`onActorCast`), for scripts that answer what a boss does:
    /// the Copperbell Mines' Kottos' 28543 sets off its markers and its jumps. Where its target stood comes after
    /// (x, y, z; nil without one): the Thousand Maws of Toto-Rak's last boss's eggs land there; then its target's actor
    /// ID (nil without one): the Navel's Granite Gaol holds the one Titan's Rock Throw was aimed at.
    pub fn on_actor_cast(
        &mut self,
        layout_id: u32,
        action_id: u32,
        actor_id: ObjectId,
        target: Option<(Position, ObjectId)>,
    ) {
        if self.lua.0.globals().get::<Function>("onActorCast").is_err() {
            return;
        }
        let mut run_script = || {
            let mut lua_director = self.create_lua_director();
            let err = self.lua.0.scope(|scope| {
                let data = scope.create_userdata_ref_mut(&mut lua_director)?;
                let func: Function = self.lua.0.globals().get("onActorCast")?;
                let (x, y, z, target_id) = match target {
                    Some((position, id)) => (
                        Some(position.0.x),
                        Some(position.0.y),
                        Some(position.0.z),
                        Some(id.0),
                    ),
                    None => (None, None, None, None),
                };
                func.call::<()>((data, layout_id, action_id, actor_id.0, x, y, z, target_id))?;
                Ok(())
            });
            self.apply_lua_director(lua_director);
            err
        };
        if let Err(err) = run_script() {
            tracing::warn!("Syntax error during onActorCast: {err:?}");
        }
    }

    pub fn on_actor_death(&mut self, bnpc_id: u32, position: Position) {
        // A room with several bosses (`boss_room` called for each with the same wall: Haukke Manor's pair) opens as
        // the last of them falls (Bob's Chronofoil captures of 2026-11-02, four runs: whichever of the two died
        // last.)
        if let Some(boss) = self.bosses.remove(&bnpc_id)
            && !self
                .bosses
                .values()
                .any(|other| other.wall_id == boss.wall_id)
        {
            self.unseal_boss_wall(boss.wall_id, boss.line_id, boss.place_name);
        }

        // Most directors (and FATEs) don't care who dies.
        if self
            .lua
            .0
            .globals()
            .get::<Function>("onActorDeath")
            .is_err()
        {
            return;
        }

        let mut run_script = || {
            let mut lua_director = self.create_lua_director();
            let err = self.lua.0.scope(|scope| {
                let data = scope.create_userdata_ref_mut(&mut lua_director)?;

                let func: Function = self.lua.0.globals().get("onActorDeath")?;

                func.call::<()>((data, bnpc_id, position))?;

                Ok(())
            });
            self.apply_lua_director(lua_director);
            err
        };
        if let Err(err) = run_script() {
            tracing::warn!("Syntax error during onActorDeath: {err:?}");
        }
    }

    pub fn build_var_segment(&self) -> ServerZoneIpcSegment {
        ServerZoneIpcSegment::new(ServerZoneIpcData::DirectorVars {
            handler_id: self.id,
            flag: self.flag,
            branch: 0,
            data: self.data,
            unk1: 0,
            unk2: 0,
            unk3: 0,
            unk4: 0,
        })
    }

    fn create_lua_director(&self) -> LuaDirector {
        LuaDirector {
            data: self.data,
            tasks: Vec::new(),
            bosses: self.bosses.clone(),
            delays: Vec::new(),
        }
    }

    fn apply_lua_director(&mut self, lua: LuaDirector) {
        if self.data != lua.data {
            self.data = lua.data;
            self.tasks.push(LuaDirectorTask::SendVariables {});
        }
        if self.bosses != lua.bosses {
            self.bosses = lua.bosses;
        }
        // Tasks queued after `director:delay(seconds)` wait that long (until the next `delay`; 0 is at once.)
        let now = Instant::now();
        for (index, task) in lua.tasks.into_iter().enumerate() {
            let delay = lua
                .delays
                .iter()
                .rev()
                .find(|(from, _)| *from <= index)
                .map(|(_, seconds)| *seconds)
                .unwrap_or(0.0);
            if delay > 0.0 {
                self.tasks.push(LuaDirectorTask::Later {
                    at: now + Duration::from_secs_f32(delay),
                    task: Box::new(task),
                });
            } else {
                self.tasks.push(task);
            }
        }
    }

    /// The range a boss being fought keeps the party in (`DirectorBoss::arena`): its layout ID.
    pub fn arena(&self) -> Option<u32> {
        self.bosses
            .values()
            .find(|boss| boss.arena != 0 && boss.state != DirectorBossState::Open)
            .map(|boss| boss.arena)
    }

    /// Actually insert tasks to seal the boss wall.
    pub fn seal_boss_wall(&mut self, id: u32, place_name: u32) {
        if self.bosses.values().any(|boss| boss.wall_id == id) {
            self.tasks.push(LuaDirectorTask::LogMessage {
                id: LogMessageType::BossRoomSealed as u32,
                params: vec![place_name],
            });
            self.tasks.push(LuaDirectorTask::ShowEObj { base_id: id });
            for boss in self.bosses.values_mut().filter(|boss| boss.wall_id == id) {
                boss.state = DirectorBossState::Closed;
            }
        }
    }

    /// Actually insert tasks to unseal the boss wall.
    pub fn unseal_boss_wall(&mut self, wall_id: u32, line_id: u32, place_name: u32) {
        self.tasks
            .push(LuaDirectorTask::HideEObj { base_id: wall_id });
        self.tasks
            .push(LuaDirectorTask::HideEObj { base_id: line_id });
        self.tasks.push(LuaDirectorTask::LogMessage {
            id: LogMessageType::BossRoomUnsealed as u32,
            params: vec![place_name],
        });
    }

    /// `id` (a layout ID) went for `target`.
    pub fn on_actor_aggro(&mut self, id: u32, target: ObjectId) {
        let key = if self.bosses.contains_key(&id) {
            Some(id)
        } else if id != 0 {
            self.bosses
                .iter()
                .find(|(_, boss)| boss.triggers.contains(&id))
                .map(|(key, _)| *key)
        } else {
            None
        };
        if let Some(boss) = key.and_then(|key| self.bosses.get_mut(&key))
            && boss.state == DirectorBossState::Open
        {
            // TODO: is there times that are longer than 15 secs?
            self.tasks.push(LuaDirectorTask::LogMessage {
                id: LogMessageType::BossRoomSealing as u32,
                params: vec![boss.place_name, 15],
            });
            // A boss room's boss may not be there yet: the wall's timer goes with whoever was pulled on.
            let actor_id = if boss.actor_id == ObjectId::default() {
                target
            } else {
                boss.actor_id
            };
            let wall_id = boss.wall_id;
            self.tasks.push(LuaDirectorTask::SealBossWall {
                actor_id,
                id: wall_id,
                place_name: boss.place_name,
                time_until: 15,
            });
            // The room's other bosses are sealed in with it.
            for boss in self
                .bosses
                .values_mut()
                .filter(|boss| boss.wall_id == wall_id)
            {
                boss.state = DirectorBossState::Aggro;
            }
        }
    }

    pub fn on_gimmick_rect(&mut self, id: u32) {
        // Skip if the fucntion isn't defined'
        if !self
            .lua
            .0
            .globals()
            .contains_key("onGimmickRect")
            .unwrap_or_default()
        {
            return;
        }

        let mut run_script = || {
            let mut lua_director = self.create_lua_director();
            let err = self.lua.0.scope(|scope| {
                let data = scope.create_userdata_ref_mut(&mut lua_director)?;

                let func: Function = self.lua.0.globals().get("onGimmickRect")?;

                func.call::<()>((data, id))?;

                Ok(())
            });
            self.apply_lua_director(lua_director);
            err
        };
        if let Err(err) = run_script() {
            tracing::warn!("Syntax error during onGimmickRect: {err:?}");
        }
    }

    pub fn get_debug_shortcut(&self, id: u32) -> u32 {
        let run_script = || {
            self.lua.0.scope(|_| {
                let func: Function = self.lua.0.globals().get("getDebugShortcut")?;

                func.call::<u32>(id)
            })
        };
        match run_script() {
            Ok(pop_range_id) => pop_range_id,
            Err(err) => {
                tracing::warn!("Syntax error during getDebugShortcut: {err:?}");

                0
            }
        }
    }

    #[allow(unused)] // TODO: temporary after refactor
    pub fn variant_vote(&mut self, vote: u32) {
        let mut run_script = || {
            let mut lua_director = self.create_lua_director();
            let err = self.lua.0.scope(|scope| {
                let data = scope.create_userdata_ref_mut(&mut lua_director)?;

                let func: Function = self.lua.0.globals().get("onVariantVote")?;

                func.call::<()>((data, vote))?;

                Ok(())
            });
            self.apply_lua_director(lua_director);
            err
        };
        if let Err(err) = run_script() {
            tracing::warn!("Syntax error during onVariantVote: {err:?}");
        }
    }

    pub fn is_gimmick_functional(&self, id: u32) -> bool {
        let run_script = || {
            self.lua.0.scope(|_| {
                let func: Function = self.lua.0.globals().get("isGimmickFunctional")?;

                func.call::<bool>(id)
            })
        };
        match run_script() {
            Ok(value) => value,
            Err(err) => {
                tracing::warn!("Syntax error during isGimmickFunctional: {err:?}");

                true
            }
        }
    }
}

/// Calls the director's `onUpdate` with the health of every living NPC.
fn update_director(instance: &mut Instance) {
    let npc_health: HashMap<u32, u32> = instance
        .actors
        .values()
        .filter_map(|actor| match actor {
            NetworkedActor::Npc { spawn, .. }
                if spawn.common.health_points > 0 && spawn.common.max_health_points > 0 =>
            {
                Some((
                    spawn.common.layout_id,
                    spawn.common.health_points * 100 / spawn.common.max_health_points,
                ))
            }
            _ => None,
        })
        .collect();
    // The player's health too (in percent), e.g. for allies that heal them.
    let player_health = instance.actors.values().find_map(|actor| match actor {
        NetworkedActor::Player { spawn, .. } if spawn.common.max_health_points > 0 => {
            Some(spawn.common.health_points * 100 / spawn.common.max_health_points)
        }
        _ => None,
    });
    // Everyone's real numbers, position and target: living NPCs by layout ID, and "player".
    let layout_of: HashMap<ObjectId, u32> = instance
        .actors
        .iter()
        .filter_map(|(id, actor)| match actor {
            NetworkedActor::Npc { spawn, .. } => Some((*id, spawn.common.layout_id)),
            _ => None,
        })
        .collect();
    let players: Vec<ObjectId> = instance
        .actors
        .iter()
        .filter(|(_, actor)| matches!(actor, NetworkedActor::Player { .. }))
        .map(|(id, _)| *id)
        .collect();
    let mut info = HashMap::new();
    for (actor_id, actor) in &instance.actors {
        let key = match actor {
            NetworkedActor::Npc { spawn, .. } if spawn.common.health_points > 0 => {
                spawn.common.layout_id.to_string()
            }
            NetworkedActor::Player { .. } => "player".to_string(),
            _ => continue,
        };
        let common = actor.get_common_spawn();
        let target = common.target_id.object_id;
        info.entry(key).or_insert(ActorInfo {
            hp: common.health_points,
            max_hp: common.max_health_points,
            position: common.position.0,
            target_layout: layout_of.get(&target).copied(),
            target_is_player: players.contains(&target),
            has_target: target.is_valid(),
            actor_id: actor_id.0,
            target_id: target.0,
        });
    }
    // Every player too, as "player1", "player2"... ("player" is one of them): a duty's line is crossed by whoever
    // reaches it first (the Thousand Maws of Toto-Rak's rooms, another player's in a Duty Finder run.)
    let mut sorted_players = players.clone();
    sorted_players.sort_by_key(|id| id.0);
    for (index, id) in sorted_players.iter().enumerate() {
        let Some(actor) = instance.find_actor(*id) else {
            continue;
        };
        let common = actor.get_common_spawn();
        let target = common.target_id.object_id;
        info.insert(
            format!("player{}", index + 1),
            ActorInfo {
                hp: common.health_points,
                max_hp: common.max_health_points,
                position: common.position.0,
                target_layout: layout_of.get(&target).copied(),
                target_is_player: players.contains(&target),
                has_target: target.is_valid(),
                actor_id: id.0,
                target_id: target.0,
            },
        );
    }
    // The party's NPCs (Duty Support members and bots, which have no layout ID) as "party1", "party2"...: whom an
    // attack on the party can fall on (Ifrit's Eruption, under one of them.)
    let mut party: Vec<ObjectId> = instance.party_bots.keys().copied().collect();
    party.sort_by_key(|id| id.0);
    for (index, id) in party.iter().enumerate() {
        let Some(actor) = instance.find_actor(*id) else {
            continue;
        };
        let common = actor.get_common_spawn();
        if common.health_points == 0 {
            continue;
        }
        let target = common.target_id.object_id;
        info.insert(
            format!("party{}", index + 1),
            ActorInfo {
                hp: common.health_points,
                max_hp: common.max_health_points,
                position: common.position.0,
                target_layout: layout_of.get(&target).copied(),
                target_is_player: players.contains(&target),
                has_target: target.is_valid(),
                actor_id: id.0,
                target_id: target.0,
            },
        );
    }
    if let Some(director) = instance.directors.first_mut() {
        director.update(&npc_health, player_health, &info);
    }
}

/// Perform any queued director tasks
pub fn director_tick(
    network: Arc<Mutex<NetworkState>>,
    instance: &mut Instance,
    game_data: Arc<Mutex<GameData>>,
) {
    // What's attached to what goes with it (`director:attach_bnpc`.)
    keep_attached(&network, instance);

    // Let the script see how the fight is going, if it wants to
    let wants_update = instance
        .directors
        .first()
        .is_some_and(|x| x.lua.0.globals().get::<Function>("onUpdate").is_ok());
    if wants_update {
        update_director(instance);
    }

    // Tasks a script queued for later (`director:delay`) whose time has come.
    if !instance.later_director_tasks.is_empty() {
        let now = Instant::now();
        let (due, later): (Vec<_>, Vec<_>) = std::mem::take(&mut instance.later_director_tasks)
            .into_iter()
            // Within a quarter second (half a tick) counts as due now, so a step comes in the tick nearest its time
            // rather than always the one after (Kottos' markers 2 s apart went out 2.5 and 1.5 s apart.)
            .partition(|(at, _)| *at <= now + Duration::from_millis(250));
        instance.later_director_tasks = later;
        if let Some(director) = instance.directors.first_mut() {
            director.tasks.extend(due.into_iter().map(|(_, task)| task));
        }
    }

    let tasks = if let Some(director) = &instance.directors.first() {
        director.tasks.clone()
    } else {
        return;
    };
    // How many of the director's tasks this tick takes: any a death in it queues (`KillBattleNpc`'s `onActorDeath`)
    // wait for the next.
    let taken = tasks.len();
    // The duty's exits, each shown as an event object.
    let tasks: Vec<LuaDirectorTask> = tasks
        .into_iter()
        .flat_map(|task| match task {
            LuaDirectorTask::ShowExits => instance
                .zone
                .exit_eobjs
                .iter()
                .map(|base_id| LuaDirectorTask::ShowEObj { base_id: *base_id })
                .collect(),
            task => vec![task],
        })
        .collect();

    let mut bosses = if let Some(director) = &instance.directors.first() {
        director.bosses.clone()
    } else {
        return;
    };

    let director_id = instance.directors.first().as_ref().unwrap().id;

    for task in &tasks {
        match task {
            LuaDirectorTask::HideEObj { base_id } => {
                let Some(actor_id) = instance.find_object_by_eobj_or_layout_id(*base_id) else {
                    tracing::warn!("Failed to find eobj {base_id} for HideEObj, it won't despawn!");
                    continue;
                };

                let state = EventState::OFF | EventState::UNK2 | EventState::UNK3;

                let mut network = network.lock();
                network.send_ac_in_range_instance(
                    instance,
                    actor_id,
                    ActorControlCategory::SetEventState { state },
                );

                // Update invisibility flags for next spawn
                if let Some(NetworkedActor::Object { object, .. }) =
                    instance.find_actor_mut(actor_id)
                {
                    object.event_state = state;
                    let layout_id = object.layout_id;
                    // Its barrier or door goes from the collision with it, as for players.
                    instance.set_gimmick_gone(layout_id, true);
                }
            }
            LuaDirectorTask::ShowEObj { base_id } => {
                let Some(actor_id) = instance.find_object_by_eobj_or_layout_id(*base_id) else {
                    tracing::warn!("Failed to find eobj {base_id} for ShowEObj, it won't despawn!");
                    continue;
                };

                let state = EventState::empty();

                let mut network = network.lock();
                network.send_ac_in_range_instance(
                    instance,
                    actor_id,
                    ActorControlCategory::SetEventState { state },
                );

                // Update invisibility flags for next spawn
                if let Some(NetworkedActor::Object { object, .. }) =
                    instance.find_actor_mut(actor_id)
                {
                    object.event_state = state;
                    let layout_id = object.layout_id;
                    // A boss room's wall sealing it: in the collision again, for everyone.
                    instance.seal_gimmick(layout_id);
                }
            }
            LuaDirectorTask::DeleteEObj { base_id } => {
                let Some(actor_id) = instance.find_object_by_eobj_or_layout_id(*base_id) else {
                    tracing::warn!(
                        "Failed to find eobj {base_id} for DeleteEObj, it won't despawn!"
                    );
                    continue;
                };

                if let Some(NetworkedActor::Object { object, .. }) = instance.find_actor(actor_id) {
                    let layout_id = object.layout_id;
                    instance.set_gimmick_gone(layout_id, true);
                }
                let mut network = network.lock();
                network.remove_actor(instance, actor_id);
            }
            LuaDirectorTask::SpawnEObj {
                base_id,
                position,
                args2,
            } => {
                let object = instance.zone.get_event_object(*base_id).or_else(|| {
                    let targetable = game_data.lock().eobj_targetable(*base_id)?;
                    Some(SpawnObject {
                        kind: ObjectKind::EventObj,
                        base_id: *base_id,
                        not_targetable: !targetable,
                        entity_id: ObjectId(fastrand::u32(..)),
                        handler_id: director_id,
                        radius: 1.0,
                        ..Default::default()
                    })
                });
                if let Some(mut object) = object {
                    if let Some(position) = position {
                        object.position = *position;
                    }
                    // A quest battle's object with no event of its own (its EObj's Data 0) is the battle's to answer
                    // when talked to: retail's rosary in In the Eyes of Gods and Men came with the director as its
                    // handler (SpawnObject, Bob's Chronofoil captures 130088ec and 3c56e09f.)
                    if object.handler_id.0 == 0
                        && director_id.handler_type() == HandlerType::QuestBattle
                    {
                        object.handler_id = director_id;
                    }
                    if let Some(args2) = args2 {
                        object.args2 = *args2;
                    }
                    instance.insert_object(object.entity_id, object, String::default()); // TODO: insert layer name
                } else {
                    tracing::warn!("Failed to find eobj {base_id} for SpawnEObj, it won't spawn!");
                }
            }
            LuaDirectorTask::SendVariables => {
                let vars = if let Some(director) = &instance.directors.first() {
                    director.build_var_segment()
                } else {
                    panic!("There's no way this could've happened!");
                };

                let mut network = network.lock();
                for id in instance
                    .actors
                    .iter()
                    .filter(|x| matches!(x.1, NetworkedActor::Player { .. }))
                    .map(|x| *x.0)
                {
                    let Some((handle, _)) = network.get_by_actor_mut(id) else {
                        continue;
                    };

                    let msg = FromServer::PacketSegment(vars.clone(), id);
                    let _ = handle.send(msg.clone()); // TODO: use result
                }
            }
            LuaDirectorTask::AbandonDuty { actor_id } => {
                let mut network = network.lock();
                network.send_to_by_actor_id(
                    *actor_id,
                    FromServer::LeaveContent,
                    DestinationNetwork::ZoneClients,
                );
            }
            LuaDirectorTask::BeginEventAction {
                actor_id,
                target,
                action_id,
            } => {
                let act = ActorControlCategory::EventAction {
                    unk1: 1,
                    id: *action_id,
                };

                let mut network = network.lock();
                network.send_to_by_actor_id(
                    *actor_id,
                    FromServer::ActorControlTarget(
                        *actor_id,
                        ObjectTypeId {
                            object_id: *target,
                            object_type: ObjectTypeKind::None,
                        },
                        act,
                    ),
                    DestinationNetwork::ZoneClients,
                );

                // TODO: set OccupiedInEvent?

                let cast_time = game_data.lock().get_event_action_cast_time(*action_id);
                instance.insert_task(
                    ClientId::default(),
                    *actor_id,
                    Duration::from_secs(cast_time as u64),
                    QueuedTaskData::CastEventAction {
                        target: *target,
                        id: *action_id,
                    },
                );
            }
            LuaDirectorTask::FinishGimmickEvent { actor_id } => {
                let mut network = network.lock();
                network.send_to_by_actor_id(
                    *actor_id,
                    FromServer::FinishEvent,
                    DestinationNetwork::ZoneClients,
                );
            }
            LuaDirectorTask::LogMessage { id, params } => {
                let ipc = EventLogMessage {
                    handler_id: director_id,
                    message_type: *id,
                    params: params.to_vec(),
                }
                .package()
                .unwrap();

                let mut network = network.lock();
                network.send_to_instance(
                    ObjectId::default(),
                    instance,
                    FromServer::PacketSegment(ipc, ObjectId::default()), // TODO: how do we just send it from the player?
                    DestinationNetwork::ZoneClients,
                );
            }
            LuaDirectorTask::SpawnBattleNpc { id, actor_id, at } => {
                if let Some(mut npc) = instance.zone.get_battle_npc(*id) {
                    npc.common.handler_id = director_id;
                    if let Some((position, rotation)) = at {
                        npc.common.position = *position;
                        if let Some(rotation) = rotation {
                            npc.common.rotation = *rotation;
                        }
                    }
                    let config = get_config();
                    let mut game_data = game_data.lock();
                    instance.insert_npc(*actor_id, npc, &config, &mut game_data);
                } else {
                    tracing::warn!("Failed to find bnpc {id} for SpawnBattleNpc, it won't spawn!");
                }
            }
            LuaDirectorTask::SpawnTetheredBattleNpc { id, to, channeling } => {
                if let Some(mut npc) = instance.zone.get_battle_npc(*id) {
                    npc.common.handler_id = director_id;
                    match instance.find_npc(*to) {
                        Some((target, _)) => {
                            // Retail's spawn carried the tether: its target and the Channeling row (Bob's Tam-Tara
                            // captures of 2026-10-24: every shield, tether 1 to Galvanth.)
                            npc.common.tether_target_id = target;
                            npc.common.tether_id = *channeling;
                        }
                        None => tracing::warn!(
                            "Bnpc {to}, which {id} is tethered to, isn't there; it spawns without the tether"
                        ),
                    }
                    let config = get_config();
                    let mut game_data = game_data.lock();
                    instance.insert_npc(ObjectId(fastrand::u32(..)), npc, &config, &mut game_data);
                } else {
                    tracing::warn!(
                        "Failed to find bnpc {id} for SpawnTetheredBattleNpc, it won't spawn!"
                    );
                }
            }
            LuaDirectorTask::DespawnBattleNpc { layout_id } => {
                // An actor ID (from `spawn_bnpc`) or a layout ID.
                if let Some(actor_id) = resolve_bnpc(instance, *layout_id) {
                    network.lock().remove_actor(instance, actor_id);
                }
            }
            LuaDirectorTask::GainEffect {
                actor_id,
                id,
                param,
                duration,
            } => {
                gain_effect_instance(
                    network.clone(),
                    ClientId::default(),
                    instance,
                    *actor_id,
                    *id,
                    *param,
                    *duration,
                    ObjectId::default(),
                    // Everyone sees it come (ActorControl GainEffect and the status list), as retail showed Ifrit's
                    // 775 and the Navel's gaoled players' 292 (Bob's Chronofoil captures b8432613 and 650dbfd8.)
                    true,
                );
            }
            LuaDirectorTask::LoseEffect { actor_id, id } => {
                lose_effect_instance(network.clone(), instance, *actor_id, *id);
            }
            LuaDirectorTask::AttachBattleNpc { who, to, distance } => {
                let (Some(child), Some(parent)) =
                    (resolve_bnpc(instance, *who), resolve_bnpc(instance, *to))
                else {
                    tracing::warn!("Failed to find bnpc {who} or {to} to attach!");
                    continue;
                };
                instance.attached_npcs.insert(child, (parent, *distance));
                keep_attached(&network, instance);
            }
            LuaDirectorTask::SetBGM { id } => {
                let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorControlSelf(
                    ActorControlSelf {
                        category: ActorControlCategory::DirectorEvent {
                            handler_id: director_id,
                            event: DirectorEvent::ContentDirector(ContentDirectorEvent::SetBGM {
                                bgm: *id,
                            }),
                        },
                    },
                ));

                let mut network = network.lock();
                network.send_to_instance(
                    ObjectId::default(),
                    instance,
                    FromServer::PacketSegment(ipc, ObjectId::default()), // TODO: how do we just send it from the player?
                    DestinationNetwork::ZoneClients,
                );
            }
            LuaDirectorTask::SealBossWall {
                actor_id,
                id,
                place_name,
                time_until,
            } => {
                instance.insert_task(
                    ClientId::default(),
                    *actor_id,
                    Duration::from_secs(*time_until as u64),
                    QueuedTaskData::SealBossWall {
                        id: *id,
                        place_name: *place_name,
                    },
                );
            }
            LuaDirectorTask::SpawnBoss {
                bnpc_id,
                wall_id,
                line_id,
                place_name,
                arena,
            } => {
                if let Some(mut npc) = instance.zone.get_battle_npc(*bnpc_id) {
                    npc.common.handler_id = director_id;

                    let actor_id = ObjectId(fastrand::u32(..));
                    let config = get_config();
                    let mut game_data = game_data.lock();
                    instance.insert_npc(actor_id, npc, &config, &mut game_data);
                    warn_if_no_arena(instance, *arena);
                    bosses.insert(
                        *bnpc_id,
                        DirectorBoss {
                            state: DirectorBossState::Open,
                            actor_id,
                            wall_id: *wall_id,
                            line_id: *line_id,
                            place_name: *place_name,
                            triggers: [0; 8],
                            arena: *arena,
                        },
                    );
                } else {
                    tracing::warn!("Failed to find bnpc {bnpc_id} for SpawnBoss, it won't spawn!");
                }
            }
            LuaDirectorTask::BossRoom {
                bnpc_id,
                wall_id,
                line_id,
                place_name,
                triggers,
                arena,
            } => {
                let mut ids = [0; 8];
                for (slot, id) in ids.iter_mut().zip(triggers) {
                    *slot = *id;
                }
                warn_if_no_arena(instance, *arena);
                bosses.insert(
                    *bnpc_id,
                    DirectorBoss {
                        state: DirectorBossState::Open,
                        actor_id: ObjectId::default(),
                        wall_id: *wall_id,
                        line_id: *line_id,
                        place_name: *place_name,
                        triggers: ids,
                        arena: *arena,
                    },
                );
            }
            LuaDirectorTask::SetInvulnerable {
                layout_id,
                invulnerable,
            } => {
                let Some((actor_id, _)) = instance.find_npc(*layout_id) else {
                    tracing::warn!("Failed to find bnpc {layout_id} for SetInvulnerable!");
                    continue;
                };
                if let Some(NetworkedActor::Npc {
                    currently_invulnerable,
                    ..
                }) = instance.find_actor_mut(actor_id)
                {
                    *currently_invulnerable = *invulnerable;
                }
            }
            LuaDirectorTask::TreasureLoot { base_id, openings } => {
                instance.loot_pools.insert(*base_id, openings.clone());
            }
            LuaDirectorTask::TreasurePool { base_id, items } => {
                instance.loot_wiki_pools.insert(*base_id, items.clone());
            }
            LuaDirectorTask::SpawnTreasure { id } => {
                if let Some(mut treasure) = instance.zone.get_treasure(*id) {
                    treasure.handler_id = director_id;
                    // A duty's coffer, closed (retail's: kind 2, event state 1; Bob's Chronofoil capture of Sastasha.)
                    treasure.coffer_kind = TreasureKind::DungeonRaid;
                    treasure.event_state = EventState::OFF;

                    // The client opens it by the ID in its spawn, so that's its actor ID too (they were two random
                    // numbers, and nothing could be opened.)
                    let actor_id = treasure.entity_id;
                    instance.insert_treasure(actor_id, treasure);
                } else {
                    tracing::warn!(
                        "Failed to find treasure {id} for SpawnTreasure, it won't spawn!"
                    );
                }
            }
            LuaDirectorTask::VariantVoteRoute { .. } => {
                // TODO: Restore, this was removed after the refactor
                // let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorControlSelf(
                //     ActorControlSelf {
                //         category: ActorControlCategory::DirectorEvent {
                //             handler_id: director_id,
                //             event: DirectorEvent::VariantVoteRoute {
                //                 votes_needed: 1, // TODO: set to the number of players in the instance
                //                 npc_route: *npc_route,
                //             },
                //         },
                //     },
                // ));
                //
                // let mut network = network.lock();
                // network.send_to_instance(
                //     ObjectId::default(),
                //     instance,
                //     FromServer::PacketSegment(ipc, ObjectId::default()), // TODO: how do we just send it from the player?
                //     DestinationNetwork::ZoneClients,
                // );
            }
            LuaDirectorTask::CompleteQuestBattle {
                sequence,
                stay,
                finish_scene,
                finish_flags,
                variables,
                fanfare,
            } => {
                let quest_id = instance.directors.first().unwrap().quest_battle_quest_id;
                let exit = instance.quest_battle_win_exit;
                let mut network = network.lock();
                // Won: no one fights on through "Duty Complete" and the closing scene.
                crate::server::npc_behavior::end_fight(instance, &mut network);
                network.send_to_instance(
                    ObjectId::default(),
                    instance,
                    FromServer::QuestBattleComplete(
                        director_id,
                        quest_id,
                        *sequence,
                        *stay,
                        *finish_scene,
                        *finish_flags,
                        variables.clone(),
                        *fanfare,
                        exit,
                    ),
                    DestinationNetwork::ZoneClients,
                );
            }
            LuaDirectorTask::EndFight => {
                let mut network = network.lock();
                crate::server::npc_behavior::end_fight(instance, &mut network);
            }
            LuaDirectorTask::PlayScene { scene, flags } => {
                let mut network = network.lock();
                network.send_to_instance(
                    ObjectId::default(),
                    instance,
                    FromServer::PlayDirectorScene(director_id, *scene, *flags),
                    DestinationNetwork::ZoneClients,
                );
            }
            LuaDirectorTask::WarpPlayers { position, rotation } => {
                let players: Vec<ObjectId> = instance
                    .actors
                    .iter()
                    .filter(|(_, actor)| matches!(actor, NetworkedActor::Player { .. }))
                    .map(|(id, _)| *id)
                    .collect();
                let mut network = network.lock();
                for actor_id in players {
                    // As retail sent it: a normal warp, the character hidden, the usual fade.
                    let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::PrepareZoning(
                        PrepareZoning {
                            territory_type_id: instance.zone.id,
                            warp_type: WarpType::Normal,
                            hide_character: 1,
                            fade_out_delay: 1,
                            ..Default::default()
                        },
                    ));
                    network.send_to_by_actor_id(
                        actor_id,
                        FromServer::PacketSegment(ipc, actor_id),
                        DestinationNetwork::ZoneClients,
                    );
                    let Some(client_id) = network
                        .clients
                        .iter()
                        .find(|(_, (handle, _))| handle.actor_id == actor_id)
                        .map(|(id, _)| *id)
                    else {
                        continue;
                    };
                    let segment =
                        ServerZoneIpcSegment::new(ServerZoneIpcData::ActorSetPos(ActorSetPos {
                            position: *position,
                            rotation: *rotation,
                            warp_type: WarpType::Normal,
                            arg: 2,
                            ..Default::default()
                        }));
                    instance.insert_task(
                        client_id,
                        actor_id,
                        WARP_DELAY,
                        QueuedTaskData::PacketSegment { segment },
                    );
                }
            }
            LuaDirectorTask::LeaveQuestBattle => {
                let mut network = network.lock();
                network.send_to_instance(
                    ObjectId::default(),
                    instance,
                    FromServer::LeaveQuestBattle(director_id),
                    DestinationNetwork::ZoneClients,
                );
            }
            LuaDirectorTask::UseAction {
                layout_id,
                action_id,
                target,
            } => {
                let Some((actor_id, _)) = instance.find_npc(*layout_id) else {
                    tracing::warn!("Failed to find bnpc {layout_id} for UseAction!");
                    continue;
                };
                let target_id = match target {
                    Some(target) => instance.find_npc(*target).map(|(id, _)| id),
                    None => instance
                        .actors
                        .iter()
                        .find(|(_, actor)| matches!(actor, NetworkedActor::Player { .. }))
                        .map(|(id, _)| *id),
                };
                let Some(target_id) = target_id else {
                    continue;
                };
                crate::server::npc_behavior::npc_use_action(
                    &network, &game_data, instance, actor_id, *action_id, target_id,
                );
            }
            LuaDirectorTask::Say { .. } | LuaDirectorTask::SayBattleTalk { .. } => {
                let (layout_id, name_override) = match task {
                    LuaDirectorTask::Say { layout_id, .. } => (*layout_id, None),
                    LuaDirectorTask::SayBattleTalk {
                        layout_id, name_id, ..
                    } => (*layout_id, *name_id),
                    _ => unreachable!(),
                };
                // Someone who isn't around (e.g. the golem once it's gone) still gets their line said.
                let actor_id = instance
                    .find_npc(layout_id)
                    .map(|(actor_id, _)| actor_id)
                    .unwrap_or_default();
                let name_id = name_override.unwrap_or_else(|| {
                    instance
                        .find_actor(actor_id)
                        .map(|actor| actor.get_common_spawn().name_id)
                        .unwrap_or_default()
                });
                let quest_id = instance
                    .directors
                    .first()
                    .map(|director| director.quest_battle_quest_id)
                    .unwrap_or_default();
                let (name, text) = {
                    let mut game_data = game_data.lock();
                    let text = match task {
                        LuaDirectorTask::Say { key, .. } => game_data.get_quest_text(key),
                        LuaDirectorTask::SayBattleTalk { number, .. } => game_data
                            .get_quest_battle_talk(quest_id)
                            .get(number.saturating_sub(1))
                            .map(|(_, text)| text.clone()),
                        _ => unreachable!(),
                    };
                    // BNpcName 0 is a speaker whose name isn't given yet, shown as "???" like on retail.
                    let name = if name_id == 0 {
                        Some("???".to_string())
                    } else {
                        game_data.get_bnpc_name(name_id)
                    };
                    (name, text)
                };
                let Some(text) = text else {
                    tracing::warn!("Couldn't find the quest text for {task:?}!");
                    continue;
                };

                let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ChatMessage(ChatMessage {
                    sender_actor_id: actor_id,
                    channel: ChatChannelType::Say,
                    // BNpcName is lower case ("short-tempered thaumaturge"); retail shows "Short-tempered Thaumaturge".
                    sender_name: capitalize_words(&name.unwrap_or_default()),
                    message: text,
                    ..Default::default()
                }));
                let mut network = network.lock();
                network.send_to_instance(
                    ObjectId::default(),
                    instance,
                    FromServer::PacketSegment(ipc, actor_id),
                    DestinationNetwork::ZoneClients,
                );
            }
            LuaDirectorTask::BattleTalk {
                layout_id,
                name_id,
                text_id,
                icon,
                duration_ms,
                kind,
            } => {
                let actor_id = instance
                    .find_npc(*layout_id)
                    .map(|(actor_id, _)| actor_id)
                    .unwrap_or_default();
                let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::DirectorPopupMessage {
                    actor_id: ObjectTypeId {
                        object_id: actor_id,
                        object_type: ObjectTypeKind::None,
                    },
                    handler_id: director_id,
                    npc_name: *name_id,
                    text_data_id: *text_id,
                    duration_msecs: *duration_ms,
                    icon: *icon,
                    kind: *kind,
                    style: 0,
                    params_count: 0,
                    param1: 0,
                    param2: 0,
                });
                let mut network = network.lock();
                network.send_to_instance(
                    ObjectId::default(),
                    instance,
                    FromServer::PacketSegment(ipc, actor_id),
                    DestinationNetwork::ZoneClients,
                );
            }
            LuaDirectorTask::PlayActionTimeline {
                layout_id,
                timeline_id,
            } => {
                let Some((actor_id, _)) = instance.find_npc(*layout_id) else {
                    continue;
                };
                let mut network = network.lock();
                network.send_ac_in_range_instance(
                    instance,
                    actor_id,
                    ActorControlCategory::PlayActionTimeline {
                        timeline_id: *timeline_id,
                    },
                );
            }
            LuaDirectorTask::PartyNpcYell {
                role,
                npc_yell_id,
                delay,
                otherwise,
            } => {
                crate::server::party_npc::queue_yell(
                    instance,
                    role,
                    *npc_yell_id,
                    *delay,
                    otherwise.clone(),
                );
            }
            LuaDirectorTask::Yell {
                layout_id,
                npc_yell_id,
            } => {
                let Some((actor_id, _)) = instance.find_npc(*layout_id) else {
                    continue;
                };
                let name_id = game_data
                    .lock()
                    .get_npc_yell_name_id(*npc_yell_id)
                    .unwrap_or_default();
                let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::NpcYell {
                    object_id: ObjectTypeId {
                        object_id: actor_id,
                        object_type: ObjectTypeKind::None,
                    },
                    name_id,
                    npc_yell_id: *npc_yell_id,
                    param1: 0,
                    param2: 0,
                    param3: 0,
                    param4: 0,
                });
                let mut network = network.lock();
                network.send_to_instance(
                    ObjectId::default(),
                    instance,
                    FromServer::PacketSegment(ipc, actor_id),
                    DestinationNetwork::ZoneClients,
                );
            }
            LuaDirectorTask::SetSequence { sequence } => {
                let vars = if let Some(director) = instance.directors.first_mut() {
                    director.flag = *sequence;
                    director.build_var_segment()
                } else {
                    continue;
                };

                let mut network = network.lock();
                for id in instance
                    .actors
                    .iter()
                    .filter(|x| matches!(x.1, NetworkedActor::Player { .. }))
                    .map(|x| *x.0)
                {
                    let Some((handle, _)) = network.get_by_actor_mut(id) else {
                        continue;
                    };
                    let _ = handle.send(FromServer::PacketSegment(vars.clone(), id));
                }
            }
            LuaDirectorTask::FailQuestBattle => {
                let mut network = network.lock();
                network.send_to_instance(
                    ObjectId::default(),
                    instance,
                    FromServer::QuestBattleFailed,
                    DestinationNetwork::ZoneClients,
                );
            }
            LuaDirectorTask::SetNotice { layout_id, notices } => {
                if *notices {
                    instance.unaware_npcs.remove(layout_id);
                } else {
                    instance.unaware_npcs.insert(*layout_id);
                }
            }
            LuaDirectorTask::SetAttackRange { layout_id, yalms } => {
                let Some((actor_id, _)) = instance.find_npc(*layout_id) else {
                    tracing::warn!("Failed to find bnpc {layout_id} for SetAttackRange!");
                    continue;
                };
                if let Some(NetworkedActor::Npc { timeline, .. }) =
                    instance.find_actor_mut(actor_id)
                {
                    timeline.attack_range = yalms.max(0.5);
                }
            }
            LuaDirectorTask::KeepNear {
                layout_id,
                anchors,
                yalms,
            } => {
                let player = instance
                    .actors
                    .iter()
                    .find(|(_, actor)| matches!(actor, NetworkedActor::Player { .. }))
                    .map(|(id, _)| *id);
                let anchor_ids: Vec<ObjectId> = anchors
                    .iter()
                    .filter_map(|anchor| match anchor {
                        Some(anchor) => instance.find_npc(*anchor).map(|(id, _)| id),
                        None => player,
                    })
                    .collect();
                let Some((actor_id, _)) = instance.find_npc(*layout_id) else {
                    tracing::warn!("Failed to find bnpc {layout_id} for KeepNear!");
                    continue;
                };
                if let Some(NetworkedActor::Npc { placement, .. }) =
                    instance.find_actor_mut(actor_id)
                {
                    match yalms {
                        Some(yalms) if !anchor_ids.is_empty() => {
                            placement.anchors = anchor_ids;
                            placement.within = *yalms;
                        }
                        _ => placement.anchors.clear(),
                    }
                }
            }
            LuaDirectorTask::StandBehind {
                layout_id,
                tank,
                yalms,
            } => {
                let tank_id = instance.find_npc(*tank).map(|(id, _)| id);
                let Some((actor_id, _)) = instance.find_npc(*layout_id) else {
                    tracing::warn!("Failed to find bnpc {layout_id} for StandBehind!");
                    continue;
                };
                if let Some(NetworkedActor::Npc { placement, .. }) =
                    instance.find_actor_mut(actor_id)
                {
                    placement.behind = tank_id.zip(*yalms);
                }
            }
            LuaDirectorTask::SetHate {
                layout_id,
                target,
                amount,
            } => {
                let target_id = match target {
                    Some(target) => instance.find_npc(*target).map(|(id, _)| id),
                    None => instance
                        .actors
                        .iter()
                        .find(|(_, actor)| matches!(actor, NetworkedActor::Player { .. }))
                        .map(|(id, _)| *id),
                };
                let Some(target_id) = target_id else {
                    tracing::warn!(
                        "Failed to find {target:?} for SetHate, {layout_id} won't attack!"
                    );
                    continue;
                };
                // An actor ID (from `spawn_bnpc`, for one of several sharing a layout ID) or a layout ID.
                let Some(actor_id) = resolve_bnpc(instance, *layout_id) else {
                    tracing::warn!("Failed to find bnpc {layout_id} for SetHate!");
                    continue;
                };

                // Picked up on the next NPC tick, the same way as being attacked.
                if let Some(actor) = instance.find_actor_mut(actor_id) {
                    actor.add_enmity(target_id, *amount);
                }
            }
            LuaDirectorTask::PlayCutscene {
                cutscene_id,
                params,
            } => {
                let mut network = network.lock();
                network.send_to_instance(
                    ObjectId::default(),
                    instance,
                    FromServer::PlayDirectorCutscene(*cutscene_id, director_id, params.clone()),
                    DestinationNetwork::ZoneClients,
                );
            }
            LuaDirectorTask::UpdateShortcut { poprange_id } => {
                instance.directors.first_mut().unwrap().shortcut_poprange_id = Some(*poprange_id);
            }
            LuaDirectorTask::UseShortcut {
                actor_id,
                pop_range_id,
            } => {
                let id = pop_range_id.or_else(|| {
                    instance
                        .directors
                        .first()
                        .and_then(|director| director.shortcut_poprange_id)
                });
                let Some(id) = id else {
                    tracing::warn!(
                        "The shortcut was used before the director pointed it anywhere!"
                    );
                    continue;
                };
                instance.insert_task(
                    ClientId::default(),
                    *actor_id,
                    Duration::from_secs(0),
                    QueuedTaskData::WarpToPopRange { id },
                );
            }
            LuaDirectorTask::CompleteDuty => {
                // The clock stops: no more time-left re-sends.
                instance.duty_clock = None;
                let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorControlSelf(
                    ActorControlSelf {
                        category: ActorControlCategory::DirectorEvent {
                            handler_id: director_id,
                            event: DirectorEvent::ContentDirector(
                                ContentDirectorEvent::InstanceContent(
                                    InstanceContentDirectorEvent::DutyCompleted {
                                        arg1: 0,
                                        arg2: 0,
                                        arg3: 0,
                                        arg4: 0,
                                    },
                                ),
                            ),
                        },
                    },
                ));

                let mut network = network.lock();
                network.send_to_instance(
                    ObjectId::default(),
                    instance,
                    FromServer::PacketSegment(ipc, ObjectId::default()), // TODO: how do we just send it from the player?
                    DestinationNetwork::ZoneClients,
                );
                // Coffers' items still to be rolled for are decided now (retail did it as the duty ended.)
                crate::server::loot::decide_all(&mut network, instance);

                // Retail took the Duty Support members away as the duty was cleared (`party_npc::dismiss`.)
                crate::server::party_npc::dismiss(&mut network, instance);

                // Their quests hear of it as they leave (quest 245 moves on then on retail).
                network.send_to_instance(
                    ObjectId::default(),
                    instance,
                    FromServer::DutyCleared(instance.content_id, !instance.newcomers.is_empty()),
                    DestinationNetwork::ZoneClients,
                );
            }
            LuaDirectorTask::MapEffect {
                index,
                timeline_id,
                state,
            } => {
                let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::DirectorMapEffect {
                    handler_id: director_id,
                    state: *state,
                    timeline_id: *timeline_id as u16,
                    index: *index as u8,
                });

                let mut network = network.lock();
                network.send_to_instance(
                    ObjectId::default(),
                    instance,
                    FromServer::PacketSegment(ipc, ObjectId::default()), // TODO: how do we just send it from the player?
                    DestinationNetwork::ZoneClients,
                );
            }
            LuaDirectorTask::MapEffectAt {
                layout_id,
                timeline_id,
                state,
                otherwise,
            } => {
                let index = game_data
                    .lock()
                    .get_map_effects(instance.content_id as u32)
                    .and_then(|ids| ids.iter().position(|id| *id as u32 == *layout_id))
                    .map(|index| index as u32)
                    .or(*otherwise);
                let Some(index) = index else {
                    tracing::warn!("No map effect for layout {layout_id} in this duty!");
                    continue;
                };
                let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::DirectorMapEffect {
                    handler_id: director_id,
                    state: *state,
                    timeline_id: *timeline_id as u16,
                    index: index as u8,
                });
                network.lock().send_to_instance(
                    ObjectId::default(),
                    instance,
                    FromServer::PacketSegment(ipc, ObjectId::default()),
                    DestinationNetwork::ZoneClients,
                );
            }
            LuaDirectorTask::Later { at, task } => {
                instance.later_director_tasks.push((*at, (**task).clone()));
            }
            LuaDirectorTask::WarpBattleNpc { who, position, .. }
            | LuaDirectorTask::JumpBattleNpc { who, position, .. }
            | LuaDirectorTask::MoveBattleNpc { who, position, .. } => {
                let Some(actor_id) = resolve_bnpc(instance, *who) else {
                    tracing::warn!("Failed to find bnpc {who} to move!");
                    continue;
                };
                let rotation = &match task {
                    LuaDirectorTask::WarpBattleNpc { rotation, .. }
                    | LuaDirectorTask::MoveBattleNpc { rotation, .. } => *rotation,
                    LuaDirectorTask::JumpBattleNpc { rotation, .. } => {
                        rotation.unwrap_or_else(|| {
                            // The way it leapt.
                            let from = instance
                                .find_actor(actor_id)
                                .map(|actor| actor.position().0)
                                .unwrap_or(position.0);
                            if (position.0 - from).length() < 0.05 {
                                instance
                                    .find_actor(actor_id)
                                    .map(|actor| actor.rotation())
                                    .unwrap_or(0.0)
                            } else {
                                f32::atan2(position.0.x - from.x, position.0.z - from.z)
                            }
                        })
                    }
                    _ => unreachable!(),
                };
                if let Some(NetworkedActor::Npc {
                    spawn,
                    navmesh_path,
                    navmesh_path_lerp,
                    last_position,
                    ..
                }) = instance.find_actor_mut(actor_id)
                {
                    spawn.common.position = *position;
                    spawn.common.rotation = *rotation;
                    navmesh_path.clear();
                    *navmesh_path_lerp = 0.0;
                    *last_position = Some(position.0);
                }
                let mut network = network.lock();
                if matches!(task, LuaDirectorTask::WarpBattleNpc { .. }) {
                    let ipc =
                        ServerZoneIpcSegment::new(ServerZoneIpcData::ActorSetPos(ActorSetPos {
                            position: *position,
                            rotation: *rotation,
                            ..Default::default()
                        }));
                    network.send_to_instance(
                        ObjectId::default(),
                        instance,
                        FromServer::PacketSegment(ipc, actor_id),
                        DestinationNetwork::ZoneClients,
                    );
                } else {
                    if matches!(task, LuaDirectorTask::JumpBattleNpc { .. }) {
                        network.send_ac_in_range_instance(
                            instance,
                            actor_id,
                            ActorControlCategory::SnapNextMove,
                        );
                    }
                    network.send_in_range_instance(
                        actor_id,
                        instance,
                        FromServer::ActorMove(
                            actor_id,
                            *position,
                            *rotation,
                            kawari::common::MoveAnimationType::empty(),
                            kawari::common::MoveAnimationState::None,
                            kawari::common::JumpState::NoneOrFalling,
                            Some(match task {
                                LuaDirectorTask::MoveBattleNpc {
                                    speed: Some(speed), ..
                                } => *speed,
                                _ => JUMP_MOVE_SPEED,
                            }),
                        ),
                        DestinationNetwork::ZoneClients,
                    );
                }
            }
            LuaDirectorTask::SetModelScale {
                who,
                model_scale_id,
            } => {
                let Some(actor_id) = resolve_bnpc(instance, *who) else {
                    tracing::warn!("Failed to find bnpc {who} for SetModelScale!");
                    continue;
                };
                if let Some(NetworkedActor::Npc { spawn, .. }) = instance.find_actor_mut(actor_id) {
                    spawn.common.model_scale_id = *model_scale_id as u8;
                }
                network.lock().send_ac_in_range_instance(
                    instance,
                    actor_id,
                    ActorControlCategory::SetModelScale {
                        model_scale_id: *model_scale_id,
                    },
                );
            }
            LuaDirectorTask::FadeOutBattleNpc { who } => {
                let Some(actor_id) = resolve_bnpc(instance, *who) else {
                    tracing::warn!("Failed to find bnpc {who} to fade out!");
                    continue;
                };
                network.lock().send_ac_in_range_instance(
                    instance,
                    actor_id,
                    ActorControlCategory::FadeOut {
                        actor: actor_id,
                        kind: 1,
                        unk: 0,
                        time: 100,
                    },
                );
            }
            LuaDirectorTask::SetTargetable { who, targetable } => {
                let Some(actor_id) = resolve_bnpc(instance, *who) else {
                    tracing::warn!("Failed to find bnpc {who} for SetTargetable!");
                    continue;
                };
                if *targetable {
                    instance.untargetable_npcs.remove(&actor_id);
                } else {
                    instance.untargetable_npcs.insert(actor_id);
                }
                network.lock().send_ac_in_range_instance(
                    instance,
                    actor_id,
                    ActorControlCategory::SetTargetable {
                        targetable: *targetable,
                    },
                );
            }
            LuaDirectorTask::BattleNpcControl {
                who,
                category,
                params,
            } => {
                let Some(actor_id) = resolve_bnpc(instance, *who) else {
                    tracing::warn!("Failed to find bnpc {who} for BattleNpcControl!");
                    continue;
                };
                network.lock().send_ac_in_range_instance(
                    instance,
                    actor_id,
                    ActorControlCategory::Unknown {
                        category: *category,
                        param1: params[0],
                        param2: params[1],
                        param3: params[2],
                        param4: params[3],
                        param5: 0,
                    },
                );
            }
            LuaDirectorTask::KillBattleNpc { who } => {
                let Some(actor_id) = resolve_bnpc(instance, *who) else {
                    tracing::warn!("Failed to find bnpc {who} for KillBattleNpc!");
                    continue;
                };
                let alive = instance.find_actor_mut(actor_id).is_some_and(|actor| {
                    let common = actor.get_common_spawn_mut();
                    let alive = common.health_points > 0;
                    common.health_points = 0;
                    alive
                });
                if alive {
                    crate::server::actor::kill_actor(network.clone(), instance, actor_id);
                }
            }
            LuaDirectorTask::Objective { done, next } => {
                let mut events = vec![ContentDirectorEvent::ObjectiveDone {
                    index: *done,
                    done: 1,
                }];
                if let Some(next) = next {
                    events.push(ContentDirectorEvent::SetObjective {
                        index: *next,
                        unk: 0,
                    });
                }
                let mut network = network.lock();
                for event in events {
                    let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorControlSelf(
                        ActorControlSelf {
                            category: ActorControlCategory::DirectorEvent {
                                handler_id: director_id,
                                event: DirectorEvent::ContentDirector(event),
                            },
                        },
                    ));
                    network.send_to_instance(
                        ObjectId::default(),
                        instance,
                        FromServer::PacketSegment(ipc, ObjectId::default()),
                        DestinationNetwork::ZoneClients,
                    );
                }
            }
            LuaDirectorTask::SetWeather { id, fade, forced } => {
                instance.weather_id = *id as u16;
                let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::WeatherId(WeatherChange {
                    weather_id: *id,
                    disables_override: *forced,
                    fade_length: *fade,
                    ..Default::default()
                }));
                network.lock().send_to_instance(
                    ObjectId::default(),
                    instance,
                    FromServer::PacketSegment(ipc, ObjectId::default()),
                    DestinationNetwork::ZoneClients,
                );
            }
            // Expanded into `ShowEObj`s above.
            LuaDirectorTask::ShowExits => {}
            LuaDirectorTask::RetreatBattleNpc { who, to } => {
                let Some(actor_id) = resolve_bnpc(instance, *who) else {
                    tracing::warn!("Failed to find bnpc {who} to send back!");
                    continue;
                };
                let cancelled = instance.cancel_actor_casts(actor_id);
                if let Some(NetworkedActor::Npc {
                    spawn,
                    state,
                    enmity,
                    navmesh_target,
                    navmesh_path,
                    navmesh_path_lerp,
                    last_position,
                    returning_home,
                    home,
                    cast,
                    ..
                }) = instance.find_actor_mut(actor_id)
                {
                    if *state == crate::server::NpcState::Dead {
                        continue;
                    }
                    if let Some(to) = to {
                        *home = *to;
                    }
                    enmity.clear();
                    *state = crate::server::NpcState::natural_state_of(spawn);
                    *navmesh_target = Some(crate::server::actor::NpcTarget::Position(*home));
                    navmesh_path.clear();
                    *navmesh_path_lerp = 0.0;
                    *last_position = Some(spawn.common.position.0);
                    *returning_home = true;
                    *cast = None;
                    spawn.common.target_id = ObjectTypeId::default();
                }
                let mut network = network.lock();
                for action_id in cancelled {
                    network.send_ac_in_range_inclusive_instance(
                        instance,
                        actor_id,
                        ActorControlCategory::CancelCast {
                            unk1: RETREAT_CANCEL,
                            unk2: 1,
                            action_id,
                            unk3: 0,
                        },
                    );
                }
                network.send_in_range_instance(
                    actor_id,
                    instance,
                    FromServer::ActorControlTarget(
                        actor_id,
                        ObjectTypeId::default(),
                        ActorControlCategory::SetTarget,
                    ),
                    DestinationNetwork::ZoneClients,
                );
            }
            LuaDirectorTask::ReleaseBattleNpc { who } => {
                let Some(actor_id) = resolve_bnpc(instance, *who) else {
                    continue;
                };
                network.lock().send_ac_in_range_instance(
                    instance,
                    actor_id,
                    ActorControlCategory::SetCompanionOwnerId {
                        new_id: ObjectId(0xE000_0000),
                    },
                );
            }
            LuaDirectorTask::RootBattleNpc { who, seconds } => {
                let Some(actor_id) = resolve_bnpc(instance, *who) else {
                    tracing::warn!("Failed to find bnpc {who} to keep where it stands!");
                    continue;
                };
                instance.rooted_npcs.insert(
                    actor_id,
                    Instant::now() + Duration::from_secs_f32(seconds.max(0.0)),
                );
            }
            LuaDirectorTask::BusyBattleNpc { who, seconds } => {
                let Some(actor_id) = resolve_bnpc(instance, *who) else {
                    tracing::warn!("Failed to find bnpc {who} to keep busy!");
                    continue;
                };
                instance.busy_npcs.insert(
                    actor_id,
                    Instant::now() + Duration::from_secs_f32(seconds.max(0.0)),
                );
            }
            LuaDirectorTask::RestartTimeline { who, anchor } => {
                let Some(actor_id) = resolve_bnpc(instance, *who) else {
                    tracing::warn!("Failed to find bnpc {who} to restart its timeline!");
                    continue;
                };
                if let Some(NetworkedActor::Npc {
                    timeline_anchor,
                    timeline_position,
                    timeline_checked,
                    ..
                }) = instance.find_actor_mut(actor_id)
                {
                    *timeline_anchor = Some(*anchor);
                    *timeline_position = 0;
                    *timeline_checked = 0;
                }
            }
            LuaDirectorTask::HoldBattleNpc { who, held } => {
                let Some(actor_id) = resolve_bnpc(instance, *who) else {
                    tracing::warn!("Failed to find bnpc {who} to hold!");
                    continue;
                };
                if *held {
                    instance.held_npcs.insert(actor_id);
                } else {
                    instance.held_npcs.remove(&actor_id);
                }
            }
            LuaDirectorTask::ActorUseAction {
                actor_id,
                action_id,
                target,
            } => {
                let actor_id = ObjectId(*actor_id);
                if instance.find_actor(actor_id).is_none() {
                    tracing::warn!("Failed to find actor {actor_id} for ActorUseAction!");
                    continue;
                }
                let target_id = target.map(ObjectId).unwrap_or(actor_id);
                crate::server::npc_behavior::npc_use_action(
                    &network, &game_data, instance, actor_id, *action_id, target_id,
                );
            }
            LuaDirectorTask::AppearBattleNpc {
                layout_id,
                action_timeline,
                delay,
            } => {
                instance.pending_appearances.push(PendingAppearance {
                    at: std::time::Instant::now() + Duration::from_secs_f32(delay.max(0.0)),
                    layout_id: *layout_id,
                    action_timeline: *action_timeline,
                });
            }
            LuaDirectorTask::EObjControl {
                id,
                category,
                params,
            } => {
                let Some(actor_id) = instance.find_object_by_eobj_or_layout_id(*id) else {
                    tracing::warn!("Failed to find eobj {id} for EObjControl!");
                    continue;
                };
                let mut network = network.lock();
                network.send_ac_in_range_instance(
                    instance,
                    actor_id,
                    ActorControlCategory::Unknown {
                        category: *category,
                        param1: params[0],
                        param2: params[1],
                        param3: params[2],
                        param4: params[3],
                        param5: 0,
                    },
                );
            }
        }
    }

    if let Some(director) = &mut instance.directors.first_mut() {
        director.tasks.drain(..taken.min(director.tasks.len()));
        director.bosses = bosses;
    }
}

/// Process director-related messages.
pub fn handle_director_messages(
    data: Arc<Mutex<WorldServer>>,
    network: Arc<Mutex<NetworkState>>,
    game_data: Arc<Mutex<GameData>>,
    msg: &ToServer,
) -> bool {
    match msg {
        ToServer::GimmickAccessor(id, from_actor_id, from_object_id, params) => {
            let mut data = data.lock();
            let Some(instance) = data.find_actor_instance_mut(*from_actor_id) else {
                tracing::warn!("Somehow failed to find an instance for actor?");
                return true;
            };

            let Some(base_id) = instance.find_base_id_by_actor_id(*from_object_id) else {
                tracing::warn!("Somehow failed to find base id from actor id {from_object_id}!");
                return true;
            };

            let Some(director) = &mut instance.directors.first_mut() else {
                tracing::warn!("Expected a director when recieving a GimmickAccessor?");
                return true;
            };

            let (gimmick_accessor_type, gimmick_param0, gimmick_param1, gimmick_param2);
            {
                let mut game_data = game_data.lock();
                (
                    gimmick_accessor_type,
                    gimmick_param0,
                    gimmick_param1,
                    gimmick_param2,
                ) = game_data.lookup_gimmick_accessor(*id).unwrap();
            }

            match gimmick_accessor_type {
                // Its prompt answered "No" (1; "Yes" is 0, as the other prompts'): nothing happens. Haukke Manor's
                // way back up warped the player either way (Bob's bundle 20261103-130104: params [1] twice, and he
                // was taken up both times.)
                GimmickAccessorType::Shortcut
                    if params.first().is_some_and(|answer| *answer != 0) =>
                {
                    director.tasks.push(LuaDirectorTask::FinishGimmickEvent {
                        actor_id: *from_actor_id,
                    });
                }
                GimmickAccessorType::Shortcut => {
                    // The duty's shortcut goes where the director points it; any other (Param1, a pop range) goes to
                    // its own place, after its line (Param2, a LogMessage, from the gimmick's own handler): Haukke
                    // Manor's way back up from the cellar (GimmickAccessor 0x41A), Bob's Chronofoil captures of
                    // 2026-11-02: scene 1, then LogMessage 2049 from 0x000F041A, EventFinish and the warp.
                    let pop_range_id =
                        (gimmick_param1 != 0 && base_id != EOBJ_SHORTCUT).then_some(gimmick_param1);
                    if pop_range_id.is_some() && gimmick_param2 != 0 {
                        let ipc = EventLogMessage {
                            handler_id: HandlerId::new(HandlerType::GimmickAccessor, *id as u16),
                            message_type: gimmick_param2,
                            params: Vec::new(),
                        }
                        .package()
                        .unwrap();
                        network.lock().send_to_by_actor_id(
                            *from_actor_id,
                            FromServer::PacketSegment(ipc, *from_actor_id),
                            DestinationNetwork::ZoneClients,
                        );
                    }
                    director.tasks.push(LuaDirectorTask::FinishGimmickEvent {
                        actor_id: *from_actor_id,
                    });
                    director.tasks.push(LuaDirectorTask::UseShortcut {
                        actor_id: *from_actor_id,
                        pop_range_id,
                    });
                }
                GimmickAccessorType::DutyExit
                | GimmickAccessorType::DeepDungeonExit
                | GimmickAccessorType::LeapOfFaithExit => {
                    // 0 means the user hit "Yes" in the prompt
                    if params[0] == 0 {
                        // A duty's exit: retail's director logged 1534 with its InstanceContent first (Haukke Manor's
                        // `[6]`, Brayflox's Longstop's `[8]`: Bob's Chronofoil captures of 2026-11-02 and 03.)
                        if matches!(gimmick_accessor_type, GimmickAccessorType::DutyExit)
                            && director.id.handler_type() == HandlerType::InstanceContent
                        {
                            director.tasks.push(LuaDirectorTask::LogMessage {
                                id: LOG_DUTY_EXIT,
                                params: vec![director.id.event_id()],
                            });
                        }
                        director.tasks.push(LuaDirectorTask::AbandonDuty {
                            actor_id: *from_actor_id,
                        });
                    }
                    director.tasks.push(LuaDirectorTask::FinishGimmickEvent {
                        actor_id: *from_actor_id,
                    });
                }
                GimmickAccessorType::GimmickPath => {
                    let pop_range_id = gimmick_param2;
                    let path_id = gimmick_param1;
                    let speed = 150; // TODO: dunno if this is defined somewhere

                    // TODO: Needs an EventAction before this happens I think
                    if let Some((position, rotation)) =
                        instance.zone.cached_pop_ranges.get(&pop_range_id)
                    {
                        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::WalkInEvent(
                            WalkInEvent {
                                path_id,
                                rotation: *rotation,
                                speed,
                                position: *position,
                                ..Default::default()
                            },
                        ));
                        let mut network = network.lock();
                        network.send_to_by_actor_id(
                            *from_actor_id,
                            FromServer::PacketSegment(ipc, *from_actor_id),
                            DestinationNetwork::ZoneClients,
                        );
                    }

                    director.tasks.push(LuaDirectorTask::FinishGimmickEvent {
                        actor_id: *from_actor_id,
                    });
                }
                GimmickAccessorType::Talk => {
                    // This handled by the client, we just need to finish it
                    director.tasks.push(LuaDirectorTask::FinishGimmickEvent {
                        actor_id: *from_actor_id,
                    });
                }
                GimmickAccessorType::FalliblePrompt => {
                    // 0 means the user hit "Yes" in the prompt
                    if params[0] == 0 {
                        if director.is_gimmick_functional(base_id) {
                            director.gimmick_accessor(*from_actor_id, base_id, params);
                            return true;
                        } else {
                            director.tasks.push(LuaDirectorTask::LogMessage {
                                id: gimmick_param1,
                                params: Vec::default(),
                            });
                        }
                    }

                    director.tasks.push(LuaDirectorTask::FinishGimmickEvent {
                        actor_id: *from_actor_id,
                    });
                }
                GimmickAccessorType::Message => {
                    if director.is_gimmick_functional(base_id) {
                        director.gimmick_accessor(*from_actor_id, base_id, params);
                    } else {
                        director.tasks.push(LuaDirectorTask::LogMessage {
                            id: gimmick_param0,
                            params: Vec::default(),
                        });
                        director.tasks.push(LuaDirectorTask::FinishGimmickEvent {
                            actor_id: *from_actor_id,
                        });
                    }
                }
                GimmickAccessorType::InfalliblePrompt => {
                    // 0 means the user hit "Yes" in the prompt
                    if params[0] == 0 {
                        director.gimmick_accessor(*from_actor_id, base_id, params);
                    } else {
                        director.tasks.push(LuaDirectorTask::FinishGimmickEvent {
                            actor_id: *from_actor_id,
                        });
                    }
                }
                _ => {
                    director.gimmick_accessor(*from_actor_id, base_id, params);
                }
            }

            true
        }
        ToServer::QuestBattleTalk(from_actor_id, target) => {
            let mut data = data.lock();
            let Some(instance) = data.find_actor_instance_mut(*from_actor_id) else {
                tracing::warn!("Somehow failed to find an instance for actor?");
                return true;
            };
            // Only an event object is the director's: its event NPCs (an onlooker) aren't there to talk.
            let base_id = instance.find_base_id_by_actor_id(*target);
            let answered = match (base_id, instance.directors.first_mut()) {
                (Some(base_id), Some(director)) => {
                    director.object_talk(*from_actor_id, *target, base_id)
                }
                _ => false,
            };
            if !answered {
                let mut network = network.lock();
                network.send_to_by_actor_id(
                    *from_actor_id,
                    FromServer::QuestBattleNotTalk,
                    DestinationNetwork::ZoneClients,
                );
            }

            true
        }
        ToServer::ContentNpc(id, from_actor_id, from_object_id, params) => {
            let mut data = data.lock();
            let Some(instance) = data.find_actor_instance_mut(*from_actor_id) else {
                tracing::warn!("Somehow failed to find an instance for actor?");
                return true;
            };
            let Some(director) = &mut instance.directors.first_mut() else {
                tracing::warn!("Expected a director when recieving a ContentNpc?");
                return true;
            };
            director.content_npc(*from_actor_id, *id, *from_object_id, params);

            true
        }
        ToServer::ReadyDirectorData(from_actor_id) => {
            let mut data = data.lock();
            let Some(instance) = data.find_actor_instance_mut(*from_actor_id) else {
                tracing::warn!("Somehow failed to find an instance for actor?");
                return true;
            };

            let mut network = network.lock();
            for director in &instance.directors {
                let flags = if instance
                    .content_settings
                    .unwrap_or_default()
                    .contains(DutyFinderSetting::EXPLORER_MODE)
                    && director.id.handler_type().is_content_director()
                {
                    1
                } else {
                    0
                };

                network.send_to_by_actor_id(
                    *from_actor_id,
                    FromServer::ActorControlSelf(ActorControlCategory::InitDirector {
                        handler_id: director.id,
                        content_id: if director.id.handler_type().is_content_director() {
                            instance.content_id
                        } else {
                            0xFFFF
                        },
                        flags,
                    }),
                    DestinationNetwork::ZoneClients,
                );
            }

            // Sync to new max level
            if let Some(synced_level) = instance.synced_level {
                network.send_to_by_actor_id(
                    *from_actor_id,
                    FromServer::SyncMaxLevel(synced_level, instance.synced_item_level),
                    DestinationNetwork::ZoneClients,
                );
            }
            if let Some(item_level) = instance.minimum_item_level {
                network.send_to_by_actor_id(
                    *from_actor_id,
                    FromServer::MinimumItemLevel(item_level),
                    DestinationNetwork::ZoneClients,
                );
            }

            true
        }
        ToServer::EnterTerritoryEvent(from_actor_id) => {
            let mut data = data.lock();
            let Some(instance) = data.find_actor_instance_mut(*from_actor_id) else {
                tracing::warn!("Somehow failed to find an instance for actor?");
                return true;
            };

            let mut network = network.lock();
            for director in &instance.directors {
                if director.id.handler_type().is_content_director() {
                    // Start the initial cutscene
                    network.send_to_by_actor_id(
                        *from_actor_id,
                        FromServer::PlayInitialCutscene(
                            instance.duration.unwrap_or_default().as_secs() as u32,
                            DutyOption::from_content_flags(
                                instance.content_settings.unwrap_or_default(),
                            )
                            .bits(),
                            director.opening_scene_flags(),
                        ),
                        DestinationNetwork::ZoneClients,
                    );
                }
            }

            true
        }
        ToServer::DirectorSceneFinished(from_actor_id, scene) => {
            let mut data = data.lock();
            let Some(instance) = data.find_actor_instance_mut(*from_actor_id) else {
                return true;
            };
            if let Some(director) = instance.directors.first_mut() {
                director.scene_finished(*scene);
            }

            true
        }
        ToServer::ReadyToCommence(from_actor_id) => {
            let mut data = data.lock();
            let Some(instance) = data.find_actor_instance_mut(*from_actor_id) else {
                tracing::warn!("Somehow failed to find an instance for actor?");
                return true;
            };

            // Quest battles don't have the InstanceContent duty start, their script decides what happens.
            if let Some(director) = instance.directors.first_mut()
                && director.id.handler_type() == HandlerType::QuestBattle
            {
                director.commence();
                return true;
            }

            // TODO: wait for all players to be ready, not just the first one!

            // Have the director commence the duty
            let ipc =
                ServerZoneIpcSegment::new(ServerZoneIpcData::ActorControlSelf(ActorControlSelf {
                    category: ActorControlCategory::DirectorEvent {
                        handler_id: instance.directors.first().unwrap().id,
                        event: DirectorEvent::ContentDirector(
                            ContentDirectorEvent::InstanceContent(
                                InstanceContentDirectorEvent::DutyCommence {
                                    time_limit: instance.duration.unwrap_or_default().as_secs()
                                        as u32,
                                },
                            ),
                        ),
                    },
                }));
            let mut network = network.lock();
            network.send_to_by_actor_id(
                *from_actor_id,
                FromServer::PacketSegment(ipc, *from_actor_id),
                DestinationNetwork::ZoneClients,
            );

            // A dungeon's script that times something from the duty's start hears it too (Brayflox's Longstop's
            // dragon flew off 3.7 s after DutyCommence: Bob's Chronofoil capture 44df02fc of 2026-11-03.)
            if let Some(director) = instance.directors.first_mut()
                && director
                    .lua
                    .0
                    .globals()
                    .contains_key("onCommence")
                    .unwrap_or_default()
            {
                director.commence();
            }

            let entrance_actor_id;
            let state = EventState::OFF | EventState::UNK2 | EventState::UNK3;

            {
                // Find the spawned entrance circle
                let Some(actor_id) = instance.find_entrance_circle() else {
                    tracing::warn!("Failed to find entrance circle, it won't despawn!");
                    return true;
                };
                entrance_actor_id = actor_id;

                // Update invisibility flags for next spawn
                if let Some(NetworkedActor::Object { object, .. }) =
                    instance.find_actor_mut(entrance_actor_id)
                {
                    object.event_state = state;
                    object.not_targetable = true;
                    // The entrance's barrier goes with it (the Copperbell Mines' held the bots up.)
                    let layout_id = object.layout_id;
                    instance.set_gimmick_gone(layout_id, true);
                }
            }

            // Make the entrance circle invisible.
            let msg = FromServer::ActorControl(
                entrance_actor_id,
                ActorControlCategory::SetEventState { state },
            );

            network.send_to_by_actor_id(*from_actor_id, msg, DestinationNetwork::ZoneClients);

            // This begins the countdown
            let ipc =
                ServerZoneIpcSegment::new(ServerZoneIpcData::ActorControlSelf(ActorControlSelf {
                    category: ActorControlCategory::DirectorEvent {
                        handler_id: instance.directors.first().unwrap().id,
                        event: DirectorEvent::ContentDirector(
                            ContentDirectorEvent::SetDutyTimeRemaining {
                                time_remaining: instance.duration.unwrap_or_default().as_secs()
                                    as u32,
                            },
                        ),
                    },
                }));
            network.send_to_by_actor_id(
                *from_actor_id,
                FromServer::PacketSegment(ipc, *from_actor_id),
                DestinationNetwork::ZoneClients,
            );
            if instance.duty_clock.is_none() {
                let now = Instant::now();
                instance.duty_clock = Some((now, now));
            }

            true
        }
        _ => false,
    }
}

/// How often a duty's time left is re-sent (retail: every 300 s, Bob's Chronofoil captures.)
/// The battle talk window's speaker kind for a BattleNpc (`name_id` a BNpcName); retail's quest battles send 2 for
/// every speaker in the fight and 3 (EventNpc, an ENpcResident name) for one who isn't (Victory in Peril's watcher,
/// Bob's Chronofoil capture of 2026-10-21.)
const BATTLE_TALK_KIND_BATTLE_NPC: u8 = 2;

pub const DUTY_CLOCK_RESYNC: Duration = Duration::from_secs(300);

/// Re-sends a running duty's time left to its players every `DUTY_CLOCK_RESYNC`, as retail does.
pub fn resync_duty_clock(network: &mut NetworkState, instance: &mut Instance) {
    let (Some((started, last)), Some(duration), Some(director)) = (
        instance.duty_clock,
        instance.duration,
        instance.directors.first(),
    ) else {
        return;
    };
    let now = Instant::now();
    if now.duration_since(last) < DUTY_CLOCK_RESYNC {
        return;
    }
    instance.duty_clock = Some((started, now));
    let remaining = duration
        .saturating_sub(now.duration_since(started))
        .as_secs() as u32;
    let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorControlSelf(ActorControlSelf {
        category: ActorControlCategory::DirectorEvent {
            handler_id: director.id,
            event: DirectorEvent::ContentDirector(ContentDirectorEvent::SetDutyTimeRemaining {
                time_remaining: remaining,
            }),
        },
    }));
    let players: Vec<ObjectId> = instance
        .actors
        .iter()
        .filter(|(_, actor)| matches!(actor, NetworkedActor::Player { .. }))
        .map(|(id, _)| *id)
        .collect();
    for player in players {
        network.send_to_by_actor_id(
            player,
            FromServer::PacketSegment(ipc.clone(), player),
            DestinationNetwork::ZoneClients,
        );
    }
}

/// Handles an incoming director trigger from a client.
pub fn handle_director_trigger(
    network: Arc<Mutex<NetworkState>>,
    game_data: Arc<Mutex<GameData>>,
    from_actor_id: ObjectId,
    handler_id: HandlerId,
    trigger: &DirectorTrigger,
) {
    if handler_id.handler_type() == HandlerType::GimmickRect {
        let ipc =
            ServerZoneIpcSegment::new(ServerZoneIpcData::ActorControlSelf(ActorControlSelf {
                category: ActorControlCategory::DirectorEvent {
                    handler_id,
                    event: DirectorEvent::GimmickRect(GimmickRectEvent::Activate { arg1: 1 }),
                },
            }));
        let mut network = network.lock();
        network.send_to_by_actor_id(
            from_actor_id,
            FromServer::PacketSegment(ipc, from_actor_id),
            DestinationNetwork::ZoneClients,
        );
    }
    // Air Force One's duty asks which course it is (`[0]`), answered with its GFateRideShooting row (retail: `[0, 6]` in
    // InstanceContent 62005, Bob's Chronofoil capture 2c98f9e8.) The GATEs' own director (GoldSaucer) is answered by the
    // connection (`zone_connection/gate.rs`.)
    if handler_id.handler_type() == HandlerType::InstanceContent
        && let DirectorTrigger::Unknown { id: 0, .. } = trigger
    {
        let course = game_data
            .lock()
            .ride_shooting_course(handler_id.event_id() as u16);
        if let Some(course) = course {
            let mut network = network.lock();
            network.send_to_by_actor_id(
                from_actor_id,
                FromServer::ActorControlSelf(ActorControlCategory::DirectorEvent {
                    handler_id,
                    event: DirectorEvent::Unknown {
                        id: 0,
                        arg1: course as u32,
                        arg2: 0,
                        arg3: 0,
                        arg4: 0,
                    },
                }),
                DestinationNetwork::ZoneClients,
            );
        }
    }

    // TODO: send triggers to Lua etc.
}

pub fn send_director_vars(
    data: Arc<Mutex<WorldServer>>,
    network: Arc<Mutex<NetworkState>>,
    game_data: Arc<Mutex<GameData>>,
    from_actor_id: ObjectId,
) {
    let mut data = data.lock();
    let Some(instance) = data.find_actor_instance_mut(from_actor_id) else {
        tracing::warn!("Somehow failed to find an instance for actor?");
        return;
    };

    let mut network = network.lock();
    for director in &instance.directors {
        if director.id.handler_type().is_content_director() {
            network.send_to_by_actor_id(
                from_actor_id,
                FromServer::PacketSegment(director.build_var_segment(), from_actor_id),
                DestinationNetwork::ZoneClients,
            );

            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::UnkDirector1 {
                unk: [
                    0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 38, 0, 0, 0,
                ],
            });
            network.send_to_by_actor_id(
                from_actor_id,
                FromServer::PacketSegment(ipc, from_actor_id),
                DestinationNetwork::ZoneClients,
            );

            let mut game_data = game_data.lock();
            if let Some(map_effects) = game_data.get_map_effects(instance.content_id as u32) {
                let mut states = Vec::new();
                for (i, layout_id) in map_effects.iter().enumerate() {
                    // A layout ID of zero means the effect should be skipped.
                    if *layout_id != 0 {
                        states.resize(i + 1, 0);
                        states[i] = 4; // 4 means to play it, I guess?
                    }
                }

                let ipc = MapEffects {
                    handler_id: director.id,
                    unk_flag: 5,
                    states,
                    ..Default::default()
                }
                .package()
                .unwrap();
                network.send_to_by_actor_id(
                    from_actor_id,
                    FromServer::PacketSegment(ipc, from_actor_id),
                    DestinationNetwork::ZoneClients,
                );
            }

            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::UnkDirector2 {
                unk: [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ],
            });
            network.send_to_by_actor_id(
                from_actor_id,
                FromServer::PacketSegment(ipc, from_actor_id),
                DestinationNetwork::ZoneClients,
            );
        }

        if director.id.handler_type() == HandlerType::PublicContent {
            let content_type;
            {
                let mut game_data = game_data.lock();
                content_type = game_data
                    .find_public_content_type(director.id.event_id())
                    .unwrap_or_default();
            }

            if content_type == PublicContentType::OccultCrescent {
                // Setup the panel
                let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::OccultCrescentSetup {
                    unk1: [
                        243, 152, 1, 0, 136, 182, 2, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 48, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 137, 21, 36, 0, 0,
                        0, 0, 0, 0, 0, 2, 5, 0, 1, 0, 0, 4, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 1, 223, 63, 0, 0, 0, 0, 0, 0, 0, 0,
                    ],
                });
                network.send_to_by_actor_id(
                    from_actor_id,
                    FromServer::PacketSegment(ipc, from_actor_id),
                    DestinationNetwork::ZoneClients,
                );

                // Setup duty actions
                network.send_to_by_actor_id(
                    from_actor_id,
                    FromServer::ActorControlSelf(ActorControlCategory::UnkDutyActions { unk1: 24 }),
                    DestinationNetwork::ZoneClients,
                );
                network.send_to_by_actor_id(
                    from_actor_id,
                    FromServer::ActorControlSelf(ActorControlCategory::EnableDutyActions {
                        enabled: true,
                    }),
                    DestinationNetwork::ZoneClients,
                );
                network.send_to_by_actor_id(
                    from_actor_id,
                    FromServer::ActorControlSelf(ActorControlCategory::SetDutyActions {
                        action_ids: [41588, 41589, 41590, 0, 0],
                    }),
                    DestinationNetwork::ZoneClients,
                );
            }
        }

        if director.id.handler_type() == HandlerType::TreasureHunt {
            network.send_to_by_actor_id(
                from_actor_id,
                FromServer::PacketSegment(director.build_var_segment(), from_actor_id),
                DestinationNetwork::ZoneClients,
            );
        }

        if director.id.handler_type().is_leve_director() {
            network.send_to_by_actor_id(
                from_actor_id,
                FromServer::PacketSegment(director.build_var_segment(), from_actor_id),
                DestinationNetwork::ZoneClients,
            );

            // Send a dummy commence for now
            // TODO: I think this can be extrapolated from Excel
            network.send_to_by_actor_id(
                from_actor_id,
                FromServer::ActorControlSelf(ActorControlCategory::DirectorEvent {
                    handler_id: director.id,
                    event: DirectorEvent::LeveDirector(LeveDirectorEvent::DutyCommence {
                        time_limit: 1200,
                        start_timestamp: timestamp_secs(),
                    }),
                }),
                DestinationNetwork::ZoneClients,
            );
        }

        if instance.zone.intended_use == TerritoryIntendedUse::CrystallineConflictCustomMatch {
            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::SpectatorList {
                object_ids: [
                    from_actor_id,
                    ObjectId::default(),
                    ObjectId::default(),
                    ObjectId::default(),
                    ObjectId::default(),
                    ObjectId::default(),
                    ObjectId::default(),
                    ObjectId::default(),
                ],
                unk1: 1,
            });
            network.send_to_by_actor_id(
                from_actor_id,
                FromServer::PacketSegment(ipc, from_actor_id),
                DestinationNetwork::ZoneClients,
            );
        }
    }
}

/// Capitalizes the first letter of each word, the way retail shows NPC names in chat.
fn capitalize_words(name: &str) -> String {
    name.split(' ')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn room(wall_id: u32) -> DirectorBoss {
        DirectorBoss {
            state: DirectorBossState::Open,
            actor_id: ObjectId::default(),
            wall_id,
            line_id: wall_id + 1,
            place_name: 609,
            triggers: [0; 8],
            arena: 0,
        }
    }

    fn unsealed(director: &DirectorData) -> bool {
        director.tasks.iter().any(|task| {
            matches!(task, LuaDirectorTask::LogMessage { id, .. } if *id == LogMessageType::BossRoomUnsealed as u32)
        })
    }

    /// A room's floor keeps the party in from the pull on, not before (Brayflox's Longstop's last boss's terrace.)
    #[test]
    fn the_arena_holds_from_the_pull() {
        let mut director = DirectorData::default();
        director.bosses.insert(
            9059658,
            DirectorBoss {
                arena: 9059667,
                ..room(2002872)
            },
        );
        assert_eq!(director.arena(), None);
        director.on_actor_aggro(9059658, ObjectId(1));
        assert_eq!(director.arena(), Some(9059667));
        director.bosses.clear();
        assert_eq!(director.arena(), None);
    }

    #[test]
    fn a_room_of_two_opens_as_the_last_falls() {
        // Haukke Manor's pair: one wall, two bosses.
        let mut director = DirectorData::default();
        director.bosses.insert(3633631, room(2001607));
        director.bosses.insert(3633633, room(2001607));

        director.on_actor_aggro(3633633, ObjectId(1));
        let sealing = director
            .tasks
            .iter()
            .filter(|task| matches!(task, LuaDirectorTask::SealBossWall { .. }))
            .count();
        assert_eq!(sealing, 1);
        assert!(
            director
                .bosses
                .values()
                .all(|boss| boss.state == DirectorBossState::Aggro)
        );
        // The other's pull doesn't seal it again.
        director.on_actor_aggro(3633631, ObjectId(1));
        let sealing_again = director
            .tasks
            .iter()
            .filter(|task| matches!(task, LuaDirectorTask::SealBossWall { .. }))
            .count();
        assert_eq!(sealing_again, 1);

        director.on_actor_death(3633633, Position::default());
        assert!(!unsealed(&director));
        director.on_actor_death(3633631, Position::default());
        assert!(unsealed(&director));
    }
}
