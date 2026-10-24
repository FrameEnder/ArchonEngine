use crate::{
    RemakeMode,
    inventory::{CrystalKind, CurrencyKind},
};
use kawari::{
    common::Position,
    ipc::zone::{EventType, GrandCompany, SceneFlags, ServerZoneIpcSegment},
    packet::PacketSegment,
};
use physis::savedata::chardat::CustomizeData;

#[derive(Clone, Debug)]
pub enum LuaTask {
    /// Opens the gathering window of the node being talked to (`ZoneConnection::send_gathering_window`.)
    OpenGatheringWindow,
    /// A gatherer's action was used (`ZoneConnection::use_gathering_action`.)
    GatheringAction {
        action_id: u32,
    },
    ChangeTerritory {
        zone_id: u16,
        exit_position: Option<Position>,
        exit_rotation: Option<f32>,
    },
    SetRemakeMode(RemakeMode),
    Warp {
        warp_id: u32,
    },
    BeginLogOut,
    FinishEvent,
    UnlockClassJob {
        classjob_id: u8,
    },
    WarpAetheryte {
        aetheryte_id: u32,
        housing_aethernet: bool,
        /// The Teleport spell (retail's warp type 4, which ends the teleport animation), rather than an aethernet or
        /// housing move.
        teleport: bool,
    },
    ToggleInvisibility {
        invisible: bool,
    },
    Unlock {
        id: u32,
    },
    UnlockAll,
    UnlockAetheryte {
        id: u32,
        on: bool,
    },
    SetLevel {
        level: u16,
    },
    /// A notice from an event, e.g. a quest's progress pop-up.
    EventNotice {
        handler_id: u32,
        notice_id: u8,
        args: Vec<u32>,
    },
    /// Only the packets of a nested event (EventStart/EventFinish with type Nest, on the current event's target):
    /// retail starts the aetheryte's event inside Close to Home's while attuning. Kawari doesn't run a handler for it.
    NestedEventPacket {
        handler_id: u32,
        start: bool,
    },
    ChangeWeather {
        id: u8,
    },
    ModifyCurrency {
        id: CurrencyKind,
        amount: i32,
        send_client_update: bool,
    },
    ModifyCrystal {
        id: CrystalKind,
        amount: i32,
        send_client_update: bool,
    },
    GmSetOrchestrion {
        value: bool,
        id: u32,
    },
    ToggleOrchestrion {
        id: u32,
    },
    AddItem {
        id: u32,
        quantity: i32,
        send_client_update: bool,
    },
    UnlockContent {
        id: u16,
    },
    UnlockAllContent,
    AddExp {
        amount: i32,
    },
    StartEvent {
        event_id: u32,
        event_type: EventType,
        event_arg: u32,
    },
    SetInnWakeup {
        watched: bool,
    },
    ToggleMount {
        id: u32,
    },
    MoveToPopRange {
        id: u32,
        fade_out: bool,
    },
    SetHP {
        hp: u32,
    },
    SetMP {
        mp: u16,
    },
    SpawnQuestBattleNpc {
        layout_id: u32,
    },
    /// A summoning bell was used: its list (scene 0), or scene 1001 without retainers.
    OpenSummoningBell,
    /// Call the retainer with this ID (from the bell's list) and play their menu.
    SummonRetainer {
        id: u64,
    },
    /// Send the called retainer away and play the list again.
    DismissRetainer,
    /// `!retainer hire`.
    HireRetainer {
        name: String,
        classjob: u8,
        level: u8,
    },
    /// `!retainer list`.
    ListRetainers,
    /// Pay the gil for teleporting to this aetheryte from here.
    PayForTeleport {
        aetheryte_id: u32,
    },
    /// An item an enemy dropped, obtained directly (with retail's loot message.)
    ObtainDrop {
        id: u32,
        quantity: i32,
    },
    /// The retainer list as the vocate sends it, carrying the menu choice.
    SendRetainerListAtVocate {
        choice: u32,
    },
    /// Hire the retainer chosen at the vocate (appearance and name from the client), answering the yield.
    HireRetainerAtVocate {
        scene: u16,
        yield_id: u8,
    },
    /// Let a retainer go at the vocate, answering the yield.
    ReleaseRetainerAtVocate {
        id: u64,
        scene: u16,
        yield_id: u8,
    },
    /// ActorControlSelf 152: how many retainers the character has.
    SendRetainerCount,
    /// Turn a condition on or off (e.g. CreatingCharacter while the retainer appearance editor is open.)
    SetCondition {
        condition: kawari::ipc::zone::Condition,
        on: bool,
    },
    /// `!retainer release`.
    ReleaseRetainer {
        name: String,
    },
    ToggleGlassesStyle {
        id: u32,
    },
    ToggleGlassesStyleAll,
    ToggleOrnament {
        id: u32,
    },
    ToggleOrnamentAll,
    /// Complete an achievement (announced), or take it away with `clear`.
    SetAchievement {
        id: u32,
        clear: bool,
    },
    /// Complete every achievement the character meets the requirements of.
    CheckAchievements,
    /// Learn what an item teaches (a mount, an emote or hairstyle by unlock link, a minion…) and use the item up, as
    /// using one does on retail. `kind` is one of `ItemUnlock`'s names (`items/Items.lua` passes them).
    LearnFromItem {
        kind: String,
        id: u32,
        item_id: u32,
    },
    UnlockBuddyEquip {
        id: u32,
    },
    UnlockBuddyEquipAll,
    ToggleChocoboTaxiStand {
        id: u32,
    },
    ToggleChocoboTaxiStandAll,
    ToggleCaughtFish {
        id: u32,
    },
    ToggleCaughtFishAll,
    ToggleCaughtSpearfish {
        id: u32,
    },
    ToggleCaughtSpearfishAll,
    ToggleTripleTriadCard {
        id: u32,
    },
    ToggleTripleTriadCardAll,
    ToggleAdventure {
        id: u32,
    },
    ToggleAdventureAll,
    ToggleCutsceneSeen {
        id: u32,
        value: bool,
    },
    ToggleCutsceneSeenAll,
    ToggleMinion {
        id: u32,
    },
    ToggleMinionAll,
    ToggleAetherCurrent {
        id: u32,
    },
    ToggleAetherCurrentAll,
    ToggleAetherCurrentCompFlgSet {
        id: u32,
    },
    ToggleAetherCurrentCompFlgSetAll,
    // previously, this was kept as a separate thing apart from tasks
    // but I discovered that this doesn't mix well - for example with play_scene (segment-based) and start_event (task)
    // this is because segments were always sent before tasks and there wasn't strong ordering
    // so be careful when changing this system!
    SendSegment {
        segment: PacketSegment<ServerZoneIpcSegment>,
    },
    // NOTE: this is mostly a workaround in a limitation in the scripting API
    StartTalkEvent,
    AcceptQuest {
        id: u32,
    },
    FinishQuest {
        id: u32,
        /// The optional reward the player picked, as an item id. 0 if there's none.
        optional_item: u32,
    },
    GainStatusEffect {
        effect_id: u16,
        effect_param: u16,
        duration: f32,
    },
    RegisterForContent {
        content_id: u16,
    },
    QuestSequence {
        id: u32,
        sequence: u8,
    },
    QuestVariable {
        id: u32,
        index: u8,
        value: u8,
    },
    CancelQuest {
        id: u32,
    },
    /// Mark a quest completed with no rewards and no scene, as retail does for a quest another one replaces (Lurkers
    /// in the Grotto, 543, completes 464 with it.)
    MarkQuestCompleted {
        id: u32,
    },
    IncompleteQuest {
        id: u32,
    },
    Kill,
    AbandonContent,
    SetHomepoint {
        homepoint: u16,
    },
    ReturnToHomepoint,
    JoinContent {
        id: u32,
    },
    JoinQuestBattle {
        id: u32,
        zone_id: u16,
    },
    /// Starts an event action (a progress bar, like when examining something) during the current quest event.
    EventAction {
        id: u32,
    },
    FinishCastingGlamour,
    PlayScene {
        scene: u16,
        scene_flags: SceneFlags,
        params: Vec<u32>,
    },
    ResumeEvent {
        scene: u16,
        resume_id: u8,
        params: Vec<u32>,
    },
    WarpPopRange {
        territory_id: u16,
        pop_range_id: u32,
    },
    RemoveCooldowns,
    ToggleHowTo {
        value: bool,
        id: u32,
    },
    SendMailboxStatus,
    SetGrandCompany {
        company: GrandCompany,
    },
    SetGrandCompanyRank {
        rank: u8,
    },
    Jump {
        name: String,
    },
    Call {
        name: String,
    },
    FinishDyeing,
    SetCustomize {
        customize: CustomizeData,
    },
}
