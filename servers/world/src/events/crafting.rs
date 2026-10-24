//! A synthesis, from the Crafting Log's Synthesize to the item (or the materials back.)
//!
//! The packets are the client's (7.57: its craft event handler, cross-checked with Teamcraft's packet reader); no
//! retail crafting has been recorded yet, so the order at the end and the unknown params are guesses
//! (docs/src/fork/crafting.md.) The rules of each step are `crate::crafting`'s.

use async_trait::async_trait;
use kawari::{
    common::{CharacterMode, ContainerType},
    ipc::zone::{ActorControlCategory, Condition, LiveEventType, SceneFlags},
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

/// The client's yields (its first result).
const YIELD_MATERIALS_CHOSEN: i32 = 0;
const YIELD_MATERIALS_LIST: i32 = 1;
const YIELD_MATERIALS_RETURNED: i32 = 3;
const YIELD_QUIT: i32 = 7;
const YIELD_ACTION: i32 = 9;
const YIELD_CRAFT_ACTION: i32 = 10;
const YIELD_COMPLETE: i32 = 11;
const YIELD_FAILED: i32 = 12;

/// "The item was added to your inventory" sort of LogMessage, as upstream had it.
const LOG_ITEM_OBTAINED: u32 = 789;

/// A synthesis in progress, kept on the connection.
#[derive(Debug)]
pub struct CraftSession {
    pub recipe: RecipeInfo,
    pub craft: Craft,
    /// What was taken from the bags: item, HQ, how many.
    consumed: Vec<(u32, bool, u32)>,
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

/// The scene a step answers with (EventScene64, 33 params; params 1–3 and 18 unknown, 0 for now.)
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
    params[6] = craft.steps;
    params[7] = craft.progress.min(craft.settings.max_progress).max(0) as u32;
    params[8] = outcome.progress_delta as u32;
    params[9] = craft.quality.min(craft.settings.max_quality).max(0) as u32;
    params[10] = outcome.quality_delta as u32;
    params[11] = craft.hq_chance();
    params[12] = craft.durability.max(0) as u32;
    params[13] = outcome.durability_delta as u32;
    params[14] = craft.condition as u32;
    params[15] = outcome.used_condition as u32;
    // Shown in the Good condition's tooltip; upstream's recorded step had 100 under Normal.
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

/// The bag a materials list names (its u16 container), if materials can be in it.
fn material_container(container: u32) -> Option<ContainerType> {
    match container {
        0 => Some(ContainerType::Inventory0),
        1 => Some(ContainerType::Inventory1),
        2 => Some(ContainerType::Inventory2),
        3 => Some(ContainerType::Inventory3),
        2001 => Some(ContainerType::Crystals),
        _ => None,
    }
}

impl CraftingEventHandler {
    /// Sends the crafter's CP and synthesis statuses.
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

    /// The synthesis is over: CP fills up again and the statuses come off (retail: "CP regenerates fully after
    /// ending a craft".)
    async fn end_synthesis(connection: &mut ZoneConnection) {
        let cp = connection
            .craft
            .as_ref()
            .map(|session| session.craft.settings.max_cp)
            .unwrap_or_default();
        connection.craft = None;
        Self::send_state(connection, cp, Vec::new()).await;
        connection
            .broadcast_actor_control(ActorControlCategory::LiveEvent {
                event: LiveEventType::EndCraft,
            })
            .await;
    }

    /// The materials a quit or a failed synthesis gives back: each one (crystals aside) kept or lost as a coin flip,
    /// a guess until a retail one is recorded. Puts them back in the bags and returns scene 8's params.
    fn return_materials(connection: &mut ZoneConnection) -> Vec<u32> {
        let mut params = vec![0u32; 22];
        params[0] = 8;
        let Some(session) = connection.craft.as_mut() else {
            return params;
        };
        let mut returned: Vec<(u32, u32, u32)> = Vec::new(); // item, NQ, HQ
        for (item_id, hq, amount) in session.consumed.clone() {
            if !session
                .recipe
                .ingredients
                .iter()
                .take(6)
                .any(|i| i.item_id == item_id)
            {
                continue; // crystals are used up
            }
            let kept = (0..amount).filter(|_| session.rng.bool()).count() as u32;
            if kept == 0 {
                continue;
            }
            match returned.iter_mut().find(|(id, _, _)| *id == item_id) {
                Some(entry) => {
                    if hq {
                        entry.2 += kept
                    } else {
                        entry.1 += kept
                    }
                }
                None => returned.push((
                    item_id,
                    if hq { 0 } else { kept },
                    if hq { kept } else { 0 },
                )),
            }
        }

        for (i, (item_id, nq, hq)) in returned.iter().take(6).enumerate() {
            params[4 + i] = *item_id;
            params[10 + i] = *nq;
            params[16 + i] = *hq;
            for (count, flags) in [(*nq, 0), (*hq, ITEM_FLAG_HQ)] {
                if count == 0 {
                    continue;
                }
                let info = connection
                    .game_data
                    .lock()
                    .get_item_info(ItemInfoQuery::ById(*item_id));
                if let Some(info) = info {
                    let mut item = Item::new(&info, count as i32);
                    item.item_flags = flags;
                    connection.player_data.inventory.add_obtained_item(item);
                }
            }
        }
        params
    }
}

#[async_trait]
impl EventHandler for CraftingEventHandler {
    async fn on_yield(
        &self,
        _event: &Event,
        connection: &mut ZoneConnection,
        _scene: u16,
        _yield_id: u8,
        results: &[i32],
        player: &mut LuaPlayer,
    ) {
        let Some(command) = results.first().copied() else {
            return;
        };
        let result = |i: usize| results.get(i).copied().unwrap_or_default();

        match command {
            YIELD_MATERIALS_CHOSEN => {
                // [0, NQ of slots 0..5, HQ of slots 0..5]
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

                let mut hq = [0u32; 6];
                for (i, hq) in hq.iter_mut().enumerate() {
                    *hq = result(7 + i).max(0) as u32;
                }
                let ingredients: Vec<_> = info
                    .ingredients
                    .iter()
                    .take(6)
                    .zip(hq)
                    .map(|(i, hq)| (i.amount, i.item_level, i.can_be_hq, hq))
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
                connection.craft = Some(Box::new(CraftSession {
                    recipe: info,
                    craft,
                    consumed: Vec::new(),
                    rng: fastrand::Rng::new(),
                }));
                Self::send_state(connection, settings.max_cp, Vec::new()).await;

                connection
                    .broadcast_actor_control(ActorControlCategory::LiveEvent {
                        event: LiveEventType::StartCraft {
                            unk1: 0,
                            unk2: 0,
                            unk3: 0,
                        },
                    })
                    .await;

                player.play_scene(
                    0,
                    SceneFlags::NO_DEFAULT_CAMERA,
                    vec![2, recipe.id, start_quality, start_hq],
                );
            }
            YIELD_MATERIALS_LIST => {
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
                let mut consumed = Vec::new();
                for i in 0..count {
                    let id = word(1 + i);
                    let amount = half(31, i);
                    let container = half(46, i);
                    let slot = half(61, i) as u16;
                    let (item_id, hq) = if id > HQ_ID_OFFSET {
                        (id - HQ_ID_OFFSET, true)
                    } else {
                        (id, false)
                    };
                    let Some(container) = material_container(container) else {
                        tracing::warn!(
                            "A material is in container {container}, which isn't a bag!"
                        );
                        continue;
                    };
                    let Some(stack) = connection
                        .player_data
                        .inventory
                        .get_item_mut(container, slot)
                    else {
                        continue;
                    };
                    if stack.item_id != item_id
                        || (stack.item_flags & ITEM_FLAG_HQ != 0) != hq
                        || stack.quantity < amount as i32
                    {
                        tracing::warn!(
                            "The client's materials list has {amount} of {id} in {container:?} {slot}, which isn't there!"
                        );
                        continue;
                    }
                    stack.quantity -= amount as i32;
                    // Crystals keep their slot (and item ID) when there are none left.
                    if stack.quantity == 0 && container != ContainerType::Crystals {
                        *stack = Item::default();
                    }
                    consumed.push((item_id, hq, amount));
                }
                if let Some(session) = connection.craft.as_mut() {
                    session.consumed = consumed;
                }
                connection.send_inventory().await;

                player.play_scene(0, SceneFlags::NO_DEFAULT_CAMERA, vec![3, 0, 0, 0]);
            }
            YIELD_ACTION | YIELD_CRAFT_ACTION => {
                let action_id = result(1) as u32;
                let craft_action = command == YIELD_CRAFT_ACTION;
                let Some(action) = CraftAction::from_id(action_id) else {
                    tracing::warn!("Crafting action {action_id} isn't one the synthesis knows!");
                    player.play_scene(0, SceneFlags::NO_DEFAULT_CAMERA, vec![7, 0, 0, 0]);
                    return;
                };
                let Some(session) = connection.craft.as_mut() else {
                    return;
                };
                let CraftSession { craft, rng, .. } = session.as_mut();
                let Some(outcome) = craft.use_action(action, rng) else {
                    // Not usable now (the client normally greys these out): scene 7 lets it act again.
                    tracing::info!("{action:?} can't be used now");
                    player.play_scene(0, SceneFlags::NO_DEFAULT_CAMERA, vec![7, 0, 0, 0]);
                    return;
                };
                let params = step_params(action_id, craft_action, craft, &outcome);
                let cp = craft.cp;
                let statuses = craft.statuses();

                // CraftAction rows have their own animation; the buffs' Action rows play theirs client-side.
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
                }
                Self::send_state(connection, cp, statuses).await;
                player.play_scene(0, SceneFlags::NO_DEFAULT_CAMERA, params);
            }
            YIELD_COMPLETE => {
                let Some(session) = connection.craft.as_mut() else {
                    return;
                };
                let hq =
                    session.recipe.can_hq && session.rng.u32(0..100) < session.craft.hq_chance();
                let item_id = session.recipe.item_id;
                let amount = session.recipe.amount_result;
                let exp = if session.recipe.exp_rewarded {
                    // A placeholder until a retail synthesis is recorded: GatheringExp for the recipe's level, up to
                    // twice that for full quality.
                    let base = connection
                        .game_data
                        .lock()
                        .get_gathering_exp(session.recipe.class_job_level as u32);
                    let quality = session
                        .craft
                        .quality
                        .clamp(0, session.craft.settings.max_quality);
                    let max = session.craft.settings.max_quality.max(1);
                    base + (base as i64 * quality as i64 / max as i64) as i32
                } else {
                    0
                };

                let info = connection
                    .game_data
                    .lock()
                    .get_item_info(ItemInfoQuery::ById(item_id));
                if let Some(info) = info {
                    let mut item = Item::new(&info, amount as i32);
                    if hq {
                        item.item_flags |= ITEM_FLAG_HQ;
                    }
                    // The crafter's signature (equipment shows it.)
                    item.crafter_content_id = connection.player_data.character.content_id as u64;
                    connection.player_data.inventory.add_obtained_item(item);
                }
                connection.send_inventory().await;

                connection
                    .actor_control_self(ActorControlCategory::LogMessage {
                        log_message: LOG_ITEM_OBTAINED,
                        id: if hq { item_id + HQ_ID_OFFSET } else { item_id },
                    })
                    .await;

                Self::end_synthesis(connection).await;
                if exp > 0 {
                    connection.add_exp(exp).await;
                }

                // Back to the Crafting Log.
                player.play_scene(0, SceneFlags::NO_DEFAULT_CAMERA, vec![4, 0, 0, 0]);
            }
            YIELD_QUIT | YIELD_FAILED => {
                if connection.craft.is_none() {
                    // Leaving the Crafting Log (no synthesis under way.)
                    player.finish_event();
                    return;
                }
                let params = Self::return_materials(connection);
                connection.send_inventory().await;
                player.play_scene(0, SceneFlags::NO_DEFAULT_CAMERA, params);
            }
            YIELD_MATERIALS_RETURNED => {
                Self::end_synthesis(connection).await;
                player.play_scene(0, SceneFlags::NO_DEFAULT_CAMERA, vec![4, 0, 0, 0]);
            }
            _ => {
                tracing::info!("Unknown crafting yield {results:?}");
            }
        }
    }

    fn condition(&self) -> Condition {
        Condition::ExecutingCraftingAction
    }

    fn character_mode(&self) -> CharacterMode {
        CharacterMode::Crafting
    }
}
