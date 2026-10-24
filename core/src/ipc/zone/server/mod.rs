use binrw::binrw;
use kawari_core_macro::opcode_data;
use physis::savedata::chardat::CustomizeData;

use super::OnlineStatusMask;
pub use super::social_list::{SocialList, SocialListUIFlags, SocialListUILanguages};

mod chara_card;
pub use chara_card::CharaCardData;

mod chara_info;
use chara_info::CharaInfoFromContentIdsData;

mod spawn_player;
pub use spawn_player::SpawnPlayer;

mod status_effect;
pub use status_effect::StatusEffect;

mod update_class_info;
pub use update_class_info::UpdateClassInfo;

mod player_setup;
pub use player_setup::PlayerSetup;

mod player_stats;
pub use player_stats::PlayerStats;

mod actor_control;
pub use actor_control::{
    ActorControl, ActorControlCategory, ActorControlSelf, ActorControlTarget, BaseStat,
    LiveEventType,
};

mod zone_init;
pub use zone_init::{ZoneInit, ZoneInitFlags};

mod spawn_npc;
pub use spawn_npc::{CharacterDataFlag, SpawnNpc};

mod common_spawn;
pub use common_spawn::{
    BattleNpcSubKind, CommonSpawn, DisplayFlag, GameMasterRank, ObjectKind, PlayerSubKind,
};

mod status_effect_list;
pub use status_effect_list::StatusEffectList;

mod weather_change;
pub use weather_change::WeatherChange;

mod container_info;
pub use container_info::ContainerInfo;

mod item_info;
pub use item_info::ItemInfo;

mod event_scene;
pub use event_scene::{EventScene, SceneFlags};

mod event_notice;
pub use event_notice::EventNotice;

mod event_start;
pub use event_start::{EventStart, EventType};

mod action_effect;
pub use action_effect::{
    ActionEffect, ActionEffectFlag, DamageElement, DamageKind, DamageType, TargetEffect,
    TargetEffectKind,
};

mod actor_set_pos;
pub use actor_set_pos::ActorSetPos;

mod equip;
pub use equip::Equip;

mod currency_info;
pub use currency_info::CurrencyInfo;

pub use super::config::Config;

mod spawn_object;
pub use spawn_object::SpawnObject;

mod quest_active_list;
pub use quest_active_list::{ActiveQuest, QuestActiveList};

mod effect_result;
pub use effect_result::{EffectEntry, EffectResult};

mod condition;
pub use condition::{Condition, Conditions};

mod chat_message;
pub use chat_message::ChatMessage;

mod free_company;
pub use free_company::FcHierarchy;

mod actor_move;
use crate::common::{
    DeepDungeonRoomFlag, HandlerId, LandData, ObjectTypeId, Position, read_packed_position,
    read_quantized_rotation, write_packed_position, write_quantized_rotation,
};
use crate::constants::{
    ACHIEVEMENT_BITMASK_SIZE, ADVENTURER_PLATE_PORTRAIT_SIZE, AVAILABLE_CLASSJOBS,
    COMPLETED_LEGACY_QUEST_BITMASK_SIZE, COMPLETED_LEVEQUEST_BITMASK_SIZE,
    COMPLETED_QUEST_BITMASK_SIZE, COMPLETED_RECIPES_BITMASK_SIZE,
    GATHERED_GATHERING_ITEMS_BITMASK_SIZE, TITLE_UNLOCK_BITMASK_SIZE,
    UNLOCKED_MAP_MARKERS_BITMASK_SIZE,
};
pub use crate::ipc::zone::server::actor_move::ActorMove;

mod server_notice;
pub use server_notice::{ServerNoticeFlags, ServerNoticeMessage};

mod quest_tracker;
pub use quest_tracker::{QuestTracker, TrackedQuest};

mod apartment_list;
pub use apartment_list::{ApartmentList, ApartmentListEntry};

mod house_list;
pub use house_list::{House, HouseExterior, HouseList, HouseStatus};

mod housing_ward;
pub use housing_ward::{HousingWardInfo, HousingWardSummaryItem};

mod housing_interior_furniture;
pub use housing_interior_furniture::{
    Furniture, FurnitureList, FurnitureTranslatedForObserver, HousingInteriorDetails,
};

mod housing_occupied_land_info;
pub use housing_occupied_land_info::HousingOccupiedLandInfo;

mod housing_vacant_land_info;
pub use housing_vacant_land_info::HousingVacantLandInfo;

mod housing_estate_greeting;
pub use housing_estate_greeting::HousingEstateGreeting;

mod trust_information;
pub use trust_information::{TrustContent, TrustInformation};

mod event_resume;
pub use event_resume::EventResume;

mod map_markers;
pub use map_markers::MapMarkers;

mod enmity_list;
pub use enmity_list::{EnmityList, PlayerEnmity};

mod hater_list;
pub use hater_list::{Hater, HaterList};

mod map_effects;
pub use map_effects::MapEffects;

mod marketboard;
pub use marketboard::{
    MARKETBOARD_HISTORY_ENTRIES, MARKETBOARD_ITEMS_PER_PAGE, MARKETBOARD_LISTINGS_PER_PAGE,
    MarketBoardHistory, MarketBoardHistoryEntry, MarketBoardItem, MarketBoardListing,
    MarketBoardPurchase,
};

mod linkshell;
pub use linkshell::*;

mod retainer;
pub use retainer::{
    ItemMarketBoardInfo, ItemMarketBoardSummary, PLAYER_RETAINER_ENTRIES, PlayerRetainerEntry,
    PlayerRetainerInfo, RETAINER_STATE_DISMISSED, RETAINER_STATE_HIRED, RETAINER_STATE_SUMMONED,
    RetainerSaleHistory, RetainerState,
};

mod fellowships;
pub use fellowships::*;

mod spawn_treasure;
pub use spawn_treasure::{SpawnTreasure, TreasureKind};

mod mogpendium;
pub use mogpendium::{Mogpendium, MogpendiumCompletionFlags};

mod cross_realm_listing;
pub use cross_realm_listing::{CrossRealmListing, CrossRealmListings};

mod prepare_zoning;
pub use prepare_zoning::{PrepareZoning, PrepareZoningFlag};

mod mail;
pub use mail::{
    AttachedItemInfo, LETTER_MSG_MAX_LENGTH, Letter, LetterPreview, LetterType, MAX_ATTACHMENTS,
    MAX_FRIEND_LETTERS, MAX_MAIL, MAX_MAIL_ATTACHMENTS_STORAGE, MAX_REWARD_LETTERS,
    MAX_SYSTEM_LETTERS, PREVIEW_MSG_MAX_LENGTH,
};

mod aoe_effect;
pub use aoe_effect::AoeEffect;

mod duty_support_information;
pub use duty_support_information::DutySupportInformation;

mod walk_in_event;
pub use walk_in_event::{WalkInEvent, WalkInEventType};

mod dice_roll_result;
pub use dice_roll_result::ZoneDiceRollResult;

mod event_log_message;
pub use event_log_message::EventLogMessage;

use crate::common::{
    CHAR_NAME_MAX_LENGTH, ContainerType, ItemOperationKind, ObjectId, read_bool_from, read_string,
    write_bool_as, write_string,
};
pub use crate::ipc::zone::black_list::{Blacklist, BlacklistedCharacter};
use crate::opcodes::ServerZoneIpcType;
use crate::packet::IpcSegment;
use crate::packet::ServerIpcSegmentHeader;

use crate::ipc::{
    chat::ChatChannel,
    zone::{
        DutyFinderSetting, PartyMemberEntry, PartyMemberPositions, PartyUpdateStatus,
        StrategyBoard, StrategyBoardUpdate, WaymarkPlacementMode, WaymarkPosition, WaymarkPreset,
    },
};

use crate::ipc::zone::social_list::{FriendGroupIconInfo, GrandCompany};
use crate::ipc::zone::{ActionType, InviteReply, InviteType, InviteUpdateType, SearchInfo};

pub type ServerZoneIpcSegment =
    IpcSegment<ServerIpcSegmentHeader<ServerZoneIpcType>, ServerZoneIpcType, ServerZoneIpcData>;

#[opcode_data(ServerZoneIpcType)]
#[binrw]
#[br(import(magic: &ServerZoneIpcType, size: &u32))]
#[derive(Debug, Clone)]
pub enum ServerZoneIpcData {
    InitResponse {
        /// The actor id of the player logging in.
        #[brw(pad_before = 8, pad_after = 4)] // empty
        actor_id: ObjectId,
    },
    ZoneInit(ZoneInit),
    ActorControlSelf(ActorControlSelf),
    PlayerStats(PlayerStats),
    PlayerSetup(PlayerSetup),
    UpdateClassInfo(UpdateClassInfo),
    SpawnPlayer(SpawnPlayer),
    LogOutComplete {
        unk: [u8; 8],
    },
    ActorSetPos(ActorSetPos),
    ServerNoticeMessage(ServerNoticeMessage),
    PrepareZoning(PrepareZoning),
    ActorControl(ActorControl),
    ActorMove(ActorMove),
    SocialList(SocialList),
    SpawnNpc(SpawnNpc),
    StatusEffectList(StatusEffectList),
    WeatherId(WeatherChange),
    UpdateItem(ItemInfo),
    ContainerInfo(ContainerInfo),
    EventResume2(#[brw(args { max_params: 2 } )] EventResume),
    EventResume4(#[brw(args { max_params: 4 } )] EventResume),
    EventResume8(#[brw(args { max_params: 8 } )] EventResume),
    EventScene2(#[brw(args { max_params: 2 } )] EventScene),
    EventScene4(#[brw(args { max_params: 4 } )] EventScene),
    EventScene8(#[brw(args { max_params: 8 } )] EventScene),
    EventScene16(#[brw(args { max_params: 16 } )] EventScene),
    EventScene32(#[brw(args { max_params: 32 } )] EventScene),
    EventScene64(#[brw(args { max_params: 64 } )] EventScene),
    EventScene128(#[brw(args { max_params: 128 } )] EventScene),
    EventScene255(#[brw(args { max_params: 255 } )] EventScene),
    EventNotice2(#[brw(args { max_args: 2 } )] EventNotice),
    EventNotice4(#[brw(args { max_args: 4 } )] EventNotice),
    EventStart(EventStart),
    UpdateHpMpTp {
        /// The new health point value.
        hp: u32,
        /// The new resource point value.
        mp: u16,
        /// The maximum MP (Bob's Chronofoil captures: 10000 for players.)
        max_mp: u16,
    },
    ActionEffect1(#[brw(args { max_targets: 1 } )] ActionEffect),
    Equip(Equip),
    DeleteActor {
        /// The index into the client-side object pool.
        spawn_index: u8,
        /// The ID of the actor being deleted.
        #[brw(pad_before = 3)] // padding
        actor_id: ObjectId,
    },
    EventFinish {
        /// ID of this event.
        handler_id: HandlerId,
        /// Type of this event.
        event_type: EventType,
        /// Arbitrary value.
        result: u8,
        /// Arbitrary value.
        #[brw(pad_before = 2)] // padding
        #[brw(pad_after = 4)] // padding
        arg: u32,
    },
    Condition(Conditions),
    ActorControlTarget(ActorControlTarget),
    CurrencyCrystalInfo(CurrencyInfo),
    Config(Config),
    InventoryActionAck {
        sequence: u32, // TODO: FFXIVClientStructs thinks is this "context_id", what does that mean...?
        action_type: u8,
        #[brw(pad_after = 2)] // not read
        unk1: u8,
        /// Only used if `action_type` is 3??
        #[brw(pad_after = 6)] // not read
        equipped_items_inventory_type: ContainerType,
    },
    PingSyncReply {
        timestamp: u32,
        #[brw(pad_after = 24)]
        transmission_interval: u32,
    },
    QuestCompleteList {
        /// Bitmask of completed quests.
        #[br(count = COMPLETED_QUEST_BITMASK_SIZE)]
        #[brw(pad_size_to = COMPLETED_QUEST_BITMASK_SIZE)]
        completed_quests: Vec<u8>,
        #[brw(pad_after = 1)] // unused I guess
        #[br(count = UNLOCKED_MAP_MARKERS_BITMASK_SIZE)]
        #[brw(pad_size_to = UNLOCKED_MAP_MARKERS_BITMASK_SIZE)]
        unlocked_map_markers: Vec<u8>,
    },
    UnkResponse2 {
        #[brw(pad_after = 7)]
        unk1: u8,
    },
    InventoryTransaction {
        /// This is later reused in InventoryTransactionFinish, so it might be some sort of sequence or context id, but it's not the one sent by the client.
        sequence: u32,
        /// Same as the one sent by the client, not the one that the server responds with in InventoryActionAck!
        operation_type: ItemOperationKind,
        src_actor_id: ObjectId,
        #[brw(pad_size_to = 4)]
        src_storage_id: ContainerType,
        src_container_index: u16,
        #[brw(pad_before = 2)]
        src_stack: u32,
        src_item_id: u32,

        /// This section was observed to be static, across two captures and a bunch of discards these never changed.
        /// Always set to 0xE000_0000, also known as no/invalid actor.
        dst_actor_id: ObjectId,
        /// Used in discard operations, both this dummy container and dst_storage_id are set to a container type of 0xFFFF.
        /// While this struct is nearly identical to ItemOperation, it deviates here by not having 2 bytes of padding.
        dummy_container: ContainerType,
        dst_storage_id: ContainerType,
        dst_container_index: u16,
        /// Always set to zero.
        #[brw(pad_before = 2)]
        dst_stack: u32,
        /// Always set to zero.
        dst_item_id: u32,
    },
    InventoryTransactionFinish {
        /// Same sequence value as in InventoryTransaction.
        sequence: u32,
        /// Repeated unk1 value. No, it's not a copy-paste error.
        sequence_repeat: u32,
        /// Unknown, seems to always be 0x00000090.
        operation_type: ItemOperationKind,
        /// Unknown, seems to always be 0x00000200.
        unk2: u32,
    },
    ContentFinderUpdate {
        /// 0 = Nothing happens
        /// 1 = Reserving server
        /// 2 = again? ^
        /// 3 = duty ready
        /// 4 = checking member status
        /// nothing appears to happen above 5
        state1: u8,
        /// The class you registered with. Index into the ClassJob Excel sheet.
        classjob_id: u8,
        unk1: [u8; 6],
        duty_settings: DutyFinderSetting,
        unk2: [u8; 4],
        /// The content IDs you registered for. Index into the ContentFinderCondition Excel sheet.
        content_ids: [u16; 5],
        unk3: [u8; 10],
    },
    ContentFinderFound {
        unk1: [u8; 8],
        duty_settings: DutyFinderSetting,
        unk2: [u8; 12],
        /// The content ID that popped. Index into the ContentFinderCondition Excel sheet.
        content_id: u16,
        unk3: [u8; 10],
    },
    SpawnObject(SpawnObject),
    ActorGauge {
        /// The class (ideally the one you actually are) to update the gauge for. Index into the ClassJob Excel sheet.
        classjob_id: u8,
        /// Arbitrary class-specific data.
        data: [u8; 15],
    },
    FreeCompanyInfo {
        unk: [u8; 80],
    },
    TitleList {
        /// Bitmask of unlocked titles.
        #[br(count = TITLE_UNLOCK_BITMASK_SIZE)]
        #[brw(pad_size_to = TITLE_UNLOCK_BITMASK_SIZE)]
        unlock_bitmask: Vec<u8>,
    },
    QuestActiveList(QuestActiveList),
    LevequestCompleteList {
        /// Bitmask of completed levequests.
        #[br(count = COMPLETED_LEVEQUEST_BITMASK_SIZE)]
        #[brw(pad_size_to = COMPLETED_LEVEQUEST_BITMASK_SIZE)]
        completed_levequests: Vec<u8>,
        #[br(count = 6)]
        #[brw(pad_size_to = 6)]
        unk2: Vec<u8>,
    },
    EventLogMessage2(#[brw(args { max_params: 2 } )] EventLogMessage),
    EventLogMessage4(#[brw(args { max_params: 4 } )] EventLogMessage),
    UpdateInventorySlot(ItemInfo),
    EffectResult(EffectResult),
    ContentFinderCommencing {
        unk1: [u8; 24],
    },
    StatusEffectList3 {
        /// List of status effects.
        status_effects: [StatusEffect; 30],
    },
    CrossworldLinkshells {
        /// List of cross-world linkshells.
        #[brw(pad_before = 8)] // Seems to be empty/zeroes
        #[br(count = CrossworldLinkshell::COUNT)]
        #[brw(pad_size_to = CrossworldLinkshell::COUNT * CrossworldLinkshell::SIZE)]
        linkshells: Vec<CrossworldLinkshell>,
    },
    SetSearchInfo(SearchInfo),
    Blacklist(Blacklist),
    WalkInEvent(WalkInEvent),
    GrandCompanyInfo {
        /// Which Grand Company this player is affiliated with.
        active_company_id: GrandCompany,
        /// Maelstrom rank.
        maelstrom_rank: u8,
        /// Twin Adder rank.
        twin_adder_rank: u8,
        /// Immortal Flames rank.
        #[brw(pad_after = 4)]
        immortal_flames_rank: u8,
    },
    CraftingLog {
        #[brw(pad_after = 7)] // unaccounted for in the CS size
        #[br(count = COMPLETED_RECIPES_BITMASK_SIZE)]
        #[brw(pad_size_to = COMPLETED_RECIPES_BITMASK_SIZE)]
        bitmask: Vec<u8>,
    },
    GatheringLog {
        #[brw(pad_after = 2)] // unaccounted for in the CS size
        #[br(count = GATHERED_GATHERING_ITEMS_BITMASK_SIZE)]
        #[brw(pad_size_to = GATHERED_GATHERING_ITEMS_BITMASK_SIZE)]
        bitmask: Vec<u8>,
    },
    Fellowships {
        unk1: [u8; 808],
    },
    DailyQuests {
        unk1: [u8; 56],
    },
    DailyQuestRepeatFlags {
        unk1: [u8; 8],
    },
    Linkshells {
        /// List of linkshells.
        #[br(count = LinkshellEntry::COUNT)]
        #[brw(pad_size_to = LinkshellEntry::SIZE * LinkshellEntry::COUNT)]
        shells: Vec<LinkshellEntry>,
    },
    ChatMessage(ChatMessage),
    LocationDiscovered {
        map_part_id: u32,
        map_id: u32,
    },
    Mount {
        /// Index into the Mount Excel sheet.
        id: u16,
        unk1: [u8; 14],
    },
    SetOnlineStatus(OnlineStatusMask),
    FreeCompanyGreeting {
        unk: u8, // TODO: What is this? Seems to commonly be 0x01 or 0x02. Could this opcode be used as a general updater? Needs more research.
        #[brw(pad_size_to = 192)]
        #[br(count = 192)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        #[brw(pad_after = 7)]
        message: String,
    },
    CharaInfoFromContentIds {
        #[brw(pad_before = 8)] // empty
        #[br(count = 10)]
        #[brw(pad_size_to = 10 * CharaInfoFromContentIdsData::SIZE)]
        info: Vec<CharaInfoFromContentIdsData>,
    },
    InviteCharacterResult {
        /// The invited character's content id.
        content_id: u64,
        /// The pre-defined LogMessage to display. 0 seems to indicate no errors, and the client will display a default message such as "You invite \[name\] to a party."
        message_id: u16,
        #[brw(pad_before = 2)]
        /// The invited character's home world id.
        world_id: u16,
        /// The type of social invite that was sent.
        invite_type: InviteType,
        unk1: u8, // TODO: What is this?
        /// The invited character's name.
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        character_name: String,
    },
    InviteReplyResult {
        content_id: u64,
        #[brw(pad_before = 4)]
        invite_type: InviteType,
        response: InviteReply,
        unk1: u8,
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        #[brw(pad_after = 1)]
        character_name: String,
    },
    InviteUpdate {
        sender_account_id: u64,
        #[brw(pad_after = 8)] // empty
        sender_content_id: u64,
        expiration_timestamp: u32, // usually the packet's timestamp + 300
        world_id: u16,
        #[brw(pad_after = 1)] // Pretty sure this is empty
        invite_type: InviteType,
        update_type: InviteUpdateType,
        unk1: u8, // TODO: Usually 1? What is this?
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        #[brw(pad_after = 6)] // empty
        sender_name: String,
    },
    /// This opcode informs the client about members leaving, joining, going offline, if the party is disbanding, and even handles ready checking directly within it. When a ready check is initiated, the target_content_id field is treated differently and used to keep track of the party's votes. While further information can be found below on the unk2 field, most of this process is described in more detail in party_misc.rs, on the ReadyCheckReply struct.
    PartyUpdate {
        execute_account_id: u64,
        target_account_id: u64,
        execute_content_id: u64,
        target_content_id: u64,
        unk1: u8, // TODO: Usually 1? What is this?
        /// This field seems to control what "mode" the target_content_id field operates in. During ready checks, this field is set to zero, and 2 otherwise. It's unclear at this time what 2 represents. When this field is set to zero, the client seems to treat the target_content_id as a pseudo-array of 8 bytes that indicate the party's yes or no votes for ready checks.
        unk2: u8,
        update_status: PartyUpdateStatus,
        unk3: u8, // TODO: Usually 2? What is this?
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        execute_name: String,
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        #[brw(pad_after = 3)] // empty
        target_name: String,
    },
    PartyList {
        #[br(count = PartyMemberEntry::NUM_ENTRIES)]
        #[brw(pad_size_to = PartyMemberEntry::NUM_ENTRIES * PartyMemberEntry::SIZE)]
        members: Vec<PartyMemberEntry>,
        party_id: u64,
        party_chatchannel: ChatChannel,
        leader_index: u8,
        #[brw(pad_after = 6)]
        member_count: u8,
    },
    PartyMemberPositions(PartyMemberPositions),
    AcceptQuest {
        /// Row ID - 65535
        #[brw(pad_after = 4)]
        quest_id: u32,
    },
    UpdateQuest {
        // TODO: index into what?
        #[brw(pad_after = 3)]
        index: u8,
        #[brw(pad_after = 4)] // seems empty
        quest: ActiveQuest,
    },
    FinishQuest {
        /// Row ID - 65535
        quest_id: u16,
        flag1: u8,
        #[brw(pad_after = 4)]
        flag2: u8,
    },
    UpdateMapMarkers4(#[brw(args { max_params: 4 } )] MapMarkers),
    UpdateMapMarkers8(#[brw(args { max_params: 8 } )] MapMarkers),
    UpdateMapMarkers16(#[brw(args { max_params: 16 } )] MapMarkers),
    UpdateMapMarkers32(#[brw(args { max_params: 32 } )] MapMarkers),
    QuestTracker(QuestTracker),
    HouseList(HouseList),
    HousingWardInfo(HousingWardInfo),
    HousingOccupiedLandInfo(HousingOccupiedLandInfo),
    HousingVacantLandInfo(HousingVacantLandInfo),
    HousingEstateGreeting(HousingEstateGreeting),
    SupportDeskNotification {
        unk1: [u8; 16],
    },
    ScenarioGuide {
        /// Not sure what this controls.
        quest_id_1: u32,
        /// Quest ID (Row ID - 65535) shown in big text. The next job quest is automatically determined.
        next_quest_id: u32,
        /// The game object ID to center on when opening the map.
        #[brw(pad_before = 4, pad_after = 16)] // seems empty
        layout_id: u32,
    },
    LegacyQuestList {
        #[br(count = COMPLETED_LEGACY_QUEST_BITMASK_SIZE)]
        #[brw(pad_size_to = COMPLETED_LEGACY_QUEST_BITMASK_SIZE)]
        bitmask: Vec<u8>,
    },
    DirectorVars {
        /// ID of this director.
        handler_id: HandlerId,
        flag: u8,
        branch: u8,
        data: [u8; 10],
        unk1: u16,
        unk2: u16,
        unk3: u16,
        unk4: u16,
    },
    UnkDirector1 {
        unk: [u8; 32],
    },
    PartyMemberPortraits {
        unk: [u8; 184],
    },
    FieldMarkerPreset(WaymarkPreset),
    DeleteObject {
        /// Index into the client-side object spawn pool.
        #[brw(pad_after = 7)] // padding
        spawn_index: u8,
    },
    GoldSaucerInformation {
        unk: [u8; 40],
    },
    UnkContentFinder {
        unk: [u8; 16],
    },
    TrustInformation(TrustInformation),
    DutySupportInformation(DutySupportInformation),
    /// The adventurer plate's portrait (the reply to `ClientTriggerCommand::OpenPortraitsWindow`), as the client
    /// sent it in `ClientZoneIpcData::SetAdventurerPlatePortrait`. The client copies it into CharaCard's
    /// CurrentBannerData (FFXIVClientStructs' BannerData): first byte set when there is one, then the expression,
    /// lighting, pose, camera, background, frame and decoration.
    PortraitsInformation {
        #[br(count = ADVENTURER_PLATE_PORTRAIT_SIZE)]
        #[brw(pad_size_to = ADVENTURER_PLATE_PORTRAIT_SIZE)]
        #[brw(pad_after = 4)]
        portrait: Vec<u8>,
    },
    /// A marketboard purchase went through: the reply to `ClientZoneIpcData::MarketBoardPurchaseHandler`, after the gil
    /// and the item's inventory updates.
    MarketBoardPurchase(MarketBoardPurchase),
    /// Loot (Bob's Chronofoil captures): `kind` 3 an item obtained directly (who, item, count), 9 gil (amount), 0 a
    /// need/greed roll and 1 its winner (who, 1 need / 2 greed, roll, item, count), 7 someone cast a lot, 2 and 8
    /// can't obtain (self, item, count, reason.)
    LootMessage {
        kind: u32,
        params: [u32; 7],
    },
    /// Sent before every RetainerState and after a retainer's containers (Bob's 2026-10-18 capture): zeros, or
    /// `market_changed` 1 before sending away a retainer whose market listings changed.
    RetainerStateNotice {
        unk: u32,
        market_changed: u32,
    },
    /// The answer to `OpenDutySupportFor`: the duty (DawnContent) the Duty Support window opens at, then 0 (Bob's
    /// Chronofoil capture of 2026-10-23, at Sastasha's entrance: 200, 0.)
    DutySupportContent {
        dawn_content_id: u32,
        unk: u32,
    },
    /// The gathering points' phases: one byte per phase slot, picking which of a point's positions the client shows
    /// (the phase modulo how many it has; QuestManager +0x7AC, read by 7.57 `0x140E8E8F0`.) Retail sends it once at
    /// login, with values 0-5 that differ every time (Bob's Chronofoil captures: 42 logins.)
    GatheringPointPhases {
        phases: [u8; 160],
    },
    /// What each progress and quality action would do now, for the Synthesis window (Bob's Chronofoil capture of
    /// 2026-10-24: before the window opens and after every step; 7.57's handler hands it to agent 0x178.)
    CraftingActionPreviews {
        actions: [CraftingActionPreview; 21],
        #[brw(pad_after = 4)]
        unk: u32,
    },
    /// How many gathers in a row at this node have succeeded so far (1, 2, 3…; 0 when the node's window closes. Bob's
    /// Chronofoil capture of 2026-10-24; 7.57's HandleGatheringSuccessfulChainCountPacket.)
    GatheringChainCount {
        #[brw(pad_after = 4)]
        count: u32,
    },
    /// One slot of the gathering points' phases changes (7.57's handler writes the byte to QuestManager +0x7AC.)
    GatheringPointPhase {
        slot: u8,
        #[brw(pad_after = 6)]
        phase: u8,
    },
    /// A retainer is called or sent away at a summoning bell, or hired.
    RetainerState(RetainerState),
    /// The character's retainers for the marketboard's listings.
    PlayerRetainerInfo(PlayerRetainerInfo),
    /// The asking price of one item a retainer has on the market.
    ItemMarketBoardInfo(ItemMarketBoardInfo),
    /// How many items a retainer has on the market.
    ItemMarketBoardSummary(ItemMarketBoardSummary),
    /// A retainer's sale history.
    RetainerSaleHistory(RetainerSaleHistory),
    /// The completed achievements, the reply to `ClientTriggerCommand::RequestAchievements` (FFXIVClientStructs'
    /// AchievementsPacket; the client's handler copies it into its Achievement state and marks it loaded.)
    Achievements {
        /// Achievement Excel sheet rows, lowest bit first.
        #[br(count = ACHIEVEMENT_BITMASK_SIZE)]
        #[brw(pad_size_to = ACHIEVEMENT_BITMASK_SIZE)]
        completed: Vec<u8>,
        /// The last five completed, newest first.
        history: [u16; 5],
        /// Copied by the client next to its near-completion data; unknown.
        #[brw(pad_after = 6)]
        unk: [u8; 26],
    },
    InitializeObfuscation {
        unk_before: [u8; 6],
        /// Zero means "no obsfucation" (not really, but functionally yes.)
        /// To enable obsfucation, you need to set this to a constant that changes every patch. See lib.rs for the constant.
        obsfucation_mode: u8,
        /// First seed used in deobsfucation on the client side.
        seed1: u8,
        /// Second seed used in deobsfucation on the client side.
        seed2: u8,
        #[brw(pad_before = 3)] // seems empty
        /// Third seed used in deobsfucation on the client side.
        seed3: u32,
    },
    StrategyBoardReceivedAck {
        /// The client ID of the player who received the board.
        content_id: u64,
        #[brw(pad_after = 4)] // Seems to be empty/always zeroes
        /// Unknown, possibly a result value. Observed as 1.
        unk: u32,
    },
    BeginStrategyBoardSession {
        /// All of these unknowns are possibly booleans or bitflags. See zone_connection/social.rs::received_strategy_board.
        unk1: u32,
        unk2: u32,
        #[brw(pad_after = 4)] // Seems to be empty/always zeroes
        unk3: u32,
    },
    StrategyBoard {
        /// The content id of the sending player.
        content_id: u64,
        /// The strategy board data.
        board_data: StrategyBoard,
    },
    StrategyBoardUpdate(StrategyBoardUpdate),
    EndStrategyBoardSession {
        unk: [u8; 16], // Always zeroes?
    },
    WaymarkUpdate {
        /// The id number of this waymark. 0 = A, 1 = B, and so on.
        id: u8,
        #[brw(pad_after = 2)] // Empty/always zeroes?
        /// The placement mode of this waymark.
        placement_mode: WaymarkPlacementMode,
        /// The waymark's position in the world.
        pos: WaymarkPosition,
    },
    FreeCompanyHierarchy {
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        leader_name: String,

        #[br(count = 16)]
        #[brw(pad_size_to = 16 * FcHierarchy::SIZE)]
        hierarchy_list: Vec<FcHierarchy>,
    },
    FreeCompanyShortMessage {
        /// The content id of the requested character.
        content_id: u64,
        /// A value the client sends, repeated back to the client.
        sequence: u32,
        /// A 32-bit Unix timestamp indicating when the message was last updated.
        time_last_updated: u32,
        #[brw(pad_size_to = 96)]
        #[br(count = 96)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        #[brw(pad_after = 8)] // Empty/zeroes
        /// The requested character's FC short message.
        short_message: String,
    },
    FreeCompanyHeader {
        /// The company's ID number. It can also be found in SocialList responses.
        company_id: u64,
        /// The company's crest ID number. Presumably used in places where the company logo is shown.
        crest_id: u64,
        /// Unknown purpose. Possibly for rankings on the Lodestone?
        company_points: u64,
        /// How many company credits the company has to spend on purchases (actions, misc. items, etc.).
        company_credits: u64,
        /// The company's standing with the Grand Company they're allied to.
        reputation: u32,
        /// The amount of points required to rank up.
        next_point: u32,
        /// How many points the company has towards their next rank up.
        current_point: u32,
        /// How many members the company has in total.
        total_members: u16,
        /// How many members in the company are currently online.
        online_members: u16,
        /// The Grand Company this fc is aligned with.
        gc_id: GrandCompany,
        /// The company's current rank (out of 30).
        fc_rank: u8,
        #[brw(pad_size_to = 22)]
        #[br(count = 22)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        /// The company's full name.
        company_name: String,
        #[brw(pad_size_to = 6)]
        #[br(count = 6)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        #[brw(pad_after = 2)] // Empty/zeroes
        /// The company's short tag.
        company_tag: String,
    },
    FreeCompanyActivityList {
        unk: [u8; 528],
    },
    UnkContentFinder2 {
        /// Index into the ContentFinderCondition Excel sheet.
        content_finder_condition_id: u32,
        unk: [u8; 12],
    },
    Playtime {
        #[brw(pad_after = 4)] // Empty/zeroes
        /// The character's total cumulative playtime, measured in minutes.
        duration: u32,
    },
    Countdown {
        /// The account id of the character that started the countdown.
        account_id: u64,
        /// The content id of the character that started the countdown.
        content_id: u64,
        /// The actor id of the character that started the countdown.
        starter_actor_id: ObjectId,
        unk: u16, // Could be a u8 with padding? Seems to always be 0x5B.
        /// The duration of the countdown in seconds.
        #[brw(pad_after = 3)]
        duration: u16,
        /// The name of the character that started the countdown.
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        #[brw(pad_after = 5)]
        starter_name: String,
    },
    DirectorPopupMessage {
        /// The actor id of who's saying the text. Unused, as the client does not use it past reading it.
        actor_id: ObjectTypeId,
        /// Should be the ID of the instance's director.
        handler_id: HandlerId,
        /// Index into the BNPCName Excel sheet.
        npc_name: u32,
        /// Index into the InstanceContentTextData Excel sheet.
        /// This value has to be in this InstanceContent's range of InstanceContentTextDataObjectiveStart and InstanceContentTextDataObjectiveEnd. If not, the client will crash.
        text_data_id: u32,
        duration_msecs: u32,
        /// The icon/image that shows on the top left
        icon: u32,
        /// What kind of object the speaker (npc_name) is, as `ObjectKind`'s first byte (2 for a BattleNpc, as retail
        /// sends it in quest battles.) Used when resolving npc_name. A single byte: `ObjectKind::BattleNpc` would
        /// also write its sub kind and make the packet a byte too long.
        kind: u8,
        /// The appearance of the text box.
        style: u8,
        /// Amount of parameters to be read. Max value is 2.
        #[brw(pad_after = 1)] // padding
        params_count: u8,
        param1: u32,
        param2: u32,
    },
    DirectorSetupMapEffects64(#[brw(args { max_params: 64 } )] MapEffects),
    DirectorSetupMapEffects128(#[brw(args { max_params: 128 } )] MapEffects),
    DirectorMapEffect {
        /// Should be the ID of the instance's director.
        handler_id: HandlerId,
        /// The new state of this map effect. Also used as a fallback if `timeline_id` doesn't work.
        /// Note that this seems to be an arbitrary value, it needs to be different than the current state - otherwise the current animation restarts?
        state: u16,
        /// Which timeline_id to start playing. This is an index into `timeline_indices` of the SGB.
        timeline_id: u16,
        /// The index of the map effect item to change.
        #[brw(pad_after = 7)] // padding, not read by the client
        index: u8,
    },
    FurnitureList(FurnitureList),
    OwnedHousing {
        #[brw(pad_after = 8)] // believe these are always empty?
        unk1: LandData,
        #[brw(pad_after = 8)]
        unk2: LandData,
        #[brw(pad_after = 8)]
        unk3: LandData,
        unk4: LandData,
        #[brw(pad_after = 8)]
        unk5: LandData,
        /// Your apartment unit.
        #[brw(pad_after = 8)]
        apartment: LandData,
    },
    UpdateFittingShop {
        /// Corresponds to the DisplayId column in the FittingShopCategoryItem Excel sheet.
        #[brw(pad_after = 8)] // empty
        display_ids: [u8; 8],
    },
    UnkSocialResponse {
        #[br(count = 80)]
        #[brw(pad_size_to = 80)]
        unk: Vec<u8>,
    },
    UnkClassRelated {
        #[brw(pad_after = 3)]
        classjob_id: u8,
        class_level: u16,
        current_level: u16,
    },
    EnmityList(EnmityList),
    HaterList(HaterList),
    DuelInformation {
        account_id: u64,
        /// The opponent's content ID.
        opponent_content_id: u64,
        /// The opponent's object ID.
        opponent_object_id: ObjectId,
        world_id: u16,
        unk1: u16,
        unk2: u8,
        /// The name of the opponent.
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        #[brw(pad_after = 7)] // empty
        opponent_name: String,
    },
    /// A page of a marketboard search (the reply to `RequestMarketBoardItems`.)
    MarketBoardItems {
        /// The items on this page, up to `MARKETBOARD_ITEMS_PER_PAGE`.
        #[br(count = MARKETBOARD_ITEMS_PER_PAGE)]
        #[brw(pad_size_to = MARKETBOARD_ITEMS_PER_PAGE * MarketBoardItem::SIZE)]
        items: Vec<MarketBoardItem>,
        /// Where the next page starts, 0 if this is the last.
        next_index: u32,
        /// 0, or 0x70000001 with no items (retail answered a search like that twice in a row, then normally.)
        error: u32,
        /// The request's, sent back.
        start_index: u32,
        request_id: u8,
        #[brw(pad_after = 2)] // empty
        kind: u8,
    },
    EffectResultBasic {
        /// How many actors are covered by this packet.
        count: u32,
        /// See ActionResult for more details.
        global_sequence: u32,
        target_id: ObjectId,
        current_hp: u32,
        unk3: u32,
        unk4: u32,
    },
    AoeEffect8(#[brw(args { max_targets: 8 } )] AoeEffect),
    ActorCast {
        /// Usually the same as `action_id`.
        spell_id: u16,
        /// What kind of action is being cast.
        action_type: ActionType,
        /// Omen Delay is for the extra effects that appear when casting, usually telegraphs, like Titan's Landslide line. If you increase that value, the line that usually immediately shows in front of him is delayed.
        omen_delay: u8,
        /// Index into the Action Excel sheet.
        action_id: u32,
        /// Cast time in seconds.
        cast_time: f32,
        /// The target of this cast.
        target: ObjectId,
        /// The cast's rotation.
        #[br(map = read_quantized_rotation)]
        #[bw(map = write_quantized_rotation)]
        rotation: f32,
        /// If true, shows special VFX around the cast bar to show that's interruptible (it currently pulsates.) This is only applicable for other targets.
        #[br(map = read_bool_from::<u16>)]
        #[bw(map = write_bool_as::<u16>)]
        interruptible: bool,
        /// Only used when ActionCategory is 11.
        ballista_entity_id: ObjectId,
        /// Position of the caster.
        #[brw(pad_after = 2)] // empty
        #[br(map = read_packed_position)]
        #[bw(map = write_packed_position)]
        position: Position,
    },
    SearchPlayersResult {
        /// The number of results found after a player search.
        #[brw(pad_after = 4)] // empty
        num_results: u32, // TODO: this might be only an u16 or an u8, since the search results window only shows up to 200 players.
    },
    FriendGroupIcon(FriendGroupIconInfo),
    DeepDungeonParty {
        /// Refers to the player actors in your party, including yourself.
        entity_ids: [ObjectId; 4],
        room_indices: [u8; 4],
    },
    DeepDungeonChests {
        types: [u8; 16],
        room_indices: [u8; 16],
    },
    DeepDungeonSetup {
        bonus_loot_item_id: u32,
        unk1: u8,
        unk2: u8,
        weapon_level: u8,
        armor_level: u8,
        return_progress: u8,
        passage_progress: u8,
        synced_gear_level: u8,
        hoard_count: u8,
        unk3: u8,
        unk4: u8,
        gimmick_effect_id_current: u8,
        gimmick_effect_id_next: u8,
        unk5: [u8; 8],
    },
    DeepDungeonMap {
        layout_initialization_type: u8,
        deep_dungeon_status_id: u8,
        deep_dungeon_ban_id: u8,
        deep_dungeon_danger_id: u8,
        unk1: u8,
        unk2: u8,
        map_data: [DeepDungeonRoomFlag; 25],
    },
    ExamineCharacterInformation {
        unk1: [u8; 640],
        /// The requested player's name.
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        name: String,
        unk2: [u8; 272],
    },
    OtherSearchInfo {
        /// The requested player's content ID.
        content_id: u64,
        unk1: [u8; 26], // seems empty but not 100%
        /// The requested player's home world. Index into the World Excel sheet.
        world_id: u16,
        /// The requested player's search comment.
        #[brw(pad_size_to = 60)]
        #[br(count = 60)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        comment: String,
        unk2: [u8; 157], // also seems empty
        /// The requested player's Grand Company rank.
        grand_company_rank: u8,
        unk3: [u8; 2], // probably also empty
        /// The requested player's class levels.
        classjob_levels: [(u16, u16); AVAILABLE_CLASSJOBS],
    },
    SetPlayerCustomizeData(CustomizeData),
    CrossworldLinkshellsEx {
        #[brw(pad_before = 8)] // Seems to be empty/zeroes
        #[br(count = CrossworldLinkshellEx::COUNT)]
        #[brw(pad_size_to = CrossworldLinkshellEx::COUNT * CrossworldLinkshellEx::SIZE)]
        linkshells: Vec<CrossworldLinkshellEx>,
    },
    CrossworldLinkshellMemberList {
        linkshell_id: u64,
        #[brw(pad_after = 2)] // Seems to be empty/zeroes
        sequence: u16,
        next_index: u16,
        current_index: u16,
        #[br(count = CWLSMemberListEntry::COUNT)]
        #[brw(pad_size_to = CWLSMemberListEntry::COUNT * CWLSMemberListEntry::SIZE)]
        members: Vec<CWLSMemberListEntry>,
    },
    SpawnTreasure(SpawnTreasure),
    OpenedTreasure {
        unk1: u32,
        unk2: u32,
        unk3: u32,
        unk4: u32,
        entity_id: ObjectId,
        unk6: u32,
    },
    TreasureFadeOut {
        unk1: u32,
        unk2: u32,
    },
    /// An opened coffer's loot, to the party (Bob's Chronofoil capture of Sastasha, 2026-10-22): how many items, the
    /// seconds left to roll (300 on opening, three times) and the items.
    LootList {
        treasure_id: ObjectId,
        count: u32,
        time_left: [f32; 3],
        items: [u32; 16],
        unk: u32,
    },
    /// One item of an opened coffer (the same capture): sent after `LootList`, and again when the player opens the
    /// coffer themselves (`opened` 1, the time left then.) `serial` counts the duty's loot from the first (retail's
    /// started at 0x3e in one run); `bonus` was set only on Aetherial Brass Wristlets (random stats, perhaps.)
    LootItem {
        treasure_id: ObjectId,
        item_id: u32,
        unk1: u32,
        time_left: f32,
        time_total: [f32; 2],
        /// 1 in every one seen.
        unk2: u8,
        unk3: u8,
        slot: u8,
        unique: u8,
        unk4: u8,
        /// 1, 1 in every one seen.
        unk5: [u8; 2],
        opened: u8,
        unk6: u32,
        serial: u32,
        unk7: u32,
        bonus: [u16; 4],
        unk8: u32,
    },
    /// The answer to the player's `LootRoll` (the same capture): `result` 1 with the `roll` (need or greed), 4 passed,
    /// 8 refused (a unique item they had already.)
    LootRollResult {
        treasure_id: ObjectId,
        slot: u32,
        result: u8,
        roll: u8,
        #[brw(pad_after = 4)]
        unk: u16,
    },
    /// Who won an item of a coffer (the same capture): the winner and their roll, `ObjectId(0xE0000000)` and 0 when
    /// everyone passed. Retail's three bytes after the slot held part of the winner's ID; zeros here.
    LootResult {
        treasure_id: ObjectId,
        slot: u8,
        unk: [u8; 3],
        winner: ObjectId,
        roll: u32,
    },
    /// Who first attacked this actor (the packet's source): its "claim". Retail sends it to everyone nearby the
    /// moment an actor first takes a hostile action, just before that action's EffectResult (an enemy hit by a player,
    /// a player hit by an enemy), and sends it again cleared when the fight is over for it. The same ID is in
    /// CommonSpawn's `combat_tagger_id` for those who see the actor spawn later. Bob's Chronofoil captures, 2026-10-14
    /// to 17 (about 1,100 of them in the Central Shroud and Thanalan).
    FirstAttack {
        /// 1: tagged by `combat_tagger`; 0: cleared (the tagger 0); 2: seen between NPCs in duties (unknown).
        kind: u32,
        unk2: u32,
        combat_tagger: ObjectId,
        unk3: u32,
    },
    UnkFate {
        /// Index into the FATE Excel sheet.
        fate_id: u32,
        unk1: u32,
        /// When the FATE was spawned. The game uses this to determine how much time is left in the FATE.
        start_timestamp: u32,
        unk3: u32,
        /// In seconds.
        time_limit: u32,
        unk5: u32,
    },
    CrossRealmListings(CrossRealmListings),
    CrossRealmListingsOverview {
        unk: [u8; 48],
    },
    CrossRealmListingInformation {
        /// The unique ID for this listing.
        listing_id: u64,
        unk: [u8; 456],
    },
    CWLinkshellNameAvailability {
        unk1: u8, // TODO: What is this? Seems to be always 1.
        /// If the desired name was available or not.
        result: CWLSNameAvailability,
        /// The desired name.
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        #[brw(pad_after = 6)]
        name: String,
    },
    NewCrossworldLinkshell {
        /// The CWLS's id number and ChatChannel information.
        ids: CWLSCommonIdentifiers,
        unk_timestamp1: u32, // Unknown 32-bit Unix timestamp, likely the cwls's creation time.
        unk_timestamp2: u32, // Seems to be the same timestamp repeated? Might be the member's join time?
        /// The member's rank in the cross-world linkshell, and the linkshell's name.
        common: CWLSCommon,
    },
    RetainerInfo {
        /// The inventory sequence (every RetainerInfo in a list has the same one, then RetainerInfoEnd.)
        sequence: u32,
        unk2: u32,
        /// Unique ID for this retainer. Zero for an empty slot of the list.
        retainer_id: u64,
        /// Which of the character's ten slots this is.
        index: u8,
        #[brw(pad_after = 2)] // appears empty
        /// How many of their inventory slots are filled.
        item_count: u8,
        /// The amount of gil in their possession.
        gil: u32,
        /// How many items they have on the market.
        market_item_count: u8,
        /// Their town (FFXIVClientStructs' RetainerManager.RetainerTown: 1 Limsa Lominsa, 2 Gridania, 3 Ul'dah...)
        town: u8,
        classjob_id: u8,
        level: u8,
        /// When their market listings end (Unix time), 0 with nothing on the market.
        market_expire: u32,
        /// Index into the RetainerTask Excel sheet, their venture.
        venture_id: u32,
        /// When their venture is done (Unix time.)
        venture_complete: u32,
        /// If set to zero, it shows "contract suspended". 1 on retail.
        available: u32,
        unk11: u8,
        /// The name of this retainer.
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        #[brw(pad_after = 3)]
        name: String,
    },
    RetainerInfoEnd {
        /// The same sequence as the RetainerInfos before it.
        sequence: u32,
        /// How many RetainerInfos came before it: 10 for the whole list, 1 for one retainer.
        entry_count: u8,
        /// How many retainers the character may hire (2 on retail without extra retainers.)
        max_retainers: u8,
        /// 1 on retail.
        #[brw(pad_after = 1)]
        unk1: u8,
        unk2: u32,
        /// 0xFFFFFFFF on retail.
        unk3: u32,
        /// 0xFFFFFFFF on retail.
        unk4: u32,
        /// 0xFFFF on retail.
        unk5: u32,
    },
    CrossworldLinkshellDisbanded {
        // The linkshell's id.
        linkshell_id: u64,
        /// The linkshell's name.
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        name: String,
    },
    CrossworldLinkshellMemberLeft {
        /// The linkshell this player is leaving.
        linkshell_id: u64,
        /// The initiator's content id.
        execute_content_id: u64,
        /// The target's content id.
        target_content_id: u64,
        /// Their home world id.
        target_homeworld_id: u16,
        unk1: u8, // Always 1? Changing it does seemingly nothing.
        /// The target's reason for leaving.
        reason_for_leaving: CWLSLeaveReason,
        /// Their name.
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        #[brw(pad_after = 4)] // Seems to be empty/zeroes
        character_name: String,
    },
    CrossworldLinkshellRenamed {
        /// The linkshell this player is renaming.
        linkshell_id: u64,
        /// The content id of the character renaming this LS.
        content_id: u64,
        /// Their home world id.
        home_world_id: u16,
        unk1: u8, // Always 1?
        unk2: u8, // TODO: This might just be padding, or part of a u16?
        /// Their name.
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        character_name: String,
        /// The linkshell's new name.
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        #[brw(pad_after = 4)] // Seems to be empty/zeroes
        new_linkshell_name: String,
    },
    CrossworldLinkshellMemberRank {
        /// The linkshell this action is taking place on.
        linkshell_id: u64,
        /// The initiator's content id.
        execute_content_id: u64,
        /// The target's content id.
        target_content_id: u64,
        /// The target's home world id.
        home_world_id: u16,
        unk1: u8, // Always 1?
        /// The rank assigned to the target.
        #[brw(pad_after = 1)] // Seems to be empty/zeroes
        permission_rank: CWLSPermissionRank,
        /// The target's name.
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        #[brw(pad_after = 3)] // Seems to be empty/zeroes
        target_name: String,
    },
    CrossworldLinkshellInvite(CrossworldLinkshellInvite),
    /// This one is sent to the joining player.
    CrossworldLinkshellJoinedSelf {
        common_ids: CWLSCommonIdentifiers,
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        linkshell_name: String,
    },
    /// This one is sent to everyone else, not to the joining player.
    CrossworldLinkshellJoined2 {
        /// The linkshell being joined.
        linkshell_id: u64,
        /// The joining player's content id.
        content_id: u64,
        /// The joining player's home world id.
        home_world_id: u16,
        unk1: u8, // Yet another always 1?
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        #[brw(pad_after = 5)]
        target_name: String,
    },
    MailboxStatus {
        /// The amount of letters still pending when the player's mailbox is full. Also affects the Delivery Moogle NPC's dialogue (caps at i32::MAX, making this an i32).
        letters_sent_back: i32,
        /// The amount of items sent by friends that have yet to be taken from letters.
        attachments_counter: u16,
        /// The total amount of new mail, displayed as a small white envelope in the server info bar (caps at 99 in the bar). Also mentioned by the Delivery Moogle when they inform the player about how many letters they have.
        unread_counter: u8,
        /// The amount of mail from friends the player has in their mailbox.
        friend_counter: u8,
        /// The amount of reward mail the player has in their mailbox. Reward mail is mail sent by the system that has cash shop items, etc., attached.
        reward_counter: u8,
        /// The amount of system mail from GMs the player in their mailbox.
        system_counter: u8,
        /// If set, the player's info bar will display that they have new mail from a GM/the system.
        #[br(map = read_bool_from::<u8>)]
        #[bw(map = write_bool_as::<u8>)]
        has_gm_mail: bool,
        /// If set, the player's info bar will display that they have new mail from the support desk. This has higher priority than has_gm_mail.
        #[br(map = read_bool_from::<u8>)]
        #[bw(map = write_bool_as::<u8>)]
        #[brw(pad_after = 4)] // Seemingly empty/zeroes, setting it does nothing noticeable
        has_support_message: bool,
    },
    MailboxPreview {
        /// The letters sent on this iteration. This is part of a series of exchanges like all the other lists in FF14.
        #[brw(pad_size_to = LetterPreview::SIZE * LetterPreview::COUNT)]
        #[br(count = LetterPreview::COUNT)]
        letters: Vec<LetterPreview>,
        /// The next batch of 5 letters' index, if applicable. If there are no more letters, it's set to zero.
        next_index: u8,
        /// The current batch of 5 letters' index.
        current_index: u8,
        #[brw(pad_after = 5)] // Probably just padding, it was observed as all 0s.
        unk: u8,
    },
    Letter(Letter),
    LetterUpdate {
        /// Seems to be a result or mode value. When a letter is sent successfully, this will be 0xDD. When a letter is deleted successfully, it will contain 0x366. When attachments are taken, it will contain 0x24E. Completely unknown purpose, as it doesn't seem to be a timestamp, actor id, or LogMessageType.
        unk_result: u32,
        unk1: u32, // Probably just padding, seems to always be 0.
        /// When deleting a letter, the sender's content id. When sending a letter, zeroes.
        sender_content_id: u64,
        /// When deleting a letter, the letter's timestamp. When sending a letter, zeroes.
        timestamp: u32,
        updated_items: [AttachedItemInfo; MAX_MAIL_ATTACHMENTS_STORAGE],
        unk2: [u8; 4], // Unknown, seems to be nothing but padding.
    },
    ShowLinkshellError {
        /// The LogMessage sheet row index to display to the client.
        #[brw(pad_after = 2)] // Seems to be empty/zeroes
        log_message: u16,
        /// Unknown. Has data and doesn't appear to be a timestamp, actor id, or content id.
        #[brw(pad_after = 16)] // Seems to be empty/zeroes
        unk: u32,
    },
    HousingInteriorDetails(HousingInteriorDetails),
    ApartmentList(ApartmentList),
    FriendRemoved {
        #[brw(pad_after = 4)] // Seems to be empty/zeroes
        content_id: u64,
        unk1: u8, // Always 1?
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        #[brw(pad_after = 3)] // Seems to be empty/zeroes
        name: String,
    },
    NpcYell {
        /// Unknown purpose.
        object_id: ObjectTypeId,
        /// Index into ENpcResident Excel sheet.
        name_id: u32,
        /// Index into the NpcYell Excel sheet.
        npc_yell_id: u32,
        /// First generic parameter.
        param1: u32,
        /// Second generic parameter.
        param2: u32,
        /// Third generic parameter.
        param3: u32,
        /// Fourth generic parameter.
        param4: u32,
    },
    InteriorFurniturePlaced {
        /// Which storage the furniture was placed into.
        storage_id: ContainerType,
        /// Which slot the furniture was placed into.
        slot: u16,
        /// The low 12 bits of the row number on the HousingFurniture sheet for this furniture. The row to that sheet can be obtained from the AdditionalData column on the Item Excel sheet. When the client receives this value, it then ORs it with 0x30000 to recreate the row number.
        item_id: u16,
        unk1: u16, // Always 1? Changing it seems to have no visible effect so far.
        /// The furniture's dye/stain.
        stain: u8,
        unk2: [u8; 3],
        /// The furniture's rotation. This is only used when placing furniture from the storeroom to ensure the furniture's front faces the player.
        rotation: f32,
        /// The furniture's position.
        position: Position,
        unk3: [u8; 4],
    },
    ExteriorFurniturePlaced {
        /// Likely the plot upon which this furniture was placed.
        plot_index: u8,
        /// The item slot the furniture was placed into.
        slot: u8,
        unk1: [u8; 2], // Likely just padding
        /// The low 12 bits of the row number on the HousingYardObject sheet for this furniture. The row to that sheet can be obtained from the AdditionalData column on the Item Excel sheet. When the client receives this value, it then ORs it with 0x20000 to recreate the row number.
        item_id: u16,
        unk2: u16, // Observed as zeroes
        /// The furniture's dye/stain.
        stain: u8,
        unk3: [u8; 3], // Likely just padding
        /// The furniture's rotation. This is only used when placing furniture from the storeroom to ensure the furniture's front faces the player.
        rotation: f32,
        /// The furniture's position.
        position: Position,
        unk4: u32, // Observed as zeroes
    },
    Mogpendium(Mogpendium),
    PlayerName {
        /// Content ID of the player in question.
        content_id: u64,
        /// Name of the player requested.
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        name: String,
    },
    EorzeanTimeOffset {
        offset: i64, // TODO: Not 100% sure this is an i64, but setting it to negative values does make the time to go back to an extent.
    },
    FurnitureTranslatedForObserver(FurnitureTranslatedForObserver),
    SharedFATEInformation {
        /// The page index, starting from zero.
        page: u8,
        /// How many FATEs were done for each zone.
        fate_count: [u8; 6],
        /// How many FATEs are needed to max rank in this zone.
        #[brw(pad_after = 3)] // not used
        fate_count_maximum: [u8; 6],
    },
    UnkDirector2 {
        unk: [u8; 24],
    },
    StatusEffectListDouble {
        unk: [u8; 720],
    },
    StatusEffectListPlayerDouble {
        unk: [u8; 720],
    },
    UnkJobGauge {
        unk: [u8; 8],
    },
    UpdateRecastTimes {
        unk: [u8; 640],
    },
    /// Another character's (or your own) adventurer plate, answering RequestAdventurerPlate. CharaCardPacket in
    /// FFXIVClientStructs; checked against a retail capture of Bob's plate being edited (2026-10-20).
    AdventurerPlate {
        /// The free company crest (CrestData); 0 without one.
        crest: u64,
        /// The character's account (as in their spawn); retail sends it.
        account_id: u64,
        content_id: u64,
        actor_id: ObjectId,
        /// 2 on retail.
        unk5: u32,
        world_id: u16,
        /// The level of the class/job they're on.
        favored_class_level: u16,
        /// The class/job they're on.
        favored_class: u8,
        /// The character's sex (0 male, 1 female), as in their customize data.
        sex: u8,
        grand_company: GrandCompany,
        grand_company_rank: u8,
        /// What the plate shows (CharaCardData), as the client last saved it.
        chara_card: CharaCardData,
        /// When the plate was last saved (Unix seconds), 0 if it never was. The client sends it back with its next
        /// UpdateAdventurerPlate.
        timestamp: u32,
        /// The search comment (SearchComment, 193 bytes).
        #[brw(pad_size_to = 193)]
        #[br(count = 193)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        comment: String,
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        name: String,
        /// The free company's name (22 bytes); empty without one.
        #[brw(pad_after = 1)] // empty
        #[brw(pad_size_to = 22)]
        #[br(count = 22)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        free_company: String,
    },
    RecordGatheringLog {
        /// Index into the bitmask.
        index: u8,
        /// New value at this index.
        #[brw(pad_after = 6)] // unused
        value: u8,
    },
    RecruitingPartyCount {
        unk1: [u8; 968], // not exactly sure what this is yet...
    },
    OccultCrescentSetup {
        unk1: [u8; 160],
    },
    SpectatorList {
        object_ids: [ObjectId; 8],
        #[brw(pad_after = 3)] // padding
        unk1: u8,
    },
    MarketBoardHistory(MarketBoardHistory),
    /// A page of an item's listings (the reply to `UnkMarketBoardRequest2`.)
    MarketBoardOfferings {
        /// Up to `MARKETBOARD_LISTINGS_PER_PAGE`.
        #[br(count = MARKETBOARD_LISTINGS_PER_PAGE)]
        #[brw(pad_size_to = MARKETBOARD_LISTINGS_PER_PAGE * MarketBoardListing::SIZE)]
        listings: Vec<MarketBoardListing>,
        /// Where the next page starts, 0 if this is the last.
        next_index: u8,
        /// The request's, sent back.
        start_index: u8,
        #[brw(pad_after = 5)] // not used, padding
        request_id: u8,
    },
    /// Sent when an item is opened (`UnkMarketBoardRequest1`), before its history.
    MarketBoardHits {
        unk1: u32,
        /// How many listings the item has (retail shows at most 100.)
        hits: u32,
    },
    ZoneDiceRollResult(ZoneDiceRollResult),
    SearchFellowshipsNoResults {
        #[brw(pad_before = 4, pad_after = 16)]
        err_code: u32, // Name is assumed, observed as 0x01240C89. Changing this to an incorrect value seems to make the client wait for more results (the UI window acts as if it's waiting with a spinner)?
    },
    SearchFellowshipsResults {
        #[brw(pad_before = 4)] // Seems to always be zeroes/padding
        unk1: u32, // Seems to always be 0xFFFF_FFFF
        sequence_end: u32, // Is typically 0xFFFF_FFFF, but seems to change to 0x28 as the final value when a number of FellowshipFinderResults have been sent. If only one is sent, it stays at 0xFFFF_FFFF and sequence_current will be 0. More research is needed for this.
        /// The current sequence value, so the client knows which packet this is so far. If it's set to 0, the client won't expect more to be coming (the appropriate page arrows in the Finder window will be disabled).
        sequence_current: u32,
        /// The actual Fellowship result items.
        #[br(count = 4)]
        #[brw(pad_size_to = 4 * 376)]
        fellowships: Vec<FellowshipSearchInfo>,
    },
}

#[cfg(test)]
mod tests {
    use crate::common::test_opcodes;

    use super::*;

    // Ensure that the IPC data size as reported matches up with what we write
    #[test]
    fn server_zone_ipc_sizes() {
        test_opcodes::<ServerZoneIpcSegment>();
    }

    /// Where the client's handler (HandleAchievementsPacket) reads each part.
    #[test]
    fn achievements_layout() {
        use binrw::BinWrite;

        let mut completed = vec![0; ACHIEVEMENT_BITMASK_SIZE];
        completed[0] = 0b10;
        let data = ServerZoneIpcData::Achievements {
            completed,
            history: [1, 2, 3, 4, 5],
            unk: [0; 26],
        };
        let mut cursor = std::io::Cursor::new(Vec::new());
        data.write_le(&mut cursor).unwrap();
        let bytes = cursor.into_inner();
        assert_eq!(bytes.len(), 552);
        assert_eq!(bytes[0], 0b10);
        assert_eq!(&bytes[0x1FE..0x202], &[1, 0, 2, 0]);
    }

    /// Where each part of AdventurerPlate lands, as retail sent Bob's plate (Chronofoil capture of 2026-10-20; its
    /// layout and numbers, with made-up IDs and text): account at 8, content ID at 16, sex at 0x25, the search comment at 0xE8, the
    /// name at 0x1A9 and the free company at 0x1C9, 480 bytes in all.
    #[test]
    fn adventurer_plate_layout() {
        use binrw::BinWrite;

        let data = ServerZoneIpcData::AdventurerPlate {
            crest: 0,
            account_id: 0x0102_0304_0506_0708,
            content_id: 0x0040_0011_2233_4455,
            actor_id: ObjectId(0x1000_0001),
            unk5: 2,
            world_id: 57,
            favored_class_level: 88,
            favored_class: 24,
            sex: 1,
            grand_company: GrandCompany::Flames,
            grand_company_rank: 10,
            chara_card: CharaCardData::default(),
            timestamp: 1_792_546_070,
            comment: "c".to_string(),
            name: "n".to_string(),
            free_company: "f".to_string(),
        };
        let mut cursor = std::io::Cursor::new(Vec::new());
        data.write_le(&mut cursor).unwrap();
        let bytes = cursor.into_inner();
        assert_eq!(bytes.len(), 480);
        assert_eq!(bytes[8], 0x08);
        assert_eq!(
            &bytes[0x10..0x18],
            &[0x55, 0x44, 0x33, 0x22, 0x11, 0x00, 0x40, 0x00]
        );
        assert_eq!(
            &bytes[0x1C..0x28],
            &[2, 0, 0, 0, 57, 0, 88, 0, 24, 1, 3, 10]
        );
        assert_eq!(&bytes[0xE4..0xE8], &1_792_546_070u32.to_le_bytes());
        assert_eq!(bytes[0xE8], b'c');
        assert_eq!(bytes[0x1A9], b'n');
        assert_eq!(bytes[0x1C9], b'f');
    }

    /// FirstAttack as retail sent it (Bob's Chronofoil capture of 2026-10-17): an enemy claimed by player 0x10065441.
    #[test]
    fn first_attack_layout() {
        use binrw::BinWrite;

        let data = ServerZoneIpcData::FirstAttack {
            kind: 1,
            unk2: 0,
            combat_tagger: ObjectId(0x10065441),
            unk3: 0,
        };
        let mut cursor = std::io::Cursor::new(Vec::new());
        data.write_le(&mut cursor).unwrap();
        assert_eq!(
            cursor.into_inner(),
            [1, 0, 0, 0, 0, 0, 0, 0, 0x41, 0x54, 0x06, 0x10, 0, 0, 0, 0]
        );
    }

    /// A coffer's loot packets as retail sent them (Bob's Chronofoil capture of Sastasha, 2026-10-22; the coffer's
    /// and the winner's IDs made up): its list of two items, the first item, a Greed roll of 33 answered, and the
    /// winner with 44.
    #[test]
    fn loot_layouts() {
        use binrw::BinWrite;

        fn write(data: ServerZoneIpcData) -> Vec<u8> {
            let mut cursor = std::io::Cursor::new(Vec::new());
            data.write_le(&mut cursor).unwrap();
            cursor.into_inner()
        }
        let coffer = ObjectId(0x4000_1234);
        let mut items = [0; 16];
        items[0] = 2684;
        items[1] = 4096;
        let list = write(ServerZoneIpcData::LootList {
            treasure_id: coffer,
            count: 2,
            time_left: [300.0; 3],
            items,
            unk: 0,
        });
        assert_eq!(list.len(), 88);
        assert_eq!(
            &list[..28],
            &hex("34120040 02000000 00009643 00009643 00009643 7c0a0000 00100000")[..]
        );
        let item = write(ServerZoneIpcData::LootItem {
            treasure_id: coffer,
            item_id: 2684,
            unk1: 0,
            time_left: 300.0,
            time_total: [300.0; 2],
            unk2: 1,
            unk3: 0,
            slot: 0,
            unique: 1,
            unk4: 0,
            unk5: [1, 1],
            opened: 0,
            unk6: 0,
            serial: 0x3e,
            unk7: 0,
            bonus: [0; 4],
            unk8: 0,
        });
        assert_eq!(
            item,
            hex("34120040 7c0a0000 00000000 00009643 00009643 00009643 01000001 00010100 00000000 3e000000
                 00000000 00000000 00000000 00000000")
        );
        let answer = write(ServerZoneIpcData::LootRollResult {
            treasure_id: coffer,
            slot: 0,
            result: 1,
            roll: 0x21,
            unk: 0,
        });
        assert_eq!(answer, hex("34120040 00000000 01210000 00000000"));
        let result = write(ServerZoneIpcData::LootResult {
            treasure_id: coffer,
            slot: 0,
            unk: [0; 3],
            winner: ObjectId(0x1000_0001),
            roll: 44,
        });
        assert_eq!(result, hex("34120040 00000000 01000010 2c000000"));
    }

    fn hex(bytes: &str) -> Vec<u8> {
        let bytes: String = bytes.split_whitespace().collect();
        (0..bytes.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&bytes[i..i + 2], 16).unwrap())
            .collect()
    }

    /// A retainer's entry in the bell's list and the end of the list, as retail sent them (Bob's Chronofoil capture of
    /// 2026-09-15; the retainer's name replaced): miner level 49 from Limsa Lominsa, 130 items, 242,668 gil, 12 items
    /// on the market.
    #[test]
    fn retainer_info_layout() {
        use binrw::BinWrite;

        let data = ServerZoneIpcData::RetainerInfo {
            sequence: 0x118,
            unk2: 0,
            retainer_id: 0x0078_0017_4aa7_db83,
            index: 0,
            item_count: 0x82,
            gil: 0x0003_b3ec,
            market_item_count: 12,
            town: 1,
            classjob_id: 16,
            level: 49,
            market_expire: 0x6abf_d9bc,
            venture_id: 0,
            venture_complete: 0,
            available: 1,
            unk11: 0,
            name: "Abc".to_string(),
        };
        let mut cursor = std::io::Cursor::new(Vec::new());
        data.write_le(&mut cursor).unwrap();
        let mut expected = hex(
            "18010000 00000000 83dba74a 17007800 00820000 ecb30300 0c011031 bcd9bf6a 00000000 00000000 01000000 00",
        );
        expected.extend_from_slice(b"Abc");
        expected.resize(80, 0);
        assert_eq!(cursor.into_inner(), expected);

        let data = ServerZoneIpcData::RetainerInfoEnd {
            sequence: 0x118,
            entry_count: 10,
            max_retainers: 2,
            unk1: 1,
            unk2: 0,
            unk3: u32::MAX,
            unk4: u32::MAX,
            unk5: 0xffff,
        };
        let mut cursor = std::io::Cursor::new(Vec::new());
        data.write_le(&mut cursor).unwrap();
        assert_eq!(
            cursor.into_inner(),
            hex("18010000 0a020100 00000000 ffffffff ffffffff ffff0000")
        );
    }
}

/// One action's line in `CraftingActionPreviews`.
#[binrw]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CraftingActionPreview {
    /// The class's CraftAction row.
    pub action_id: u32,
    /// Its progress efficiency now (potency times the buffs, %), 0x8000 set when that isn't its usual potency.
    pub progress_efficiency: u16,
    /// The same for quality.
    pub quality_efficiency: u16,
    /// The progress and quality it would add now.
    pub progress: u32,
    pub quality: u32,
    /// 0 usable, 1 not learnt yet, 2 not enough CP, 3 not usable now (first step only, needs Inner Quiet, a Good condition…).
    pub state: u32,
}
