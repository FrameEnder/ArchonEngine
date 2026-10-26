//! A synthesis, from the Crafting Log's Synthesize to the item (or the materials lost.)
//!
//! Everything here follows Bob's Chronofoil capture of 2026-10-24 (a level 71–72 Carpenter: four syntheses, one
//! quick synthesis, a failure and quits), with the client's code for what the packets mean
//! (docs/src/fork/crafting.md.) The rules of each step are `crate::crafting`'s.

use async_trait::async_trait;
use kawari::{
    common::{CharacterMode, ContainerType, INVENTORY_ACTION_ACK_SHOP},
    constants::COMPLETED_RECIPES_BITMASK_SIZE,
    ipc::zone::{
        ActorControlCategory, Condition, CraftingActionPreview, EventLogMessage, LiveEventType,
        SceneFlags, ServerZoneIpcData, ServerZoneIpcSegment,
    },
};

use crate::{
    Event, EventHandler, ItemInfoQuery, ToServer, ZoneConnection,
    crafting::{Craft, CraftAction, CraftSettings, StepOutcome, initial_quality},
    gamedata::RecipeInfo,
    inventory::Item,
    lua::LuaPlayer,
};

/// An item's HQ flag (`Item::item_flags`.)
const ITEM_FLAG_HQ: u8 = 1;
/// The client gives HQ items as their ID plus this (the materials list, log messages.)
const HQ_ID_OFFSET: u32 = 1_000_000;

/// What the client sends (its first result), as yields and returns of scene 0.
const CLIENT_MATERIALS_CHOSEN: i32 = 0; // a return: [0, NQ of slots 0..5, HQ of slots 0..5]
const CLIENT_MATERIALS_LIST: i32 = 1; // a return of 77 results: the stacks the synthesis takes
const CLIENT_BACK_TO_LOG: i32 = 3; // after a failure's scene 8
const CLIENT_SYNTHESIZE_AGAIN: i32 = 4; // [4, recipe, 0, 0] from the Crafting Log
const CLIENT_QUICK_SYNTHESIS: i32 = 5; // [5, recipe, how many, ?]
const CLIENT_QUIT: i32 = 7; // the log's (or the synthesis') Quit: [7, synthesis under way, …]
const CLIENT_QUIT_DONE: i32 = 8; // after scene 6
const CLIENT_ACTION: i32 = 9; // an Action row (the buffs), which also comes as an ActionRequest
const CLIENT_CRAFT_ACTION: i32 = 10; // a CraftAction row
const CLIENT_COMPLETE: i32 = 11; // after a step's StateFlags 0x4
const CLIENT_FAILED: i32 = 12; // after StateFlags 0x8
const CLIENT_QUICK_MATERIALS_LIST: i32 = 13; // the list again, for a quick synthesis

/// The LiveEvent actor controls retail sent (their numbers; no names known for most.)
const LIVE_CONDITION: u32 = 10; // the condition's look: the condition plus one (2 Normal, 3 Good)
const LIVE_BACK_TO_LOG: u32 = 11;
const LIVE_COMPLETE_NQ: u32 = 15;
const LIVE_COMPLETE_HQ: u32 = 16;
const LIVE_COMPLETE_HQ_AFTER: u32 = 17;
const LIVE_QUICK_RESULT: u32 = 18; // [item, how many made so far]
const LIVE_FAILED: u32 = 20;
const LIVE_QUICK_START: u32 = 21;
const LIVE_QUIT: u32 = 40;
/// StartCraft's second argument, the same in every synthesis of the capture (meaning unknown.)
const START_CRAFT_UNKNOWN: u32 = 136_153;

/// LogMessage rows the craft event sent with its EventLogMessages.
const LOG_SYNTHESIS_DONE: u32 = 1156; // the item
const LOG_ITEM_MADE: u32 = 1158; // the item
const LOG_QUICK_SYNTHESIS_STARTED: u32 = 1150; // the item
const LOG_FAILED: u32 = 1177;
const LOG_MATERIALS_LOST: u32 = 1160;
const LOG_RECORDED_IN_LOG: u32 = 1178; // the item (NQ); a recipe's first synthesis

/// A synthesis in progress, kept on the connection.
#[derive(Debug)]
pub struct CraftSession {
    pub recipe: RecipeInfo,
    pub craft: Craft,
    /// The stacks the client listed: container, slot, item, HQ, how many. Taken when the synthesis ends (done,
    /// failed or quit), as retail did.
    materials: Vec<(ContainerType, u16, u32, bool, u32)>,
    /// How many a quick synthesis makes (None for a normal synthesis.)
    quick: Option<u32>,
    rng: fastrand::Rng,
}

/// For crafting events.
#[derive(Debug)]
pub struct CraftingEventHandler;

impl Default for CraftingEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl CraftingEventHandler {
    pub fn new() -> Self {
        Self {}
    }
}

/// The scene a step answers with (EventScene64, 33 params; params 1–3 and 18 were 0 on retail.)
fn step_params(
    action_id: u32,
    craft_action: bool,
    craft: &Craft,
    outcome: &StepOutcome,
) -> Vec<u32> {
    let mut params = vec![0u32; 33];
    // The server's 9 answers the client's 10 (a CraftAction row) and its 10 the client's 9 (an Action row.)
    params[0] = if craft_action { 9 } else { 10 };
    params[4] = action_id;
    params[5] = outcome.cp_recovered.max(0) as u32;
    params[6] = craft.display_step();
    params[7] = craft.progress.min(craft.settings.max_progress).max(0) as u32;
    params[8] = outcome.progress_delta as u32;
    params[9] = craft.quality.min(craft.settings.max_quality).max(0) as u32;
    params[10] = outcome.quality_delta as u32;
    params[11] = craft.hq_chance();
    params[12] = craft.durability.max(0) as u32;
    params[13] = outcome.durability_delta as u32;
    params[14] = craft.condition as u32;
    params[15] = outcome.used_condition as u32;
    // 150 when the next condition is Good, else 100 (retail.)
    params[16] = if craft.condition == crate::crafting::CraftCondition::Good {
        150
    } else {
        100
    };
    params[17] = craft.state_flags(outcome.success);
    for (i, (status, param)) in outcome.worn_off.iter().take(7).enumerate() {
        params[19 + i] = *status as u32;
        params[26 + i] = *param as u32;
    }
    params
}

/// The item the synthesis under way makes.
fn recipe_item(connection: &ZoneConnection) -> u32 {
    connection
        .craft
        .as_ref()
        .map(|session| session.recipe.item_id)
        .unwrap_or_default()
}

/// The bag a materials list names (its u16 container): the bags by number, and 255 for the crystals (retail.)
fn material_container(container: u32) -> Option<ContainerType> {
    match container {
        0 => Some(ContainerType::Inventory0),
        1 => Some(ContainerType::Inventory1),
        2 => Some(ContainerType::Inventory2),
        3 => Some(ContainerType::Inventory3),
        255 | 2001 => Some(ContainerType::Crystals),
        _ => None,
    }
}

async fn live_event(
    connection: &mut ZoneConnection,
    event: u32,
    param1: u32,
    param2: u32,
    param3: u32,
) {
    connection
        .broadcast_actor_control(ActorControlCategory::LiveEvent {
            event: LiveEventType::Unknown {
                event,
                param1,
                param2,
                param3,
            },
        })
        .await;
}

async fn log_message(
    connection: &mut ZoneConnection,
    event: &Event,
    message_type: u32,
    params: Vec<u32>,
) {
    let message = EventLogMessage {
        handler_id: event.id,
        message_type,
        params,
    };
    if let Some(ipc) = message.package() {
        connection.send_ipc_self(ipc).await;
    }
}

/// The crafter's CP and synthesis statuses.
async fn send_state(connection: &mut ZoneConnection, cp: i32, statuses: Vec<(u16, u16)>) {
    connection
        .handle
        .send(ToServer::SetCraftingState(
            connection.id,
            connection.player_data.character.actor_id,
            cp.max(0) as u16,
            statuses,
        ))
        .await;
}

/// The crafter's tools as LiveEvent 38 and 39 show them (the weapon models' two halves.) Retail's off hand was always
/// empty (39 with 0, a secondary tool equipped or not: Bob's Chronofoil capture of 2026-10-24); the client brings the
/// station itself, and with the main hand's sub model (the workbench) sent too there were two.
async fn show_tools(connection: &mut ZoneConnection) {
    let (main, sub) = {
        let mut game_data = connection.game_data.lock();
        let inventory = &connection.player_data.inventory;
        let classjob = connection.player_data.classjob.current_class as u8;
        (
            u64::from(inventory.get_main_weapon_id(&mut game_data)),
            u64::from(inventory.get_shown_sub_weapon_id(&mut game_data, classjob)),
        )
    };
    connection
        .broadcast_actor_control(ActorControlCategory::LiveEvent {
            event: LiveEventType::SetMainHand {
                model_id: main as u32,
                unk1: (main >> 32) as u32,
                unk2: 0,
            },
        })
        .await;
    connection
        .broadcast_actor_control(ActorControlCategory::LiveEvent {
            event: LiveEventType::SetOffHand {
                model_id: sub as u32,
                unk1: (sub >> 32) as u32,
                unk2: 0,
            },
        })
        .await;
}

/// Sets the conditions of a synthesis (ExecutingCraftingAction and 73) or of the Crafting Log (PreparingToCraft.)
async fn set_synthesizing(connection: &mut ZoneConnection, synthesizing: bool) {
    connection
        .conditions
        .toggle_condition(Condition::ExecutingCraftingAction, synthesizing);
    connection
        .conditions
        .toggle_condition(Condition::Synthesizing, synthesizing);
    connection
        .conditions
        .toggle_condition(Condition::PreparingToCraft, !synthesizing);
    connection.send_conditions().await;
}

/// Starts a synthesis of `connection.recipe` (`variant`: 0 from Synthesize, 3 Synthesize Again, 4 a quick
/// synthesis): the tools, StartCraft and scene 1 (`[1, 0, item, 0]`), which the client answers with the materials
/// chosen.
pub async fn begin_synthesis(connection: &mut ZoneConnection, event: &Event, variant: u32) {
    let Some(recipe) = connection.recipe else {
        return;
    };
    if variant != 0 {
        connection
            .set_character_mode(CharacterMode::Crafting, 0, true)
            .await;
        set_synthesizing(connection, true).await;
    }
    show_tools(connection).await;
    connection
        .broadcast_actor_control(ActorControlCategory::LiveEvent {
            event: LiveEventType::StartCraft {
                unk1: variant,
                unk2: START_CRAFT_UNKNOWN,
                unk3: 1,
            },
        })
        .await;
    connection
        .event_scene(
            event,
            0,
            SceneFlags::NO_DEFAULT_CAMERA,
            vec![1, 0, recipe.item_id.max(0) as u32, 0],
        )
        .await;
    // CP full and no statuses (retail's StatusEffectList after scene 1.)
    let cp = connection.base_parameters().cp as i32;
    send_state(connection, cp, Vec::new()).await;
}

impl CraftingEventHandler {
    /// The previews of what each action would do now (CraftingActionPreviews, before the window opens and with every
    /// step.)
    async fn send_previews(connection: &mut ZoneConnection) {
        let Some(session) = connection.craft.as_ref() else {
            return;
        };
        let previews = session.craft.previews();
        let class_job = connection.player_data.classjob.current_class as u8;
        let mut actions = [CraftingActionPreview::default(); 21];
        {
            let mut game_data = connection.game_data.lock();
            for (line, preview) in actions.iter_mut().zip(previews) {
                *line = CraftingActionPreview {
                    action_id: game_data
                        .craft_action_for_class(preview.action.base_id(), class_job),
                    progress_efficiency: preview.progress_efficiency,
                    quality_efficiency: preview.quality_efficiency,
                    progress: preview.progress,
                    quality: preview.quality,
                    state: preview.state,
                };
            }
        }
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::CraftingActionPreviews { actions });
        connection.send_ipc_self(ipc).await;
    }

    /// Takes the listed stacks out of the bags (once per synthesis made.)
    /// A synthesis ended, done or failed: the gear wears and bonds by the item made's item level (retail sent the worn
    /// pieces' slots after each synthesis; `crate::spiritbond`.)
    async fn wear_gear(connection: &mut ZoneConnection, item_id: u32) {
        let crafted_item_level = connection
            .game_data
            .lock()
            .get_item_info(ItemInfoQuery::ById(item_id))
            .map(|info| info.item_level)
            .unwrap_or_default();
        connection
            .use_gear(crate::GearUse::Synthesis { crafted_item_level })
            .await;
    }

    /// Returns the slots it changed.
    fn take_materials(connection: &mut ZoneConnection, times: u32) -> Vec<(ContainerType, u16)> {
        let Some(session) = connection.craft.as_ref() else {
            return Vec::new();
        };
        let materials = session.materials.clone();
        let mut changed = Vec::new();
        for _ in 0..times {
            for (container, slot, item_id, hq, amount) in &materials {
                let Some(stack) = connection
                    .player_data
                    .inventory
                    .get_item_mut(*container, *slot)
                else {
                    continue;
                };
                if stack.item_id != *item_id
                    || (stack.item_flags & ITEM_FLAG_HQ != 0) != *hq
                    || stack.quantity < *amount as i32
                {
                    tracing::warn!(
                        "The materials list has {amount} of {item_id} in {container:?} {slot}, which isn't there!"
                    );
                    continue;
                }
                stack.quantity -= *amount as i32;
                // Crystals keep their slot (and item ID) when there are none left.
                if stack.quantity == 0 && *container != ContainerType::Crystals {
                    *stack = Item::default();
                }
                if !changed.contains(&(*container, *slot)) {
                    changed.push((*container, *slot));
                }
            }
        }
        changed
    }

    /// Sends the slots a synthesis changed, one UpdateInventorySlot each, as retail did (Bob's Chronofoil capture of
    /// 2026-10-24: the materials' and crystals' slots, InventoryActionAck 6, then the made item's slot.) An emptied
    /// slot goes as an empty item: the client doesn't apply a whole-inventory resend (ItemInfo and ContainerInfo) to
    /// the slots it already has, so with that an emptied stack stayed in its bags (Bob's 2026-10-26 bundles.)
    async fn send_slots(connection: &mut ZoneConnection, slots: &[(ContainerType, u16)]) {
        for (container, slot) in slots {
            connection.send_inventory_slot(*container, *slot).await;
            connection.player_data.item_sequence += 1;
        }
    }

    /// A recipe's first synthesis marks it done in the Crafting Log (which a quick synthesis needs): the byte of the
    /// completed recipes bitmask (RecordCraftingLog) and LogMessage 1178 with the item, as retail sent them after the
    /// synthesis' messages.
    async fn record_in_crafting_log(
        connection: &mut ZoneConnection,
        event: &Event,
        recipe: &RecipeInfo,
    ) {
        let (_, index) = kawari::common::value_to_flag_byte_index_value_quests(recipe.id);
        let completed = &mut connection.player_data.quest.completed_recipes;
        // Saved before recipes were (the column's default, `[]`): as long as the client's bitmask, all unset.
        if completed.data.len() < COMPLETED_RECIPES_BITMASK_SIZE {
            completed.data.resize(COMPLETED_RECIPES_BITMASK_SIZE, 0);
        }
        if (index as usize) >= completed.data.len() || completed.contains(recipe.id) {
            return;
        }
        completed.set(recipe.id);
        let Some(value) = completed.data.get(index as usize).copied() else {
            return;
        };
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::RecordCraftingLog { index, value });
        connection.send_ipc_self(ipc).await;
        log_message(connection, event, LOG_RECORDED_IN_LOG, vec![recipe.item_id]).await;
    }

    /// Gives the made item: HQ or not, with the crafter's signature. Returns the slot it went into.
    fn give_item(
        connection: &mut ZoneConnection,
        item_id: u32,
        amount: u32,
        hq: bool,
    ) -> Option<(ContainerType, u16)> {
        let info = connection
            .game_data
            .lock()
            .get_item_info(ItemInfoQuery::ById(item_id))?;
        let mut item = Item::new(&info, amount as i32);
        if hq {
            item.item_flags |= ITEM_FLAG_HQ;
        }
        item.crafter_content_id = connection.player_data.character.content_id as u64;
        let slot = connection.player_data.inventory.add_obtained_item_at(item);
        if slot.is_none() {
            tracing::warn!("No room for the {item_id} just made!");
        }
        slot
    }

    /// The materials taken, the ack, then the item made (retail's order.)
    async fn send_synthesis_slots(
        connection: &mut ZoneConnection,
        taken: &[(ContainerType, u16)],
        made: Option<(ContainerType, u16)>,
    ) {
        Self::send_slots(connection, taken).await;
        connection
            .send_inventory_ack(u32::MAX, INVENTORY_ACTION_ACK_SHOP, 0)
            .await;
        Self::send_slots(connection, made.as_slice()).await;
    }

    /// The EXP for a synthesis. A stand-in fitted to the capture: 0.72 % of the recipe level's ExpToNext (21,837
    /// for a level 71 recipe, made by a level 71 or 72 Carpenter, before rested EXP), up to 3.125 times that at full
    /// quality (68,250), and less for a recipe far below the crafter (a level 50 one at 72 gave 923, 30 %.)
    fn synthesis_exp(
        connection: &ZoneConnection,
        recipe: &RecipeInfo,
        quality: i32,
        max_quality: i32,
        level: u8,
    ) -> i32 {
        if !recipe.exp_rewarded {
            return 0;
        }
        let to_next = connection
            .game_data
            .lock()
            .get_max_exp(recipe.class_job_level.max(1) as u32) as f64;
        let below = (level as f64 - recipe.class_job_level as f64 - 5.0).max(0.0);
        let penalty = (1.0 - below * 0.041).max(0.05);
        let quality = quality.clamp(0, max_quality.max(1)) as f64 / max_quality.max(1) as f64;
        (to_next * 0.007_235_6 * penalty * (1.0 + 2.125 * quality)).round() as i32
    }

    /// Back to the Crafting Log: its conditions, LiveEvent 11, scene 4, CP full and the statuses off.
    async fn back_to_log(connection: &mut ZoneConnection, player: &mut LuaPlayer) {
        let cp = connection
            .craft
            .take()
            .map(|session| session.craft.settings.max_cp)
            .unwrap_or_else(|| connection.base_parameters().cp as i32);
        set_synthesizing(connection, false).await;
        live_event(connection, LIVE_BACK_TO_LOG, 0, 0, 0).await;
        player.play_scene(0, SceneFlags::NO_DEFAULT_CAMERA, vec![4, 0, 0, 0]);
        send_state(connection, cp, Vec::new()).await;
    }

    async fn handle(
        &self,
        event: &Event,
        connection: &mut ZoneConnection,
        results: &[i32],
        player: &mut LuaPlayer,
    ) {
        let Some(command) = results.first().copied() else {
            return;
        };
        let result = |i: usize| results.get(i).copied().unwrap_or_default();

        match command {
            CLIENT_MATERIALS_CHOSEN => {
                let Some(recipe) = connection.recipe else {
                    return;
                };
                let level = connection.current_level(&connection.game_data.lock()) as u8;
                let info = connection
                    .game_data
                    .lock()
                    .get_recipe_info(recipe.id, level);
                let Some(info) = info else {
                    tracing::warn!("Recipe {} isn't in the game data!", recipe.id);
                    player.finish_event();
                    return;
                };

                let ingredients: Vec<_> = info
                    .ingredients
                    .iter()
                    .take(6)
                    .enumerate()
                    .map(|(i, ingredient)| {
                        (
                            ingredient.amount,
                            ingredient.item_level,
                            ingredient.can_be_hq,
                            result(7 + i).max(0) as u32,
                        )
                    })
                    .collect();
                let start_quality =
                    initial_quality(info.max_quality, info.material_quality_factor, &ingredients);

                let parameters = connection.base_parameters();
                let settings = CraftSettings::new(
                    &info,
                    level,
                    parameters.craftmanship,
                    parameters.control,
                    parameters.cp,
                );
                let craft = Craft::new(settings, start_quality as i32);
                let start_hq = craft.hq_chance();
                let quick = connection.quick_synthesis.take();
                connection.craft = Some(Box::new(CraftSession {
                    recipe: info,
                    craft,
                    materials: Vec::new(),
                    quick,
                    rng: fastrand::Rng::new(),
                }));

                match quick {
                    // A quick synthesis: scene 12 with the recipe and how many.
                    Some(count) => player.play_scene(
                        0,
                        SceneFlags::NO_DEFAULT_CAMERA,
                        vec![12, recipe.id, count, 0],
                    ),
                    None => player.play_scene(
                        0,
                        SceneFlags::NO_DEFAULT_CAMERA,
                        vec![2, recipe.id, start_quality, start_hq],
                    ),
                }
            }
            CLIENT_MATERIALS_LIST | CLIENT_QUICK_MATERIALS_LIST => {
                // u32 command, u32 item[30] (HQ + 1,000,000), then u16 amount[30], container[30], slot[30] (15
                // words each), then u32 count.
                let word = |i: usize| result(i) as u32;
                let half = |base: usize, i: usize| {
                    let w = word(base + i / 2);
                    if i.is_multiple_of(2) {
                        w & 0xFFFF
                    } else {
                        w >> 16
                    }
                };
                let count = (word(76) as usize).min(30);
                let mut materials = Vec::new();
                for i in 0..count {
                    let id = word(1 + i);
                    let (item_id, hq) = if id > HQ_ID_OFFSET {
                        (id - HQ_ID_OFFSET, true)
                    } else {
                        (id, false)
                    };
                    let Some(container) = material_container(half(46, i)) else {
                        tracing::warn!(
                            "A material is in container {}, which isn't a bag!",
                            half(46, i)
                        );
                        continue;
                    };
                    materials.push((container, half(61, i) as u16, item_id, hq, half(31, i)));
                }
                let Some(session) = connection.craft.as_mut() else {
                    return;
                };
                session.materials = materials;

                if command == CLIENT_MATERIALS_LIST {
                    live_event(connection, LIVE_CONDITION, 2, 0, 0).await;
                    Self::send_previews(connection).await;
                    player.play_scene(0, SceneFlags::NO_DEFAULT_CAMERA, vec![3, 0, 0, 0]);
                    return;
                }

                // A quick synthesis: every item at once (retail sent one every two seconds), NQ, with the EXP of a
                // synthesis of no quality.
                let (item_id, amount, count) = (
                    session.recipe.item_id,
                    session.recipe.amount_result,
                    session.quick.unwrap_or(1),
                );
                let recipe = session.recipe.clone();
                let max_quality = session.craft.settings.max_quality;
                log_message(
                    connection,
                    event,
                    LOG_QUICK_SYNTHESIS_STARTED,
                    vec![item_id],
                )
                .await;
                live_event(connection, LIVE_CONDITION, 2, 0, 0).await;
                live_event(connection, LIVE_QUICK_START, 0, 0, 0).await;
                let level = connection.current_level(&connection.game_data.lock()) as u8;
                for made in 1..=count {
                    let taken = Self::take_materials(connection, 1);
                    let slot = Self::give_item(connection, item_id, amount, false);
                    log_message(connection, event, LOG_ITEM_MADE, vec![item_id]).await;
                    Self::send_synthesis_slots(connection, &taken, slot).await;
                    live_event(connection, LIVE_QUICK_RESULT, item_id, made, 0).await;
                    let exp = Self::synthesis_exp(connection, &recipe, 0, max_quality, level);
                    if exp > 0 {
                        connection.add_exp_with_bonus(exp, 0).await;
                    }
                }
                Self::back_to_log(connection, player).await;
            }
            CLIENT_ACTION | CLIENT_CRAFT_ACTION => {
                let action_id = result(1) as u32;
                let craft_action = command == CLIENT_CRAFT_ACTION;
                let Some(action) = CraftAction::from_id(action_id) else {
                    tracing::warn!("Crafting action {action_id} isn't one the synthesis knows!");
                    player.play_scene(0, SceneFlags::NO_DEFAULT_CAMERA, vec![7, 0, 0, 0]);
                    return;
                };
                let Some(session) = connection.craft.as_mut() else {
                    return;
                };
                let CraftSession { craft, rng, .. } = session.as_mut();
                let condition_before = craft.condition;
                let statuses_before = craft.statuses();
                let Some(outcome) = craft.use_action(action, rng) else {
                    // Not usable now (the client normally greys these out): scene 7 lets it act again.
                    tracing::info!("{action:?} can't be used now");
                    player.play_scene(0, SceneFlags::NO_DEFAULT_CAMERA, vec![7, 0, 0, 0]);
                    return;
                };
                let params = step_params(action_id, craft_action, craft, &outcome);
                let cp = craft.cp;
                let statuses = craft.statuses();
                let condition = craft.condition;

                // CraftAction rows have their own animation; the buffs' Action rows are the action pipeline's.
                if craft_action {
                    let (start, end) = connection
                        .game_data
                        .lock()
                        .get_craft_action_animations(action_id);
                    connection
                        .broadcast_actor_control(ActorControlCategory::LiveEvent {
                            event: LiveEventType::PlayAnimation {
                                animation_start: start as u32,
                                animation_end: end as u32,
                            },
                        })
                        .await;
                    // Statuses gained or changed by it (Inner Quiet's stacks, Muscle Memory…), as retail announced them.
                    let actor_id = connection.player_data.character.actor_id;
                    for (status, param) in &statuses {
                        if !statuses_before.contains(&(*status, *param)) {
                            connection
                                .broadcast_actor_control(ActorControlCategory::GainEffect {
                                    effect_id: *status as u32,
                                    param: *param as u32,
                                    source_actor_id: actor_id,
                                })
                                .await;
                        }
                    }
                }
                if condition != condition_before {
                    live_event(connection, LIVE_CONDITION, condition as u32 + 1, 0, 0).await;
                }
                send_state(connection, cp, statuses).await;
                Self::send_previews(connection).await;
                player.play_scene(0, SceneFlags::NO_DEFAULT_CAMERA, params);
            }
            CLIENT_COMPLETE => {
                let Some(session) = connection.craft.as_mut() else {
                    return;
                };
                let hq =
                    session.recipe.can_hq && session.rng.u32(0..100) < session.craft.hq_chance();
                let recipe = session.recipe.clone();
                let quality = session.craft.quality;
                let max_quality = session.craft.settings.max_quality;
                let level = session.craft.settings.level;

                let taken = Self::take_materials(connection, 1);
                let slot = Self::give_item(connection, recipe.item_id, recipe.amount_result, hq);
                Self::send_synthesis_slots(connection, &taken, slot).await;
                Self::wear_gear(connection, recipe.item_id).await;
                live_event(
                    connection,
                    if hq {
                        LIVE_COMPLETE_HQ
                    } else {
                        LIVE_COMPLETE_NQ
                    },
                    0,
                    0,
                    0,
                )
                .await;
                let shown_id = if hq {
                    recipe.item_id + HQ_ID_OFFSET
                } else {
                    recipe.item_id
                };
                log_message(connection, event, LOG_ITEM_MADE, vec![shown_id]).await;
                log_message(connection, event, LOG_SYNTHESIS_DONE, vec![shown_id]).await;
                Self::record_in_crafting_log(connection, event, &recipe).await;
                let exp = Self::synthesis_exp(connection, &recipe, quality, max_quality, level);
                if exp > 0 {
                    connection.add_exp_with_bonus(exp, 0).await;
                }
                if hq {
                    live_event(connection, LIVE_COMPLETE_HQ_AFTER, 0, 0, 0).await;
                }
                // Retail went back to the log 1.5–3 s later (the finish animation); here at once.
                Self::back_to_log(connection, player).await;
            }
            CLIENT_FAILED => {
                // Every material is lost (retail: the stacks emptied, scene 8 listing nothing back.)
                live_event(connection, LIVE_FAILED, 0, 0, 0).await;
                let taken = Self::take_materials(connection, 1);
                Self::send_slots(connection, &taken).await;
                Self::wear_gear(connection, recipe_item(connection)).await;
                log_message(connection, event, LOG_FAILED, Vec::new()).await;
                log_message(connection, event, LOG_MATERIALS_LOST, Vec::new()).await;
                let mut params = vec![0u32; 22];
                params[0] = 8;
                player.play_scene(0, SceneFlags::NO_DEFAULT_CAMERA, params);
            }
            CLIENT_BACK_TO_LOG => {
                Self::back_to_log(connection, player).await;
            }
            CLIENT_SYNTHESIZE_AGAIN | CLIENT_QUICK_SYNTHESIS => {
                let id = result(1) as u32;
                if connection.recipe.map(|recipe| recipe.id) != Some(id) {
                    let recipe = connection.game_data.lock().get_recipe(id);
                    connection.recipe = Some(recipe);
                }
                let quick = (command == CLIENT_QUICK_SYNTHESIS).then(|| result(2).max(1) as u32);
                // How many a quick synthesis makes waits for the materials chosen.
                connection.craft = None;
                connection.quick_synthesis = quick;
                begin_synthesis(connection, event, if quick.is_some() { 4 } else { 3 }).await;
            }
            CLIENT_QUIT => {
                player.play_scene(0, SceneFlags::NO_DEFAULT_CAMERA, vec![6, 0, 0, 0]);
            }
            CLIENT_QUIT_DONE => {
                // Quitting a synthesis loses its materials (retail.)
                if connection.craft.is_some() {
                    let taken = Self::take_materials(connection, 1);
                    Self::send_slots(connection, &taken).await;
                }
                let cp = connection.base_parameters().cp as i32;
                connection.craft = None;
                send_state(connection, cp, Vec::new()).await;
                connection
                    .conditions
                    .remove_condition(Condition::PreparingToCraft);
                show_tools(connection).await;
                live_event(connection, LIVE_QUIT, 0, 0, 0).await;
                live_event(connection, LIVE_BACK_TO_LOG, 0, 0, 0).await;
                player.finish_event();
            }
            _ => {
                tracing::info!("Unknown crafting yield {results:?}");
            }
        }
    }
}

#[async_trait]
impl EventHandler for CraftingEventHandler {
    async fn on_yield(
        &self,
        event: &Event,
        connection: &mut ZoneConnection,
        _scene: u16,
        _yield_id: u8,
        results: &[i32],
        player: &mut LuaPlayer,
    ) {
        self.handle(event, connection, results, player).await;
    }

    /// The materials chosen and their list come as scene returns (EventReturnHandler16 and 128), the rest as yields;
    /// none of them ends the event.
    async fn on_return(
        &self,
        event: &Event,
        connection: &mut ZoneConnection,
        _scene: u16,
        results: &[i32],
        player: &mut LuaPlayer,
    ) {
        self.handle(event, connection, results, player).await;
    }

    fn condition(&self) -> Condition {
        Condition::ExecutingCraftingAction
    }

    fn extra_conditions(&self) -> Vec<Condition> {
        vec![Condition::Synthesizing]
    }

    fn character_mode(&self) -> CharacterMode {
        CharacterMode::Crafting
    }
}
