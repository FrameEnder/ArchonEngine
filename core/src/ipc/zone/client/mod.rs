use binrw::binrw;
use kawari_core_macro::opcode_data;
use physis::savedata::chardat::CustomizeData;
use physis::sestring::SeString;

mod action_request;
pub use crate::ipc::zone::client::action_request::{ActionRequest, ActionType};

mod send_chat_message;
pub use send_chat_message::SendChatMessage;

mod client_trigger;
pub use crate::ipc::zone::client::client_trigger::{
    ClientTrigger, ClientTriggerCommand, ClientTriggerTarget,
};

mod event_yield_handler;
pub use crate::ipc::zone::client::event_yield_handler::EventYieldHandler;

mod item_operation;
pub use crate::ipc::zone::client::item_operation::ItemOperation;

mod event_return_handler;
pub use crate::ipc::zone::client::event_return_handler::EventReturnHandler;

mod mail;
pub use mail::{MailItemInfo, TakeAttachmentsInfo};

mod queue_duties;
pub use queue_duties::{DutyFinderSetting, QueueDuties};

mod dye_information;
pub use dye_information::DyeInformation;

use crate::ipc::zone::{
    CWLSPermissionRank, InviteReply, InviteType, LETTER_MSG_MAX_LENGTH, LinkshellInviteResponse,
    MAX_MAIL_ATTACHMENTS_STORAGE, OnlineStatusMask, SearchInfo, SearchUIClassJobMask,
    SearchUIGrandCompanies, SocialListUILanguages, StrategyBoard, StrategyBoardUpdate,
    WaymarkPreset,
};

use crate::ipc::zone::black_list::RequestBlacklist;

pub use super::social_list::{
    FriendGroupIconInfo, PlayerEntry, SocialList, SocialListRequest, SocialListRequestType,
};

use super::config::Config;
use crate::common::{
    CHAR_NAME_MAX_LENGTH, ClientLanguage, HandlerId, HouseId, JumpState, MoveAnimationState,
    MoveAnimationType, ObjectId, Position, read_bool_from, read_quantized_rotation, read_string,
    write_bool_as, write_quantized_rotation, write_string,
};
use crate::opcodes::ClientZoneIpcType;
use crate::packet::ServerIpcSegmentHeader;

use crate::common::{ContainerType, ObjectTypeId};
use crate::packet::IpcSegment;

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ContentFinderUserAction {
    /// Accepted the duty.
    #[default]
    Accepted = 0,
    /// Withdrawn from the duty.
    Withdrawn = 1,
    /// Let the duty announcement time out.
    Timeout = 2,
}

pub type ClientZoneIpcSegment =
    IpcSegment<ServerIpcSegmentHeader<ClientZoneIpcType>, ClientZoneIpcType, ClientZoneIpcData>;

#[opcode_data(ClientZoneIpcType)]
#[binrw]
#[br(import(magic: &ClientZoneIpcType, size: &u32))]
#[derive(Debug, Clone)]
pub enum ClientZoneIpcData {
    InitRequest {
        #[brw(pad_before = 40)] // seems to be empty?
        #[brw(pad_size_to = 32)]
        #[br(count = 32)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        unk1: String,
        #[br(count = 48)]
        #[brw(pad_size_to = 48)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        unk2: String,
    },
    FinishLoading {
        unk: [u8; 72],
    },
    ClientTrigger(ClientTrigger),
    UnkSocialEvent {
        unk: [u8; 8],
    },
    SocialListRequest(SocialListRequest),
    UpdatePositionHandler {
        /// In radians.
        rotation: f32,
        anim_type: MoveAnimationType,
        anim_state: MoveAnimationState,
        #[brw(pad_after = 1)] // empty
        jump_state: JumpState,
        #[brw(pad_after = 4)] // empty
        position: Position,
    },
    LogOut {
        unk: [u8; 8],
    },
    Disconnected {
        unk: [u8; 8],
    },
    SendChatMessage(SendChatMessage),
    GMCommand {
        command: u32,
        arg0: i32,
        arg1: i32,
        arg2: i32,
        arg3: i32,
        #[brw(pad_after = 4)]
        target: u64,
    },
    ZoneJump {
        exit_box: u32,
        position: Position,
        #[brw(pad_after = 4)] // padding
        landset_index: i32,
    },
    ActionRequest(ActionRequest),
    /// A ground-targeted action (Asylum, Sacred Soil, Doton...): retail's layout (Bob's Chronofoil captures, every
    /// sample Asylum): the action, a byte, the action type, the sequence, the rotation twice, then where on the ground.
    ActionRequestGroundTargeted {
        action_id: u32,
        unk1: u8,
        action_type: ActionType,
        sequence: u16,
        #[br(map = read_quantized_rotation)]
        #[bw(map = write_quantized_rotation)]
        rotation: f32,
        unk2: u16,
        position: Position,
    },
    PingSync {
        timestamp: u32,
        /// Sapphire calls it this, but it never seems to have the player's actor id or any values resembling ids of any sort in it?
        origin_entity_id: u32,
        #[brw(pad_before = 4)]
        position: Position,
        #[brw(pad_after = 4)]
        rotation: f32,
    },
    ItemOperation(ItemOperation),
    StartTalkEvent {
        actor_id: ObjectTypeId,
        #[brw(pad_after = 4)] // padding
        handler_id: HandlerId,
    },
    EventYieldHandler2(EventYieldHandler<2>),
    EventYieldHandler4(EventYieldHandler<4>),
    EventYieldHandler16(EventYieldHandler<16>),
    EventYieldHandler32(EventYieldHandler<32>),
    EventYieldHandler128(EventYieldHandler<128>),
    StandardControlsPivot {
        /// Set to 4 when beginning to pivot.
        /// Set to 0 when pivoting ends.
        #[brw(pad_after = 4)]
        is_pivoting: u32,
    },
    EventReturnHandler2(EventReturnHandler<2>),
    EventReturnHandler8(EventReturnHandler<8>),
    EventReturnHandler16(EventReturnHandler<16>),
    /// A scene finished with up to 128 results: the crafting event's list of the stacks a synthesis takes (77 results;
    /// Bob's Chronofoil capture of 2026-10-24.)
    EventReturnHandler128(EventReturnHandler<128>),
    /// Sent as the marketboard's window opens, before any search; retail answers with an empty
    /// `ServerZoneIpcData::MarketBoardItems` whose request ID is this one.
    MarketBoardOpened {
        /// Counts up with the searches: 0 on a new session, then one more than the last search's.
        request_id: u32,
        unk: u32,
    },
    /// Buying a marketboard listing: what the listing said when the player chose it (Bob's Chronofoil captures of
    /// 2026-09-15; the server answers with `ServerZoneIpcData::MarketBoardPurchase`.)
    MarketBoardPurchaseHandler {
        retainer_id: u64,
        listing_id: u64,
        /// Index into the Item Excel sheet.
        item_id: u32,
        quantity: u32,
        price_per_unit: u32,
        total_tax: u32,
        /// Unknown: `03 00 00 04 01 00 00 00` and `00 00 00 01 00 00 00 00` on retail.
        unk: [u8; 8],
    },
    /// The client asks for its retainers for the marketboard's listings, as it opens a retainer's sale list (580 on
    /// 2026.09.15, Bob's Chronofoil captures; all zeros there.) Answered with `ServerZoneIpcData::PlayerRetainerInfo`.
    RequestPlayerRetainerInfo {
        unk: u64,
    },
    /// The appearance chosen for a new retainer in the retainer vocate's editor (Bob's 2026-10-18 capture: the new
    /// retainer's SpawnNpc had exactly this voice and CustomizeData.)
    SetRetainerAppearance {
        voice: u8,
        look: CustomizeData,
        /// 0xFF on retail.
        #[brw(pad_after = 4)]
        unk: u8,
    },
    /// The client asks for a retainer's sale history (243 on 2026.09.15, Bob's Chronofoil captures.) Answered with
    /// `ServerZoneIpcData::RetainerSaleHistory`.
    RequestRetainerSaleHistory {
        retainer_id: u64,
    },
    /// The client sets its adventurer plate's portrait (the Portraits window), as BannerData in FFXIVClientStructs. The
    /// server sends it back in `ServerZoneIpcData::PortraitsInformation`.
    SetAdventurerPlatePortrait {
        #[brw(pad_after = 4)]
        portrait: [u8; 52],
    },
    /// The client saves its adventurer plate (the Adventurer Plate window): what the plate shows, as the server sends
    /// it back in `ServerZoneIpcData::AdventurerPlate`. Sent by CharaCard (`0xC4AB2E` writes the opcode, 216 bytes
    /// with the header). Retail capture (2026-10-20): the server keeps it as sent except `unk14`, which the client
    /// fills with leftover memory and retail stores as 0.
    UpdateAdventurerPlate {
        /// Which part of the Adventurer Plate window was saved: 1 portrait, 2 profile (title, active hours, play
        /// styles), 3 design (base plate, decorations), 4 settings (who can see it).
        section: u32,
        chara_card: crate::ipc::zone::CharaCardData,
        /// The timestamp of the plate the client last received (0 if it was never saved).
        #[brw(pad_after = 4)] // leftover memory
        timestamp: u32,
    },
    Config(Config),
    UnkCall2 {
        unk1: [u8; 8],
    },
    QueueDuties(QueueDuties),
    EquipGearset {
        /// Index into the list of gearsets that the client keeps on its side.
        gearset_index: u32,
        /// In order: weapon, off-hand, head, body, hands, invalid/waist, legs, feet, earrings, neck, wrist, left ring, right ring, soul crystal
        /// When a container is irrelevant, it is marked as 9999/ContainerType::Invalid.
        containers: [ContainerType; 14],
        /// Indices into the containers.
        indices: [i16; 14],
        /// For the moment, it is completely unclear what unk1 and unk2 are used for or represent.
        #[brw(pad_before = 6)]
        unk1: u16,
        #[brw(pad_after = 2)]
        unk2: u16,
    },
    EquipGearset2 {
        /// Index into the list of gearsets that the client keeps on its side.
        gearset_index: u32,
        /// In order: weapon, off-hand, head, body, hands, invalid/waist, legs, feet, earrings, neck, wrist, left ring, right ring, soul crystal
        /// When a container is irrelevant, it is marked as 9999/ContainerType::Invalid.
        containers: [ContainerType; 14],
        /// Indices into the containers.
        indices: [i16; 14],
        /// For the moment, it is completely unclear what unk1/unk2/unk3 are used for or represent.
        #[brw(pad_before = 6)]
        unk1: u16,
        #[brw(pad_after = 2)]
        unk2: u16,
        #[br(count = 56)]
        #[brw(pad_size_to = 56)]
        unk3: Vec<u8>,
    },
    StartWalkInEvent {
        event_arg: u32,
        handler_id: HandlerId,
        #[brw(pad_after = 4)]
        pos: Position,
    },
    ContentFinderAction {
        action: ContentFinderUserAction,
        unk1: [u8; 7],
    },
    NewDiscovery {
        layout_id: u32,
        pos: Position,
    },
    GMCommandName {
        command: u32,
        arg0: i32,
        arg1: i32,
        arg2: i32,
        arg3: i32,
        /// Originating World ID of the caller.
        world_id: u16,
        /// Name passed to the command.
        #[brw(pad_size_to = 30)]
        #[br(count = 30)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        name: String,
    },
    GMCommandName2 {
        command: u32,
        arg0: i32,
        arg1: i32,
        #[brw(pad_size_to = 28)]
        #[br(count = 28)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        data: String,
    },
    RequestBlacklist(RequestBlacklist),
    RequestFellowships {
        unk: [u8; 8],
    },
    RequestCrossworldLinkshells {
        unk: [u8; 8],
    },
    SearchFellowships {
        #[br(count = 112)]
        #[brw(pad_size_to = 112)]
        unk: Vec<u8>,
    },
    StartCountdown {
        /// The actor id of the character who initiated the countdown.
        starter_actor_id: ObjectId,
        /// The duration of the countdown in seconds.
        #[brw(pad_after = 2)] // Empty/zeroes, doesn't appear to have anything?
        duration: u16,
        /// The name of the character who initiated the countdown.
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        starter_name: String,
    },
    RequestPlaytime {
        unk: [u8; 8],
    },
    SetFreeCompanyGreeting {
        #[brw(pad_size_to = 192)]
        #[br(count = 192)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        #[brw(pad_after = 8)]
        message: String,
    },
    SetClientLanguage {
        #[brw(pad_before = 4)] // empty
        #[brw(pad_after = 3)] // empty
        language: ClientLanguage,
    },
    RequestCharaInfoFromContentIds {
        content_ids: [u64; 10],
    },
    PartyLeave {
        unk: [u8; 8], // seems to always be zeroes?
    },
    PartyDisband {
        unk: [u8; 8], // seems to always be zeroes?
    },
    PartyMemberKick {
        content_id: u64,
        unk: u16, // Always 0x003F?

        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        #[brw(pad_after = 6)] // empty
        character_name: String,
    },
    PartyChangeLeader {
        /// The actor id of the new leader.
        content_id: u64,
        unk: u16, // Always 0x003F?

        /// The name of the new leader.
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        #[brw(pad_after = 6)] // empty
        character_name: String,
    },
    InviteCharacter {
        content_id: u64,
        world_id: u16,
        invite_type: InviteType,
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        #[brw(pad_after = 5)] // "empty", but see above
        character_name: String,
    },
    InviteReply {
        inviter_content_id: u64,
        inviter_world_id: u16, // The current world id
        invite_type: InviteType,
        #[brw(pad_after = 4)] // empty
        response: InviteReply,
    },
    InviteReply2 {
        inviter_content_id: u64, // The inviter's content_id
        inviter_world_id: u16,   // The current world id
        unk1: u8,
        invite_type: InviteType,
        response: InviteReply,
        #[brw(pad_after = 3)]
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        inviter_character_name: String,
    },
    RequestSearchInfo {
        content_id: u64,
        unk: [u8; 16], // unsure if this is always empty
    },
    RequestAdventurerPlate {
        actor_id: ObjectId,
        unk: [u8; 12],
    },
    SearchPlayers {
        /// The classjobs to filter by.
        classjobs: SearchUIClassJobMask,
        /// The minimum level to filter by.
        minimum_level: u16,
        /// The maximum level to filter by.
        maximum_level: u16,
        /// The grand companies to filter by, if any. If *none* are searched for, it will be 255.
        #[brw(pad_before = 4)] // empty
        #[brw(pad_after = 7)] // empty
        grand_company: SearchUIGrandCompanies,
        /// The languages to filter by. At least one is always selected.
        #[brw(pad_after = 7)] //empty
        languages: SocialListUILanguages,
        /// The online statuses to filter by.
        online_status: OnlineStatusMask,
        /// The areas to filter by. These are row indices into the PlaceName sheet. If an entry is non-zero, it's actively being searched for.
        areas: [u16; 50],
        /// The name of the character to search for. This is optional, and can be blank. If searching for a last name, the first character of the string will be a space (0x20).
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        name: String,
        /// Unknown, but has data in it on retail. Doesn't appear to be a timestamp, sequence value, or ObjectId.
        unk: u32,
    },
    EditSearchInfo(SearchInfo),
    RequestOwnSearchInfo {
        unk: [u8; 8],
    },
    WalkOutsideEvent {
        event_arg: u32,
        handler_id: HandlerId,
        #[brw(pad_after = 4)]
        pos: Position,
    },
    EnterTerritoryEvent {
        #[brw(pad_after = 4)] // empty
        handler_id: HandlerId,
    },
    Trade {
        sequence: u32,
        action_type: u32, // Observed as 1 when beginning a trade, and 7 when doing the second trade opcode
        target_actor_id: ObjectId,
        unk3: [u8; 4],
    },
    ShareStrategyBoard {
        /// When the content id is 0, the client is starting a non-real-time session, or is initiating one but isn't ready yet.
        content_id: u64,
        board_data: StrategyBoard,
    },
    StrategyBoardReceived {
        content_id: u64,
        #[brw(pad_after = 4)] // Seems to be empty/always zeroes
        unk: u32,
    },
    StrategyBoardUpdate(StrategyBoardUpdate),
    RealtimeStrategyBoardFinished {
        /// Both unknowns have data, but it's unclear what they are. They don't appear to be party ids or content ids.
        unk1: u32,
        unk2: u32,
    },
    ApplyFieldMarkerPreset(WaymarkPreset),
    RequestFreeCompanyShortMessage {
        /// The content id of the requested character.
        content_id: u64,
        #[brw(pad_after = 4)]
        /// A sequence value that is repeated by the server later on in FreeCompanyShortMessage.
        sequence: u32,
    },
    QueueRoulette {
        /// See the ContentRoulette Excel sheet.
        roulette_id: u8,
        unk1: [u8; 15],
        /// The languages to match with.
        languages: SocialListUILanguages,
        unk2: [u8; 7],
    },
    InitiateReadyCheck {
        unk: [u8; 8], // Seems to always be zeroes/unused
    },
    ReadyCheckResponse {
        /// The player's response to the ready check. 1 indicates yes, 0 indicates no.
        /// If the player fails to respond before the vote ends, their vote is automatically cast as no.
        #[brw(pad_before = 1, pad_after = 6)] // Seems to be empty/zeroes
        response: u8,
    },
    /// A page of a marketboard search: the items of one ItemSearchCategory.
    RequestMarketBoardItems {
        /// Where the page starts (0, 20, 40...)
        start_index: u32,
        /// Counts up with each search; every page of one search has the same.
        request_id: u8,
        /// 0x0A for a search's first page, 0x0B for the ones after.
        kind: u8,
        /// Index into the ItemSearchCategory Excel sheet.
        category: u8,
        /// 1 on the Weapons and Armor tabs, 0 on Items and Housing (Bob's 2026-10-17 session; always 0 in his
        /// 2026-09-15 captures, which only browsed Items.)
        flags: u8,
        /// With `flags` 1, the first byte is the list's sort (0x13 as the window opened, 0x12 after Bob changed it to
        /// level: his bundle of 2026-10-23; see `marketboard::search_order` in the world server); otherwise search
        /// text and leftovers of earlier replies (the client doesn't clear it.)
        #[br(count = 128)]
        #[brw(pad_size_to = 128)]
        unk: Vec<u8>,
    },
    /// Named after creating a local linkshell (upstream), but on 2026.10.06 this opcode is also how the client answers
    /// an event's text input: a yield with a string, e.g. a retainer's name at the retainer vocate (Bob's Chronofoil
    /// capture of 2026-10-18, 48 bytes: handler u32, scene u16, yield u8, u8, u32 4, u32 0, the text from byte 16.)
    /// Kept as raw bytes since the two don't agree on the layout; see `event_text`. Only the first 40 bytes are read
    /// (the size upstream knows), which holds a text of up to 24 bytes: retainer names are at most 20.
    CreateLocalLinkshellRequest {
        data: [u8; 40],
    },
    CrossworldLinkshellMemberListRequest {
        linkshell_id: u64,
        #[brw(pad_after = 6)] // Seems to be empty/zeroes
        sequence: u16,
    },
    SetFriendGroupIcon(FriendGroupIconInfo),
    OpenTreasure {
        /// Object ID of the treasure that this player is opening.
        #[brw(pad_after = 4)] // empty
        entity_id: ObjectId,
    },
    /// Need, Greed or Pass on one item of an open coffer (Bob's Chronofoil capture of Sastasha, 2026-10-22): `choice`
    /// 1 need, 2 greed, 5 pass (greed and pass seen; need by the LootMessages' 1.)
    LootRoll {
        treasure_id: ObjectId,
        slot: u32,
        choice: u32,
        unk: u32,
    },
    CrossRealmListingsRequest1 {
        unk1: u32,
        unk2: u32,
        max_results: u32,
        unk3: [u8; 44],
    },
    CrossRealmListingsRequest2 {
        unk1: [u8; 40],
    },
    ViewCrossRealmListing {
        #[brw(pad_after = 8)] // empty
        listing_id: u64,
    },
    CheckCWLinkshellNameAvailability {
        unk1: u8, // TODO: What is this? Seems to be always 1?
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        name: String,
        unk2: [u8; 7], // Unknown data, likely garbage since several other client opcodes have been discovered to leave garbage behind around name strings
    },
    CreateNewCrossworldLinkshell {
        /// The name of the cross-world linkshell.
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        name: String,
    },
    DisbandCrossworldLinkshell {
        /// The id of the linkshell to delete.
        linkshell_id: u64,
    },
    LeaveCrossworldLinkshell {
        /// The id of the linkshell to leave.
        linkshell_id: u64,
    },
    RenameCrossworldLinkshell {
        /// The id of the linkshell to rename.
        linkshell_id: u64,
        /// The desired name.
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        name: String,
    },
    SetCWLSMemberRank {
        /// The id of the linkshell to promote a member in.
        linkshell_id: u64,
        /// The content id of the member to promote.
        content_id: u64,
        /// The rank to assign to this member.
        #[brw(pad_after = 3)] // Seems to be empty/zeroes
        rank: CWLSPermissionRank,
        unk: [u8; 4], // Contains unknown data, possibly garbage. Doesn't seem to be a timestamp, actor id, or anything recognisable at this time, and the server doesn't echo it back.
    },
    RemoveCWLSMember {
        /// The linkshell to remove this member from.
        linkshell_id: u64,
        /// The target member's content id.
        content_id: u64,
    },
    InviteCharacterToCWLS {
        linkshell_id: u64,
        #[brw(pad_after = 8)] // Seems to be empty/zeroes
        content_id: u64,
    },
    LinkshellInviteReply {
        linkshell_id: u64,
        #[brw(pad_after = 7)]
        // Seems to be empty/zeroes, but it's unclear if the response is larger than 1 byte
        response: LinkshellInviteResponse,
    },
    RequestMailbox {
        /// The current index in the sequence. This increments by 5 for every request, until the mailbox's entire contents has been sent (up to 100 letters for player mail, 20 for item rewards, and 10 for GM letters).
        current_index: u8,
        unk1: u8,      // Probably just padding
        unk2: u16, // Observed as 0x98DA at the beginning of a RequestMailbox <-> MailboxPreview exchange. No idea what it means.
        unk3: [u8; 4], // Probably just padding
    },
    SendLetter {
        /// The recipient's content id.
        recipient_content_id: u64,
        /// The items attached to the letter, if any. Gil must always be the last item!
        attached_items: [MailItemInfo; MAX_MAIL_ATTACHMENTS_STORAGE],
        /// The message to send. In-game it's only 200 characters, but it's larger than 201 bytes due to multi-byte characters.
        #[brw(pad_size_to = LETTER_MSG_MAX_LENGTH)]
        #[br(count = LETTER_MSG_MAX_LENGTH)]
        #[brw(pad_after = 7)] // Seems to just be padding/garbage
        message: SeString,
    },
    ViewLetter {
        /// The sender's content id.
        sender_content_id: u64,
        /// The time the letter was sent.
        timestamp: u32,
        unk: u32, // Always 1?
    },
    DeleteLetter {
        /// The sender of the letter to delete.
        sender_content_id: u64,
        /// The timestamp of the letter to delete.
        timestamp: u32,
        unk: u32, // Probably just padding
    },
    RewardDeliveryRequest {
        unk: [u8; 8],
    },
    TakeLetterAttachments {
        sender_content_id: u64,
        timestamp: u32,
        items_taken: [TakeAttachmentsInfo; MAX_MAIL_ATTACHMENTS_STORAGE],
        unk: [u8; 4],
    },
    RemoveFriend {
        content_id: u64,
        home_world_id: u16,
        #[brw(pad_size_to = CHAR_NAME_MAX_LENGTH)]
        #[br(count = CHAR_NAME_MAX_LENGTH)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        #[brw(pad_after = 6)] // Seems to just be padding/garbage
        name: String,
    },
    Dive {
        rotation: f32,
        target_position: Position,
        #[brw(pad_after = 4)]
        original_position: Position,
    },
    WorldInteraction {
        action: u32,
        param1: u32,
        param2: u32,
        param3: u32,
        param4: u32,
        position: Position,
    },
    PlaceFurniture {
        /// Likely the plot upon which this furniture was placed. When indoors, this should be zero.
        plot_index: u8,
        unk1: [u8; 7], // Zeroes? Haven't seen it change yet.
        /// The container the furniture item came from.
        container: ContainerType,
        /// The slot the item came from.
        slot: u16,
        /// Where in the world the client has placed it.
        position: Position,
        /// The direction the furniture will be rotated towards when placed. This matters more for placing items from the storeroom, since in that case the furniture's front will face the player. The client sends 0.0 otherwise.
        rotation: f32,
        /// If true, this furniture will be spawned in the world, if false it should be placed into the storeroom. This seems to be false only when right-clicking furniture in the main inventory and selecting "Store". Otherwise, it seems to be always true.
        #[br(map = read_bool_from::<u32>)]
        #[bw(map = write_bool_as::<u32>)]
        spawn_furniture: bool,
        unk2: [u8; 8], // Zeroes?
    },
    TranslateFurniture {
        /// Which house this affects.
        house_id: HouseId,
        /// Index of the housing item being moved. This is across all placed item containers, e.g. 50 here means the second container's first slot.
        slot: u16,
        /// Unknown.
        unk2: u16,
        /// The new position of this furniture.
        position: Position,
        /// The new rotation of this furniture.
        rotation: f32,
        /// Unknown.
        unk3: u32,
    },
    UpdatePositionHandlerInstance {
        rotation: f32,
        interpolate_rotation: f32,
        anim_type: MoveAnimationType,
        anim_state: MoveAnimationState,
        #[brw(pad_after = 1)] // empty
        jump_state: JumpState,
        position: Position,
        interpolate_position: Position,
        unk1: u32, // Seems to be zeroed?
    },
    FallFromArena {
        unk1: [u8; 24],
    },
    DyeInformation(DyeInformation),
    RequestRecruitingPartyCount {
        unk1: [u8; 8], // empty, as far as i've seen
    },
    UnkMarketBoardRequest1 {
        /// Corresponds to an item on the marketboard. Index into the Item Excel sheet.
        item_id: u32,
        unk2: u32,
    },
    /// A page of an item's listings, after `UnkMarketBoardRequest1` opened it.
    UnkMarketBoardRequest2 {
        /// Where the page starts (0, 10, 20...), the previous page's `next_index`.
        start_index: u8,
        /// Counts up with each item opened.
        #[brw(pad_after = 6)] // leftovers, not cleared by the client
        request_id: u8,
    },
    PerformanceNote {
        unk1: [u8; 24],
    },
    CreateFellowship {
        #[brw(pad_size_to = 60)]
        #[br(count = 60)]
        #[br(map = read_string)]
        #[bw(map = write_string)]
        #[brw(pad_after = 4)] // Seems to just be padding/garbage
        name: String,
    },
    SearchFellowshipsJoin {
        community_id: u64,
    },
}

#[cfg(test)]
mod tests {
    use crate::common::test_opcodes;

    use super::*;

    // Ensure that the IPC data size as reported matches up with what we write
    #[test]
    fn client_zone_ipc_sizes() {
        test_opcodes::<ClientZoneIpcSegment>();
    }
}

/// The text an event yield carried (`ClientZoneIpcData::CreateLocalLinkshellRequest` in its 48-byte form): the event's
/// handler, scene, yield ID and the text.
#[derive(Debug, Clone, PartialEq)]
pub struct EventText {
    pub handler_id: u32,
    pub scene: u16,
    pub yield_id: u8,
    pub text: String,
}

/// Reads an event yield's text from the packet's bytes (the 48-byte layout; the text from byte 16 to the end of what was
/// read.) None if it's too short or isn't for an event (no handler.)
pub fn event_text(data: &[u8]) -> Option<EventText> {
    if data.len() < 40 || data[..4] == [0; 4] {
        return None;
    }
    let text = &data[16..];
    let end = text
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(text.len());
    Some(EventText {
        handler_id: u32::from_le_bytes(data[0..4].try_into().ok()?),
        scene: u16::from_le_bytes(data[4..6].try_into().ok()?),
        yield_id: data[6],
        text: String::from_utf8_lossy(&text[..end]).into_owned(),
    })
}

#[cfg(test)]
mod event_text_tests {
    use super::*;

    /// The retainer vocate's name yield as retail's client sent it (Bob's 2026-10-18 capture, the name replaced.)
    #[test]
    fn reads_event_text() {
        let mut data = vec![0x09, 0x00, 0x0b, 0x00, 1, 0, 5, 0, 4, 0, 0, 0, 0, 0, 0, 0];
        data.extend_from_slice(b"Abc");
        data.resize(48, 0);
        assert_eq!(
            event_text(&data),
            Some(EventText {
                handler_id: 0x000b_0009,
                scene: 1,
                yield_id: 5,
                text: "Abc".to_string(),
            })
        );
        assert_eq!(event_text(&data[..40]).unwrap().text, "Abc");
        assert_eq!(event_text(&data[..30]), None);
    }
}
