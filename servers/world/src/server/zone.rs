use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use glam::{Affine3A, EulerRot, Vec3A};
use parking_lot::Mutex;
use physis::{
    layer::{
        ExitRangeInstanceObject, InstanceObject, LayerEntryData, PopRangeInstanceObject,
        ServerPathInstanceObject, TriggerBoxShape,
    },
    lgb::Lgb,
    lvb::Lvb,
};

use crate::{
    ClientId, FromServer, GameData, StatusEffects, TerritoryNameKind, ToServer,
    lua::{KawariLua, KawariLuaState, LuaZone},
    server::{
        NetworkedActor, WorldServer,
        actor::{create_npc_common_spawn, field_enemy_base_hp, set_character_mode},
        fate::inform_fate_spawn,
        instance::{Instance, QueuedTaskData, remove_actor_from_instance},
        job_gauge::JobGauge,
        network::{DestinationNetwork, NetworkState},
        party_bot, party_npc,
    },
    zone_connection::{BaseParameters, TeleportQuery},
};
use kawari::{
    common::{
        CharacterMode, DistanceRange, DropIn, DropInLayer, DropInObjectData, ENTRANCE_CIRCLE_IDS,
        EOBJ_EXIT, EOBJ_HOUSING_ENTRANCE, EOBJ_SHORTCUT, EOBJ_SHORTCUT_EXPLORER_MODE, EventState,
        GimmickAccessorType, GimmickRectTriggerType, HandlerType, ObjectId, Position,
        TerritoryIntendedUse, WARP_DELAY, WarpType, euler_to_direction, internal_housing_row,
    },
    config::get_config,
    ipc::zone::{
        ActorControlCategory, ActorSetPos, CharacterDataFlag, CommonSpawn, Conditions, DisplayFlag,
        ObjectKind, PrepareZoning, PrepareZoningFlag, ServerZoneIpcData, ServerZoneIpcSegment,
        SpawnNpc, SpawnObject, SpawnTreasure,
    },
};

#[derive(Debug)]
pub enum MapGimmick {
    /// Seen for final boss triggers in Sastasha
    Generic,
    /// Jump pads like the ones in Gold Saucer.
    Jump {
        /// The position to land on.
        to_position: Vec3A,
        /// The GimmickJump type.
        gimmick_jump_type: u32,
        /// The animation ID to play for the EObj.
        sgb_animation_id: u32,
        /// The EObj's instance ID to play the animation for.
        eobj_instance_id: u32,
    },
    /// Unsure of what to call these, but these are "exit lines" like as seen in the overworld but go to another poprange in the same zone.
    /// Used heavily in instanced content.
    FakeExit { exit_pop_range_id: u32 },
}

/// Simpler form of a MapRange object designed for collision detection.
#[derive(Debug, Default)]
pub struct MapRange {
    /// Trigger box shape.
    pub trigger_box_shape: TriggerBoxShape,
    /// Position of this range in the world.
    pub position: Vec3A,
    /// Relative scale of this range.
    pub scale: Vec3A,
    /// Whether this map range represents a sanctuary.
    pub sanctuary: bool,
    /// Whether this map range represents a PvP duel area.
    pub duel: bool,
    /// Whether this map range represents a gimmick, like a jumping pad.
    pub gimmick: Option<MapGimmick>,
    /// Game Object ID. Also known as the layout ID. The client sends this when discovering new areas.
    pub instance_id: u32,
    /// The MapRange's discovery index. Unclear if this is the same as DiscoveryIndex on the Map sheet.
    pub discovery_id: Option<u8>,
    /// Whether this map range represents an instance exit.
    pub entrance: bool,
    /// What FATE this MapRange is associated with, if any.
    pub fate: Option<u32>,
}

#[derive(Debug)]
struct HousingPlot {
    entrance_position: Vec3A,
}

/// Represents a loaded zone
#[derive(Debug, Default)]
pub struct Zone {
    pub id: u16,
    pub internal_name: String,
    pub region_name: String,
    pub place_name: String,
    pub intended_use: TerritoryIntendedUse,
    pub layer_groups: Vec<Lgb>,
    pub navimesh_path: String,
    pub map_id: u16,
    cached_npc_base_ids: HashMap<ObjectId, u32>,
    pub map_ranges: Vec<MapRange>,
    dropin_layers: Vec<DropInLayer>,
    cached_objects: HashMap<u32, SpawnObject>,
    cached_npcs: HashMap<u32, SpawnNpc>,
    cached_treasure: HashMap<u32, SpawnTreasure>,
    layer_set: i32,
    bg_path: String,
    cached_housing_plots: Vec<HousingPlot>,
    cached_eobj_base_ids: HashMap<u32, u32>,
    pub cached_pop_ranges: HashMap<u32, (Position, f32)>,
    pub cached_paths: HashMap<u32, ServerPathInstanceObject>,
    /// Layout IDs of the drop-ins' battle NPCs that stand where they are instead of wandering.
    pub stationary_npcs: HashSet<u32>,
    /// The drop-ins' unspoiled, legendary and ephemeral gathering points, spawned while they're up
    /// (`Instance::update_timed_gathering_points`.)
    pub timed_gathering_points: Vec<TimedGatheringPoint>,
    /// The duty's exits (EObj IDs), hidden until it's complete (`get_event_objects`): every event object whose gimmick
    /// is the duty exit (GimmickAccessor of the DutyExit kind), not only 2000139, which most dungeons use; the Thousand
    /// Maws of Toto-Rak's is 2000683 (its EObj Data the same GimmickAccessor, 5; retail spawned it hidden and showed it
    /// as the duty was complete, Bob's Chronofoil captures caa5e908 and 1fac2416 of 2026-10-30.)
    pub exit_eobjs: Vec<u32>,
}

/// Whether this event object's gimmick is the duty exit (its EObj Data a GimmickAccessor of the DutyExit kind.)
fn is_duty_exit(game_data: &mut GameData, base_id: u32) -> bool {
    if base_id == EOBJ_EXIT {
        return true;
    }
    let data = game_data.get_eobj_data(base_id);
    data >> 16 == HandlerType::GimmickAccessor as u32
        && game_data
            .lookup_gimmick_accessor(data & 0xFFFF)
            .is_some_and(|(kind, ..)| matches!(kind, GimmickAccessorType::DutyExit))
}

/// A gathering point that's only up at certain Eorzea times.
#[derive(Debug, Clone)]
pub struct TimedGatheringPoint {
    pub spawn: SpawnObject,
    /// When it's up, in Eorzea minutes of the day (start, end.)
    pub windows: Vec<(u32, u32)>,
}

impl Zone {
    pub fn load(game_data: &mut GameData, id: u16) -> Self {
        let mut zone = Self {
            id,
            ..Default::default()
        };

        let Some(row) = game_data.territory_type_sheet.row(id as u32) else {
            tracing::warn!("Invalid zone id {id}, allowing anyway...");
            return zone;
        };

        if let Some(intended_use) = TerritoryIntendedUse::from_repr(row.TerritoryIntendedUse) {
            zone.intended_use = intended_use;
        } else {
            tracing::warn!(
                "Unknown TerritoryIntendedUse {}, certain featureswill not function. Please report this as a bug.",
                row.TerritoryIntendedUse
            );
        }
        zone.map_id = row.Map;

        // e.g. ffxiv/fst_f1/fld/f1f3/level/f1f3
        let bg_path = row.Bg;
        if bg_path.is_empty() {
            tracing::warn!("Bg path is empty for zone id {id}, allowing anyway...");
            return zone;
        }

        let path = format!("bg/{}.lvb", bg_path);
        if let Ok(lvb) = game_data.resource.parsed::<Lvb>(&path) {
            let mut load_lgb = |path: &str| -> Option<Lgb> {
                // Skip LGBs that aren't relevant for the server
                if path.ends_with("bg.lgb")
                    || path.ends_with("vfx.lgb")
                    || path.ends_with("sound.lgb")
                {
                    return None;
                }

                let lgb = game_data.resource.parsed::<Lgb>(path);
                if let Err(e) = &lgb {
                    tracing::warn!(
                        "Failed to parse {path}: {e}, this is most likely a bug in Physis and should be reported somewhere!"
                    )
                }

                lgb.ok()
            };

            for path in &lvb.sections[0].lgb_paths {
                if let Some(lgb) = load_lgb(path) {
                    zone.layer_groups.push(lgb);
                }
            }

            for layer_set in &lvb.sections[0].layer_sets.layer_sets {
                if layer_set.territory_type_id == id {
                    zone.layer_set = layer_set.id;
                    zone.navimesh_path = layer_set
                        .nvm_path
                        .value
                        .replace("/server/data/", "")
                        .to_string();

                    break;
                }
            }
            log_layer_filters(
                id,
                &lvb.sections[0].layer_sets.layer_sets,
                &zone.layer_groups,
            );

            let mut search_dirs: Vec<String> = get_config()
                .filesystem
                .additional_resource_paths
                .iter()
                .cloned()
                .map(|mut x| {
                    x.push_str("/dropins/");
                    x
                })
                .collect();
            search_dirs.push("resources/dropins/".to_string());

            // Every drop-in for this zone in the first folder that has any (a zone can have several, e.g. Middle La
            // Noscea's gathering points and its battle NPCs; it used to stop at the first, so only one of them loaded.)
            for search_dir in search_dirs {
                let Ok(search_dir) = std::fs::read_dir(search_dir) else {
                    continue;
                };
                let mut entries: Vec<_> = search_dir.flatten().collect();
                entries.sort_by_key(|entry| entry.file_name());
                let mut loaded_any = false;

                // Load drop-ins
                for entry in entries {
                    if !entry.file_name().to_str().unwrap().ends_with(".json") {
                        continue;
                    }

                    if let Ok(contents) = std::fs::read_to_string(entry.path()) {
                        match serde_json::from_str::<DropIn>(&contents) {
                            Ok(mut dropin) => {
                                if lvb.sections[0].lgb_paths.contains(&dropin.appends) {
                                    tracing::info!("Loaded dropin from {:?}", entry.path());
                                    zone.dropin_layers.append(&mut dropin.layers);
                                    loaded_any = true;
                                }
                            }
                            Err(err) => {
                                tracing::warn!("Failed to load drop-in {:?}: {err:?}", entry.path())
                            }
                        }
                    }
                }
                if loaded_any {
                    break;
                }
            }

            zone.bg_path = lvb.sections[0].general.bg_path.value.clone();
        }

        // create NPC ID cache
        for layer_group in &zone.layer_groups {
            for chunk in &layer_group.chunks {
                for layer in &chunk.layers {
                    // NOTE: We don't check layer set here because it fails pop ranges for Moonfire Faire?

                    for object in &layer.objects {
                        let (scale, rotation, translation) =
                            Affine3A::from(object.transform).to_scale_rotation_translation();

                        if let LayerEntryData::EventNPC(npc) = &object.data {
                            zone.cached_npc_base_ids.insert(
                                ObjectId(object.instance_id),
                                npc.parent_data.parent_data.base_id,
                            );
                        }
                        // Aetherytes and shards are part of the map (the server doesn't spawn them), and the client
                        // talks to them by layout ID; their base ID is the Aetheryte row, so quests can check
                        // `BASE_ID == AETHERYTE0` (Close to Home in Ul'dah.)
                        if let LayerEntryData::Aetheryte(aetheryte) = &object.data {
                            zone.cached_eobj_base_ids
                                .insert(object.instance_id, aetheryte.parent_data.base_id);
                        }
                        if let LayerEntryData::MapRange(map_range) = &object.data {
                            zone.map_ranges.push(MapRange {
                                trigger_box_shape: map_range.parent_data.trigger_box_shape,
                                position: translation.into(),
                                scale: scale.into(),
                                sanctuary: map_range.rest_bonus_enabled,
                                instance_id: object.instance_id,
                                discovery_id: if map_range.discovery_enabled {
                                    Some(map_range.discovery_id)
                                } else {
                                    None
                                },
                                ..Default::default()
                            });
                        }
                        if let LayerEntryData::EventRange(event_range) = &object.data {
                            zone.map_ranges.push(MapRange {
                                trigger_box_shape: event_range.parent_data.trigger_box_shape,
                                position: translation.into(),
                                scale: scale.into(),
                                duel: object.instance_id == 6445254, // From the LVD_duel_01 layer in Wolves Den Pier
                                instance_id: object.instance_id,
                                fate: game_data.find_fate_by_event_range(object.instance_id),
                                ..Default::default()
                            });
                        }
                        if let LayerEntryData::PopRange(_) = &object.data {
                            zone.cached_pop_ranges.insert(
                                object.instance_id,
                                (
                                    Position(translation.into()),
                                    euler_to_direction(rotation.to_euler(EulerRot::XYZ)),
                                ),
                            );
                        }
                    }

                    // Second pass for eobjs
                    for object in &layer.objects {
                        if !layer.header.has_layer_set(zone.layer_set as u32) {
                            continue;
                        }

                        if let LayerEntryData::EventObject(eobj) = &object.data {
                            let eobj_data = game_data.get_eobj_data(eobj.parent_data.base_id);
                            let event_type = HandlerType::from_repr(eobj_data >> 16);

                            if let Some(HandlerType::GimmickRect) = event_type {
                                // GimmickRects are used for stuff like the Golden Saucer jumping pads, and is handled server-side.
                                // Thus, we need to go through and mark these MapRanges to play said event.
                                if let Some(gimmick_rect_info) =
                                    game_data.get_gimmick_rect_info(eobj_data & 0xFFFF)
                                    && let Some(target_pop_range) =
                                        zone.find_pop_range(gimmick_rect_info.Params[1])
                                {
                                    let gimmick_jump_type = gimmick_rect_info.Params[0];
                                    let target_event_range = gimmick_rect_info.LayoutID;
                                    let sgb_animation_id = gimmick_rect_info.Params[2];

                                    // 8 seems to indicate a jumping pad
                                    if GimmickRectTriggerType::from_repr(
                                        gimmick_rect_info.TriggerIn,
                                    )
                                    .unwrap()
                                        == GimmickRectTriggerType::Jump
                                    {
                                        let (_, _, translation) =
                                            Affine3A::from(target_pop_range.0.transform)
                                                .to_scale_rotation_translation();

                                        let map_gimmick = MapGimmick::Jump {
                                            to_position: translation.into(),
                                            gimmick_jump_type,
                                            sgb_animation_id,
                                            eobj_instance_id: object.instance_id,
                                        };

                                        for map_range in &mut zone.map_ranges {
                                            if map_range.instance_id == target_event_range {
                                                map_range.gimmick = Some(map_gimmick);
                                                break;
                                            }
                                        }
                                    } else {
                                        tracing::warn!(
                                            "Unsupported Gimmick trigger for EventObj {}",
                                            gimmick_rect_info.TriggerIn
                                        );
                                    }
                                } else {
                                    tracing::warn!(
                                        "Failed to lookup Gimmick {}?!",
                                        eobj_data & 0xFFFF
                                    );
                                }
                            }
                        } else if let LayerEntryData::EventRange(_) = &object.data
                            && let Some(gimmick_rect_info) =
                                game_data.lookup_gimmick_rect(object.instance_id)
                        {
                            let mut map_gimmick = None;
                            match GimmickRectTriggerType::from_repr(gimmick_rect_info.TriggerIn)
                                .unwrap()
                            {
                                GimmickRectTriggerType::Cutscene
                                | GimmickRectTriggerType::VVDRouteTrigger => {
                                    // FIXME: We should make this less "generic".
                                    map_gimmick = Some(MapGimmick::Generic {});
                                }
                                GimmickRectTriggerType::FakeExit => {
                                    // Seen for same-zone "exit ranges" like the one in the beginning of Sycrus Tower
                                    map_gimmick = Some(MapGimmick::FakeExit {
                                        exit_pop_range_id: gimmick_rect_info.Params[0],
                                    });
                                }
                                GimmickRectTriggerType::FaithTalk => {
                                    // TODO: not sure what the param for this one means. It obviously plugs into NpcYell somehow
                                }
                                _ => tracing::warn!(
                                    "Unknown GimmickRect type: {} for EventRange {}",
                                    gimmick_rect_info.TriggerIn,
                                    object.instance_id
                                ),
                            }

                            for map_range in &mut zone.map_ranges {
                                if map_range.instance_id == object.instance_id {
                                    map_range.gimmick = map_gimmick;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        // load names
        let fallback = "<Unable to load name!>";
        zone.internal_name = game_data
            .get_territory_name(id as u32, TerritoryNameKind::Internal)
            .unwrap_or(fallback.to_string());
        zone.region_name = game_data
            .get_territory_name(id as u32, TerritoryNameKind::Region)
            .unwrap_or(fallback.to_string());
        zone.place_name = game_data
            .get_territory_name(id as u32, TerritoryNameKind::Place)
            .unwrap_or(fallback.to_string());

        // create housing plot cache
        if zone.intended_use == TerritoryIntendedUse::HousingOutdoor {
            let land_sets = game_data
                .get_land_sets(internal_housing_row(id).unwrap())
                .unwrap();
            for land_set in land_sets {
                let map_ranges: Vec<&MapRange> = zone
                    .map_ranges
                    .iter()
                    .filter(|x| x.instance_id == land_set.MapRange)
                    .collect();
                if map_ranges.is_empty() {
                    tracing::warn!(
                        "Failed to find map range for a plot! The entrance won't spawn!"
                    );
                } else {
                    let map_range = map_ranges.first().unwrap();

                    zone.cached_housing_plots.push(HousingPlot {
                        entrance_position: map_range.position,
                    });
                }
            }
        }

        // create path cache
        for layer in &zone.dropin_layers {
            for object in &layer.objects {
                if let DropInObjectData::ServerPath(server_path) = &object.data {
                    zone.cached_paths
                        .insert(object.layout_id, server_path.clone());
                }
            }
        }

        zone
    }

    /// Search for an exit box matching an id.
    pub fn find_exit_box(
        &self,
        instance_id: u32,
    ) -> Option<(&InstanceObject, &ExitRangeInstanceObject)> {
        for layer_group in &self.layer_groups {
            for layer in &layer_group.chunks[0].layers {
                if !layer.header.has_layer_set(self.layer_set as u32) {
                    continue;
                }

                for object in &layer.objects {
                    if let LayerEntryData::ExitRange(exit_range) = &object.data
                        && object.instance_id == instance_id
                    {
                        return Some((object, exit_range));
                    }
                }
            }
        }

        None
    }

    /// Any layout object by its instance ID, in the current layer set.
    pub fn find_layout_object(&self, instance_id: u32) -> Option<&InstanceObject> {
        self.layer_groups
            .iter()
            .flat_map(|layer_group| layer_group.chunks[0].layers.iter())
            .filter(|layer| layer.header.has_layer_set(self.layer_set as u32))
            .flat_map(|layer| layer.objects.iter())
            .find(|object| object.instance_id == instance_id)
    }

    pub fn find_pop_range(
        &self,
        instance_id: u32,
    ) -> Option<(&InstanceObject, &PopRangeInstanceObject)> {
        for layer_group in &self.layer_groups {
            for layer in &layer_group.chunks[0].layers {
                if !layer.header.has_layer_set(self.layer_set as u32) {
                    continue;
                }

                for object in &layer.objects {
                    if let LayerEntryData::PopRange(pop_range) = &object.data
                        && object.instance_id == instance_id
                    {
                        return Some((object, pop_range));
                    }
                }
            }
        }

        None
    }

    pub fn to_lua_zone(&self, weather_id: u16) -> LuaZone {
        LuaZone {
            zone_id: self.id,
            weather_id,
            internal_name: self.internal_name.clone(),
            region_name: self.region_name.clone(),
            place_name: self.place_name.clone(),
            intended_use: self.intended_use,
            map_id: self.map_id,
            cached_npc_base_ids: self.cached_npc_base_ids.clone(),
            cached_eobj_base_ids: self.cached_eobj_base_ids.clone(),
            cached_pop_ranges: self.cached_pop_ranges.clone(),
            ..Default::default()
        }
    }

    fn find_entrance_from_base_id(&self, base_id: u32) -> Option<&InstanceObject> {
        // First, we need to find the EventObject for the entrance:
        let mut bound_id = None;
        for layer_group in &self.layer_groups {
            for layer in &layer_group.chunks[0].layers {
                if !layer.header.has_layer_set(self.layer_set as u32) {
                    continue;
                }

                for object in &layer.objects {
                    if let LayerEntryData::EventObject(eobj) = &object.data
                        && eobj.parent_data.base_id == base_id
                    {
                        bound_id = Some(eobj.bound_instance_id);
                        break;
                    }
                }
            }
        }

        bound_id?;

        // Then find the linked instance object, which is usually a SGB.
        for layer_group in &self.layer_groups {
            for layer in &layer_group.chunks[0].layers {
                if !layer.header.has_layer_set(self.layer_set as u32) {
                    continue;
                }

                for object in &layer.objects {
                    if object.instance_id == bound_id.unwrap() {
                        return Some(object);
                    }
                }
            }
        }

        None
    }

    /// Tries to locate the entrance circle used in instanced content.
    pub fn find_entrance(&self) -> Option<&InstanceObject> {
        for base_id in ENTRANCE_CIRCLE_IDS {
            if let Some(object) = self.find_entrance_from_base_id(base_id) {
                return Some(object);
            }
        }

        None
    }

    /// Where a retainer called at the summoning bell with this layout ID stands: the pop range the bell is bound to
    /// (its EventObject's bound instance), or failing that the pop range nearest the bell; one of its points, as
    /// retail did (docs/src/fork/retainers.md). The pop range's layout ID, the spot, and how it was found.
    pub fn retainer_spot(&self, bell_id: u32) -> Option<(u32, Position, &'static str)> {
        let mut bell = None;
        let mut pop_ranges = Vec::new();
        for layer_group in &self.layer_groups {
            for layer in &layer_group.chunks[0].layers {
                if !layer.header.has_layer_set(self.layer_set as u32) {
                    continue;
                }
                for object in &layer.objects {
                    match &object.data {
                        LayerEntryData::EventObject(eobj) if object.instance_id == bell_id => {
                            bell = Some((
                                Vec3A::from_array(object.transform.translation),
                                eobj.bound_instance_id,
                            ));
                        }
                        LayerEntryData::PopRange(_) => pop_ranges.push(object),
                        _ => {}
                    }
                }
            }
        }

        let (bell_position, bound) = bell?;
        if let Some(object) = pop_ranges
            .iter()
            .find(|object| bound != 0 && object.instance_id == bound)
        {
            return Some((
                object.instance_id,
                pick_point_in_pop_range(object),
                "bound to the bell",
            ));
        }
        pop_ranges
            .iter()
            .map(|object| {
                let distance =
                    Vec3A::from_array(object.transform.translation).distance(bell_position);
                (distance, object)
            })
            .filter(|(distance, _)| *distance <= RETAINER_SPOT_SEARCH_DISTANCE)
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, object)| {
                (
                    object.instance_id,
                    pick_point_in_pop_range(object),
                    "nearest the bell",
                )
            })
    }

    /// Returns a random pop range in this zone.
    pub fn find_random_pop_range(&self) -> Option<&InstanceObject> {
        let object_id = fastrand::choice(&self.cached_pop_ranges)?.0;
        self.find_pop_range(*object_id).map(|x| x.0)
    }

    /// Returns a list of event objects to spawn by default. If `explorer_mode`, replaces the shortcut object.
    ///
    /// For example, the Gold Saucer arcade machines or shortcuts in dungeons.
    pub fn get_event_objects(
        &mut self,
        game_data: &mut GameData,
        explorer_mode: bool,
    ) -> Vec<(SpawnObject, String)> {
        let mut object_spawns = Vec::new();

        for layer_group in &self.layer_groups {
            for layer in &layer_group.chunks[0].layers {
                if !layer.header.has_layer_set(self.layer_set as u32) {
                    continue;
                }

                for object in &layer.objects {
                    let (_, rotation, translation) =
                        Affine3A::from(object.transform).to_scale_rotation_translation();

                    if let LayerEntryData::EventObject(eobj) = &object.data {
                        let base_id = if eobj.parent_data.base_id == EOBJ_SHORTCUT && explorer_mode
                        {
                            EOBJ_SHORTCUT_EXPLORER_MODE
                        } else {
                            eobj.parent_data.base_id
                        };

                        // Hide shortcuts and exits, these are spawned later by the director.
                        let exit = is_duty_exit(game_data, eobj.parent_data.base_id);
                        if exit && !self.exit_eobjs.contains(&eobj.parent_data.base_id) {
                            self.exit_eobjs.push(eobj.parent_data.base_id);
                        }
                        let event_state = if (eobj.parent_data.base_id == EOBJ_SHORTCUT
                            && !explorer_mode)
                            || exit
                        {
                            EventState::OFF | EventState::UNK2 | EventState::UNK3
                        } else if explorer_mode {
                            // HACK: disable all walls in Explorer Mode
                            EventState::OFF | EventState::UNK2 | EventState::UNK3
                        } else {
                            EventState::empty()
                        };

                        let (pop_type, targetable) =
                            game_data.get_eobj_extra_data(eobj.parent_data.base_id);

                        let spawn = SpawnObject {
                            kind: ObjectKind::EventObj,
                            base_id,
                            not_targetable: !targetable,
                            event_state,
                            entity_id: ObjectId(fastrand::u32(..)),
                            layout_id: object.instance_id,
                            bind_layout_id: eobj.bound_instance_id,
                            radius: object.transform.scale[0],
                            rotation: euler_to_direction(rotation.to_euler(EulerRot::XYZ)),
                            position: Position(translation.into()),
                            ..Default::default()
                        };
                        self.cached_objects.insert(eobj.parent_data.base_id, spawn);
                        // NOTE: I think the use of layout_id here is intentional(?) Try the Grand Company Barracks entrance.
                        self.cached_eobj_base_ids
                            .insert(spawn.layout_id, spawn.base_id);

                        if pop_type == 1 {
                            object_spawns.push((spawn, layer.header.name.value.clone()));
                        }
                    }

                    if let LayerEntryData::Treasure(treasure) = &object.data
                        && !explorer_mode
                    {
                        self.cached_treasure.insert(
                            treasure.parent_data.base_id,
                            SpawnTreasure {
                                base_id: treasure.parent_data.base_id,
                                entity_id: ObjectId(fastrand::u32(..)),
                                layout_id: object.instance_id,
                                rotation: euler_to_direction(rotation.to_euler(EulerRot::XYZ)),
                                position: Position(translation.into()),
                                ..Default::default()
                            },
                        );
                    }
                }
            }
        }

        // Only dropins are checked for gathering points, because they strip that from retail LGBs.
        object_spawns.extend(self.gathering_point_spawns(game_data));

        // housing plot entrances
        for (i, plot) in self.cached_housing_plots.iter().enumerate() {
            let spawn = SpawnObject {
                kind: ObjectKind::EventObj,
                base_id: EOBJ_HOUSING_ENTRANCE,
                entity_id: ObjectId(fastrand::u32(..)),
                radius: 1.0,
                position: Position(plot.entrance_position),
                args2: u32::from_le_bytes([0, i as u8, 0, 0]),
                ..Default::default()
            };
            object_spawns.push((spawn, String::default()));
        }

        object_spawns
    }

    /// The drop-ins' gathering points that are always up, and (in `timed_gathering_points`) the timed ones. As retail
    /// (Bob's Chronofoil captures), every position of a point is sent; the client shows the one its phase table picks
    /// (`args1`: integrity, phase slot, this position's index, how many positions; 7.57 `0x140E8E8F0`.) A layout in
    /// two drop-ins is spawned once, from the one with retail's arguments; points without them get theirs worked out
    /// here (positions in descending layout ID order, as retail numbered them, and a free phase slot.)
    fn gathering_point_spawns(&mut self, game_data: &mut GameData) -> Vec<(SpawnObject, String)> {
        let mut points: Vec<(u32, u32, Position, Option<u32>)> = Vec::new(); // layout, point, position, args
        for layer in &self.dropin_layers {
            for object in &layer.objects {
                if let DropInObjectData::GatheringPoint {
                    base_id,
                    spawn_args,
                } = object.data
                {
                    match points.iter_mut().find(|p| p.0 == object.layout_id) {
                        Some(existing) => {
                            if existing.3.is_none() {
                                *existing =
                                    (object.layout_id, base_id, object.position, spawn_args);
                            }
                        }
                        None => {
                            points.push((object.layout_id, base_id, object.position, spawn_args))
                        }
                    }
                }
            }
        }

        let used_slots: HashSet<u8> = points
            .iter()
            .filter_map(|p| p.3.map(|args| args.to_le_bytes()[1]))
            .collect();
        let mut free_slots = (1..crate::gathering::PHASE_SLOTS as u8)
            .rev()
            .filter(|slot| !used_slots.contains(slot));
        let mut derived: HashMap<u32, (u8, Vec<u32>)> = HashMap::new(); // point -> slot, its layouts
        for (layout, point, _, args) in &points {
            if args.is_none() {
                derived
                    .entry(*point)
                    .or_insert_with(|| (free_slots.next().unwrap_or(0), Vec::new()))
                    .1
                    .push(*layout);
            }
        }

        let mut spawns = Vec::new();
        self.timed_gathering_points.clear();
        for (layout, point, position, args) in points {
            let (_, _, count, _, point_type) = game_data.get_gathering_point(point);
            // Levequest points only appear for their leve.
            if point_type == 3 {
                continue;
            }
            let args1 = args.unwrap_or_else(|| {
                let (slot, mut layouts) = derived[&point].clone();
                layouts.sort_unstable_by(|a, b| b.cmp(a));
                let index = layouts.iter().position(|l| *l == layout).unwrap_or(0);
                u32::from_le_bytes([count, slot, index as u8, layouts.len().min(3) as u8])
            });
            let spawn = SpawnObject {
                kind: ObjectKind::GatheringPoint,
                base_id: point,
                entity_id: ObjectId(fastrand::u32(..)),
                layout_id: layout,
                radius: 1.0,
                args1,
                position,
                ..Default::default()
            };
            let windows = game_data.get_gathering_point_windows(point);
            if !windows.is_empty() {
                self.timed_gathering_points
                    .push(TimedGatheringPoint { spawn, windows });
                continue;
            }
            self.cached_objects.insert(point, spawn);
            spawns.push((spawn, String::default()));
        }
        spawns
    }

    /// Returns an SpawnObject for the given base ID.
    pub fn get_event_object(&self, base_id: u32) -> Option<SpawnObject> {
        self.cached_objects.get(&base_id).cloned()
    }

    /// The event object (of the zone's layout, spawned or the client's own) nearest `position` on the ground, within
    /// `radius` yalms of it, for something thrown or placed there (a ground-targeted key item, `action::
    /// event_item_on_ground_landed`.)
    pub fn event_object_near(&self, position: Vec3A, radius: f32) -> Option<SpawnObject> {
        self.cached_objects
            .values()
            .filter(|object| object.kind == ObjectKind::EventObj)
            .map(|object| {
                let offset = object.position.0 - position;
                (object, offset.x.hypot(offset.z))
            })
            .filter(|(_, distance)| *distance <= radius)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(object, _)| *object)
    }

    /// Returns an SpawnNpc for the given instance ID.
    pub fn get_battle_npc(&self, instance_id: u32) -> Option<SpawnNpc> {
        self.cached_npcs.get(&instance_id).cloned()
    }

    /// Returns an SpawnTreasure for the given base ID.
    pub fn get_treasure(&self, base_id: u32) -> Option<SpawnTreasure> {
        self.cached_treasure.get(&base_id).cloned()
    }

    /// Returns a list of battle NPCs to spawn.
    pub fn get_npcs(&mut self, game_data: &mut GameData) -> Vec<SpawnNpc> {
        let mut npc_spawns = Vec::new();

        // Only dropins are checked for battle npcs, because they strip that from retail LGBs.
        for layer in &self.dropin_layers {
            for object in &layer.objects {
                if let DropInObjectData::BattleNpc {
                    base_id,
                    name_id,
                    hp,
                    level,
                    nonpop,
                    hostile,
                    gimmick_id,
                    max_links,
                    link_family,
                    link_range,
                    display_flags,
                    stationary,
                    character_data_flags,
                    behavior,
                    ref appearance,
                } = object.data
                {
                    if stationary {
                        self.stationary_npcs.insert(object.layout_id);
                    }
                    // Without a set HP, an ordinary enemy of its zone's expansion at its level.
                    let hp = hp.or_else(|| {
                        let ex_version = game_data.get_ex_version(self.id).unwrap_or(0);
                        field_enemy_base_hp(ex_version, level as u8)
                    });
                    let base_npc = create_npc_common_spawn(game_data, base_id, name_id, hp, level);
                    let mut spawn = SpawnNpc {
                        gimmick_id,
                        character_data_flags: match character_data_flags {
                            Some(flags) => CharacterDataFlag::from_bits_retain(flags),
                            None if hostile => CharacterDataFlag::HOSTILE,
                            None => CharacterDataFlag::empty(),
                        },
                        max_links,
                        link_family,
                        link_range,
                        common: CommonSpawn {
                            position: object.position,
                            rotation: object.rotation,
                            layout_id: object.layout_id,
                            display_flags: display_flags
                                .map(DisplayFlag)
                                .unwrap_or(base_npc.common.display_flags),
                            behavior: behavior.unwrap_or(base_npc.common.behavior),
                            ..base_npc.common
                        },
                        ..base_npc
                    };
                    if let Some(appearance) = appearance {
                        apply_appearance(&mut spawn, appearance);
                    }

                    self.cached_npcs.insert(object.layout_id, spawn.clone());
                    if !nonpop {
                        npc_spawns.push(spawn);
                    }
                }
                if let DropInObjectData::EventNpc { base_id, nonpop } = object.data {
                    let (model_chara, customize, equip, behavior) =
                        game_data.find_enpc(base_id).unwrap();

                    let spawn = SpawnNpc {
                        common: CommonSpawn {
                            base_id,
                            name_id: base_id,
                            model_chara,
                            object_kind: ObjectKind::EventNpc,
                            position: object.position,
                            rotation: object.rotation,
                            look: customize,
                            layout_id: object.layout_id,
                            behavior,
                            ..game_data.get_npc_equip(equip as u32).unwrap_or_default()
                        },
                        ..Default::default()
                    };

                    self.cached_npcs.insert(object.layout_id, spawn.clone());
                    if !nonpop {
                        npc_spawns.push(spawn);
                    }
                }
            }
        }

        npc_spawns
    }

    /// Returns a list of MapRanges that overlap this position.
    pub fn get_overlapping_map_ranges(&self, position: Vec3A) -> Vec<&MapRange> {
        let mut overlapping = Vec::new();

        for map_range in &self.map_ranges {
            match map_range.trigger_box_shape {
                TriggerBoxShape::Box => {
                    // TODO: support oriented boxes (this is used by sanctuary boundaries, for some reason)
                    let min_x = map_range.position.x - (map_range.scale[0]);
                    let max_x = map_range.position.x + (map_range.scale[0]);

                    let min_y = map_range.position.y - (map_range.scale[1]);
                    let max_y = map_range.position.y + (map_range.scale[1]);

                    let min_z = map_range.position.z - (map_range.scale[2]);
                    let max_z = map_range.position.z + (map_range.scale[2]);

                    if position.x >= min_x
                        && position.x <= max_x
                        && position.y >= min_y
                        && position.y <= max_y
                        && position.z >= min_z
                        && position.z <= max_z
                    {
                        overlapping.push(map_range);
                    }
                }
                TriggerBoxShape::Cylinder => {
                    // TODO: support arbitrarily-rotated cylinders
                    let length = map_range.scale[1] * 2.0;
                    let length_sq = f32::powi(length, 2);

                    let pt1 = Vec3A::from_array([
                        map_range.position.x,
                        map_range.position.y - map_range.scale[1],
                        map_range.position.z,
                    ]);
                    let pt2 = Vec3A::from_array([
                        map_range.position.x,
                        map_range.position.y + map_range.scale[1],
                        map_range.position.z,
                    ]);

                    let radius = map_range.scale[0]; // TODO: support individual radii (if that's even a thing, assert please)
                    let radius_sq = f32::powi(radius, 2);

                    if Self::cylinder_test(pt1, pt2, length_sq, radius_sq, position) != -1.0 {
                        overlapping.push(map_range);
                    }
                }
                _ => {} // TODO: support other box shapes
            }
        }

        overlapping
    }

    // From https://www.flipcode.com/archives/Fast_Point-In-Cylinder_Test.shtml
    fn cylinder_test(
        pt1: Vec3A,
        pt2: Vec3A,
        length_sq: f32,
        radius_sq: f32,
        test_pt: Vec3A,
    ) -> f32 {
        let dx = pt2.x - pt1.x;
        let dy = pt2.y - pt1.y;
        let dz = pt2.z - pt1.z;

        let pdx = test_pt.x - pt1.x;
        let pdy = test_pt.y - pt1.y;
        let pdz = test_pt.z - pt1.z;

        let dot = pdx * dx + pdy * dy + pdz * dz;
        if dot < 0.0 || dot > length_sq {
            -1.0
        } else {
            let dsq = (pdx * pdx + pdy * pdy + pdz * pdz) - dot * dot / length_sq;

            if dsq > radius_sq { -1.0 } else { dsq }
        }
    }
}

/// How PrepareZoning hides the character for a warp. Retail's Teleport and Return (warp types 4 and 7) hide the
/// character, with flags 0x02 within a zone and 0x0A to another one (Bob's Chronofoil capture of 2026-10-21: all 15
/// of them). Teleporting within a zone with warp type 1 and nothing hidden, as Kawari did, left the player in the
/// teleport animation after landing (Bentbranch, 2026-10-16 bundle; Moraby Drydocks, 2026-10-22 bundle.)
fn warp_appearance(warp_type: WarpType, same_zone: bool) -> (u8, PrepareZoningFlag) {
    match warp_type {
        WarpType::Teleport | WarpType::Return | WarpType::Ticket => {
            let mut flags = PrepareZoningFlag::UNK2;
            if !same_zone {
                flags |= PrepareZoningFlag::PRESERVE_COMPANION;
            }
            (1, flags)
        }
        // A chocobo porter's ride: retail's PrepareZoning hid the character, with 0x02 (Bob's Chronofoil captures,
        // nine rides; `zone_connection/chocobo_taxi.rs`.)
        WarpType::ChocoboTaxi => (1, PrepareZoningFlag::UNK2),
        // The story moved a zone's layer filter on where the player stands (`refresh_layer_filter`): retail's
        // PrepareZoning hid the character, flags 0 (Bob's Chronofoil captures, every one of them.)
        WarpType::Unk18 => (1, PrepareZoningFlag::default()),
        _ => (0, PrepareZoningFlag::default()),
    }
}

fn begin_change_zone<'a>(
    data: &'a mut WorldServer,
    network: &mut NetworkState,
    game_data: &mut GameData,
    lua: &KawariLua,
    destination_zone_id: Option<u16>,
    actor_id: ObjectId,
    warp_type: WarpType,
    loading_screen_vfx_id: u16,
) -> (&'a mut Instance, bool) {
    if let Some(destination_zone_id) = destination_zone_id {
        let mut needs_init_zone = false;
        let same_zone = data
            .find_actor_instance(actor_id)
            .is_some_and(|instance| instance.zone.id == destination_zone_id);
        let (hide_character, flags) = warp_appearance(warp_type, same_zone);
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::PrepareZoning(PrepareZoning {
            territory_type_id: destination_zone_id,
            warp_type,
            log_message: 0,
            vfx_id: 0,
            loading_screen_vfx_id,
            hide_character,
            fade_out_delay: 1,
            flags,
        }));

        network.send_to_by_actor_id(
            actor_id,
            FromServer::PacketSegment(ipc, actor_id),
            DestinationNetwork::ZoneClients,
        );

        // inform the players in this zone that this actor left
        if let Some(current_instance) = data.find_actor_instance_mut(actor_id) {
            // HACK: This is to prevent actors from disappearing when warping within the same zone.
            if current_instance.zone.id != destination_zone_id {
                needs_init_zone = true;
            }
        }

        if needs_init_zone {
            remove_actor_from_instance(data, network, actor_id);
        } else if data.find_actor_instance(actor_id).is_some() {
            // Within the same zone they stay in the instance they're in. A private area (the Waking Sands) would
            // otherwise get a new instance for every warp in it, and the player stayed behind in the old one as well:
            // the next zone change took them out of the old one only, so they were in two instances, and the quest
            // battle they entered next was loaded from Eastern Thanalan's (its monsters, its FATEs, no director;
            // Bob's bundle of 2026-10-28.)
            return (data.find_actor_instance_mut(actor_id).unwrap(), false);
        }

        // then find or create a new instance with the zone id
        let instance = data.ensure_exists(destination_zone_id, game_data, lua);
        // Insert an empty actor that will be filled later
        instance.insert_empty_actor(actor_id);

        (instance, needs_init_zone)
    } else {
        let instance = data.find_actor_instance_mut(actor_id).unwrap();
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::PrepareZoning(PrepareZoning {
            territory_type_id: instance.zone.id,
            warp_type,
            log_message: 0,
            vfx_id: 0,
            loading_screen_vfx_id,
            hide_character: 0,
            fade_out_delay: 1,
            flags: PrepareZoningFlag::default(),
        }));

        network.send_to_by_actor_id(
            actor_id,
            FromServer::PacketSegment(ipc, actor_id),
            DestinationNetwork::ZoneClients,
        );

        (instance, false)
    }
}

/// How far from a summoning bell its retainers' pop range may be, when the bell doesn't name it. Retail's retainers
/// stood 1.0 to 2.8 yalms from the player at the bell (Bob's Chronofoil captures, clients 2026.09.15 and 2026.10.06).
const RETAINER_SPOT_SEARCH_DISTANCE: f32 = 5.0;

/// Finds a random position from the pop range. Most have several positions available to stop people from spawning in the same place.
fn pick_point_in_pop_range(object: &InstanceObject) -> Position {
    let LayerEntryData::PopRange(pop_range) = &object.data else {
        return Position(glam::Vec3A::from_array(object.transform.translation));
    };

    let Some(rand) = fastrand::choice(&pop_range.positions) else {
        return Position(glam::Vec3A::from_array(object.transform.translation));
    };

    Position(glam::Vec3A::from_array(object.transform.translation) + glam::Vec3A::from_slice(rand))
}

/// Sends the needed information to ZoneConnection for a zone change.
pub fn change_zone_warp_to_pop_range(
    data: &mut WorldServer,
    network: &mut NetworkState,
    game_data: &mut GameData,
    lua: &KawariLua,
    destination_zone_id: Option<u16>,
    destination_instance_id: u32,
    actor_id: ObjectId,
    from_id: ClientId,
    warp_type: WarpType,
    loading_screen_vfx_id: u16,
) {
    let (target_instance, needs_init_zone) = begin_change_zone(
        data,
        network,
        game_data,
        lua,
        destination_zone_id,
        actor_id,
        warp_type,
        loading_screen_vfx_id,
    );

    let exit_position;
    let exit_rotation;
    if let Some((destination_object, _)) =
        target_instance.zone.find_pop_range(destination_instance_id)
    {
        let (_, rotation, _) =
            Affine3A::from(destination_object.transform).to_scale_rotation_translation();
        exit_position = Some(pick_point_in_pop_range(destination_object));
        exit_rotation = Some(euler_to_direction(rotation.to_euler(EulerRot::XYZ)));
    } else if let Some(npc) = target_instance.zone.get_battle_npc(destination_instance_id) {
        // An NPC's layout ID: in front of it, facing it. The GATEs' "pop ranges" (GFATEParams' LGBPopRange) are their
        // entrances' NPCs, and retail's GATE Keepers put the player 4.5 yalms in front of Sabotender Guia, turned to
        // her (Bob's Chronofoil capture 2c98f9e8: ActorSetPos at (-53.36, 3.26, -65.00), facing -1.82, from her at
        // (-57.86, 3.26, -65.40) facing 1.35.)
        const IN_FRONT: f32 = 4.5;
        let facing = npc.common.rotation;
        let position = npc.common.position.0
            + Vec3A::new(facing.sin() * IN_FRONT, 0.0, facing.cos() * IN_FRONT);
        exit_position = Some(Position(position));
        let turned = facing + std::f32::consts::PI;
        exit_rotation = Some(if turned > std::f32::consts::PI {
            turned - std::f32::consts::TAU
        } else {
            turned
        });
    } else {
        tracing::warn!(
            "Failed to find pop range {destination_instance_id} in zone {}",
            target_instance.zone.id
        );
        exit_position = None;
        exit_rotation = None;
    }

    do_change_zone(
        network,
        target_instance,
        needs_init_zone,
        exit_position,
        exit_rotation,
        from_id,
        warp_type,
    );
}

/// Sends the needed information to ZoneConnection for a zone change.
pub fn change_zone_warp_to_entrance(
    network: &mut NetworkState,
    game_data: &mut GameData,
    target_instance: &mut Instance,
    needs_init_zone: bool,
    from_id: ClientId,
    warp_type: WarpType,
) {
    let mut destiation_object = None;

    if let Some(director) = &target_instance
        .directors
        .iter()
        .find(|x| x.id.handler_type() == HandlerType::PublicContent)
    {
        // Some PublicContent has the pop range defined in the Excel sheet, we can use that if available.
        if let Some(pop_range_id) = game_data.find_public_content_pop_range(director.id.event_id())
        {
            destiation_object = target_instance
                .zone
                .find_pop_range(pop_range_id)
                .map(|(x, _)| x);
        }
    }

    // An InstanceContent's players face the way its entrance rect (an EventRange) does, as Sapphire does it: the
    // entrance circle's own object faced 0 in the Copperbell Mines, where retail turned players to 1.788 in all three
    // runs (Bob's captures and bundle of 2026-10-25).
    let entrance_rect_rotation = target_instance
        .content_handler_id()
        .filter(|id| id.handler_type() == HandlerType::InstanceContent)
        .and_then(|id| game_data.find_instance_content_entrance_rect(id.event_id()))
        .and_then(|rect| {
            let object = target_instance.zone.find_layout_object(rect);
            if object.is_none() {
                tracing::warn!("Entrance rect {rect} isn't in the zone's layout!");
            }
            object
        })
        .map(|rect| {
            let (_, rotation, _) = Affine3A::from(rect.transform).to_scale_rotation_translation();
            let facing = euler_to_direction(rotation.to_euler(EulerRot::XYZ));
            tracing::info!("The entrance rect {} faces {facing}", rect.instance_id);
            facing
        });

    let exit_position;
    let mut exit_rotation;
    if let Some(destination_object) = destiation_object.or(target_instance.zone.find_entrance()) {
        let (_, rotation, _) =
            Affine3A::from(destination_object.transform).to_scale_rotation_translation();
        exit_position = Some(pick_point_in_pop_range(destination_object));
        exit_rotation = Some(euler_to_direction(rotation.to_euler(EulerRot::XYZ)));
    } else {
        tracing::warn!(
            "Failed to find instanced content entrance?! This is a bug in Kawari, please report it!"
        );
        // I strongly suspect things like the Battlehall actuall end up doing this anyway (but not as a fallback)
        if let Some(destination_object) = target_instance.zone.find_random_pop_range() {
            tracing::warn!("Picking a random poprange as a fallback...");

            let (_, rotation, _) =
                Affine3A::from(destination_object.transform).to_scale_rotation_translation();
            exit_position = Some(pick_point_in_pop_range(destination_object));
            exit_rotation = Some(euler_to_direction(rotation.to_euler(EulerRot::XYZ)));
        } else {
            exit_position = None;
            exit_rotation = None;
        }
    }
    // Which way they face: a duty script's `ENTRANCE_ROTATION`, else retail's recorded facing (`entrance_facing.json`),
    // else the entrance rect's, else the entrance object's. The rect's was wrong in most duties (Toto-Rak 0 for
    // retail's 1.5708; Bob's bundle 20261030-165554.)
    let script_rotation = target_instance
        .directors
        .iter()
        .find(|director| director.id.handler_type() == HandlerType::InstanceContent)
        .and_then(|director| director.entrance_rotation());
    let territory = target_instance.zone.id;
    if let Some(rotation) = script_rotation {
        exit_rotation = Some(rotation);
    } else if let Some(rotation) = crate::entrance_facing::entrance_facing(territory) {
        tracing::info!("Players come in facing {rotation} (retail's, recorded for {territory})");
        exit_rotation = Some(rotation);
    } else if entrance_rect_rotation.is_some() {
        exit_rotation = entrance_rect_rotation;
    }

    do_change_zone(
        network,
        target_instance,
        needs_init_zone,
        exit_position,
        exit_rotation,
        from_id,
        warp_type,
    );
}

/// Teleports one player to another.
pub fn change_zone_to_player(
    network: &mut NetworkState,
    data: &mut WorldServer,
    game_data: &mut GameData,
    lua: &KawariLua,
    from_id: ClientId,
    to_actor_id: ObjectId,
) {
    let destination_zone_id;
    {
        let Some(target_instance) = data.find_actor_instance(to_actor_id) else {
            return;
        };

        destination_zone_id = target_instance.zone.id;
    }

    let from_actor_id = network.clients.get(&from_id).unwrap().0.actor_id;

    let (target_instance, needs_init_zone) = begin_change_zone(
        data,
        network,
        game_data,
        lua,
        Some(destination_zone_id),
        from_actor_id,
        WarpType::Normal,
        0,
    );

    let Some(target_actor) = target_instance.find_actor(to_actor_id) else {
        return;
    };

    do_change_zone(
        network,
        target_instance,
        needs_init_zone,
        Some(target_actor.position()),
        Some(target_actor.rotation()),
        from_id,
        WarpType::Normal,
    );
}

/// Sends the needed information to ZoneConnection for a zone change.
pub fn do_change_zone(
    network: &mut NetworkState,
    target_instance: &mut Instance,
    needs_init_zone: bool,
    exit_position: Option<Position>,
    exit_rotation: Option<f32>,
    from_id: ClientId,
    warp_type: WarpType,
) {
    let actor_id = network.clients.get(&from_id).unwrap().0.actor_id;
    let state = network.get_state_mut(from_id).unwrap();

    if needs_init_zone {
        // Clear spawn pools
        state.actor_allocator.clear();
        state.object_allocator.clear();

        // now that we have all of the data needed, inform the connection of where they need to be
        let msg = FromServer::ChangeZone(
            target_instance.zone.id,
            target_instance.content_finder_condition_id,
            target_instance.weather_id,
            exit_position.unwrap_or_default(),
            exit_rotation.unwrap_or_default(),
            target_instance.zone.to_lua_zone(target_instance.weather_id),
            target_instance.content_handler_id(),
        );
        network.send_to(from_id, msg, DestinationNetwork::ZoneClients);
    } else {
        // Where they'll be, kept now: a client that doesn't move once it's there sends no position to say so, and
        // what goes by the player's position in the meantime went by where they had been. After a wipe, the party's
        // NPCs got up where the player had fallen, beside the boss brought back, and started the fight again before
        // the recommencement (Bob's bundle 20261109-195426, the Navel.)
        if let Some(position) = exit_position
            && let Some(NetworkedActor::Player { spawn, .. }) =
                target_instance.find_actor_mut(actor_id)
        {
            spawn.common.position = position;
            if let Some(rotation) = exit_rotation {
                spawn.common.rotation = rotation;
            }
        }
        // We want to delay sending this to give time for the client to fade out.
        let segment = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorSetPos(ActorSetPos {
            position: exit_position.unwrap_or_default(),
            rotation: exit_rotation.unwrap_or_default(),
            warp_type,
            arg: 2, // unknown
            ..Default::default()
        }));
        target_instance.insert_task(
            from_id,
            actor_id,
            WARP_DELAY,
            QueuedTaskData::PacketSegment { segment },
        );
    }
}

/// Process zone-related messages.
pub fn handle_zone_messages(
    data: Arc<Mutex<WorldServer>>,
    network: Arc<Mutex<NetworkState>>,
    game_data: Arc<Mutex<GameData>>,
    lua: Arc<Mutex<KawariLua>>,
    msg: &ToServer,
) -> bool {
    match msg {
        ToServer::ZoneLoaded(from_id, from_actor_id, player_spawn) => {
            tracing::info!(
                "Client {from_id:?} has now loaded into the zone, sending them existing player data."
            );

            let mut data = data.lock();

            // replace the connection's actor in the table
            let instance = data.find_actor_instance_mut(*from_actor_id).unwrap();
            *instance.find_actor_mut(*from_actor_id).unwrap() = NetworkedActor::Player {
                spawn: player_spawn.clone(),
                status_effects: StatusEffects::default(),
                teleport_query: TeleportQuery::default(),
                distance_range: DistanceRange::Normal,
                conditions: Conditions::default(),
                executing_gimmick_jump: false,
                inside_instance_exit: false,
                parameters: BaseParameters::default(),
                dueling_opponent_id: ObjectId::default(),
                remove_cooldowns: false,
                last_combo_action: 0,
                combo_sequence: 0,
                autoattack_target: None,
                autoattack_timing: 0,
                job_gauge: JobGauge::default(),
            };

            // The bots of their party that came with them (`party_bot::pack_for_travel`) appear beside them, in the
            // party. In through the Duty Finder: the party's empty places are filled with bots (`server/party_bot.rs`),
            // for the roles no one in it plays, unless the party went in as it is (Unrestricted Party, Explorer Mode).
            // Through Duty Support: the members chosen first (party NPCs, `server/party_npc.rs`), announced to the
            // client as retail does, then bots for any role still missing. The client is told its party list.
            let content = instance.content_finder_condition_id;
            let filling = instance.pending_duty_bots.remove(from_actor_id);
            let chosen = instance.pending_duty_support.remove(from_actor_id);
            let settings = instance.content_settings.unwrap_or_default();
            let usable: HashMap<u32, String> = {
                let lua = lua.lock();
                let state = lua.0.app_data_ref::<KawariLuaState>().unwrap();
                state.action_scripts.clone()
            };
            let mut game_data = game_data.lock();
            // In a duty, its one party's places besides the player; none in a solo duty (a quest battle) or an
            // alliance's, so the party's bots wait outside until the player comes back out.
            let room = (content != 0).then(|| {
                game_data
                    .content_party_roles(content)
                    .map(|roles| roles.members.saturating_sub(1) as usize)
                    .unwrap_or(0)
            });
            let mut party_roles = Vec::new();
            if let Some(chosen) = chosen {
                let (_, roles) =
                    party_npc::fill_duty_support(instance, &mut game_data, *from_actor_id, &chosen);
                party_roles = roles;
                party_npc::announce(&network, instance, *from_actor_id);
            }
            let room = room.map(|room| room.saturating_sub(party_roles.len()));
            party_roles.extend(party_bot::arrive(
                &mut data,
                &mut game_data,
                &usable,
                *from_actor_id,
                room,
            ));
            if filling {
                let filled = if crate::server::duty_finder::fills_the_party(settings) {
                    party_bot::fill_duty_party(
                        &mut data,
                        &mut game_data,
                        &usable,
                        *from_actor_id,
                        content,
                        &party_roles,
                    )
                    .len()
                } else {
                    0
                };
                if let Some(instance) = data.find_actor_instance_mut(*from_actor_id) {
                    instance.loot_rule = crate::server::duty_finder::loot_rule(
                        settings,
                        game_data.content_allows_loot_rules(content),
                        filled,
                    );
                    tracing::info!(
                        "Duty Finder: {from_actor_id} went in with {settings:?}: {filled} places filled, loot {:?}",
                        instance.loot_rule
                    );
                }
            }
            if let Some(instance) = data.find_actor_instance(*from_actor_id)
                && let Some(members) = party_bot::party_list(instance, *from_actor_id)
            {
                let mut network = network.lock();
                network.send_to(
                    *from_id,
                    FromServer::BotParty(Some(members)),
                    DestinationNetwork::ZoneClients,
                );
            }

            // Epic Echo for an unrestricted party without Level Sync, by the duty's level and the highest member's
            // (bots too); then everyone gets the duty's Echo (`server/duty_finder.rs`.)
            if content != 0
                && let Some(instance) = data.find_actor_instance_mut(*from_actor_id)
            {
                use crate::server::duty_finder;
                if duty_finder::unsynced_unrestricted(settings) {
                    let duty_level = game_data.find_content_synced_level(content).unwrap_or(0);
                    let highest = instance
                        .actors
                        .iter()
                        .filter(|(id, actor)| {
                            matches!(actor, NetworkedActor::Player { .. })
                                || instance.party_bots.contains_key(id)
                        })
                        .map(|(_, actor)| actor.get_common_spawn().level)
                        .max()
                        .unwrap_or(0);
                    instance.epic_echo = duty_finder::epic_echo_percent(duty_level, highest);
                }
                if instance.echo > 0 || instance.epic_echo > 0 {
                    duty_finder::refresh_echo(&network, instance);
                }
            }

            true
        }
        ToServer::ChangeZone(
            from_id,
            actor_id,
            zone_id,
            new_position,
            new_rotation,
            warp_type,
            loading_screen_vfx_id,
        ) => {
            tracing::info!("{from_id:?} is requesting to go to zone {zone_id}");

            let mut data = data.lock();
            let mut network = network.lock();
            let mut game_data = game_data.lock();
            let lua = lua.lock();

            let (target_instance, needs_init_zone) = begin_change_zone(
                &mut data,
                &mut network,
                &mut game_data,
                &lua,
                Some(*zone_id),
                *actor_id,
                *warp_type,
                *loading_screen_vfx_id,
            );
            do_change_zone(
                &mut network,
                target_instance,
                needs_init_zone,
                *new_position,
                *new_rotation,
                *from_id,
                *warp_type,
            );

            true
        }
        ToServer::EnterZoneJump(
            from_id,
            actor_id,
            exitbox_id,
            warp_type,
            loading_screen_vfx_id,
        ) => {
            let mut data = data.lock();
            let mut network = network.lock();

            // first, find the zone jump in the current zone
            let mut destination_zone_id;
            let destination_instance_id;
            if let Some(current_instance) = data.find_actor_instance(*actor_id) {
                let Some((_, new_exit_box)) = current_instance.zone.find_exit_box(*exitbox_id)
                else {
                    tracing::warn!("Couldn't find exit box {exitbox_id}?!");
                    return true;
                };
                destination_zone_id = new_exit_box.territory_type;

                // Seen when attempting to enter underwater portals in Ruby Sea
                if new_exit_box.territory_type == 0
                    && new_exit_box.zone_id == 0
                    && new_exit_box.exit_type == physis::layer::ExitType::Invisible
                {
                    destination_zone_id = current_instance.zone.id;
                }

                destination_instance_id = new_exit_box.destination_instance_id;
            } else {
                tracing::warn!("Actor isn't in the instance it was expected in. This is a bug!");
                return true;
            }

            let mut game_data = game_data.lock();
            let lua = lua.lock();
            change_zone_warp_to_pop_range(
                &mut data,
                &mut network,
                &mut game_data,
                &lua,
                Some(destination_zone_id),
                destination_instance_id,
                *actor_id,
                *from_id,
                *warp_type,
                *loading_screen_vfx_id,
            );

            true
        }
        ToServer::Warp(from_id, actor_id, warp_id) => {
            let mut data = data.lock();
            let mut network = network.lock();
            let mut game_data = game_data.lock();
            let lua = lua.lock();

            // first, find the warp and it's destination
            let (destination_instance_id, destination_zone_id) = game_data
                .get_warp(*warp_id)
                .expect("Failed to find the warp!");

            change_zone_warp_to_pop_range(
                &mut data,
                &mut network,
                &mut game_data,
                &lua,
                Some(destination_zone_id),
                destination_instance_id,
                *actor_id,
                *from_id,
                WarpType::Normal,
                0,
            );

            true
        }
        ToServer::WarpAetheryte(from_id, actor_id, aetheryte_id, housing_aethernet, warp_type) => {
            let mut data = data.lock();
            let mut network = network.lock();
            let mut game_data = game_data.lock();
            let lua = lua.lock();

            // first, find the warp and it's destination
            let (destination_instance_id, destination_zone_id) = game_data
                .get_aetheryte(*aetheryte_id, *housing_aethernet)
                .expect("Failed to find the aetheryte!");

            change_zone_warp_to_pop_range(
                &mut data,
                &mut network,
                &mut game_data,
                &lua,
                Some(destination_zone_id),
                destination_instance_id,
                *actor_id,
                *from_id,
                *warp_type,
                0,
            );

            true
        }
        ToServer::WarpPopRange(from_id, from_actor_id, territory_id, pop_range_id, warp_type) => {
            let mut data = data.lock();
            let mut network = network.lock();
            let mut game_data = game_data.lock();
            let lua = lua.lock();

            change_zone_warp_to_pop_range(
                &mut data,
                &mut network,
                &mut game_data,
                &lua,
                Some(*territory_id),
                *pop_range_id,
                *from_actor_id,
                *from_id,
                *warp_type,
                0,
            );

            true
        }
        ToServer::ZoneIn(from_id, from_actor_id, is_teleport) => {
            tracing::info!("Player {from_id:?} has finally zoned in, informing other players...");

            // HACK: maybe there's states where you're dead, zone in and yet not a raise?
            let mut data = data.lock();
            let mut is_raise = false;
            if let Some(instance) = data.find_actor_instance_mut(*from_actor_id)
                && let Some(actor) = instance.find_actor_mut(*from_actor_id)
            {
                is_raise = actor.get_common_spawn().mode == CharacterMode::Dead;
            }

            // Inform all clients to play the zone in animation
            let mut network = network.lock();
            let mut to_remove = Vec::new();
            for (id, (handle, _)) in &mut network.clients {
                let id = *id;

                let category = ActorControlCategory::ZoneIn {
                    warp_finish_anim: 1,
                    raise_anim: if is_raise { 1 } else { 0 },
                    unk1: if *is_teleport { 110 } else { 0 },
                };

                if id == *from_id {
                    let msg = FromServer::ActorControlSelf(category);

                    if handle.send(msg).is_err() {
                        to_remove.push(id);
                    }
                } else {
                    let msg = FromServer::ActorControl(*from_actor_id, category);

                    if handle.send(msg).is_err() {
                        to_remove.push(id);
                    }
                }
            }
            network.to_remove.append(&mut to_remove);

            // Then update the PlayerSpawn so respawning this player doesn't appear invisible again
            if let Some(instance) = data.find_actor_instance_mut(*from_actor_id)
                && let Some(actor) = instance.find_actor_mut(*from_actor_id)
            {
                actor
                    .get_common_spawn_mut()
                    .display_flags
                    .remove(DisplayFlag::INVISIBLE);

                // Send them the current list of FATEs
                if let Some(director) = &instance.directors.first()
                    && director.id.handler_type() == HandlerType::Fate
                {
                    for fate in &instance.fates {
                        inform_fate_spawn(&mut network, *from_actor_id, fate);
                    }
                }
            }

            if let Some(instance) = data.find_actor_instance_mut(*from_actor_id)
                && is_raise
            {
                set_character_mode(
                    instance,
                    &mut network,
                    *from_actor_id,
                    CharacterMode::Normal,
                    0,
                    true,
                );
            }

            true
        }
        ToServer::MoveToPopRange(from_id, from_actor_id, id, warp_type) => {
            let zone_id;
            {
                let data = data.lock();
                let Some(instance) = data.find_actor_instance(*from_actor_id) else {
                    return false;
                };

                zone_id = instance.zone.id;
            }

            let mut data = data.lock();
            let mut network = network.lock();
            let mut game_data = game_data.lock();
            let lua = lua.lock();
            change_zone_warp_to_pop_range(
                &mut data,
                &mut network,
                &mut game_data,
                &lua,
                Some(zone_id),
                *id,
                *from_actor_id,
                *from_id,
                *warp_type,
                0,
            );

            true
        }
        ToServer::NewLocationDiscovered(from_id, layout_id, _pos, zone_id) => {
            let data = data.lock();
            let mut network = network.lock();

            for instance in &data.instances {
                if instance.zone.id == *zone_id {
                    for range in &instance.zone.map_ranges {
                        if range.instance_id == *layout_id
                            && let Some(discovery_id) = range.discovery_id
                        {
                            // TODO: Check if the player is actually in this range?
                            // Most maps are revealed part by part as the player runs into them; A Realm Reborn's
                            // field zones all at once, with the part retail sends for that (see WHOLE_MAP_DISCOVERY.)
                            let mut game_data = game_data.lock();
                            let Some(map_id) = game_data.get_territory_info_map_data(*zone_id)
                            else {
                                tracing::error!(
                                    "Unable to get Map column data from TerritoryInfo sheet for zone id {zone_id}"
                                );
                                return true;
                            };

                            let part = if game_data.reveals_whole_map(*zone_id) {
                                crate::common::WHOLE_MAP_DISCOVERY
                            } else {
                                discovery_id.into()
                            };
                            let msg = FromServer::LocationDiscovered(map_id.into(), part);
                            network.send_to(*from_id, msg, DestinationNetwork::ZoneClients);
                            return true;
                        }
                    }

                    // If we somehow didn't get any discoverable ranges, exit early. Is that even possible?
                    break;
                }
            }

            true
        }
        ToServer::PlaceFurniture(
            from_actor_id,
            container,
            slot,
            item_id,
            stain,
            position,
            indoors,
            rotation,
            plot_index,
        ) => {
            let mut network = network.lock();
            let data = data.lock();

            let Some(instance) = data.find_actor_instance(*from_actor_id) else {
                return true;
            };

            let msg = FromServer::FurniturePlaced(
                *container,
                *slot,
                *item_id,
                *stain,
                *position,
                *indoors,
                *rotation,
                *plot_index,
            );

            // We *do* want to include the sender here
            network.send_in_range_inclusive_instance(
                *from_actor_id,
                instance,
                msg,
                DestinationNetwork::ZoneClients,
            );

            true
        }
        ToServer::TranslateFurniture(
            from_actor_id,
            plot_info,
            slot,
            position,
            rotation,
            indoors,
        ) => {
            let mut network = network.lock();
            let data = data.lock();

            let Some(instance) = data.find_actor_instance(*from_actor_id) else {
                return true;
            };

            let msg =
                FromServer::FurnitureTranslated(*plot_info, *slot, *position, *rotation, *indoors);

            // We *don't* want to include the sender here
            network.send_in_range_instance(
                *from_actor_id,
                instance,
                msg,
                DestinationNetwork::ZoneClients,
            );

            true
        }
        _ => false,
    }
}

/// Logs a zone's layer filters (its LVB's layer sets) and which of its layers each one shows, with their event NPCs,
/// once per zone, when it has more than one: where the story changes a place (the Waking Sands), retail's server picks
/// the filter by quest progress (`layer_filters`), and this is what tells which key goes with which part of the story.
fn log_layer_filters(
    territory: u16,
    layer_sets: &[physis::scn::ScnLayerSet],
    layer_groups: &[Lgb],
) {
    static LOGGED: std::sync::Mutex<Option<HashSet<u16>>> = std::sync::Mutex::new(None);
    if layer_sets.len() < 2 {
        return;
    }
    if let Ok(mut logged) = LOGGED.lock()
        && !logged.get_or_insert_with(HashSet::new).insert(territory)
    {
        return;
    }
    let sets: Vec<String> = layer_sets
        .iter()
        .map(|set| {
            format!(
                "{} (territory {}, duty {})",
                set.id, set.territory_type_id, set.content_finder_condition_id
            )
        })
        .collect();
    tracing::info!(
        "Zone {territory}'s layer filters: {}; the key sent is `layer_filters`'",
        sets.join(", ")
    );
    for lgb in layer_groups {
        for chunk in &lgb.chunks {
            for layer in &chunk.layers {
                let filter = &layer.header.layer_set_referenced_list;
                if matches!(
                    filter.referenced_type,
                    physis::layer::LayerSetReferencedType::All
                ) {
                    continue;
                }
                let npcs: Vec<u32> = layer
                    .objects
                    .iter()
                    .filter_map(|object| match &object.data {
                        LayerEntryData::EventNPC(npc) => Some(npc.parent_data.parent_data.base_id),
                        _ => None,
                    })
                    .collect();
                tracing::info!(
                    "  layer {} ({}): {:?} {:?}, {} objects, event NPCs {:?}",
                    layer.header.layer_id,
                    layer.header.name.value,
                    filter.referenced_type,
                    filter.layer_set_ids,
                    layer.objects.len(),
                    npcs
                );
            }
        }
    }
}

/// A drop-in's recorded looks over what the sheets gave (`DropInAppearance`): each part only when it's the right size.
fn apply_appearance(spawn: &mut SpawnNpc, appearance: &kawari::common::DropInAppearance) {
    use binrw::BinRead;
    fn read<T: for<'a> BinRead<Args<'a> = ()>>(bytes: &[u8], size: usize) -> Option<T> {
        (bytes.len() == size)
            .then(|| T::read_le(&mut std::io::Cursor::new(bytes)).ok())
            .flatten()
    }
    spawn.common.class_job = appearance.class_job;
    if let Some(weapon) = read(&appearance.main_weapon, 8) {
        spawn.common.main_weapon_model = weapon;
    }
    if let Some(weapon) = read(&appearance.sec_weapon, 8) {
        spawn.common.sec_weapon_model = weapon;
    }
    if let Some(models) = read(&appearance.models, 40) {
        spawn.common.models = models;
    }
    if let Some(look) = read(&appearance.look, 26) {
        spawn.common.look = look;
    }
}

#[cfg(test)]
mod appearance_tests {
    use super::*;

    /// Trial by Wind's conjurer as retail spawned him (Bob's Chronofoil capture b23b91ef, the drop-in's numbers.)
    #[test]
    fn retail_appearance() {
        let appearance = kawari::common::DropInAppearance {
            class_job: 6,
            main_weapon: vec![189, 2, 1, 0, 6, 0, 0, 0],
            sec_weapon: vec![0; 8],
            models: [
                7, 0, 129, 0, 7, 0, 36, 0, 7, 0, 117, 0, 7, 0, 82, 0, 10, 0, 148, 0,
            ]
            .into_iter()
            .chain([0; 20])
            .collect(),
            look: vec![
                1, 1, 1, 50, 1, 3, 9, 1, 17, 59, 13, 0, 0, 0, 2, 59, 1, 3, 2, 0, 0, 75, 0, 25, 0, 0,
            ],
        };
        let mut spawn = SpawnNpc::default();
        apply_appearance(&mut spawn, &appearance);
        assert_eq!(spawn.common.class_job, 6);
        assert_eq!(spawn.common.main_weapon_model.id, 701);
        assert_eq!(spawn.common.models[0].id, 7);
        assert_eq!(spawn.common.models[0].variant, 129);
        assert_eq!(spawn.common.look.face, 3);
        assert_eq!(spawn.common.look.hair, 9);

        // A part of the wrong size is left as it was.
        let mut spawn = SpawnNpc::default();
        let short = kawari::common::DropInAppearance {
            look: vec![1, 2, 3],
            ..appearance
        };
        apply_appearance(&mut spawn, &short);
        assert_eq!(spawn.common.look.face, SpawnNpc::default().common.look.face);
    }
}

#[cfg(test)]
mod event_object_near_tests {
    use super::*;

    /// Rock of Rancor's throws: the destination where retail's landed, a gathering point closer still (never a target),
    /// and a throw too far off.
    #[test]
    fn nearest_event_object_within_reach() {
        let mut zone = Zone::default();
        let object = |kind, layout_id, x, z| SpawnObject {
            kind,
            layout_id,
            position: Position(Vec3A::new(x, -66.3, z)),
            ..Default::default()
        };
        zone.cached_objects.insert(
            2001983,
            object(ObjectKind::EventObj, 4273290, -222.8, 396.7),
        );
        zone.cached_objects.insert(
            2001984,
            object(ObjectKind::EventObj, 4273291, -212.0, 380.0),
        );
        zone.cached_objects
            .insert(30001, object(ObjectKind::GatheringPoint, 9, -223.0, 396.8));

        let landed = |x, z| {
            zone.event_object_near(Vec3A::new(x, -66.35, z), 2.0)
                .map(|object| object.layout_id)
        };
        assert_eq!(landed(-222.89, 396.55), Some(4273290));
        assert_eq!(landed(-225.17, 394.39), None);
    }
}
