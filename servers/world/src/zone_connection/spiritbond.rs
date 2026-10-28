//! Gear wear on the connection: condition and spiritbond after a synthesis, a gather or completed content, and the
//! Materialize events that use them (materia extraction, self-repair with Dark Matter). The rules are
//! `crate::spiritbond`'s; docs/src/fork/spiritbond.md has the packets and where each number comes from.

use std::io::Cursor;

use binrw::BinRead;
use kawari::{
    common::{
        ContainerType, HandlerId, HandlerType, INVENTORY_ACTION_ACK_GENERAL, ITEM_CONDITION_MAX,
        ObjectTypeId, ObjectTypeKind,
    },
    ipc::zone::{
        ActorControlCategory, EventLogMessage, EventType, SceneFlags, ServerZoneIpcData,
        ServerZoneIpcSegment,
    },
};

use crate::{
    Event, EventHandler, ItemInfoQuery, ItemRow, ZoneConnection,
    inventory::{Item, Storage},
    lua::LuaPlayer,
    spiritbond::{self, ContentKind, SOUL_CRYSTAL_SLOT, SPIRITBOND_MAX},
};

/// An item's HQ flag (`Item::item_flags`.)
const ITEM_FLAG_HQ: u8 = 1;

/// The Materialize handlers (Item.MaterializeType's events; FFXIVClientStructs' EventHandler and the client's Repair
/// agent, which starts 0x390003 for self-repair.)
pub const MATERIA_EXTRACTION: u16 = 1;
pub const SELF_REPAIR: u16 = 3;

/// "Forging the Spirit", which unlocks materia extraction (Quest sheet row.)
const QUEST_FORGING_THE_SPIRIT: u32 = 66174;

/// LogMessage rows (their text is the client's.)
const LOG_SPIRITBOND_COMPLETE: u32 = 744; // [item]: retail sent it right after the piece's slot reached 100%
const LOG_EXTRACTED: u32 = 1200; // [materia, item]
const LOG_REPAIRED_ONE: u32 = 1385; // [item], as the mender's
const LOG_REPAIRED_SOME: u32 = 1388; // [how many]
const LOG_CANNOT_REPAIR: u32 = 1386; // [item]
const LOG_REPAIR_LEVEL: u32 = 1390;
const LOG_NO_DARK_MATTER: u32 = 1392;
const LOG_NO_REPAIR_NEEDED: u32 = 1394; // [item]
const LOG_NO_ROOM: u32 = 4160;

/// What wore the gear.
#[derive(Debug, Clone, Copy)]
pub enum GearUse {
    /// A synthesis ended (done or failed), making an item of this item level.
    Synthesis { crafted_item_level: u16 },
    /// A gathering attempt at a node of this level; spiritbond only if something was gathered.
    Gather {
        node_level: u8,
        rare: bool,
        gathered: bool,
    },
    /// Content was completed.
    Content {
        kind: ContentKind,
        content_level: u8,
    },
}

/// A materia extraction or self-repair asked for (ClientTrigger 2800), kept until the event's scene yields.
#[derive(Debug, Clone)]
pub struct MaterializeRequest {
    pub handler_id: HandlerId,
    /// The items it's for: one, or every one a self-repair's "repair all" covers.
    pub items: Vec<(ContainerType, u16, u32)>,
}

fn container_from(value: u32) -> Option<ContainerType> {
    ContainerType::read_le(&mut Cursor::new((value as u16).to_le_bytes())).ok()
}

fn melds(item: &Item) -> u32 {
    item.materia.iter().filter(|materia| **materia != 0).count() as u32
}

impl ZoneConnection {
    fn item_row(&self, item_id: u32) -> Option<ItemRow> {
        self.game_data
            .lock()
            .get_item_info(ItemInfoQuery::ById(item_id))
    }

    /// Wears what the player has on: condition lost and spiritbond gained, each changed slot sent again as retail did
    /// (one UpdateInventorySlot per piece, then LogMessage 744 for a piece reaching 100%.)
    pub async fn use_gear(&mut self, gear_use: GearUse) {
        let mut rng = fastrand::Rng::new();
        let sb_gain = self.base_parameters().increased_spiritbond_gain;
        let above_level = |connection: &Self, content_level: u8| {
            let level = connection.current_level(&connection.game_data.lock());
            connection.synced_level.is_some() || level > content_level as u16
        };
        let content_above = match gear_use {
            GearUse::Content { content_level, .. } => above_level(self, content_level),
            _ => false,
        };

        let mut changed = Vec::new();
        let mut bonded = Vec::new();
        let mut broke = false;
        let slots = self.player_data.inventory.equipped.max_slots() as u16;
        for slot in (0..slots).filter(|slot| *slot != SOUL_CRYSTAL_SLOT) {
            let item = *self.player_data.inventory.equipped.get_slot(slot);
            if item.is_empty_slot() {
                continue;
            }
            let Some(row) = self.item_row(item.item_id) else {
                continue;
            };
            let hq = item.item_flags & ITEM_FLAG_HQ != 0;
            let (wear, points) = match gear_use {
                GearUse::Synthesis { crafted_item_level } => (
                    spiritbond::synthesis_wear(slot, &mut rng),
                    spiritbond::points(
                        spiritbond::crafting_base(
                            row.item_level.saturating_sub(crafted_item_level),
                        ),
                        sb_gain,
                        melds(&item),
                        hq,
                        &mut rng,
                    ),
                ),
                GearUse::Gather {
                    node_level,
                    rare,
                    gathered,
                } => (
                    spiritbond::gather_wear(slot),
                    if gathered {
                        spiritbond::points(
                            spiritbond::gathering_base(
                                (row.equip_level as u16).saturating_sub(node_level as u16),
                                rare,
                            ),
                            sb_gain.min(spiritbond::gathering_sb_gain_cap(rare)),
                            melds(&item),
                            hq,
                            &mut rng,
                        )
                    } else {
                        0
                    },
                ),
                GearUse::Content {
                    kind,
                    content_level,
                } => (
                    spiritbond::content_wear(slot, kind, content_above, &mut rng),
                    spiritbond::points(
                        spiritbond::content_base(kind, content_level, row.equip_level),
                        sb_gain,
                        melds(&item),
                        hq,
                        &mut rng,
                    ),
                ),
            };

            // No spiritbond in a duty run as an unrestricted party without Level Sync (the gear still wears.)
            let points = if self.unsynced_unrestricted() {
                0
            } else {
                points
            };
            let slot_item = self.player_data.inventory.equipped.get_slot_mut(slot);
            let before = (slot_item.condition, slot_item.spiritbond_or_collectability);
            // Gear at 0% gains nothing (the wiki), and only gear that can be extracted from bonds.
            if slot_item.condition > 0
                && row.materialize_type != 0
                && slot_item.spiritbond_or_collectability < SPIRITBOND_MAX
            {
                slot_item.spiritbond_or_collectability =
                    (slot_item.spiritbond_or_collectability as u32 + points)
                        .min(SPIRITBOND_MAX as u32) as u16;
                if slot_item.spiritbond_or_collectability == SPIRITBOND_MAX {
                    bonded.push((slot, item.item_id));
                }
            }
            slot_item.condition = slot_item.condition.saturating_sub(wear);
            if slot_item.condition == 0 && before.0 > 0 {
                broke = true;
            }
            if (slot_item.condition, slot_item.spiritbond_or_collectability) != before {
                changed.push(slot);
            }
        }

        for slot in changed {
            self.send_inventory_slot(ContainerType::Equipped, slot)
                .await;
            if let Some((_, item_id)) = bonded.iter().find(|(s, _)| *s == slot) {
                self.actor_control_self(ActorControlCategory::LogMessage {
                    log_message: LOG_SPIRITBOND_COMPLETE,
                    id: *item_id,
                })
                .await;
            }
        }
        // A piece at 0% gives no stats.
        if broke {
            self.send_stats().await;
        }
    }

    /// Completed content (InstanceContent `content_id`) wears the gear: its kind from InstanceContentType (an eight-player
    /// raid without one party's roles is an alliance raid) and its level the duty's sync level, else the player's.
    pub async fn wear_gear_for_content(&mut self, content_id: u16) {
        use kawari::common::InstanceContentType;
        let cfc = self.content_finder_condition_id;
        let (kind, content_level) = {
            let mut game_data = self.game_data.lock();
            let kind = match game_data.find_type_for_content(content_id) {
                Some(InstanceContentType::Dungeon) => ContentKind::Dungeon,
                Some(InstanceContentType::Trial | InstanceContentType::UnrealTrial) => {
                    ContentKind::Trial
                }
                Some(InstanceContentType::Raid) => {
                    if game_data.content_party_roles(cfc).is_some() {
                        ContentKind::Raid
                    } else {
                        ContentKind::AllianceRaid
                    }
                }
                Some(InstanceContentType::TreasureHuntDungeon) => ContentKind::TreasureDungeon,
                _ => ContentKind::Other,
            };
            let level = game_data
                .find_content_synced_level(cfc)
                .unwrap_or_else(|| self.current_level(&game_data).min(u8::MAX as u16) as u8);
            (kind, level)
        };
        self.use_gear(GearUse::Content {
            kind,
            content_level,
        })
        .await;
    }

    /// A FATE's rewards came: one unit of wear, its spiritbond (`spiritbond::content_base`.)
    pub async fn wear_gear_for_fate(&mut self, fate_id: u32) {
        let level = {
            let mut game_data = self.game_data.lock();
            game_data
                .get_fate_max_level(fate_id)
                .unwrap_or_else(|| self.current_level(&game_data).min(u8::MAX as u16) as u8)
        };
        self.use_gear(GearUse::Content {
            kind: ContentKind::Fate,
            content_level: level,
        })
        .await;
    }

    /// The items a ClientTrigger 2800 names, for the handlers Kawari does: extraction `[container, slot, item]`, and
    /// self-repair `[(slot << 16) | 1, container, item]` (one item) or `[2, container, 0]` (all in a container.)
    fn materialize_items(
        &self,
        handler_id: HandlerId,
        param1: u32,
        param2: u32,
        param3: u32,
    ) -> Option<Vec<(ContainerType, u16, u32)>> {
        match handler_id.event_id() as u16 {
            MATERIA_EXTRACTION => Some(vec![(container_from(param1)?, param2 as u16, param3)]),
            SELF_REPAIR => match param1 & 0xFFFF {
                1 => Some(vec![(
                    container_from(param2)?,
                    (param1 >> 16) as u16,
                    param3,
                )]),
                2 => {
                    let container = container_from(param2)?;
                    let storage = self.player_data.inventory.get_container(container)?;
                    Some(
                        (0..storage.max_slots() as u16)
                            .map(|slot| (container, slot, storage.get_slot(slot).item_id))
                            .filter(|(_, _, item_id)| *item_id != 0)
                            .collect(),
                    )
                }
                _ => None,
            },
            _ => None,
        }
    }

    /// Whether this item can have materia extracted now; otherwise the LogMessage to refuse with, if there's one.
    fn check_extraction(
        &self,
        container: ContainerType,
        slot: u16,
        item_id: u32,
    ) -> Result<(), Option<u32>> {
        if !self.quest_completed(QUEST_FORGING_THE_SPIRIT) {
            return Err(None);
        }
        let item = self
            .player_data
            .inventory
            .get_item(container, slot)
            .filter(|item| !item.is_empty_slot() && item.item_id == item_id)
            .ok_or(None)?;
        if item.spiritbond_or_collectability < SPIRITBOND_MAX {
            return Err(None);
        }
        if self
            .item_row(item_id)
            .is_none_or(|row| row.materialize_type == 0)
        {
            return Err(None);
        }
        Ok(())
    }

    /// The Dark Matter in the bags that repairs this item (its own grade or a higher one, lowest first), or the
    /// LogMessage to refuse with.
    fn check_self_repair(
        &self,
        container: ContainerType,
        slot: u16,
        item_id: u32,
        dark_matter: &[u32],
    ) -> Result<u32, u32> {
        let item = self
            .player_data
            .inventory
            .get_item(container, slot)
            .filter(|item| !item.is_empty_slot() && item.item_id == item_id)
            .ok_or(LOG_CANNOT_REPAIR)?;
        let row = self.item_row(item_id).ok_or(LOG_CANNOT_REPAIR)?;
        if row.item_repair <= 0 || row.class_job_repair == 0 {
            return Err(LOG_CANNOT_REPAIR);
        }
        if item.condition >= ITEM_CONDITION_MAX {
            return Err(LOG_NO_REPAIR_NEEDED);
        }
        let level = self
            .level_for(row.class_job_repair as u16)
            .unwrap_or_default();
        if !spiritbond::can_self_repair(level, row.equip_level) {
            return Err(LOG_REPAIR_LEVEL);
        }
        let grade = row.item_repair as usize;
        dark_matter
            .iter()
            .skip(grade.saturating_sub(1))
            .copied()
            .find(|dark_matter| self.player_data.inventory.count_in_bags(*dark_matter) > 0)
            .ok_or(LOG_NO_DARK_MATTER)
    }

    /// ClientTrigger 2800: start the Materialize event, as retail started a desynthesis (Bob's Chronofoil capture of
    /// 2026-09-15): EventStart (type 0x19), condition 14, ActorControlSelf 138 with the handler, then scene 0 with flags
    /// 0x40000 and no params. The client plays it (an ActionRequest for the animation) and yields; the result is sent
    /// then (`finish_materialize`.) A request Kawari can't do still gets an EventFinish, as the client started the event
    /// on its side already.
    pub async fn begin_materialize(
        &mut self,
        handler_id: u32,
        params: [u32; 3],
        events: &mut Vec<(Box<dyn EventHandler>, Event)>,
        lua_player: &LuaPlayer,
    ) {
        let handler_id = HandlerId(handler_id);
        let refuse = |reason: &str| {
            tracing::info!(
                "Materialize {handler_id:?} {params:?} refused: {reason} (only extraction and self-repair are done)"
            );
        };
        let items = if handler_id.handler_type() == HandlerType::Materialize {
            self.materialize_items(handler_id, params[0], params[1], params[2])
        } else {
            None
        };
        let refusal = match &items {
            None => {
                refuse("not a request Kawari knows");
                Some(None)
            }
            Some(items) if handler_id.event_id() as u16 == MATERIA_EXTRACTION => {
                let (container, slot, item_id) = items[0];
                self.check_extraction(container, slot, item_id).err()
            }
            Some(items) => {
                let dark_matter = self.game_data.lock().dark_matter_items();
                let results: Vec<_> = items
                    .iter()
                    .map(|(c, s, i)| self.check_self_repair(*c, *s, *i, &dark_matter))
                    .collect();
                if results.iter().any(|result| result.is_ok()) {
                    None
                } else {
                    Some(results.first().and_then(|result| result.err()))
                }
            }
        };
        if let Some(log_message) = refusal {
            if let Some(log_message) = log_message {
                let item_id = items
                    .as_ref()
                    .and_then(|items| items.first())
                    .map(|item| item.2)
                    .unwrap_or_default();
                self.log_message(log_message, item_id).await;
            } else {
                refuse("its conditions aren't met");
            }
            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::EventFinish {
                handler_id,
                event_type: EventType::Materialize,
                result: 1,
                arg: 0,
            });
            self.send_ipc_self(ipc).await;
            self.send_conditions().await;
            return;
        }

        self.materialize = Some(MaterializeRequest {
            handler_id,
            items: items.unwrap_or_default(),
        });
        let actor_id = ObjectTypeId {
            object_id: self.player_data.character.actor_id,
            object_type: ObjectTypeKind::None,
        };
        if !self
            .start_event(
                actor_id,
                handler_id,
                EventType::Materialize,
                0,
                events,
                lua_player,
            )
            .await
        {
            self.materialize = None;
            return;
        }
        let Some((_, event)) = events.last() else {
            return;
        };
        let event = Event {
            id: event.id,
            event_type: event.event_type,
            event_arg: event.event_arg,
            condition: event.condition,
            actor_id: event.actor_id,
        };
        self.event_scene(&event, 0, SceneFlags::from_bits_retain(0x40000), Vec::new())
            .await;
    }

    /// The Materialize scene yielded: the extraction's materia or the repair, then the client returns the scene and the
    /// event finishes.
    pub async fn finish_materialize(&mut self, event: &Event) {
        let Some(request) = self.materialize.take() else {
            return;
        };
        if request.handler_id.event_id() as u16 == MATERIA_EXTRACTION {
            let (container, slot, item_id) = request.items[0];
            if self.check_extraction(container, slot, item_id).is_ok() {
                self.extract_materia(event, container, slot, item_id).await;
            }
        } else {
            self.self_repair(&request.items).await;
        }
    }

    /// Takes the item's spiritbond back to 0 and gives a materia of its role (`spiritbond::gear_role`) at the grade its
    /// level gives.
    async fn extract_materia(
        &mut self,
        event: &Event,
        container: ContainerType,
        slot: u16,
        item_id: u32,
    ) {
        let Some(row) = self.item_row(item_id) else {
            return;
        };
        let classjob = self.player_data.classjob.current_class as u8;
        let Some(role) = spiritbond::gear_role(row.materialize_type, classjob) else {
            return;
        };
        let mut rng = fastrand::Rng::new();
        let params = spiritbond::materia_params(role);
        let base_param = params[rng.usize(0..params.len())];
        let grade = spiritbond::materia_grade(row.equip_level, row.item_level, &mut rng);
        let materia = {
            let mut game_data = self.game_data.lock();
            game_data
                .materia_item(base_param, grade)
                .and_then(|materia| game_data.get_item_info(ItemInfoQuery::ById(materia)))
        };
        let Some(materia) = materia else {
            tracing::warn!("No materia of BaseParam {base_param} at grade index {grade}");
            return;
        };
        if !self
            .player_data
            .inventory
            .add_obtained_item(Item::new(&materia, 1))
        {
            self.log_message(LOG_NO_ROOM, 0).await;
            return;
        }
        if let Some(item) = self.player_data.inventory.get_item_mut(container, slot) {
            item.spiritbond_or_collectability = 0;
        }
        self.send_inventory_slot(container, slot).await;
        self.send_inventory_ack(u32::MAX, INVENTORY_ACTION_ACK_GENERAL, 0)
            .await;
        self.send_inventory().await;
        let message = EventLogMessage {
            handler_id: event.id,
            message_type: LOG_EXTRACTED,
            params: vec![materia.id, item_id],
        };
        if let Some(ipc) = message.package() {
            self.send_ipc_self(ipc).await;
        }
    }

    /// Repairs each item that can be (to 100%), using up one Dark Matter each.
    async fn self_repair(&mut self, items: &[(ContainerType, u16, u32)]) {
        let dark_matter = self.game_data.lock().dark_matter_items();
        let mut repaired = Vec::new();
        for &(container, slot, item_id) in items {
            let Ok(used) = self.check_self_repair(container, slot, item_id, &dark_matter) else {
                continue;
            };
            if !self.player_data.inventory.use_up_one(used) {
                continue;
            }
            if let Some(item) = self.player_data.inventory.get_item_mut(container, slot) {
                item.condition = ITEM_CONDITION_MAX;
            }
            self.send_inventory_slot(container, slot).await;
            repaired.push((container, item_id));
        }
        match repaired.as_slice() {
            [] => return,
            [(_, item_id)] => self.log_message(LOG_REPAIRED_ONE, *item_id).await,
            _ => {
                self.log_message(LOG_REPAIRED_SOME, repaired.len() as u32)
                    .await
            }
        }
        self.send_inventory().await;
        if repaired
            .iter()
            .any(|(container, _)| *container == ContainerType::Equipped)
        {
            self.send_stats().await;
        }
    }
}
