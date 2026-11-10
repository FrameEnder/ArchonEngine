//! A world without a game, for tests of what the server does in a situation one of Bob's bundles showed
//! (docs/src/fork/testing.md): an `Instance` with players, each with a client whose messages are read back, enemies
//! and a party's Player NPCs, all made by hand. Nothing here reads the game's data, so the tests run in CI; what
//! needs it (stats, sheets, the navmesh) isn't here, and a test of that belongs with a real install.
//!
//! A test puts the actors in, does what the bundle's player did (`casting`, `kill_actor`, a wipe's return...) and
//! checks the instance and what each client was sent (`sent`).

use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
    time::{Duration, Instant},
};

use kawari::{
    common::{ObjectId, ObjectTypeId, ObjectTypeKind, Position},
    ipc::zone::{
        ActionRequest, ActorControlCategory, Conditions, ServerZoneIpcData, SpawnNpc, SpawnPlayer,
    },
};
use parking_lot::Mutex;
use tokio::sync::mpsc::{Receiver, channel};

use crate::{
    ClientHandle, ClientId, FromServer,
    server::{
        ClientState,
        actor::{NetworkedActor, NpcState},
        instance::{Instance, QueuedTaskData},
        network::NetworkState,
        party_bot::{BotLook, BotRole, PartyBot, ShownAsPlayer},
    },
};

pub struct TestWorld {
    pub network: Arc<Mutex<NetworkState>>,
    pub instance: Instance,
    receivers: HashMap<ObjectId, Receiver<FromServer>>,
    clients: HashMap<ObjectId, ClientId>,
}

impl TestWorld {
    pub fn new() -> Self {
        Self {
            network: Arc::new(Mutex::new(NetworkState::default())),
            instance: Instance::default(),
            receivers: HashMap::new(),
            clients: HashMap::new(),
        }
    }

    /// A player with a client, alive, at 1000 of 1000 HP.
    pub fn player(&mut self, id: u32, at: [f32; 3]) -> ObjectId {
        let actor_id = ObjectId(id);
        let mut spawn = SpawnPlayer::default();
        spawn.common.health_points = 1000;
        spawn.common.max_health_points = 1000;
        spawn.common.resource_points = 10000;
        spawn.common.max_resource_points = 10000;
        spawn.common.level = 41;
        spawn.common.position = Position(glam::Vec3A::from_array(at));
        self.instance.actors.insert(
            actor_id,
            NetworkedActor::Player {
                spawn,
                status_effects: Default::default(),
                teleport_query: Default::default(),
                distance_range: kawari::common::DistanceRange::Normal,
                conditions: Conditions::default(),
                executing_gimmick_jump: false,
                inside_instance_exit: false,
                parameters: Default::default(),
                dueling_opponent_id: ObjectId::default(),
                remove_cooldowns: false,
                last_combo_action: 0,
                combo_sequence: 0,
                autoattack_target: None,
                autoattack_timing: 0,
                job_gauge: Default::default(),
            },
        );
        let client = ClientId::for_tests(self.clients.len() + 1);
        let (sender, receiver) = channel(crate::common::CLIENT_QUEUE_SIZE);
        self.network.lock().clients.insert(
            client,
            (
                ClientHandle {
                    id: client,
                    channel: sender,
                    actor_id,
                    content_id: id as u64,
                    account_id: id as u64,
                },
                ClientState::default(),
            ),
        );
        self.receivers.insert(actor_id, receiver);
        self.clients.insert(actor_id, client);
        actor_id
    }

    fn npc(
        &mut self,
        id: u32,
        at: [f32; 3],
        hp: u32,
        shown_as_player: Option<ShownAsPlayer>,
    ) -> ObjectId {
        let actor_id = ObjectId(id);
        let mut spawn = SpawnNpc::default();
        spawn.common.health_points = hp;
        spawn.common.max_health_points = hp;
        spawn.common.resource_points = 10000;
        spawn.common.max_resource_points = 10000;
        spawn.common.level = 41;
        spawn.common.position = Position(glam::Vec3A::from_array(at));
        let home = spawn.common.position.0;
        self.instance.actors.insert(
            actor_id,
            NetworkedActor::Npc {
                state: NpcState::Wander,
                navmesh_path: VecDeque::default(),
                navmesh_path_lerp: 0.0,
                navmesh_target: None,
                last_position: None,
                spawn,
                timeline: serde_json::from_str(
                    r#"{"autoattack_action_id": 870, "timeline_always_plays": false, "timepoints": []}"#,
                )
                .expect("a timeline"),
                timeline_position: 0,
                timeline_checked: 0,
                timeline_anchor: None,
                timeline_hp_done: 0,
                enmity: HashMap::new(),
                autoattack_timing: 0,
                currently_invulnerable: false,
                status_effects: Default::default(),
                last_wander_timestamp: Instant::now(),
                parameters: Default::default(),
                home,
                home_rotation: 0.0,
                returning_home: false,
                cast: None,
                rotation_state: Default::default(),
                placement: Default::default(),
                shown_as_player,
            },
        );
        // Every client has it spawned, as one in range would.
        for (_, state) in self.network.lock().clients.values_mut() {
            state.actor_allocator.reserve(actor_id);
        }
        actor_id
    }

    /// An enemy, alive, with this much HP.
    pub fn enemy(&mut self, id: u32, at: [f32; 3], hp: u32) -> ObjectId {
        self.npc(id, at, hp, None)
    }

    /// A Player NPC (a bot shown as another player) of this player's party, in their party list.
    pub fn player_npc(&mut self, id: u32, leader: ObjectId, at: [f32; 3]) -> ObjectId {
        let content_id = 0x00B0_0000_0000_0000 | id as u64;
        let actor_id = self.npc(
            id,
            at,
            1500,
            Some(ShownAsPlayer {
                account_id: content_id,
                content_id,
                world_id: 0,
                handler_id: Default::default(),
            }),
        );
        if let Some(NetworkedActor::Npc { state, .. }) = self.instance.find_actor_mut(actor_id) {
            *state = NpcState::Follow;
        }
        let order = self.instance.party_bots.len();
        self.instance.party_bots.insert(
            actor_id,
            PartyBot {
                leader,
                gauge: Default::default(),
                known: Vec::new(),
                role: BotRole::Dps,
                attack_range: 3.0,
                heal_action: None,
                heal_below: 0.0,
                next_heal: Instant::now(),
                aoe_action: None,
                pull_action: None,
                next_special: Instant::now(),
                answers: HashMap::new(),
                pulled: HashSet::new(),
                order,
                walking_itself: true,
                path: Default::default(),
                reaimed: Instant::now(),
                walled_since: None,
                far_since: None,
                in_party: true,
                equipped: Default::default(),
                item_level: 0,
                content_id,
                glamour_name: None,
                dead_since: None,
                revive_seen: None,
                raise_action: None,
                raising: None,
                party_npc: None,
                dodge: None,
                braced_for: None,
                brace_ready: HashMap::new(),
                look_kind: BotLook::Player,
                duty_filled: true,
            },
        );
        actor_id
    }

    /// The actor is this far from finishing a cast of this action (its task queued, as `action.rs` queues it.)
    pub fn casting(&mut self, actor: ObjectId, action_id: u32, left: Duration, target: ObjectId) {
        let client = self.clients.get(&actor).copied().unwrap_or_default();
        self.instance.insert_task(
            client,
            actor,
            left,
            QueuedTaskData::CastAction {
                request: ActionRequest {
                    action_id,
                    target: ObjectTypeId {
                        object_id: target,
                        object_type: ObjectTypeKind::None,
                    },
                    ..Default::default()
                },
                interruptible: true,
            },
        );
    }

    pub fn casts_queued(&self, actor: ObjectId) -> usize {
        self.instance
            .queued_task
            .iter()
            .filter(|task| {
                task.from_actor_id == actor
                    && matches!(task.data, QueuedTaskData::CastAction { .. })
            })
            .count()
    }

    /// Everything this player's client has been sent since the last call.
    pub fn sent(&mut self, player: ObjectId) -> Vec<FromServer> {
        let mut out = Vec::new();
        if let Some(receiver) = self.receivers.get_mut(&player) {
            while let Ok(message) = receiver.try_recv() {
                out.push(message);
            }
        }
        out
    }
}

/// Whether a message is this actor's packet of the kind `is` picks out.
pub fn packet_from(
    message: &FromServer,
    actor: ObjectId,
    is: impl Fn(&ServerZoneIpcData) -> bool,
) -> bool {
    matches!(message, FromServer::PacketSegment(segment, from) if *from == actor && is(&segment.data))
}

/// Whether a message is this ActorControl about `actor` to someone else (`ActorControl`).
pub fn control_from(
    message: &FromServer,
    actor: ObjectId,
    is: impl Fn(&ActorControlCategory) -> bool,
) -> bool {
    matches!(message, FromServer::ActorControl(from, category) if *from == actor && is(category))
}
