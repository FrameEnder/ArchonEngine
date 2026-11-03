use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use icarus::Quest::QuestSheet;
use tokio::sync::mpsc::{Sender, error::TrySendError};

use crate::{
    StatusEffects,
    lua::LuaTask,
    server::Party,
    zone_connection::{BaseParameters, TeleportQuery},
};
use kawari::{
    common::{
        CharacterMode, ContainerType, HandlerId, JumpState, LegacyEquipmentModelId,
        MoveAnimationState, MoveAnimationType, ObjectId, ObjectTypeId, Position, WarpType,
        WeaponModelId,
    },
    ipc::{
        chat::{
            CWLinkshellMessage, ChatChannelType, ChatDiceRollResult, PartyMessage, TellMessage,
        },
        zone::{
            ActionRequest, ActorControlCategory, CWLSLeaveReason, CWLSPermissionRank,
            ClientTrigger, Condition, Conditions, Config, CrossworldLinkshellInvite,
            DutyFinderSetting, InviteReply, InviteType, OnlineStatus, PartyMemberEntry,
            PartyMemberPositions, PartyUpdateStatus, ReadyCheckReply, ServerZoneIpcSegment,
            SpawnNpc, SpawnObject, SpawnPlayer, SpawnTreasure, StrategyBoard, StrategyBoardUpdate,
            WaymarkPlacementMode, WaymarkPosition, WaymarkPreset, ZoneDiceRollResult,
        },
    },
};

use super::lua::LuaZone;

use physis::sestring::SeString;

#[derive(Copy, Clone, Default, Eq, PartialEq, Hash)]
pub struct ClientId(usize);

impl std::fmt::Debug for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ClientId({})", self.0)
    }
}

/// A type encapsulating the different spawn types.
/// Note that event object (eobj) spawning is handled elsewhere in connection.rs.
#[derive(Clone, Debug)]
pub enum SpawnKind {
    /// A player's spawn data is contained within.
    Player(SpawnPlayer),
    /// An NPC's spawn data is contained within.
    Npc(SpawnNpc),
}

/// A type encapsulating party update sender and recipient info.
/// This is internal to Kawari, hence it being placed here.
#[derive(Clone, Debug, Default)]
pub struct PartyUpdateTargets {
    pub execute_account_id: u64,
    pub execute_content_id: u64,
    pub execute_name: String,
    pub target_account_id: u64,
    pub target_content_id: u64,
    pub target_name: String,
}

/// A type encapsulating various information about a zone chat mesage to be sent.
#[derive(Clone, Debug, Default)]
pub struct MessageInfo {
    /// The sender's actor id.
    pub sender_actor_id: ObjectId,
    /// The sender's account id. Likely used by the client to know to ignore the message if this player is blocked.
    pub sender_account_id: u64,
    /// The sender's home world id. Used for purposes of displaying their home world in the chat window.
    pub sender_world_id: u16,
    /// The sender's name.
    pub sender_name: String,
    /// The sender's position in the zone, used for creating a radius around which the message is heard (not yet implemented on Kawari).
    pub sender_position: Position,
    /// The channel the message is intended for (say, shout, yell, custom emote (/em)).
    pub channel: ChatChannelType,
    /// The chat message itself.
    pub message: SeString,
}

/// The "part" retail sends in LocationDiscovered to reveal a whole map at once: on entering one of A Realm Reborn's
/// field zones for the first time (Bob's Chronofoil capture, 2026-10-14: Central Shroud, map 4, part 164; North Shroud
/// too). Every part of the map counts as discovered, which completes its "Mapping the Realm" achievement ("Visit ... and
/// unlock the area map").
pub const WHOLE_MAP_DISCOVERY: u32 = 164;

#[derive(Clone, Debug)]
pub enum FromServer {
    /// A chat message.
    Message(MessageInfo),
    /// An actor has been spawned.
    ActorSpawn(ObjectId, SpawnKind),
    /// An actor moved to a new position.
    ActorMove(
        ObjectId,
        Position,
        f32,
        MoveAnimationType,
        MoveAnimationState,
        JumpState,
        /// The speed byte to send (tenths of a yalm a second), or None to work it out from the animation.
        Option<u8>,
    ),
    // An actor should be despawned.
    DeleteActor(ObjectId, u8),
    // An object should be despawned.
    DeleteObject(u8),
    /// We need to update an actor
    ActorControl(ObjectId, ActorControlCategory),
    /// We need to update an actor's target
    ActorControlTarget(ObjectId, ObjectTypeId, ActorControlCategory),
    /// We need to update the player actor
    ActorControlSelf(ActorControlCategory),
    /// Update an actor's model IDs.
    ActorEquip(
        ObjectId,
        WeaponModelId,
        WeaponModelId,
        [LegacyEquipmentModelId; 10],
        [u8; 10],
    ),
    /// We need to summon a player's minion, and tell other clients
    ActorSummonsMinion(u32),
    /// We need to despawn a player's minion, and tell other clients
    ActorDespawnsMinion,
    /// The player should lose this effect.
    LoseEffect(u16, u16, ObjectId),
    // TODO: temporary
    Conditions(Conditions),
    /// Sets or clears one condition, leaving the others as the connection has them.
    ToggleCondition(Condition, bool),
    /// To inform the connection of the zone they're loading into.
    ChangeZone(u16, u16, u16, Position, f32, LuaZone, Option<HandlerId>),
    /// We need to inform the recipent about the direct message they're receiving.
    TellMessageReceived(ObjectId, TellMessage),
    /// We need to tell our chat connection that our zone connection has disconnected.
    ChatDisconnected,
    /// Inform the chat connection that its zone connection has joined a party.
    SetPartyChatChannel(u32),
    /// Inform the client that they've received a social invite (party, friends list, free company). Other social features have their own dedicated opcodes and FromServer responses.
    SocialInvite(u64, u64, String, InviteType),
    /// Inform the client about the results of an invite sent to another player.
    InvitationResult(u64, u64, String, InviteType, InviteReply),
    /// The client who received the invite also needs to be informed.
    InvitationReplyResult(u64, String, InviteType, InviteReply),
    /// A chat message from the client's party has been received.
    PartyMessageReceived(PartyMessage),
    /// Members of this party need to be informed of an update.
    PartyUpdate(
        PartyUpdateTargets,
        PartyUpdateStatus,
        Option<(u64, u32, ObjectId, Vec<PartyMemberEntry>)>,
    ),
    /// The party list of a player and the bots that joined them in a duty (`server/party_bot.rs`), or None when they
    /// leave: it isn't a party the server keeps.
    BotParty(Option<Vec<PartyMemberEntry>>),
    /// A bot party changed: its list (None when it's gone), what happened and to whom.
    BotPartyUpdate(
        Option<Vec<PartyMemberEntry>>,
        PartyUpdateStatus,
        PartyUpdateTargets,
    ),
    /// A party bot accepted the player's invite (to the party, or a friend request): its account ID, content ID,
    /// name, and the invite's kind.
    BotInviteAccepted(u64, u64, String, InviteType),
    /// A tell went to a name that's no character and none of the player's bots: that name.
    BotTellNotFound(String),
    /// Inform the client of the result of a social invite they have sent.
    InviteCharacterResult(u64, u16, InviteType, String),
    /// Inform the client they were in a party, and request that they inform us of their return.
    RejoinPartyAfterDisconnect(u64),
    /// Send an arbitrary IPC segment to the client.
    PacketSegment(ServerZoneIpcSegment, ObjectId),
    /// Set of Lua tasks queued up from the server.
    NewTasks(Vec<LuaTask>),
    /// New copy of the status effects list, for use in Lua scripting.
    NewStatusEffects(StatusEffects),
    /// An event object was spawned.
    SpawnObject(SpawnObject),
    /// Inform the client about the location they discovered, so their map can be revealed: map, part (or
    /// `WHOLE_MAP_DISCOVERY`.)
    LocationDiscovered(u32, u32),
    /// Inform the client that a member of their party has shared a strategy board.
    StrategyBoardShared(u64, StrategyBoard),
    /// Inform the sending client that another client received their strategy board.
    StrategyBoardSharedAck(u64),
    /// Inform the client that the board sharer has made a real-time update.
    StrategyBoardRealtimeUpdate(StrategyBoardUpdate),
    /// Inform the client that the board sharer has ended their real-time sharing session.
    StrategyBoardRealtimeFinished,
    /// Inform the client that a waymark was placed or updated.
    WaymarkUpdated(u8, WaymarkPlacementMode, WaymarkPosition, i32),
    /// Inform the client that a waymark preset was applied.
    WaymarkPreset(WaymarkPreset, i32),
    /// Inform the client that they entered an instance exit range.
    EnteredInstanceEntranceRange(u32),
    /// Inform the client to increment the rested EXP by 10 seconds.
    IncrementRestedExp,
    /// Inform the client about a countdown that was started in their party.
    Countdown(u64, u64, String, ObjectId, u16),
    /// Inform the client that a sign/marker was applied to a target by someone in their party.
    TargetSignToggled(u32, ObjectId, ObjectTypeId),
    /// Request the client to begin preparing to leave this content.
    LeaveContent,
    /// The duty the player is in was cleared: its InstanceContent ID, and whether someone there hadn't cleared it
    /// before (`Instance::newcomers`: the first-timer bonus is everyone's then.)
    DutyCleared(u16, bool),
    /// Request the client to finish their current event.
    FinishEvent,
    /// When a fish bites.
    FishBite,
    /// A battle NPC this player (or their party) was fighting was defeated: its BNpcBase ID (what quests' ENEMY
    /// parameters are), BNpcName ID, layout ID and level, and the gil it dropped for this player (`actor::kill_gil`.)
    BattleNpcKilled {
        actor_id: ObjectId,
        base_id: u32,
        name_id: u32,
        layout_id: u32,
        level: u8,
        gil: u32,
    },
    /// The quest battle was won: its director, and the quest and sequence it advances.
    /// Whether the players stay afterwards (until `LeaveQuestBattle`), and the scene played before they leave, with its
    /// scene flags; quest variables (index, value) set with the sequence; whether "Duty Complete" plays first; and
    /// where the players go when they leave it, if not back where they entered it from (territory, position, rotation.)
    QuestBattleComplete(
        HandlerId,
        u32,
        u8,
        bool,
        u16,
        u32,
        Vec<(u8, u8)>,
        bool,
        Option<(u16, Position, f32)>,
    ),
    /// The player won this item from a coffer (`server/loot.rs`): it goes in their inventory.
    LootWon(u32),
    /// An item of a coffer that's this player's alone (Duty Support, `server/loot.rs`): its LootMessage and the item.
    PersonalLoot(u32),
    /// Send the player back from the won quest battle they stayed in.
    LeaveQuestBattle(HandlerId),
    /// The quest battle was lost, send them back to where they entered it.
    QuestBattleFailed,
    /// An event action (the progress bar for examining things and such) finished casting during a quest event.
    QuestEventActionCast,
    /// A key item's cast bar ended (an ActionRequest of type EventItem): the item, and what it was used on. The
    /// zone connection starts its quest's event (type EventItem.)
    EventItemUsed(u32, ObjectTypeId),
    /// An action (its Action row) landed on an event NPC or object the server doesn't have as an actor: a quest that
    /// listens for it starts its event (type ActionResult.)
    ActionOnEventObject(u32, ObjectTypeId),
    /// An item was used and is used up (Gysahl Greens): the zone connection takes one from the bags.
    ItemUsedUp(u32),
    /// The player's companion gained this much EXP from a kill: the zone connection ranks it up (`companion.rs`.)
    CompanionExp(u32),
    /// The player used an emote (its Emote row) on an actor: the zone connection starts the quest event that listens
    /// for it, if one does (type Emote.)
    EmotedAt(u32, ObjectTypeId),
    /// An enemy's attack interrupted the event action (the progress bar) during a quest event.
    QuestEventActionInterrupted,
    /// Inform the client that another player has dismounted.
    ActorDismounted(ObjectId),
    /// Inform the client of the whereabouts of their party members.
    PartyMemberPositionsUpdate(PartyMemberPositions),
    /// Use this connections's database to commit the party list for the server.
    CommitParties(HashMap<u64, Party>),
    /// Treasure was spawned.
    TreasureSpawn(SpawnTreasure),
    /// A chat message from one of the client's cwlses has been received.
    CWLSMessageReceived(CWLinkshellMessage),
    /// Inform the zone and chat connections about their linkshell channels.
    SetLinkshellChatChannels(Vec<u32>, Vec<u32>, bool),
    /// Inform the client that one of their linkshells has been disbanded.
    LinkshellDisbanded(u64, String),
    /// Inform the client that a member left one of their linkshells.
    LinkshellLeft(ObjectId, u64, u64, String, CWLSLeaveReason, u64),
    /// Inform the client that an owner of one of their linkshells has renamed it.
    LinkshellRenamed(u64, String, u64, String),
    /// Inform the client that a member of one of their linkshells had their rank changed.
    LinkshellRankChanged(u64, u64, u64, CWLSPermissionRank, String),
    /// Inform the client that they have received a new cross-world linkshell invitation.
    LinkshellInviteReceived(CrossworldLinkshellInvite),
    /// Inform the client that someone has joined their linkshell.
    LinkshellInviteAccepted(u64, u64, String, String),
    /// Inform the zone connection about their current mount's model id. Helps persist mounts across zone transitions and through log-outs.
    SetCurrentMount(u16),
    /// Inform the chat connection that it needs to refresh its non-party ChatChannels due to some event necessitating it.
    MustRefreshChatChannels,
    /// Inform the client that a friend removal has taken place.
    FriendRemoved(u64, String),
    /// Inform the client that a new letter has arrived in their mailbox.
    NewLetterArrived,
    /// Plays a cutscene from the instance director: the cutscene, the director, and the scene's parameters after the
    /// cutscene's ID (see `DEFAULT_CUTSCENE_PARAMS`).
    PlayDirectorCutscene(u32, HandlerId, Vec<u32>),
    /// Plays one of the director's scenes, with these scene flags.
    PlayDirectorScene(HandlerId, u16, u32),
    /// Inform the client that another player has placed a piece of furniture.
    FurniturePlaced(ContainerType, u16, u16, u8, Position, bool, f32, u8),
    /// Inform the client that another player has moved or rotated a piece of furniture.
    FurnitureTranslated((bool, u8), u16, Position, f32, bool),
    /// Inform the client that another player in their party has offered them a teleport.
    TeleportOffered(u32, TeleportQuery),
    /// Sync to this new max level, and item level if the content has its own (ContentFinderCondition.ItemLevelSync.)
    SyncMaxLevel(u8, Option<u16>),
    /// Minimum IL: bring the character's item level down to this (`Instance::minimum_item_level`.)
    MinimumItemLevel(u16),
    /// The Echo's multiplier on the character's maximum HP (`duty_finder::refresh_echo`.)
    EchoMultiplier(f32),
    /// Play the initial cutscene for instanced content: its duration, settings, and for a quest battle its scene
    /// flags (`OPENING_SCENE_FLAGS` in its script, if it sets them.)
    PlayInitialCutscene(u32, u32, Option<u32>),
    /// A FATE was completed. This is meant to track SHARED FATE completion and not much else yet.
    FATEComplete(u32),
    /// A FATE this player took part in was completed: give its rewards.
    FateReward(u32),
    /// This player died: their gear wears down.
    PlayerDied,
    /// Instruct the client's zone connection to check if the teleport can be shared.
    CheckTeleportSharingEligibility(u32),
    /// Inform the client of their player's new CharacterMode.
    SetCharacterMode(CharacterMode, u8),
    /// Inform the chat connection that someone (or their own player) in one of their ChatChannels sent a dice roll.
    ChatDiceRoll(ObjectId, ChatDiceRollResult),
    /// Inform the zone connection that someone (or their own player) in the vicinity sent a dice roll.
    ZoneDiceRoll(ObjectId, ZoneDiceRollResult),
}

/// How many messages can wait for a zone or chat connection to pick them up. A zone-in alone sends well over a
/// hundred (every NPC and object nearby), on top of NPCs moving every half second.
pub const CLIENT_QUEUE_SIZE: usize = 4096;

#[derive(Debug, Clone)]
pub struct ClientHandle {
    pub id: ClientId,
    pub channel: Sender<FromServer>,
    pub actor_id: ObjectId,
    pub content_id: u64,
    pub account_id: u64,
}

impl ClientHandle {
    /// Send a message to this client actor. Only fails if the connection is gone, which gets the client removed.
    ///
    /// If its queue is full (the connection is busy, e.g. zoning in or running a long script), the message is dropped
    /// with a warning instead. This used to count as an error too, which quietly removed the player from the world
    /// server while their connection carried on: monsters froze and nothing could be attacked until they relogged.
    pub fn send(&mut self, msg: FromServer) -> Result<(), std::io::Error> {
        match self.channel.try_send(msg) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(msg)) => {
                let kind = format!("{msg:?}");
                tracing::warn!(
                    "{:?}'s queue is full ({} messages), dropping a {}",
                    self.id,
                    CLIENT_QUEUE_SIZE,
                    kind.split(['(', ' ', '{']).next().unwrap_or_default()
                );
                Ok(())
            }
            Err(TrySendError::Closed(_)) => Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "The connection is gone",
            )),
        }
    }

    /// Kill the actor.
    pub fn kill(self) {
        // run the destructor
        drop(self);
    }
}

/// What a player did to a party with bots in it, or to a bot (`ToServer::BotPartyAction`.)
#[derive(Debug, Clone)]
pub enum BotPartyAction {
    /// Invited the character with this content ID and name (a bot's are made up.)
    Invite { content_id: u64, name: String },
    /// Removed the member with this content ID and name.
    Kick { content_id: u64, name: String },
    /// Made the member with this content ID leader.
    Promote { content_id: u64 },
    /// Left the party, or disbanded it.
    Leave,
    /// Sent a friend request to the character with this content ID and name.
    Befriend { content_id: u64, name: String },
}

#[derive(Debug)]
pub enum ToServer {
    /// A new zone connection has started.
    NewClient(ClientHandle),
    /// A new chat connection has started.
    NewChatClient(ClientHandle),
    /// The connection sent a message.
    Message(ClientId, ObjectId, MessageInfo),
    /// The connection's player moved.
    ActorMoved(
        ObjectId,
        Position,
        f32,
        MoveAnimationType,
        MoveAnimationState,
        JumpState,
        Option<u64>,
    ),
    /// The connection has recieved a client trigger.
    ClientTrigger(ClientId, ObjectId, ClientTrigger),
    /// The connection loaded into a zone.
    // TODO: the connection should not be in charge and telling the global server what zone they just loaded in! but this will work for now
    ZoneLoaded(ClientId, ObjectId, SpawnPlayer),
    /// The connection wants to enter a new zone.
    // TODO: temporary as this is only used for commands (and diving/surfacing due to this being the least intrusive way to do this currently) and those aren't run on global server state yet
    ChangeZone(
        ClientId,
        ObjectId,
        u16,
        Option<Position>,
        Option<f32>,
        WarpType,
        u16,
    ),
    /// The player walks through a zone change line or passes through an underwater portal (e.g. in Ruby Sea).
    EnterZoneJump(ClientId, ObjectId, u32, WarpType, u16),
    /// The connection disconnected.
    Disconnected(ClientId, ObjectId),
    /// A fatal error occured.
    FatalError(std::io::Error),
    /// Request to perform an action
    ActionRequest(ClientId, ObjectId, ActionRequest),
    /// We want to update our own equip display flags.
    Config(ClientId, ObjectId, Config),
    /// Tell the server what models IDs we have equipped.
    Equip(
        ObjectId,
        WeaponModelId,
        WeaponModelId,
        [LegacyEquipmentModelId; 10],
        [u8; 10],
    ),
    /// The player gains an effect.
    GainEffect(ClientId, ObjectId, u16, u16, f32, ObjectId),
    /// The player loses an effect.
    LoseEffect(ClientId, ObjectId, u16, u16, ObjectId),
    /// A crafter's CP and synthesis statuses (status, param) after a step: the statuses change without a gain or loss
    /// message (the step's scene has the log lines), and an empty list takes them all off (`crafting.rs`.)
    SetCraftingState(ClientId, ObjectId, u16, Vec<(u16, u16)>),
    /// The player's chocobo companion as it is now (at zone-in and whenever it changes: its stance, barding, skills.)
    SetCompanion(ObjectId, crate::server::CompanionProfile),
    /// The player withdrew their companion (client trigger 1700 with Withdraw.)
    WithdrawCompanion(ObjectId),
    /// Something the player fought was defeated, worth this much of their EXP (0 for none): their companion, if it's
    /// out, gets it (`FromServer::CompanionExp`.)
    CompanionKill(ObjectId, u32),
    /// Warp with the specified id.
    Warp(ClientId, ObjectId, u32),
    /// Warp with the specified aetheryte id.
    WarpAetheryte(ClientId, ObjectId, u32, bool, WarpType),
    /// The player opens a coffer (`server/loot.rs`.)
    OpenTreasure(ObjectId, ObjectId),
    /// The player chooses for an item of an open coffer: the coffer, the item's slot, Need/Greed/Pass.
    LootRoll(ObjectId, ObjectId, u32, u32),
    /// The player does something to their party that concerns party bots (`server/party_bot.rs`): they're in no party
    /// the server keeps (their `party_id` is 0), or they invite someone who isn't a character.
    BotPartyAction(ObjectId, BotPartyAction),
    /// The player examines someone: the examiner, the one examined.
    ExamineCharacter(ObjectId, ObjectId),
    /// A tell to a name no character has (maybe one of the player's party bots): the sender, the name, the text.
    BotTell(ObjectId, String, String),
    /// Ready to spawn the player (this happens during initrequest)
    ReadySpawnPlayer(ClientId, ObjectId, u16, Position, f32, Option<u8>),
    /// Ready to send the ZoneIn ACS
    ZoneIn(ClientId, ObjectId, bool),
    /// We need to summon a player's minion, and tell other clients
    ActorSummonsMinion(ObjectId, u32),
    /// We need to despawn a player's minion, and tell other clients
    ActorDespawnsMinion(ObjectId),
    /// Move the player's actor to the specified pop range.
    MoveToPopRange(ClientId, ObjectId, u32, WarpType),
    /// The connection sent a direct message to another client. This needs the sender's actor id too for purposes of `send_ipc_from`.
    TellMessageSent(ObjectId, ObjectId, TellMessage),
    /// The client invited another player to join their party, friend list, or free company.
    InvitePlayerTo(
        ObjectId,
        u64,
        u64,
        String,
        ObjectId,
        u64,
        String,
        InviteType,
    ),
    /// The client replied to another player's invite.
    InvitationResponse(
        ObjectId,
        u64,
        u64,
        String,
        ObjectId,
        u64,
        String,
        InviteType,
        InviteReply,
    ),
    /// The party leader is adding a member to their party.
    AddPartyMember(u64, ObjectId, u64),
    /// The client sent a message to their party.
    PartyMessageSent(PartyMessage),
    /// The client is designating another player in the party as leader.
    PartyChangeLeader(u64, u64, u64, String, u64, String),
    /// The client is removing another player from the party.
    PartyMemberKick(u64, u64, u64, String, u64, String),
    /// The client changed areas.
    PartyMemberChangedAreas(u64, u64, u64, ObjectId, String, i32),
    /// The client left their party.
    PartyMemberLeft(u64, u64, u64, ObjectId, String),
    /// The client disbands their party.
    PartyDisband(u64, u64, u64, String),
    /// The chat connection acknowledges the shutdown notice, and now we need to remove it from our internal state.
    ChatDisconnected(ClientId),
    /// The client went offline and we need to inform other party members.
    PartyMemberOffline(u64, u64, u64, ObjectId, String),
    /// The client returned online and we need to inform other party members.
    PartyMemberReturned(ObjectId, i32, u64),
    /// The client is requesting to join the following content.
    /// The Duty Support members chosen (DawnQuestMember, in party slot order), when registered through Duty Support.
    JoinContent(ClientId, ObjectId, u16, DutyFinderSetting, Option<Vec<u8>>),
    /// A key item used on the ground where the client aimed it (ActionRequestGroundTargeted of type EventItem.)
    EventItemOnGround(ClientId, ObjectId, ActionRequest, Position),
    /// The client started an event action (its EventAction row, last) during a quest event, which finishes casting
    /// after this many seconds.
    QuestEventAction(ClientId, ObjectId, u8, u32),
    /// The client is entering a quest battle: the QuestBattle ID, and the territory it takes place in.
    JoinQuestBattle(ClientId, ObjectId, u32, u16),
    /// This player is in a duty they haven't cleared before (`Instance::newcomers`.)
    DutyNewcomer(ObjectId),
    /// The c lient is requesting to leave their current content, the connection is in charge of keeping track of the old position.
    LeaveContent(ClientId, ObjectId, u16, Position, f32),
    /// Update the global server state of the client's conditions.
    UpdateConditions(ObjectId, Conditions),
    /// (Temporary) Signal to the server to kill this actor.
    Kill(ClientId, ObjectId),
    /// Inform the server to update our HP to this value.
    SetHP(ClientId, ObjectId, u32),
    /// Inform the server to update our MP to this value.
    SetMP(ClientId, ObjectId, u16),
    /// Gives back this many resource points (MP, CP or GP), up to the maximum: a gatherer's GP for each gather.
    RestoreResourcePoints(ObjectId, u16),
    /// A quest (the handler) brings out the battle NPC at this layout ID (from the zone's drop-ins), already after this
    /// player; the last u32 is what brought it out (the layout ID of the object searched or the range walked into.)
    SpawnQuestBattleNpc(ClientId, ObjectId, u32, HandlerId, u32),
    /// A player (the first ID) called a retainer at a summoning bell (its layout ID), standing at this position: they
    /// appear as this NPC with the second ID, at the bell's spot for retainers if the zone has one.
    SpawnRetainer(ObjectId, ObjectId, Box<SpawnNpc>, u32, Position),
    /// A player (the first ID) sent their retainer (the second) away.
    DespawnRetainer(ObjectId, ObjectId),
    /// The client discovered a new location in this zone.
    NewLocationDiscovered(ClientId, u32, Position, u16),
    /// The client is sharing a strategy board with their party.
    ShareStrategyBoard(ObjectId, u64, u64, u64, StrategyBoard),
    /// The client received a strategy board from another member in their party.
    StrategyBoardReceived(u64, u64, u64),
    /// The client is making edits to their strategy board via real-time sharing.
    StrategyBoardRealtimeUpdate(ObjectId, u64, u64, StrategyBoardUpdate),
    /// The client finished their real-time sharing session.
    StrategyBoardRealtimeFinished(u64),
    /// The client applied a waymark preset for their party.
    ApplyWaymarkPreset(ObjectId, u64, WaymarkPreset, i32),
    /// Inform the server of our new basic stat values.
    SetNewStatValues(ObjectId, u8, u8, BaseParameters),
    /// The client started a countdown in their party.
    StartCountdown(u64, ObjectId, u64, u64, String, ObjectId, u16),
    /// The client yields from a GimmickAccessor.
    GimmickAccessor(u32, ObjectId, ObjectId, Vec<i32>),
    /// The client's talk with a duty's ContentNpc returned (its row, the player, the event NPC, the scene's results.)
    ContentNpc(u32, ObjectId, ObjectId, Vec<i32>),
    /// The client begins fishing.
    Fish(ClientId, ObjectId),
    /// Warp to a specified pop range in a new territory.
    WarpPopRange(ClientId, ObjectId, u16, u32, WarpType),
    /// Request the global server state to reload its Lua state.
    ReloadScripts,
    /// The client dismounted.
    Dismounted(ObjectId, Option<u64>),
    /// A chocobo porter's chocobo (its Mount, or 0 as the ride ends; `zone_connection/chocobo_taxi.rs`.)
    ChocoboTaxiMount(ObjectId, u16),
    /// Inform the server of this actor's new online status.
    SetOnlineStatus(ObjectId, OnlineStatus),
    /// The client is requesting to ride pillion with a party member's mount.
    RidePillionRequest(ObjectId, Option<u64>, ObjectId, u32),
    /// Inform the server of this actor's new CharacterMode.
    SetCharacterMode(ObjectId, CharacterMode, u8),
    /// Broadcasts an actor control to other players.
    BroadcastActorControl(ObjectId, ActorControlCategory),
    /// The client initiated a ready check for their party.
    ReadyCheckInitiated(Option<u64>, ObjectId, u64, u64, String),
    /// The client responded to an on-going ready check in their party.
    ReadyCheckResponse(Option<u64>, ObjectId, u64, u64, String, ReadyCheckReply),
    /// Removes action cooldowns for this player.
    RemoveCooldowns(ObjectId),
    /// The client's zone connection wishes to inform its chat connection about any linkshells the player belongs to.
    SetLinkshells(ObjectId, Vec<u64>),
    /// The client sent a message to a cross-world linkshell.
    CWLSMessageSent(CWLinkshellMessage),
    /// The client disbanded their linkshell, and online members need to be informed.
    DisbandLinkshell(u64, String),
    /// The client left a linkshell, and online members need to be informed.
    LeaveLinkshell(ObjectId, u64, u64, String, CWLSLeaveReason, u64),
    /// The client renamed their linkshell.
    RenameLinkshell(u64, String, u64, String),
    /// The client changed the rank of a member in their linkshell.
    SetLinkshellRank(u64, u64, u64, CWLSPermissionRank, String),
    /// The client invited another character to join their linkshell.
    SendLinkshellInvite(ObjectId, CrossworldLinkshellInvite),
    /// The client accepted an invite to a linkshell.
    AcceptedLinkshellInvite(ObjectId, u64, u64, String, String),
    /// The client removes a player from their friend list.
    FriendRemoved(ObjectId, u64, String, ObjectId, u64, String),
    /// The client sends a letter to a friend.
    SendLetterTo(ObjectId),
    /// Teleport to this player.
    Jump(ClientId, String),
    /// Teleport the player to this one.
    Call(ObjectId, String),
    /// The client places a piece of furniture.
    PlaceFurniture(
        ObjectId,
        ContainerType,
        u16,
        u16,
        u8,
        Position,
        bool,
        f32,
        u8,
    ),
    /// The client moves or rotates a piece of furniture.
    TranslateFurniture(ObjectId, (bool, u8), u16, Position, f32, bool),
    /// The client offers a teleport to nearby party members.
    OfferTeleportToParty(Option<u64>, ObjectId, u16, TeleportQuery),
    /// The client equips new glasses.
    EquipGlasses(ObjectId, u32, u32),
    /// The client is ready to initialize its directors.
    ReadyDirectorData(ObjectId),
    /// The client is ready to initialize map effects and other director data.
    EnterTerritoryEvent(ObjectId),
    /// This player is now ready for the instanced content to begin.
    ReadyToCommence(ObjectId),
    /// This player finished a scene the director played (`FromServer::PlayDirectorScene`.)
    DirectorSceneFinished(ObjectId, u16),
    /// The client uses the /dice command to send a dice roll to one of their ChatChannels.
    ChatDiceRoll(ObjectId, ChatDiceRollResult),
    /// The client uses the /random command to send a dice roll to nearby players in the vicinity.
    ZoneDiceRoll(ObjectId, ZoneDiceRollResult),
    /// Tell the server to reload the active festivals in all running instances.
    ReloadFestivals,
    /// For the SpawnBatteNpc custom IPC.
    SpawnBattleNpc(u32),
}

#[derive(Clone, Debug)]
pub struct ServerHandle {
    pub chan: Sender<ToServer>,
    pub next_id: Arc<AtomicUsize>,
}

impl ServerHandle {
    pub async fn send(&mut self, msg: ToServer) {
        if self.chan.send(msg).await.is_err() {
            panic!("Main loop has shut down.");
        }
    }
    pub fn next_id(&self) -> ClientId {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        ClientId(id)
    }
}

/// Turns a Quest row ID into a "normal" one. For example: 65537 to 1.
/// Whether a ClassJob row is a Disciple of the Hand (Carpenter to Culinarian), whose resource is CP.
pub fn is_crafter(class_job: u8) -> bool {
    (8..=15).contains(&class_job)
}

/// Whether a ClassJob row is a Disciple of the Land (Miner, Botanist, Fisher), whose resource is GP.
pub fn is_gatherer(class_job: u8) -> bool {
    (16..=18).contains(&class_job)
}

/// The most resource points (MP, CP or GP) someone of this class has with these parameters. The client keeps the
/// three apart and StatusEffectList fills the one the class uses (7.57, `0x140B38500`: ClassJobCategory 32 GP, 33
/// CP, else MP.)
pub fn max_resource_points(
    class_job: u8,
    parameters: &crate::zone_connection::BaseParameters,
) -> u16 {
    let points = if is_crafter(class_job) {
        parameters.cp
    } else if is_gatherer(class_job) {
        parameters.gp
    } else {
        parameters.mp
    };
    points.min(u16::MAX as u32) as u16
}

pub fn adjust_quest_id(quest_id: u32) -> u32 {
    quest_id.saturating_sub(QuestSheet::STARTING_ROW)
}

pub fn fetch_entries<T>(
    next_index: &mut u16,
    data: &mut Vec<T>,
    increment_by: usize,
    state: &mut usize,
) -> Vec<T>
where
    T: Clone + std::default::Default,
{
    let mut ret: Vec<T>;
    if data.len() > increment_by {
        *next_index += increment_by as u16;
        ret = data.drain(0..increment_by).collect();
    } else {
        *next_index = 0;
        ret = std::mem::take(data);
        ret.resize(increment_by, T::default());
    }

    if !data.is_empty() {
        *state += increment_by;
    } else {
        *state = 0;
    }

    ret
}

/// Rolls an n-sided die, up to n = 999 inclusive.
pub fn roll_die(num_sides: u16) -> (u16, u16) {
    let mut sides_clamped = num_sides;

    // We can't use the standard clamp for this: when the side count is 0, it means the client wants the default of 999 sides.
    // This does matter: it causes the client to display different text.
    // Examples:
    // /dice will display {Random! <some value from 1-999>} (in the client-sent packet, the number of sides specified would be 0, and the server will also send back 0 sides).
    // /dice 999 will display {Random! <some value from 1-999> (1-999)}.
    // /dice 456 will display {Random! <some value from 1-456> (1-456)}.
    if !(1..=999).contains(&sides_clamped) {
        sides_clamped = 999;
    }

    // We do a clamp here from 0-999 inclusive to ensure the client will display the correct text, as described above.
    let sides_result = num_sides.clamp(0, 999);

    let roll_result = fastrand::u16(1..sides_clamped + 1); // +1 since this range is exclusive, and we *do* allow the maximum value to be rolled.

    (sides_result, roll_result)
}
