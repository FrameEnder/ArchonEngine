use binrw::binrw;
use strum_macros::IntoStaticStr;

use crate::{
    common::{
        CharacterMode, DirectorEvent, EquipDisplayFlag, EventState, FateState, HandlerId, ObjectId,
        ObjectTypeId, SharedGroupTimelineState, read_bool_from, read_packed_float, write_bool_as,
        write_packed_float,
    },
    ipc::zone::{online_status::OnlineStatus, server::ContainerType},
};

#[binrw]
#[brw(repr = u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BaseStat {
    Health = 1,
    Mana = 3,
    GatheringPoints = 7,
    CraftingPoints = 8,
}

#[binrw]
#[derive(Debug, PartialEq, Clone)]
pub enum LiveEventType {
    /// "\[Item\] recorded in gathering log". Only prints the text, doesn't set the bitmask.
    #[brw(magic = 5u32)]
    RecordInGatheringLog {
        /// The actual Item ID, not the GathertingItem one.
        item_id: u32,
    },

    /// Begins a new crafting session.
    #[brw(magic = 8u32)]
    StartCraft {
        /// 0, 1, 2 plays an ActionTimeline (???) and 3, 4, 5 does something else?
        unk1: u32,
        unk2: u32,
        unk3: u32,
    },

    /// Ends the current crafting session.
    #[brw(magic = 12u32)]
    PlayAnimation {
        /// Index into the ActionTimeline Excel sheet.
        animation_start: u32,
        /// Index into the ActionTimeline Excel sheet.
        animation_end: u32,
    },

    /// Ends the current crafting session.
    #[brw(magic = 15u32)]
    EndCraft,

    /// Sets the main hand weapon model.
    #[brw(magic = 38u32)]
    SetMainHand { model_id: u32, unk1: u32, unk2: u32 },

    /// Sets the off hand weapon model.
    #[brw(magic = 39u32)]
    SetOffHand { model_id: u32, unk1: u32, unk2: u32 },

    #[doc(hidden)]
    Unknown {
        event: u32,
        param1: u32,
        param2: u32,
        param3: u32,
    },
}

#[binrw]
#[derive(Debug, PartialEq, Clone, IntoStaticStr)]
pub enum ActorControlCategory {
    #[brw(magic = 0u32)]
    ToggleWeapon {
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        shown: bool,
        /// This seems to always be set to 1. If set to another value, the animation glitches for other clients.
        unk_flag: u32,
        unk2: u32,
    },

    /// Sets auto-attack mode of this entity.
    #[brw(magic = 1u32)]
    SetAutoAttack {
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        auto_attack: bool,
    },

    /// Sets the character mode, such as for mounts.
    #[brw(magic = 2u32)]
    SetMode {
        #[brw(pad_size_to = 4)]
        mode: CharacterMode,
        mode_arg: u32,
    },

    /// Begins an event action.
    #[brw(magic = 3u32)]
    EventAction {
        /// Seems to be 1 to start it?
        unk1: u32,
        /// Index into the EventAction Excel sheet.
        id: u32,
    },

    /// Toggles whether an enemy has a red nameplate or not.
    #[brw(magic = 4u32)]
    SetBattle {
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        battle: bool,
    },

    /// Only plays the VFX and nothing else.
    #[brw(magic = 5u32)]
    ClassJobChangeVFX {
        /// Index into the ClassJob Excel sheet.
        classjob_id: u32,
    },

    /// Shows that this enemy was defeated in the log, and probably other stuff.
    #[brw(magic = 6u32)]
    Defeated { id1: ObjectId, id2: ObjectId },

    /// Only shows the floating message, it doesn't actually update your EXP!
    #[brw(magic = 7u32)]
    EXPFloatingMessage {
        /// The classjob ID to gain experience for.
        classjob_id: u32,
        /// The amount of EXP.
        amount: u32,
        /// Percentage of bonus EXP to display. Note that this *doesn't* apply it client-side, this is simply an indicator.
        bonus_percent: u32,
        /// The defeated enemy the EXP is for, or 0xE0000000 for EXP from anything else (a quest, a FATE, a duty), as
        /// retail sends it (Bob's Chronofoil captures.)
        source_actor_id: ObjectId,
    },

    /// Only updates the UI, it's wiped on the next ClassInfo update.
    #[brw(magic = 8u32)]
    UnlockClass { classjob_id: u32 },

    /// Only updates the UI, it's wiped on the next ClassInfo update.
    #[brw(magic = 9u32)]
    SetLevel { classjob_id: u32, level: u32 },

    /// Shows the message and also updates some character data.
    /// It's also interesting in that it handles the in-between levels. So you can give it a level of 99 and it'll go through *each* level.
    #[brw(magic = 10u32)]
    LevelUpMessage {
        classjob_id: u32,
        level: u32,
        unk2: u32,
        unk3: u32,
    },

    #[brw(magic = 11u32)]
    Unk11 { unk1: ObjectId, unk2: u32 },

    /// A kill that counts towards an EXP chain (ActorControlSelf, sent right before the EXP message): its place in the
    /// chain from 0, and how long the next kill has to keep it going, in hundredths of a second (Bob's Chronofoil
    /// captures: 10000, 8000, 6000, 4000, 2000, then 1000.)
    #[brw(magic = 12u32)]
    ExpChain { index: u32, window: u32 },

    /// Does what it says!
    #[brw(magic = 13u32)]
    SetBaseStat { base_stat: BaseStat, amount: u32 },

    /// Kills this actor, including playing an animation and setting HP/MP to zero.
    /// Does *not* change the state of the actor.
    #[brw(magic = 14u32)]
    Kill {
        /// Index into the ActionTimeline sheet. If 0, plays the default death animation.
        animation_id: u32,
    },

    #[brw(magic = 15u32)]
    CancelCast {
        /// 538 when interrupted, 540 when the caster died mid-cast (retail's bosses, Bob's Copperbell Mines captures of
        /// 2026-10-25), 0 for 28764's tank-buster ending.
        unk1: u32,
        unk2: u32,
        /// Action ID of the cast being cancelled.
        action_id: u32,
        /// 1 when the caster died mid-cast (retail), otherwise 0.
        unk3: u32,
    },

    /// Sets the current value of the cooldown timer.
    #[brw(magic = 16u32)]
    SetCooldownTimer {
        /// This corresponds to (CooldownGroup - 1) from the Action sheet.
        cooldown_group: u32,
        unk1: u32,
        unk2: u32,
    },

    /// Sets the upper bound of the cooldown timer. *Only* has an effect if the cooldown timer is actually running.
    #[brw(magic = 17u32)]
    SetCooldownTimerMax {
        /// This corresponds to (CooldownGroup - 1) from the Action sheet.
        cooldown_group: u32,
        /// Index in the Action Excel sheet.
        action_id: u32,
        /// 10 * milliseconds value.
        milliseconds: u32,
    },

    #[brw(magic = 20u32)]
    GainEffect {
        effect_id: u32,
        param: u32,
        source_actor_id: ObjectId,
    },

    #[brw(magic = 21u32)]
    LoseEffect {
        effect_id: u32,
        unk2: u32,
        source_actor_id: ObjectId,
    },

    #[brw(magic = 23u32)]
    FloatingNumberVFX {
        unk1: u32,
        /// See TargetEffectKind.
        kind: u32,
        amount: u32,
        object_id: ObjectId,
    },

    /// Updates the rested EXP bonus shown in the EXP bar.
    #[brw(magic = 24u32)]
    UpdateRestedExp { exp: u32 },

    #[brw(magic = 27u32)]
    Flee { speed: u16 },

    #[brw(magic = 34u32)]
    ShowLockonVfx {
        /// Index into the Lockon Excel sheet.
        id: u32,
        /// Who to show the lockon VFX on.
        target: ObjectId,
    },

    /// Brings an NPC spawned hidden (display flag INVISIBLE) into view, as retail does with the Copperbell Mines'
    /// ambushing gigas 0.6-0.9 s after their spawn, with 158 (Bob's Chronofoil captures of 2026-10-25). From the client's
    /// ActorControl handler (2026.10.16, case 0x24): it flags the object for an update (UpdateFlags 0x80), fades it in
    /// over a second when `fade_in` is set, and plays `action_timeline` (ActionTimeline) on it when it isn't 0.
    #[brw(magic = 36u32)]
    Appear {
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        fade_in: bool,
        action_timeline: u32,
    },

    /// Fades an NPC out before it's taken away (a DeleteActor 0.6-1.4 s later), when it leaves without dying: retail's
    /// `[itself, 1, 0, 100]` for Brotherly Love's beast, Claxio and the fallen conjurer as "Duty Complete" played (Bob's
    /// Chronofoil capture 5fcc970c), and before most despawns of living NPCs in his other captures (also seen:
    /// `[itself, 2, 197, 100]` and `[itself, 1, 0, 300]`; what `kind`, `unk` and `time` mean isn't known.)
    #[brw(magic = 607u32)]
    FadeOut {
        actor: ObjectId,
        kind: u32,
        unk: u32,
        time: u32,
    },

    /// The next ActorMove puts the character there at once rather than walking it: the client's handler (2026.10.16,
    /// case 0xEF) sets a flag in its MoveController. Retail sent it right before each ActorMove that landed Kottos'
    /// jumps (Bob's Copperbell Mines captures of 2026-10-25.)
    #[brw(magic = 239u32)]
    SnapNextMove,

    #[brw(magic = 38u32)]
    ToggleInvisibility {
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        invisible: bool,
    },

    /// Seen while dead actors fade away.
    #[brw(magic = 39u32)]
    DeadFadeOut,

    /// Fall damage: the HP a fall took, as flying text over the one who fell (Bob's Chronofoil captures: 49 falls, all
    /// with `unk` 1 but two of 450 at level 87; see docs/src/fork/combat.md.)
    #[brw(magic = 45u32)]
    FallDamage { damage: u32, unk: u32 },

    /// After a gil drop's LootMessage (kind 9), always and only then: retail's 136 of 136 (Bob's Chronofoil captures
    /// c99fe374 and cc62ad94), no values. What the client does with it isn't known.
    #[brw(magic = 42u32)]
    GilObtained,

    #[brw(magic = 41u32)]
    ToggleUnlock {
        /// Corresponds to an UnlockLink. Could be a spell, action, emote, etc.
        // See https://github.com/Haselnussbomber/HaselDebug/blob/main/HaselDebug/Tabs/UnlocksTabs/UnlockLinks/UnlockLinksTable.cs
        id: u32,
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        unlocked: bool,
    },

    /// Sets the EXP, but doesn't display anything visually.
    #[brw(magic = 43u32)]
    SetEXP {
        classjob_id: u32,
        amount: u32,
        rested_exp: u32,
    },

    /// Changes a character's size: the ModelScale row it's drawn at (the client's handler, 2026.10.16 case 0x30, sets
    /// ModelContainer.ModelScaleId and multiplies its base scale by the row's). Retail grew the Copperbell Mines'
    /// second boss each round (16, 17, 18; Bob's captures of 2026-10-25.)
    #[brw(magic = 48u32)]
    SetModelScale { model_scale_id: u32 },

    #[brw(magic = 50u32)]
    SetTarget,

    /// Whether an object can be targeted: the client's handler (2026.10.16, case 0x36) sets TargetableStatus' first
    /// bit and IsTargetable to it. Retail sent it (1) right after `Appear` (Bob's captures of 2026-10-25.)
    #[brw(magic = 54u32)]
    SetTargetable {
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        targetable: bool,
    },

    #[brw(magic = 58u32)]
    SetSoftTarget,

    /// Plays this character's idle animation, I guess?
    #[brw(magic = 60u32)]
    PlayIdleAnimation,

    // Sets the player's HP and seems to deal unique damage(?) Seen while falling off an Eden arena.
    #[brw(magic = 80u32)]
    DamageEffect {
        /// How much damage you take.
        amount: u32,
    },

    // Calls into many inventory-related functions, haven't looked too far yet.
    #[brw(magic = 84u32)]
    UnkInventoryRelated {
        unk1: u32,
        unk2: u32,
        unk3: u32,
        unk4: u32,
    },

    /// Set the actor into a state of riding pillion.
    #[brw(magic = 89u32)]
    RidePillion {
        /// The target actor to bind to.
        target_actor_id: ObjectId,
        /// The target seat to ride on the mount.
        target_seat_index: u32,
    },

    #[brw(magic = 100u32)]
    InitDirector {
        /// The director to intiailize.
        handler_id: HandlerId,
        /// Content ID for this director. Can be 0xFFFF to indicate there should be an associated content ID.
        #[brw(pad_after = 2)] // padding
        content_id: u16,
        /// If set to 1, enables explorer mode. Probably does a lot more too!
        flags: u32,
    },

    #[brw(magic = 101u32)]
    TerminateDirector {
        /// The director to terminate.
        handler_id: HandlerId,
    },

    /// Updates the state for an event object.
    #[brw(magic = 106u32)]
    SetEventState { state: EventState },

    #[brw(magic = 109u32)]
    DirectorEvent {
        handler_id: HandlerId,
        #[br(args(handler_id))]
        event: DirectorEvent,
    },

    #[brw(magic = 131u32)]
    UnlockInstanceContent {
        /// Index into InstanceContent Excel sheet
        id: u32,
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        unlocked: bool,
    },

    /// Calls into some GCSupply function, unsure what it does yet.
    #[brw(magic = 137u32)]
    UnkGCSupplyRelated {
        unk1: u32,
        unk2: u32,
        unk3: u32,
        unk4: u32,
    },

    /// Unknown purpose, maybe positioning related?
    #[brw(magic = 138u32)]
    DisableEventPosRollback { handler_id: HandlerId },

    /// If enabled, the client sends UpdatePositionHandlerInstance. When disabled (the default) then regular UpdatePositionHandler packets instead.
    #[brw(magic = 142u32)]
    EnableInstancePositionHandler {
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        enabled: bool,
    },

    #[brw(magic = 142u32)]
    SetImmediateAction { unk1: u32 },

    /// How many retainers the character has, sent as a retainer vocate visit ends (Bob's Chronofoil capture of
    /// 2026-10-18: 1 after hiring one, 0 after letting them go.)
    #[brw(magic = 152u32)]
    RetainerCount { count: u32 },

    #[brw(magic = 156u32)]
    ToggleAdventureUnlock {
        id: u32, // Index to Adventure sheet
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        all_vistas_recorded: bool,
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        unlocked: bool,
    },

    #[brw(magic = 164u32)]
    ToggleAetherCurrentUnlock {
        id: u32, // Index to AetherCurrent sheet
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        attunement_complete: bool, // If true, then "Attunement Complete" will show in the Aether Currents menu, and screen_image_id will show on screen
        // padding, screen_image_id and zone_id are technically a single u32 in the client, but this is more readable
        padding: u8,
        screen_image_id: u16, // Index to ScreenImage sheet. Will only show if attunement_complete is true.
        zone_id: u8,          // Index to AetherCurrentCompFlgSet sheet.
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        unk1: bool, // Same value as attunement_complete
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        show_flying_mounts_help: bool, // Will only be used if attunement_complete is true.
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        remove_aether_current: bool, // If true, attunement_complete, screen_image_id and show_flying_mounts_help will be ignored.
    },

    // Calls into some EventFramework function, haven't looked too far yet.
    #[brw(magic = 199u32)]
    UnkEventRelated {
        unk1: u32,
        unk2: u32,
        unk3: u32,
        unk4: u32,
    },

    #[brw(magic = 200u32)]
    ZoneIn {
        /// When set to 1, it slowly fades the character in.
        warp_finish_anim: u32,
        /// When set to 1, it plays the effects *and* animation for raising.
        raise_anim: u32,
        /// If set to 110, then it plays the "we teleported here" animation.
        #[brw(pad_before = 4)] // empty
        unk1: u32,
    },

    #[brw(magic = 203u32)]
    TeleportStart {
        insufficient_gil: u32,
        aetheryte_id: u32,
    },

    /// The player is offered a teleport by someone in their party.
    #[brw(magic = 204u32)]
    TeleportOffered {
        /// If the player is ineligible for this teleport, it'll be set to true.
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        ineligible_for_teleport: bool,
        /// The destination aetheryte.
        aetheryte_id: u32,
        /// The party member who offered the teleport. It only affects the name of the offerer, getting it wrong shows either the wrong name or no name at all.
        party_member_index: u32,
    },

    /// Used for things like the water pads in Gold Saucer.
    #[brw(magic = 220u32)]
    ExecuteGimmickJump {
        /// Y position to land on.
        #[br(map = read_packed_float)]
        #[bw(map = write_packed_float)]
        landing_position_y: f32,
        /// X position to land on.
        #[br(map = read_packed_float)]
        #[bw(map = write_packed_float)]
        landing_position_x: f32,
        /// Z position to land on.
        #[br(map = read_packed_float)]
        #[bw(map = write_packed_float)]
        #[brw(pad_after = 2)] // empty
        landing_position_z: f32,
        /// Index into the GimmickJump Excel sheet.
        gimmick_jump_type: u32,
        unk1: u32,
    },

    /// Changes something in UpdatePositionHandler?
    #[brw(magic = 236u32)]
    MovementRelatedUnk { unk1: u32 },

    /// Updates the summoned status and time remaining of your chocobo companion.
    #[brw(magic = 253u32)]
    CompanionUpdate {
        /// When true, the chocobo is marked as "Summoned" in the Companion UI.
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        summoned: bool,
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        set_time_remaining: bool,
        /// In centiseconds. Only used if `set_time_remaining` is true, for some reason.
        time_remaining: u32,
        unk4: u32,
        unk5: u32,
        unk6: u32,
    },

    #[brw(magic = 254u32)]
    BuddyEquipUnlock {
        id: u32, // Index to BuddyEquip sheet
    },

    /// Barding the chocobo companion now wears, to its owner: retail's answer to client trigger 1701 and, for each
    /// slot, at My Little Chocobo's end (Bob's Chronofoil captures 0845b9a2 and bab8c255.) The Companion window's
    /// appearance tab shows what this says.
    #[brw(magic = 255u32)]
    CompanionBarding {
        /// 0 head, 1 body, 2 legs.
        slot: u32,
        /// A BuddyEquip row (0 none.)
        equip: u32,
    },

    /// Also seen during Carbuncle spawning.
    #[brw(magic = 257u32)]
    SetupPet {
        /// The owner's actor ID.
        owner_id: ObjectId,
        /// Index into the Pet Excel sheet. If set to 0, hides the pet hotbar.
        pet_id: u32,
        /// The object ID of the pet.
        pet_actor_id: ObjectId,
        unk2: u32,
        unk3: u32,
    },

    /// Seen while dismissing your chocobo companion.
    #[brw(magic = 258u32)]
    DismissCompanion { unk1: u32 },

    /// Sets up "pets" like carbuncles.
    #[brw(magic = 260u32)]
    SetPetParameters {
        /// Index into the Pet Excel sheet. If set to 0, hides the pet hotbar.
        pet_id: u32,
        unk2: u32,
        unk3: u32,
        /// Usually 7?
        unk4: u32,
    },

    /// Seen while summoning your chocobo companion.
    #[brw(magic = 264u32)]
    UnkCompanionRelated {
        unk1: ObjectId,
        unk2: u32, // TODO: maybe object type?
    },

    #[brw(magic = 270u32)]
    MinionSpawnControl {
        /// When set to 0, the player's minion is despawned.
        minion_id: u32,
    },

    #[brw(magic = 271u32)]
    ToggleMinionUnlock {
        minion_id: u32,
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        unlocked: bool,
    },

    /// The chocobo companion's colour, to its owner: the Companion window's appearance tab shows it. Retail sent it at
    /// every login, after PlayerSetup (whose colour byte was 0), with 36 (Desert Yellow) for chocobos never recoloured
    /// (all of Bob's Chronofoil captures to 2026-10-29.) Upstream called it UpdateHater.
    #[brw(magic = 274u32)]
    CompanionColor {
        /// A Stain row.
        stain: u32,
    },

    #[brw(magic = 295u32)]
    Pose { unk1: u32, pose: u32 },

    #[brw(magic = 263u32)]
    SetPetEntityId { unk1: u32 },

    #[brw(magic = 290u32)]
    Emote {
        /// The id of the emote.
        emote: u32,
        /// 0/false = text shown, 1/true = text hidden
        #[brw(pad_before = 4)] // blank
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        hide_text: bool,
    },

    /// Interrupts the currently playing emote.
    #[brw(magic = 291u32)]
    InterruptEmote,

    /// Generic catch-all for crafting/gathering actions.
    #[brw(magic = 300u32)]
    LiveEvent {
        #[brw(pad_size_to = 16)] // pad for LiveEvents that don't use all params
        event: LiveEventType,
    },

    #[brw(magic = 324u32)]
    SetCaughtFishBitmask { index: u32, value: u32 },

    #[brw(magic = 343u32)]
    SetCaughtSpearfishBitmask { index: u32, value: u32 },

    /// A currency the client counts outside the inventory, such as Achievement Certificates. Retail sends it for each
    /// of them at login with the count held, and again when the count changes (Bob's Chronofoil capture b23b91ef: two
    /// items, 0 at login, 1 each after Jonathas gave a certificate; see docs/src/fork/character-windows.md.)
    #[brw(magic = 378u32)]
    PlayerCurrency {
        unk1: u32,
        /// Index into the Item Excel sheet.
        item_id: u32,
        /// Max number of held currency.
        max_count: u32,
        /// How many are held now.
        count: u32,
        /// 1 every time on retail.
        unk2: u32,
    },

    #[brw(magic = 384u32)]
    SetupGatheringPoint {
        /// Index into the GatheringPoint Excel sheet.
        id: u32,
        /// Index into the GatheringPointBase Excel sheet.
        base_id: u32,
        /// The level of this gathering node. This can be read from the GatheringPointBase Excel sheet.
        level: u32,
        /// Count column from the GatheringPoint Excel sheet.
        count: u32,
        /// The remaining count of this GatheringPoint.
        remaining_count: u32,
        /// Unknown purpose, only seen zero so far.
        unk1: u16,
        /// The gathering bonuses for this GatheringPoint.
        gathering_point_bonus: [u16; 2],
    },

    // Unsure what the purpose of this is, it's sent when someone else rides pillion on the player's mount
    #[brw(magic = 400u32)]
    PillionDriverRelatedUnk {
        target_seat_index: u32,
        from_actor_id: ObjectId,
    },

    /// Plays an animation for a NPC or player.
    #[brw(magic = 407u32)]
    PlayActionTimeline {
        /// See the ActionTimeline Excel sheet.
        timeline_id: u32,
    },

    /// Seen for giant clams.
    #[brw(magic = 409u32)]
    SetSharedGroupTimelineState {
        state: SharedGroupTimelineState,
        arg2: u32,        // unsued?
        object_type: u32, // If 0, then search for an EventObject using the ActorControl Entity ID
        layout_id: u32, // If arg3 is not 0, then search in g_Client::Game::Object::StandObjectManager_Instance using this layoutId
    },

    /// Plays an animation for a SharedGroup object.
    #[brw(magic = 410u32)]
    PlaySharedGroupTimeline { timeline_id: u32 },

    /// Sets the `companion_owner_id` for a this object.
    #[brw(magic = 417u32)]
    SetCompanionOwnerId { new_id: ObjectId },

    #[brw(magic = 500u32)]
    SetTitle { title_id: u32 },

    #[brw(magic = 501u32)]
    UnlockTitle { title_id: u32 },

    /// Sets or removes a marker on a given target (Ignore Target, Bind Target, etc.).
    #[brw(magic = 502u32)]
    ToggleSign {
        /// The ID of the sign to apply.
        sign_id: u32,
        /// The actor id of the player who marked the target.
        from_actor_id: ObjectId,
    },

    #[brw(magic = 503u32)]
    Unk503 { unk1: u32, unk2: ObjectId },

    #[brw(magic = 504u32)]
    SetStatusIcon { icon: OnlineStatus },

    #[brw(magic = 504u32)]
    SetLimitBreak {
        level: u32,
        amount: u32,
        limit_type: u32,
    },

    /// Controls *something* about limit breaks.
    #[brw(magic = 505u32)]
    UnkLimitBreakController {
        unk1: u32,
        unk2: u32,
        unk3: u32,
        unk4: u32,
    },

    #[brw(magic = 507u32)]
    SetHomepoint { id: u32 },

    #[brw(magic = 509u32)]
    LearnTeleport {
        id: u32,
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        unlocked: bool,
    },

    #[brw(magic = 510u32)]
    ToggleChocoboTaxiStandUnlock {
        id: u32, // id + 1179648 = Index to ChocoboTaxiStand sheet
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        unlocked: bool,
    },

    #[brw(magic = 511u32)]
    MapMarkerUpdateBegin { flags: u32 },

    #[brw(magic = 512u32)]
    MapMarkerUpdateEnd,

    /// The result of storing in or withdrawing from the armoire (ClientTrigger 424 and 425): 0 done (3 when the item
    /// wasn't there to store), the Cabinet row, 1 stored or 0 withdrawn (Bob's Chronofoil capture of 2026-10-24.)
    #[brw(magic = 513u32)]
    ArmoireResult {
        result: u32,
        cabinet_id: u32,
        stored: u32,
    },

    #[brw(magic = 516u32)]
    ToggleCutsceneSeen {
        id: u32, // Index to Cutscene sheet
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        unlocked: bool,
    },

    #[brw(magic = 517u32)]
    LogMessage {
        /// Index into the LogMessage Excel sheet.
        log_message: u32,
        /// Index to the variable sheet, depending on LogMessage.
        id: u32,
    },

    /// `LogMessage` with two values (fork): the Jumbo Cactpot's purchase line, 4735, with the ticket's number and its
    /// price (Bob's Chronofoil capture deaf7e28.) Shares 517, so it's only ever written; a read gives `LogMessage`.
    #[brw(magic = 517u32)]
    LogMessageValues {
        log_message: u32,
        value1: u32,
        value2: u32,
    },

    /// A log line with a number for the player (fork): retail's MGP from the Gil to MGP exchange, LogMessage 4765 with
    /// 0 and the MGP (Bob's Chronofoil capture deaf7e28.)
    #[brw(magic = 527u32)]
    LogMessageNumber {
        log_message: u32,
        unk: u32,
        value: u32,
    },

    /// A log line naming an actor and a number (fork): retail's MGP won at the Gold Saucer, LogMessage 4765 with the
    /// player's actor ID and the MGP (Bob's Chronofoil capture deaf7e28: the Cactpot, the arcade machines, GATEs.)
    #[brw(magic = 528u32)]
    LogMessageActorNumber {
        log_message: u32,
        actor_id: ObjectId,
        value: u32,
    },

    #[brw(magic = 519u32)]
    SetPartyMemberCutsceneFlags { actor_id: ObjectId, unk2: u32 },

    #[brw(magic = 521u32)]
    SetItemLevel { level: u32 },

    #[brw(magic = 533u32)]
    LogMessage2 {
        log_message: u32, // Index to LogMessage sheet
    },

    /// How far along an achievement is, the reply to `ClientTriggerCommand::RequestAchievementProgress` (the client
    /// calls Achievement::ReceiveAchievementProgress.)
    #[brw(magic = 514u32)]
    AchievementProgressInfo { id: u32, current: u32, max: u32 },

    /// An achievement was just completed: the client marks it complete, adds it to its recent history and shows the
    /// "Achievement unlocked" pop-up.
    #[brw(magic = 515u32)]
    AchievementCompleted { id: u32 },

    /// Someone completed an achievement: retail sent it from the player, to the player and to everyone around them,
    /// right after `AchievementCompleted` (Bob's Chronofoil capture cc62ad94: all eight of the player's, and four
    /// other players'.) What the client does with it isn't known.
    #[brw(magic = 518u32)]
    AchievementEarned { id: u32 },

    /// Calls into some Achievement method, unsure what this does yet. (The client empties the nearly completed list
    /// whose index is `unk1`, 0 for the login notice and 1 for the Achievements window's; the fork sends it as the
    /// reply to `ClientTriggerCommand::RequestNearCompletionAchievements`.)
    /// A Hunting Log target was counted: the entry's MonsterNote row, the target (0-3), the count now (Bob's Chronofoil
    /// capture b23b91ef, every kill of a target; see docs/src/fork/hunting-log.md.)
    #[brw(magic = 542u32)]
    HuntingLogTargetCounted { note: u32, target: u32, count: u32 },

    /// A Hunting Log entry was completed: its MonsterNote row and the class (6 for the Conjurer's entries: it could as
    /// well be the log's index plus one), then retail gives its EXP.
    #[brw(magic = 543u32)]
    HuntingLogEntryCompleted { note: u32, class: u32 },

    /// Sent with a job's soul crystal as its first quest gave it, with that quest's HOW_TO_JOB (the HowTo row on jobs:
    /// Seer Folly's 94; Bob's Chronofoil capture 8d550e33, 2026-11-05, between the reward's InventoryActionAck and the
    /// crystal's slot). Probably makes that HowTo entry available; what the client does with it wasn't checked.
    #[brw(magic = 540u32)]
    JobHowTo { how_to: u32 },

    #[brw(magic = 538u32)]
    UnkAchievementRelated {
        unk1: u32,
        unk2: u32,
        unk3: u32,
        unk4: u32,
    },

    /// Calls into some MobHunt method, not sure what it does yet.
    #[brw(magic = 583u32)]
    UnkMobHuntRelated {
        unk1: u32,
        unk2: u32,
        unk3: u32,
        unk4: u32,
    },

    /// The client marks `id` complete if `unk1` is 1, and takes `unk2` and `unk3` as its progress (current, max).
    /// One glamour dresser slot changed: its index and the item now there (0 when restored; same capture.)
    #[brw(magic = 1800u32)]
    GlamourDresserSlot { index: u32, item_id: u32 },

    #[brw(magic = 600u32)]
    AchievementProgress {
        id: u32,
        unk1: u32,
        unk2: u32,
        unk3: u32,
        unk4: u32,
    },

    #[brw(magic = 608u32)]
    SetEquipDisplayFlags { display_flag: EquipDisplayFlag },

    #[brw(magic = 609u32)]
    ToggleWireframeRendering(),

    /// The server refused an action the client asked for: the client prints the LogMessage and drops the request.
    /// Retail (Bob's Chronofoil captures, 2026-06 to 2026-10, 18 of these): 563 "Invalid target." when the target had
    /// died, 579 "Cannot execute at this time." (the caster had died, among others), 582 "Not yet ready." with
    /// `elapsed` the recast group's elapsed time in hundredths of a second, 1952 for an item used where it can't be.
    #[brw(magic = 700u32)]
    ActionRejected {
        /// Index into the LogMessage Excel sheet.
        log_message: u32,
        /// The request's action type (1 for an action, 2 for an item.)
        action_type: u32,
        action_id: u32,
        /// 0, except with 582.
        elapsed: u32,
    },

    #[brw(magic = 801u32)]
    GearSetEquipped { gearset_index: u32 },

    /// Multiple festivals can be set at the same time.
    #[brw(magic = 902u32)]
    SetFestival {
        festival1: u32,
        festival2: u32,
        festival3: u32,
        festival4: u32,
    },

    #[brw(magic = 903u32)]
    ToggleMountUnlock {
        /// From the Order column from the Excel row.
        order: u32,
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        unlocked: bool,
        /// Row ID of the Mount Excel sheet.
        id: u32,
    },

    /// Sets some variable in QuestManager, unsure which yet.
    #[brw(magic = 907u32)]
    UnkQuestManagerRelated {
        unk1: u32,
        unk2: u32,
        unk3: u32,
        unk4: u32,
    },

    /// For the Gold Saucer.
    /// MGP won and where (fork: Bob's Chronofoil capture deaf7e28, after each payout: 8 Cuff-a-Cur, 10 The Moogle's
    /// Paw, 13 The Finer Miner, 14 Out on a Limb, 16 a GATE.)
    #[brw(magic = 910u32)]
    GoldSaucerMgpWon { source: u32, mgp: u32 },

    #[brw(magic = 911u32)]
    UnkGoldSaucerRelated { unk1: u32 },

    /// For the Gold Saucer.
    #[brw(magic = 912u32)]
    SetWeeklyLotOffsetTime { offset_time: u32 },

    /// For the Gold Saucer.
    #[brw(magic = 914u32)]
    SetGoldSaucerFlags { flags: u32 },

    /// Forces the player off their current mount.
    #[brw(magic = 915u32)]
    Dismount { sequence: u32 },

    #[brw(magic = 919u32)]
    ToggleOrchestrionUnlock {
        song_id: u32, // Index to Orchestrion sheet
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        unlocked: bool,
        item_id: u32, // Index to Item sheet
    },

    /// Replies to a ServerRequestCallbackManager request (client trigger 9003), such as the quest battle prompt.
    /// Found with Kawari Probe: the client hands this to ServerRequestCallbackManager::ProcessPacket.
    #[brw(magic = 925u32)]
    ServerRequestReply {
        /// The callback slot from the request.
        callback_index: u32,
        /// The command from the request.
        command: u32,
        /// The reply itself, 0 means OK for the quest battle prompt.
        result: u32,
    },

    /// Calls some method in PlayerState, unsure what it does currently.
    #[brw(magic = 924u32)]
    UnkPlayerStateRelated {
        unk1: u32,
        unk2: u32,
        unk3: u32,
        unk4: u32,
    },

    /// Unsure what this is for, but it's sent when riding as a passenger.
    #[brw(magic = 928u32)]
    PillionPassengerRelatedUnk { unk: u32 },

    // TODO: rename this again if it's apparent that it's not purely for the dismount animation, but it's what actually plays the animation when networked...
    // TODO: What do those 3 unks represent? They don't seem to matter to the client (dismounting still works even if they're all zero).
    #[brw(magic = 930u32)]
    PlayDismountAnimation { unk1: u32, unk2: u32, unk3: u32 },

    #[brw(magic = 931u32)]
    BeginContentsReplay {
        unk1: u32, // Always 1
    },

    #[brw(magic = 932u32)]
    EndContentsReplay {
        unk1: u32, // Always 1
    },

    #[brw(magic = 938u32)]
    ToggleOrnamentUnlock {
        id: u32, // Index to Ornament sheet
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        unlocked: bool,
    },

    #[brw(magic = 940u32)]
    SetOrnament {
        /// Index into the Ornament Excel sheet.
        id: u32,
    },

    #[brw(magic = 944u32)]
    SetGlasses {
        // Guessed
        index: u32,
        /// Index into the Glasses Excel sheet.
        id: u32,
    },

    #[brw(magic = 945u32)]
    ToggleGlassesStyleUnlock {
        /// Index to GlassesStyle Excel sheet.
        id: u32,
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        unlocked: bool,
    },

    /// The server acknowledges the client's request to remove an item from the world and put it back into their inventory. Doesn't seem to have any values, but we should keep unknowns for now, just in case...
    #[brw(magic = 1009u32)]
    FurnitureRemovedToInventoryAck {
        unk1: u32,
        unk2: u32,
        unk3: u32,
        unk4: u32,
    },

    /// The server acknowledges the client's request to place an item. Doesn't seem to have any values, but we should keep unknowns for now, just in case...
    #[brw(magic = 1011u32)]
    FurniturePlacedAck {
        unk1: u32,
        unk2: u32,
        unk3: u32,
        unk4: u32,
    },

    /// The server displays the Interior Furnishings menu for the client.
    #[brw(magic = 1015u32)]
    FurnitureMenu,

    /// The server acknowledges the client's request to translate (move or rotate) an item. The purpose of sending the item's container is unclear.
    #[brw(magic = 1017u32)]
    FurnitureTranslatedAck {
        /// What the furniture was stored in.
        storage_id: ContainerType,
        /// Seems to be the plot number of the interior being modified, or zero if in an apartment. Unknown purpose.
        plot_number: u16,
        /// The slot the furniture occupied.
        slot: u32,
    },

    /// The server sets the interior lighting level for an observing client.
    #[brw(magic = 1034u32)]
    InteriorLightLevelForObserver {
        /// The light level set by the resident.
        level: u32,
        /// Same unk from the resident's client trigger. It's always 1.
        unk1: u32,
    },

    /// The server sets the interior lighting level for the client.
    #[brw(magic = 1035u32)]
    InteriorLightLevel {
        unk1: u32,
        /// The light level set by the client.
        level: u32,
        /// This repeats the client trigger's unk back to them. It's always 1.
        unk2: u32,
    },

    #[brw(magic = 1204u32)]
    ToggleTripleTriadCardUnlock {
        id: u32, // Index to TripleTriadCard sheet
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        unlocked: bool,
    },

    /// Called during dueling.
    #[brw(magic = 1504u32)]
    SetPvPState { state: u32 },

    /// Called during dueling.
    #[brw(magic = 1506u32)]
    StartDuelCountdown { opponent_id: ObjectId },

    /// Needed for EnableDutyActions/SetDutyActions to function.
    #[brw(magic = 1512u32)]
    UnkDutyActions { unk1: u32 },

    /// Sets up duty-specific actions like the ones seen in Occult Crescent.
    #[brw(magic = 1514u32)]
    SetDutyActions {
        /// Index into the Action Excel sheet.
        action_ids: [u32; 5],
    },

    /// Unsure, but needed for SetDutyActions to work?
    #[brw(magic = 1515u32)]
    EnableDutyActions {
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        enabled: bool,
    },

    /// Calls into animation-related functions I think?
    #[brw(magic = 1529u32)]
    UnkAnimationRelated,

    /// That's the name of the function in FFXIVClientStructs: ProcessHotDot. On retail it's a heal over time's tick
    /// (Bob's Chronofoil captures): `status_id` 0 for all of a target's HoTs together (a ground effect's status when
    /// it ticks on its own: Asylum 1911, Sacred Soil 1944...), `critical` 1 for a critical tick (×1.5.) Damage over time
    /// ticks are DamageFlyingText (1541).
    #[brw(magic = 1540u32)]
    HotDot {
        /// The source status of this HoT(?)
        status_id: u32,
        /// Amount of damage to display.
        amount: u32,
        /// Who did the damage?
        source_actor_id: ObjectId,
        critical: u32,
    },

    /// Shows the flying text and some VFX on the target, but doesn't display in the log or change anything else. On
    /// retail this is a damage over time tick: `unk1` 0, `unk4` 0xFFFFFFFF (Bob's Chronofoil captures; every DoT on the
    /// target together.)
    #[brw(magic = 1541u32)]
    DamageFlyingText {
        unk1: u32,
        /// Amount of damage to display.
        amount: u32,
        /// Who did the damage?
        source_actor_id: ObjectId,
        unk4: u32,
    },

    #[brw(magic = 1545u32)]
    UnkCooldownsRelated {
        unk1: u32,
        unk2: u32,
        unk3: u32,
        unk4: u32,
    },

    /// Initializes the SpectatorManager on the client.
    #[brw(magic = 1600u32)]
    InitializeSpectatorManager { row_id: u32, unk2: u32, unk3: u32 },

    /// This hides the spectator UI (including the uninitialized one.)
    #[brw(magic = 1601u32)]
    HideSpectatorUI,

    /// This actually shows the Spectator UI.
    #[brw(magic = 1607u32)]
    ShowSpectatorUI,

    /// Calls some method in PvPProfile, unsure what it does yet.
    #[brw(magic = 1610u32)]
    UnkPvPProfileRelated {
        unk1: u32,
        unk2: u32,
        unk3: u32,
        unk4: u32,
    },

    /// Collection UI stuff.
    #[brw(magic = 2251u32)]
    UpdateMcGuffin {
        /// Data specific to this McBuffin.
        data: u32,
        /// Index into the McGuffin Excel sheet.
        id: u32,
        /// If this McGuffin is unlocked or not.
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        unlocked: bool,
    },

    /// Adds this to the FateStartNPCs list and sets the type to what's specified in `ObjectTypeId`.
    #[brw(magic = 2351u32)]
    FateStartNpc { actor_id: ObjectTypeId },

    #[brw(magic = 2352u32)]
    UnkFate2 {
        unk1: u32,
        unk2: u32,
        unk3: u32,
        unk4: u32,
    },

    #[brw(magic = 2353u32)]
    FateInit {
        /// Index into the FATE Excel sheet.
        fate_id: u32,
        /// What state this FATE is in.
        fate_state: FateState,
    },

    #[brw(magic = 2354u32)]
    UnkFate4 {
        unk1: u32,
        unk2: u32,
        unk3: u32,
        unk4: u32,
    },

    #[brw(magic = 2355u32)]
    UnkFate5 {
        unk1: u32,
        unk2: u32,
        unk3: u32,
        unk4: u32,
    },

    /// Creates the FateContext on the client.
    #[brw(magic = 2356u32)]
    CreateFateContext {
        fate_id: u32,
        /// Controls the IsBonus field in some way.
        is_bonus: u32,
    },

    /// Removes a FATE's context from the client. Retail sends it for every FATE of the zone when the player leaves it,
    /// right after PrepareZoning and before terminating the FATE director (Bob's Chronofoil captures, 2026-10-14).
    /// Terminating the director alone left the client's contexts pointing at nothing, and it crashed updating them on
    /// the next map (2026-10-17 crash report). The name is the fork's, from what it does.
    #[brw(magic = 2357u32)]
    RemoveFateContext {
        /// Index into the FATE Excel sheet.
        fate_id: u32,
    },

    /// Associates an NPC with a preparing FATE. You need to talk to this NPC to begin, and also to return items to (if applicable?)
    #[brw(magic = 2359u32)]
    SetupMotivationNpc {
        /// Index into the FATE Excel sheet.
        fate_id: u32,
        /// The NPC to be used.
        motivation_npc: ObjectId,
        unk1: u32,
        /// Only used if the BNpc is not spawned yet.
        #[bw(map = |x: &f32| (*x * 1000.0) as i32)]
        #[br(map = |x: i32| (x as f32) / 1000.0)]
        x: f32,
        #[bw(map = |x: &f32| (*x * 1000.0) as i32)]
        #[br(map = |x: i32| (x as f32) / 1000.0)]
        y: f32,
        #[bw(map = |x: &f32| (*x * 1000.0) as i32)]
        #[br(map = |x: i32| (x as f32) / 1000.0)]
        z: f32,
    },

    /// Puts the collection point icon on top of their head.
    #[brw(magic = 2360u32)]
    SetCollectionPointNpc {
        /// Index into the FATE Excel sheet.
        fate_id: u32,
        /// The NPC to be used.
        motivation_npc: ObjectId,
    },

    /// Needs to be sent otherwise the Level Sync button disappears.
    #[brw(magic = 2362u32)]
    FateLevelSynced { fate_id: u32 },

    #[brw(magic = 2363u32)]
    UnkFate7 {
        unk1: u32,
        unk2: u32,
        unk3: u32,
        unk4: u32,
    },

    /// Sets progress and other fields too. Unsure why this is used sometimes instead of FateProgress?
    #[brw(magic = 2364u32)]
    FateUpdate {
        /// Index into the FATE Excel sheet.
        fate_id: u32,
        progress: u32,
        param: u32,
    },

    #[brw(magic = 2365u32)]
    UnkFate8 {
        unk1: u32,
        unk2: u32,
        unk3: u32,
        unk4: u32,
    },

    #[brw(magic = 2366u32)]
    FateProgress {
        /// Index into the FATE Excel sheet.
        fate_id: u32,
        /// From 0-100 assumedly?
        progress: u32,
    },

    #[brw(magic = 2367u32)]
    UnkFate10 {
        unk1: u32,
        unk2: u32,
        unk3: u32,
        unk4: u32,
    },

    #[brw(magic = 2369u32)]
    UnkFate11 {
        unk1: u32,
        unk2: u32,
        unk3: u32,
        unk4: u32,
    },

    /// Sets something for all objects bound to this FATE...
    #[brw(magic = 2370u32)]
    FateUpdateTargetableStatus {
        /// Index into the FATE Excel sheet.
        fate_id: u32,
    },

    #[brw(magic = 2500u32)]
    FauxHollowsData {
        unk1: u32,
        unk2: u32,
        unk3: u32,
        unk4: u32,
    },

    #[doc(hidden)]
    Unknown {
        category: u32,
        param1: u32,
        param2: u32,
        param3: u32,
        param4: u32,
        param5: u32,
    },
}

impl Default for ActorControlCategory {
    fn default() -> Self {
        Self::Unknown {
            category: 0,
            param1: 0,
            param2: 0,
            param3: 0,
            param4: 0,
            param5: 0,
        }
    }
}

#[binrw]
#[derive(Debug, Clone, Default)]
pub struct ActorControl {
    #[brw(pad_size_to = 24)] // take into account categories without params
    pub category: ActorControlCategory,
}

#[binrw]
#[derive(Debug, Clone, Default)]
pub struct ActorControlSelf {
    #[brw(pad_size_to = 40)] // take into account categories without params
    pub category: ActorControlCategory,
}

#[binrw]
#[derive(Debug, Clone, Default)]
pub struct ActorControlTarget {
    #[brw(pad_size_to = 24)] // take into account categories without params
    pub category: ActorControlCategory,
    pub target: ObjectTypeId,
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use binrw::{BinRead, BinWrite};

    use super::*;

    /// Retail's two ActorControls bringing an ambushing gigas into view, byte for byte (Bob's Copperbell Mines captures
    /// of 2026-10-25.)
    #[test]
    fn appear_and_set_targetable_match_retail() {
        let retail = [
            (
                "24000000010000009e000000000000000000000000000000",
                ActorControlCategory::Appear {
                    fade_in: true,
                    action_timeline: 158,
                },
            ),
            (
                "360000000100000000000000000000000000000000000000",
                ActorControlCategory::SetTargetable { targetable: true },
            ),
        ];
        for (hex, category) in retail {
            let mut written = Cursor::new(Vec::new());
            ActorControl {
                category: category.clone(),
            }
            .write_le(&mut written)
            .unwrap();
            let written: String = written
                .into_inner()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect();
            assert_eq!(written, hex);

            let bytes: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                .collect();
            let read = ActorControl::read_le(&mut Cursor::new(bytes)).unwrap();
            assert_eq!(format!("{:?}", read.category), format!("{category:?}"));
        }
    }

    /// Retail's ActorControlSelf with Seer Folly's soul crystal (Bob's Chronofoil capture 8d550e33.)
    #[test]
    fn job_how_to_matches_retail() {
        let mut written = Cursor::new(Vec::new());
        ActorControlSelf {
            category: ActorControlCategory::JobHowTo { how_to: 94 },
        }
        .write_le(&mut written)
        .unwrap();
        let mut retail = vec![0x1c, 0x02, 0, 0, 0x5e, 0, 0, 0];
        retail.resize(40, 0);
        assert_eq!(written.into_inner(), retail);
    }
}
