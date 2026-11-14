//! All things zone related, such as changing the weather or warping.

use crate::{
    ObsfucationData, TeleportReason, ToServer, ZoneConnection, inventory::BuyBackList,
    lua::LuaZone, zone_connection::TeleportQuery,
};
use kawari::{
    common::{
        CharacterMode, FestivalId, HandlerId, HandlerType, HouseId, HouseUnit, HousingFlag,
        LandData, Position, TerritoryIntendedUse, WarpType, timestamp_secs,
    },
    constants::{
        DISCOVERY_MAPS_WITH_UP_TO_16_REGIONS_BITMASK_SIZE, MAP_DISCOVERY_BITMASK_SIZE,
        OBFUSCATION_ENABLED_MODE,
    },
    ipc::zone::{
        ActorControlCategory, Condition, DutyFinderSetting, FurnitureList, House, HouseExterior,
        HouseList, HouseStatus, HousingInteriorDetails, PlotSize, ServerZoneIpcData,
        ServerZoneIpcSegment, WeatherChange, ZoneInit, ZoneInitFlags,
    },
    packet::{ConnectionState, PacketSegment, ScramblerKeyGenerator, SegmentData, SegmentType},
};

impl ZoneConnection {
    /// Request the global server state to change our zone.
    pub async fn change_zone(
        &mut self,
        new_zone_id: u16,
        new_position: Option<Position>,
        new_rotation: Option<f32>,
        warp_type: WarpType,
        loading_screen_vfx_id: u16,
    ) {
        self.teleport_reason = TeleportReason::NotSpecified;
        self.handle
            .send(ToServer::ChangeZone(
                self.id,
                self.player_data.character.actor_id,
                new_zone_id,
                new_position,
                new_rotation,
                warp_type,
                loading_screen_vfx_id,
            ))
            .await;
    }

    /// Handle the zone change information from the server state.
    pub async fn handle_zone_change(
        &mut self,
        new_zone_id: u16,
        content_finder_condition_id: u16,
        weather_id: u16,
        exit_position: Position,
        exit_rotation: f32,
        lua_zone: &LuaZone,
        handler_id: Option<HandlerId>,
    ) {
        self.spawned_in = false;
        self.fate_motivation_npcs.clear();
        self.content_handler_id = handler_id;
        self.content_finder_condition_id = content_finder_condition_id;
        if content_finder_condition_id == 0 {
            self.entered_with = None;
            self.entered_roulette = None;
        }

        // In a duty they haven't cleared, they're one of its newcomers (the first-timer bonus is everyone's then.)
        if let Some(handler) = handler_id
            && handler.handler_type() == HandlerType::InstanceContent
            && !self.has_cleared_content(handler.event_id() as u16)
        {
            self.handle
                .send(ToServer::DutyNewcomer(self.player_data.character.actor_id))
                .await;
        }

        // Quest battles are instanced content too, but don't come from the Duty Finder.
        let bound_by_duty = content_finder_condition_id != 0
            || handler_id.is_some_and(|x| x.handler_type() == HandlerType::QuestBattle);

        // Riding into a zone without mounts gets them off: retail's spawn there had no mount and the Condition after
        // the load none (Bob's Chronofoil captures: into Ul'dah, the Waking Sands, the Twins' Gridania; elsewhere it
        // re-sent the mount.) Kawari kept the mount in the spawn, and they rode through Ul'dah (Bob's bundle
        // 20261029-162434.)
        if self.player_data.volatile.current_mount != 0
            && !self.game_data.lock().zone_allows_mounts(new_zone_id)
        {
            tracing::info!(
                "{} rode into zone {new_zone_id}, which has no mounts: dismounted",
                self.player_data.character.name
            );
            self.player_data.volatile.current_mount = 0;
            if self.player_data.volatile.mode == CharacterMode::Mounted {
                self.player_data.volatile.mode = CharacterMode::Normal;
                self.player_data.volatile.mode_arg = 0;
            }
        }

        // Commit back our zone id and other volatile info on zone change.
        {
            self.player_data.volatile.zone_id = new_zone_id as i32;
            self.player_data.volatile.position = exit_position;
            self.player_data.volatile.rotation = exit_rotation as f64;

            let mut db = self.database.lock();
            db.commit_volatile(&self.player_data);
        }

        // Generate obsfucation-related keys if needed.
        if self.config.enable_packet_obsfucation {
            // What's queued was sent under the keys the client has now.
            self.flush_segments().await;
            let seed1 = fastrand::u8(..);
            let seed2 = fastrand::u8(..);
            let seed3 = fastrand::u32(..);

            let generator = ScramblerKeyGenerator::new();

            self.obsfucation_data = ObsfucationData {
                seed1,
                seed2,
                seed3,
            };

            let ConnectionState::Zone { scrambler_keys, .. } = &mut self.state else {
                panic!("Unexpected connection type!");
            };
            *scrambler_keys = Some(generator.generate(seed1, seed2, seed3));

            tracing::info!(
                "You enabled packet obsfucation in your World config, if things break please report it!",
            );
        }

        // they send the initialize packet again for some reason
        {
            self.send_segment(PacketSegment {
                segment_type: SegmentType::Initialize,
                data: SegmentData::Initialize {
                    actor_id: self.player_data.character.actor_id,
                    timestamp: timestamp_secs(),
                },
                ..Default::default()
            })
            .await;
        }

        // Clear the server's copy of the buyback list.
        self.player_data.buyback_list = BuyBackList::default();

        // Send obsfucation init
        if self.config.enable_packet_obsfucation {
            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::InitializeObfuscation {
                unk_before: [0; 6],
                obsfucation_mode: OBFUSCATION_ENABLED_MODE,
                seed1: !self.obsfucation_data.seed1,
                seed2: !self.obsfucation_data.seed2,
                seed3: !self.obsfucation_data.seed3,
            });
            self.send_ipc_self(ipc).await;
            // In a frame that ends with it: what follows is written with the keys it tells the client to make.
            self.flush_segments().await;
        } else {
            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::InitializeObfuscation {
                unk_before: [0; 6],
                obsfucation_mode: 0,
                seed1: 0,
                seed2: 0,
                seed3: 0,
            });
            self.send_ipc_self(ipc).await;
        }

        self.send_ipc_self(ServerZoneIpcSegment::new(ServerZoneIpcData::InitResponse {
            actor_id: self.player_data.character.actor_id,
        }))
        .await;

        // Send owned housing list (unsure where this fits in before ZoneInit?!)
        {
            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::OwnedHousing {
                unk1: Default::default(),
                unk2: Default::default(),
                unk3: Default::default(),
                unk4: Default::default(),
                unk5: Default::default(),
                apartment: LandData {
                    id: HouseId {
                        unit: HouseUnit {
                            apartment_division_plot_index: 0,
                            apartment_flag: true,
                        },
                        unk1: 0,
                        ward_index: 0,
                        room_number: 1,
                        territory_type_id: 340,
                        world_id: self.config.world_id,
                    },
                    flags: 19,
                    unk1: 0,
                },
            });
            self.send_ipc_self(ipc).await;
        }

        // Send the list of available items if we're in an inn, since its only accessible via their beds.
        if lua_zone.intended_use == TerritoryIntendedUse::Inn {
            let display_ids;
            {
                let mut game_data = self.game_data.lock();
                display_ids = game_data.get_latest_fittingshop_display_ids();
            }

            let ipc =
                ServerZoneIpcSegment::new(ServerZoneIpcData::UpdateFittingShop { display_ids });
            self.send_ipc_self(ipc).await;
        }

        let mut game_festival_ids = self.config.active_festivals.map(FestivalId);
        let mut game_festival_phases = [0; 8];

        // Set up ocean fishing
        if bound_by_duty {
            let mut game_data = self.game_data.lock();
            if let Some(spots) =
                game_data.lookup_ikd_route_spots_via_content(content_finder_condition_id as u32)
            {
                // Special festivals used for ocean fishing.
                game_festival_ids = [
                    FestivalId(101),
                    FestivalId(102),
                    FestivalId::default(),
                    FestivalId::default(),
                    FestivalId::default(),
                    FestivalId::default(),
                    FestivalId::default(),
                    FestivalId::default(),
                ];

                game_festival_phases = [
                    spots[0] as u16 + 1, // IKDSpot + 1
                    23,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ];
            }
        }

        // Init Zone
        {
            let mut flags = if self.initial_login {
                ZoneInitFlags::INITIAL_LOGIN
            } else if bound_by_duty {
                ZoneInitFlags::INSTANCED_CONTENT | ZoneInitFlags::UNK5
            } else {
                ZoneInitFlags::default()
            };

            if !bound_by_duty {
                // Flying, where the zone has an AetherCurrentCompFlgSet and the player has completed it (A Realm
                // Reborn's 19 for its field zones), as retail's ZoneInit sets it: 641 of them across Bob's Chronofoil
                // captures, his flying character's (sets 1-19) and his story characters' (none), not one otherwise.
                // Upstream set it everywhere outside duties, and new characters flew in Southern Thanalan (Bob's
                // bundle 20261031-203841).
                let flying_set = self.game_data.lock().flying_set(new_zone_id);
                if flying_set.is_some_and(|set| {
                    self.player_data
                        .aether_current
                        .comp_flg_set
                        .contains(set - 1)
                }) {
                    flags |= ZoneInitFlags::ENABLE_FLYING;
                }
            } else {
                self.offered_teleport = None; // Discard any previously offered teleports once we're in a duty.

                let mut game_data = self.game_data.lock();
                if game_data.is_duty_recorder_allowed(content_finder_condition_id) {
                    flags |= ZoneInitFlags::ENABLE_RECORD_READY_CHECK;
                }
            }

            let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ZoneInit(ZoneInit {
                territory_type: new_zone_id,
                // Which of the zone's layer filters to show, by the story (`layer_filters`.)
                transition_territory_filter_key: self.layer_filter_key(new_zone_id),
                weather_id: weather_id as u8,
                flags,
                content_finder_condition_id,
                game_festival_ids,
                game_festival_phases,
                ui_festival_ids: self.config.active_festivals.map(FestivalId),
                unk1: 8.59375,
                // Where the player comes in: retail fills it in every ZoneInit, out in the world and in duties (Bob's
                // Chronofoil capture b23b91ef: Trial by Wind's start spot, the East Shroud's arrivals); Kawari sent
                // the origin.
                position: exit_position,
                ..Default::default()
            }));
            self.send_ipc_self(ipc).await;
        }

        let level;
        {
            let mut game_data = self.game_data.lock();

            level = self
                .player_data
                .inventory
                .equipped
                .calculate_item_level(&mut game_data) as u32;
        }

        self.actor_control_self(ActorControlCategory::SetItemLevel { level })
            .await;

        // NOTE: This is currently disabled because - at least in my testing - we quickly desync and I'm unsure if this is actually important right now.
        // Let's re-enable this once we're not trash
        // builder.actor_control_self(ActorControlCategory::EnableInstancePositionHandler {
        //     enabled: bound_by_duty,
        // });

        // send some weird thing to make the zone load correctly
        if !bound_by_duty {
            self.send_ipc_self(ServerZoneIpcSegment::new(ServerZoneIpcData::DailyQuests {
                unk1: [0; 56],
            }))
            .await;

            self.send_ipc_self(ServerZoneIpcSegment::new(
                ServerZoneIpcData::DailyQuestRepeatFlags { unk1: [0; 8] },
            ))
            .await;
        }

        if self.initial_login {
            self.send_quest_information().await;
            self.send_crafting_gathering_information().await;
            self.send_player_currencies().await;
            self.send_hunting_logs().await;
        }

        // The GATE on in the Gold Saucer, if any: its director, before the lotteries' (`zone_connection/gate.rs`.)
        self.send_gate_director().await;
        // A new Air Force One takes its score again.
        self.ride_shooting_scored = false;

        // The Gold Saucer's lotteries, as retail told them at every zone change: when the Jumbo Cactpot's weeks begin
        // (912) and what's been played (914; Bob's Chronofoil captures, `gold_saucer.rs`.)
        self.actor_control_self(ActorControlCategory::SetWeeklyLotOffsetTime {
            offset_time: crate::gold_saucer::WEEKLY_LOT_OFFSET_TIME as u32,
        })
        .await;
        self.send_gold_saucer_flags().await;

        if lua_zone.intended_use == TerritoryIntendedUse::HousingOutdoor {
            let mut houses = [House::default(); 30];

            // First, populate the houses in this ward. Note that for now, we treat every ward the same.
            // TODO: For now, we hardcode 3 prefab houses as demonstration units until we implement more of the system. One cottage, one house and one mansion, all set to be individually owned and locked. Plots 5, 6, and 12 were chosen due to their close proximity to each other.
            // Glade house (Wood)
            houses[4] = House {
                plot_size: PlotSize::Medium,
                status: HouseStatus::HouseBuilt,
                flags: HousingFlag::OPEN,
                exterior: HouseExterior {
                    roof_id: 1029,
                    walls_id: 3589,
                    windows_id: 2562,
                    door_id: 514,
                    ..Default::default()
                },
                ..Default::default()
            };

            // Hingan mansion (Mokuzo)
            houses[5] = House {
                plot_size: PlotSize::Large,
                status: HouseStatus::HouseBuilt,
                flags: HousingFlag::OPEN,
                exterior: HouseExterior {
                    roof_id: 1081,
                    walls_id: 3632,
                    windows_id: 2579,
                    door_id: 531,
                    ..Default::default()
                },
                ..Default::default()
            };

            // Highland cottage (Wood)
            houses[11] = House {
                plot_size: PlotSize::Small,
                status: HouseStatus::HouseBuilt,
                flags: HousingFlag::OPEN | HousingFlag::OWNED_BY_FC,
                exterior: HouseExterior {
                    roof_id: 1136,
                    walls_id: 3687,
                    windows_id: 2598,
                    door_id: 550,
                    ..Default::default()
                },
                ..Default::default()
            };

            self.send_ipc_self(ServerZoneIpcSegment::new(ServerZoneIpcData::HouseList(
                HouseList {
                    land_id: 0,
                    ward: 0,
                    territory_type_id: lua_zone.zone_id,
                    world_id: self.config.world_id,
                    subdivision: 257, // TODO: Figure out more about subdivisions
                    houses,
                },
            )))
            .await;

            // Finally, populate the exterior furniture.
            // TODO: Actually send some real furniture, once we can do that!
            for index in 0..15 {
                self.send_ipc_self(ServerZoneIpcSegment::new(ServerZoneIpcData::FurnitureList(
                    FurnitureList {
                        count: 15,
                        index,
                        ..Default::default()
                    },
                )))
                .await;
            }
        }

        if lua_zone.intended_use == TerritoryIntendedUse::HousingIndoor {
            // Bare minimum stuff to make housing interiors load
            self.send_ipc_self(ServerZoneIpcSegment::new(
                ServerZoneIpcData::HousingInteriorDetails(HousingInteriorDetails::default()),
            ))
            .await;

            // TODO: Limit this by interior size once we have furniture persistence
            for index in 0..6 {
                // The LandId is currently set so that plugins like HousingPos/Buildingway can plop stuff down
                self.send_ipc_self(ServerZoneIpcSegment::new(ServerZoneIpcData::FurnitureList(
                    FurnitureList {
                        id: HouseId {
                            unit: HouseUnit {
                                apartment_division_plot_index: 0,
                                apartment_flag: true,
                            },
                            unk1: 0,
                            room_number: 1,
                            ward_index: 0,
                            territory_type_id: 340,
                            world_id: self.config.world_id,
                        },
                        count: 6,
                        index,
                        unk2: 100, // Indoors
                        ..Default::default()
                    },
                )))
                .await;
            }
        }

        self.conditions
            .toggle_condition(Condition::BoundByDuty, bound_by_duty);
        self.conditions
            .toggle_condition(Condition::BoundByDuty56, bound_by_duty);
        // No one arrives in a zone fighting or casting: the new zone's enemies set InCombat again if any are after
        // them. A player who lost a quest battle came out with InCombat still set (the server's copy of their
        // conditions starts empty in the new zone, so nothing there ever cleared it), and the client refused to
        // change gear, teleport or mount out in the open (Bob's bundle 20261110-095652.)
        self.conditions.remove_condition(Condition::InCombat);
        self.conditions.remove_condition(Condition::Casting);

        self.send_conditions().await;

        self.synced_level = None;
        self.synced_item_level = None;
        self.minimum_item_level = None;
        self.echo_multiplier = 1.0;
        self.synced_fate = None;
        self.match_class_to_equipment();
        self.update_class_info().await;
        self.send_stats().await;

        // Inform the server we're ready for director data
        self.handle
            .send(ToServer::ReadyDirectorData(
                self.player_data.character.actor_id,
            ))
            .await;

        self.initial_login = false;
    }

    /// On retail, the Teleport general action unlocks once you've attuned to a second Aetheryte (shards don't count.)
    /// Unlocks it then; `notify` tells the client straight away (otherwise it's in PlayerSetup, at login.)
    pub async fn check_teleport_unlock(&mut self, notify: bool) {
        let (link, attuned) = {
            let mut game_data = self.game_data.lock();
            let Some(link) = game_data.get_teleport_unlock_link() else {
                return;
            };
            let unlocked = &self.player_data.aetheryte.unlocked;
            (
                link,
                game_data.count_main_aetherytes(|id| unlocked.contains(id)),
            )
        };
        if attuned < 2 || self.player_data.unlock.unlocks.contains(link) {
            return;
        }

        tracing::info!("Unlocking Teleport (unlock link {link}): {attuned} aetherytes attuned.");
        self.player_data.unlock.unlocks.set(link);
        if notify {
            self.actor_control_self(ActorControlCategory::ToggleUnlock {
                id: link,
                unlocked: true,
            })
            .await;
        }
    }

    pub async fn warp(&mut self, warp_id: u32) {
        self.teleport_reason = TeleportReason::NotSpecified;
        self.handle
            .send(ToServer::Warp(
                self.id,
                self.player_data.character.actor_id,
                warp_id,
            ))
            .await;
    }

    pub async fn warp_aetheryte(
        &mut self,
        aetheryte_id: u32,
        housing_aethernet: bool,
        taking_offered_teleport: bool,
        warp_type: WarpType,
    ) {
        self.teleport_reason = TeleportReason::Aetheryte;
        self.handle
            .send(ToServer::WarpAetheryte(
                self.id,
                self.player_data.character.actor_id,
                aetheryte_id,
                housing_aethernet,
                warp_type,
            ))
            .await;

        if self.can_offer_teleport() {
            let teleport_info = TeleportQuery {
                aetheryte_id: aetheryte_id as u16,
            };

            self.handle
                .send(ToServer::OfferTeleportToParty(
                    Some(self.party_id),
                    self.player_data.character.actor_id,
                    self.player_data.volatile.zone_id as u16,
                    teleport_info,
                ))
                .await;
            // Reset it so we don't inadvertently share an invalid aetheryte (housing/city aethernet shard)
            self.can_share_teleport = false;
        }

        if taking_offered_teleport {
            self.offered_teleport = None;
        }
    }

    /// What teleporting from here to this aetheryte costs: free to the character's free destination, half to a
    /// favourite (retail; see `GameData::teleport_cost`.)
    pub fn teleport_cost(&self, aetheryte_id: u32) -> u32 {
        let aetheryte = &self.player_data.aetheryte;
        if aetheryte.free_aetheryte != 0 && aetheryte.free_aetheryte as u32 == aetheryte_id {
            return 0;
        }
        let favourite = aetheryte
            .favorite_aetherytes
            .0
            .iter()
            .any(|favourite| *favourite as u32 == aetheryte_id);
        self.game_data.lock().teleport_cost(
            self.player_data.volatile.zone_id as u16,
            aetheryte_id,
            favourite,
        )
    }

    /// Takes a teleport's gil as it lands, as retail: the gil slot, then the log line (LogMessage 4590, the cost.)
    pub async fn pay_for_teleport(&mut self, aetheryte_id: u32) {
        let cost = self.teleport_cost(aetheryte_id);
        if cost == 0 {
            return;
        }
        let gil = &mut self.player_data.inventory.currency.gil;
        gil.quantity = gil.quantity.saturating_sub(cost as i32).max(0);
        self.send_inventory().await;
        self.actor_control_self(ActorControlCategory::LogMessage {
            log_message: TELEPORT_PAID_LOG,
            id: cost,
        })
        .await;
    }

    pub async fn change_weather(&mut self, new_weather_id: u8) {
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::WeatherId(WeatherChange {
            weather_id: new_weather_id,
            fade_length: 1.0,
            ..Default::default()
        }));
        self.send_ipc_self(ipc).await;
    }

    /// Where a map's discovered parts are kept in the discovery bitmask: the byte they start at, and how many parts
    /// there's room for.
    fn map_discovery_slot(&self, map_id: u32) -> Option<(usize, u32)> {
        let (index, two_bytes) = self.game_data.lock().get_map_discovery_index(map_id)?;
        if index < 0 {
            return None;
        }
        Some(if two_bytes {
            (index as usize * 2, 16)
        } else {
            (
                DISCOVERY_MAPS_WITH_UP_TO_16_REGIONS_BITMASK_SIZE * 2 + index as usize * 4,
                32,
            )
        })
    }

    /// The parts of a map discovered so far, as bits (part n is bit n, like Map's DiscoveryFlag.)
    pub fn discovered_map_parts(&self, map_id: u32) -> u32 {
        let Some((offset, width)) = self.map_discovery_slot(map_id) else {
            return 0;
        };
        let discovered = &self.player_data.unlock.map_discovery.data;
        (0..width / 8)
            .map(|byte| discovered.get(offset + byte as usize).copied().unwrap_or(0) as u32)
            .enumerate()
            .fold(0, |parts, (byte, bits)| parts | (bits << (byte * 8)))
    }

    pub async fn discover_location(&mut self, map_id: u32, map_part_id: u32) {
        // Remember it, so the map stays discovered after logging out (sent back in PlayerSetup.)
        // The whole map at once (A Realm Reborn's field zones) is every part in Map's DiscoveryFlag.
        let parts = if map_part_id == crate::common::WHOLE_MAP_DISCOVERY {
            self.game_data
                .lock()
                .get_map_discovery_flag(map_id)
                .unwrap_or(0)
        } else if map_part_id < 32 {
            1 << map_part_id
        } else {
            0
        };
        match self.map_discovery_slot(map_id) {
            Some((offset, width)) => {
                let discovered = &mut self.player_data.unlock.map_discovery.data;
                discovered.resize(MAP_DISCOVERY_BITMASK_SIZE, 0);
                if parts != 0
                    && (width == 32 || parts >> width == 0)
                    && offset + (width as usize / 8) <= discovered.len()
                {
                    for byte in 0..width as usize / 8 {
                        discovered[offset + byte] |= (parts >> (byte * 8)) as u8;
                    }
                } else {
                    tracing::warn!(
                        "Map {map_id} part {map_part_id} doesn't fit its discovery bitmask, it won't be remembered"
                    );
                }
            }
            _ => tracing::warn!("Map {map_id} has no discovery index, it won't be remembered"),
        }

        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::LocationDiscovered {
            map_id,
            map_part_id,
        });
        self.send_ipc_self(ipc).await;

        // The whole map discovered completes its "Mapping the Realm" achievement.
        self.check_achievements(true).await;
    }

    pub async fn join_content(&mut self, id: u16, settings: DutyFinderSetting) {
        self.entered_with = Some(settings);
        self.entered_roulette = self.queued_roulette.take();
        // Store our old information, for when we leave the instance
        self.old_zone_id = self.player_data.volatile.zone_id as u16;
        self.old_position = self.player_data.volatile.position;
        self.old_rotation = self.player_data.volatile.rotation as f32;

        self.handle
            .send(ToServer::JoinContent(
                self.id,
                self.player_data.character.actor_id,
                id,
                settings,
                self.duty_support.take(),
            ))
            .await;
    }

    /// Enters a quest battle, `zone_id` being the territory it takes place in.
    pub async fn join_quest_battle(&mut self, quest_battle_id: u32, zone_id: u16) {
        // Store our old information, for when we leave the instance
        self.old_zone_id = self.player_data.volatile.zone_id as u16;
        self.old_position = self.player_data.volatile.position;
        self.old_rotation = self.player_data.volatile.rotation as f32;

        self.handle
            .send(ToServer::JoinQuestBattle(
                self.id,
                self.player_data.character.actor_id,
                quest_battle_id,
                zone_id,
            ))
            .await;
    }

    /// Ensure the player is placed in a valid zone, and if they aren't they are teleported back to their homepoint.
    pub async fn ensure_valid_zone(&mut self) {
        let zone_id = self.player_data.volatile.zone_id;

        // If the player isn't in a valid zone, or in instanced content (both crash the game) then we need to reset them.
        let should_reset;
        {
            let mut game_data = self.game_data.lock();
            should_reset = !game_data.is_zone_valid(zone_id as u16)
                || game_data.is_zone_associated_with_content(zone_id as u16);
        }
        if should_reset {
            // TODO: teleport them to their homepoint instead
            self.player_data.volatile.zone_id = 132;
            self.player_data.volatile.position = Position::default();

            self.send_notice("Moved you to a safe area to prevent a crash!")
                .await;
        }
    }

    /// Changes the time of day in the current zone for our client.
    pub async fn set_eorzean_time(&mut self, offset: i64) {
        self.send_ipc_self(ServerZoneIpcSegment::new(
            ServerZoneIpcData::EorzeanTimeOffset { offset },
        ))
        .await;
    }
}

/// LogMessage for a teleport's gil (retail's, with the cost.)
const TELEPORT_PAID_LOG: u32 = 4590;
