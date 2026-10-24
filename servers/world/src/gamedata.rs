use crate::common::adjust_quest_id;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use icarus::Achievement::AchievementSheet;
use icarus::Action::ActionSheet;
use icarus::AetherCurrentCompFlgSet::AetherCurrentCompFlgSetSheet;
use icarus::Aetheryte::AetheryteSheet;
use icarus::AetheryteSystemDefine::AetheryteSystemDefineSheet;
use icarus::BNpcBase::BNpcBaseSheet;
use icarus::BNpcCustomize::BNpcCustomizeSheet;
use icarus::BaseParam::{BaseParamRow, BaseParamSheet};
use icarus::Battalion::BattalionSheet;
use icarus::ClassJob::ClassJobSheet;
use icarus::ContentDirectorManagedSG::ContentDirectorManagedSGSheet;
use icarus::ContentFinderCondition::ContentFinderConditionSheet;
use icarus::ContentMemberType::ContentMemberTypeSheet;
use icarus::CraftAction::CraftActionSheet;
use icarus::CustomTalk::CustomTalkSheet;
use icarus::DawnContent::DawnContentSheet;
use icarus::ENpcBase::ENpcBaseSheet;
use icarus::EObj::EObjSheet;
use icarus::Emote::EmoteSheet;
use icarus::EquipSlotCategory::EquipSlotCategorySheet;
use icarus::EventAction::EventActionSheet;
use icarus::Fate::FateSheet;
use icarus::FateProgressUI::FateProgressUISheet;
use icarus::FateShop::FateShopSheet;
use icarus::FittingShopCategoryItem::FittingShopCategoryItemSheet;
use icarus::GFATE::{GFATERow, GFATESheet};
use icarus::GathererCrafterLvAdjustTable::GathererCrafterLvAdjustTableSheet;
use icarus::GatheringExp::GatheringExpSheet;
use icarus::GatheringItem::GatheringItemSheet;
use icarus::GatheringItemLevelConvertTable::GatheringItemLevelConvertTableSheet;
use icarus::GatheringPoint::GatheringPointSheet;
use icarus::GatheringPointBase::GatheringPointBaseSheet;
use icarus::GatheringPointBonus::GatheringPointBonusSheet;
use icarus::GatheringPointTransient::GatheringPointTransientSheet;
use icarus::GatheringRarePopTimeTable::GatheringRarePopTimeTableSheet;
use icarus::GeneralAction::GeneralActionSheet;
use icarus::GilShopItem::GilShopItemSheet;
use icarus::GimmickAccessor::GimmickAccessorSheet;
use icarus::GimmickRect::{GimmickRectRow, GimmickRectSheet};
use icarus::Glasses::GlassesSheet;
use icarus::GoldSaucerContent::GoldSaucerContentSheet;
use icarus::HalloweenNpcSelect::HalloweenNpcSelectSheet;
use icarus::HousingAethernet::HousingAethernetSheet;
use icarus::HousingLandSet::{HousingLandSetSheet, LandSetElement};
use icarus::IKDRoute::IKDRouteSheet;
use icarus::InstanceContent::InstanceContentSheet;
use icarus::Item::ItemSheet;
use icarus::ItemAction::ItemActionSheet;
use icarus::ItemLevel::ItemLevelRow;
use icarus::ItemLevel::ItemLevelSheet;
use icarus::ItemSearchCategory::ItemSearchCategorySheet;
use icarus::ItemSortCategory::ItemSortCategorySheet;
use icarus::Level::LevelSheet;
use icarus::Map::MapSheet;
use icarus::Mount::MountSheet;
use icarus::NpcEquip::NpcEquipSheet;
use icarus::NpcYell::NpcYellSheet;
use icarus::Omen::OmenSheet;
use icarus::OnlineStatus::OnlineStatusSheet;
use icarus::Opening::OpeningSheet;
use icarus::ParamGrow::{ParamGrowRow, ParamGrowSheet};
use icarus::PlaceName::PlaceNameSheet;
use icarus::PreHandler::PreHandlerSheet;
use icarus::PublicContent::PublicContentSheet;
use icarus::Quest::{QuestRow, QuestSheet};
use icarus::QuestBattle::QuestBattleSheet;
use icarus::Recipe::RecipeSheet;
use icarus::RecipeLevelTable::RecipeLevelTableSheet;
use icarus::SpecialShop::SpecialShopSheet;
use icarus::SwitchTalkVariation::{SwitchTalkVariationRow, SwitchTalkVariationSheet};
use icarus::TerritoryType::TerritoryTypeSheet;
use icarus::TopicSelect::TopicSelectSheet;
use icarus::Trait::TraitSheet;
use icarus::Tribe::TribeRow;
use icarus::WarpLogic::WarpLogicSheet;
use icarus::WeatherRate::WeatherRateSheet;
use icarus::{Tribe::TribeSheet, Warp::WarpSheet};
use kawari::ipc::zone::{CommonSpawn, DamageElement, PlotSize};
use physis::Language;
use physis::equipment::EquipSlotCategory;
use physis::race::{Gender, Race, Tribe};
use physis::resource::{Resource, ResourceResolver, SqPackResource, UnpackedResource};
use physis::savedata::chardat::CustomizeData;
use physis::sestring::SeString;

use kawari::common::{
    ActionCategory, FateRule, GimmickAccessorType, InstanceContentType, PublicContentType,
    TerritoryIntendedUse, get_aether_current_comp_flg_set_to_screenimage,
};
use kawari::common::{LegacyEquipmentModelId, WeaponModelId, timestamp_secs};
use kawari::config::get_config;
use strum_macros::FromRepr;

/// What an achievement asks for, see `GameData::achievement_rules`.
#[derive(Debug, Clone, Default)]
pub struct AchievementRule {
    /// Its row in the Achievement sheet.
    pub id: u32,
    /// The Achievement sheet's Type, which says what `data` means (see `zone_connection/achievement.rs`.)
    pub kind: u8,
    /// The Achievement sheet's Key (e.g. a counter for counting achievements.)
    pub key: i32,
    pub data: [i32; 8],
}

/// A QuestBattle, see `GameData::get_quest_battle`.
#[derive(Debug, Default)]
pub struct QuestBattleInfo {
    /// The quest it belongs to.
    pub quest_id: u32,
    /// In minutes.
    pub time_limit: u16,
    /// Its variables, such as the layout IDs of the NPCs it spawns.
    pub variables: Vec<(String, u32)>,
}

/// A key item's EventItem row (see `GameData::get_event_item`.)
#[derive(Debug, Clone, Copy)]
pub struct EventItemInfo {
    /// The Quest row it belongs to (65536 + the quest ID), 0 for none.
    pub quest: u32,
    pub cast_time_ms: u32,
    /// The Action row its use animates as (ActorCast and ActionEffect's spell ID.)
    pub action: u16,
}

/// One party's make-up in a duty (the ContentMemberType sheet.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartyRoles {
    pub members: u8,
    pub tanks: u8,
    pub healers: u8,
    pub melee: u8,
    pub ranged: u8,
}

/// Convenient methods built on top of Physis to access data relevant to the server
#[derive(Clone)]
pub struct GameData {
    pub resource: ResourceResolver,

    // Remember to keep frequently accessed or large sheets here, until we have a better caching solution.
    pub item_sheet: ItemSheet,
    pub classjob_exp_indexes: Vec<i8>,
    pub weather_rate_sheet: WeatherRateSheet,
    pub territory_type_sheet: TerritoryTypeSheet,
    pub quest_sheet: QuestSheet,
    pub warp_sheet: WarpSheet,
    pub action_sheet: ActionSheet,
    pub place_name_sheet: PlaceNameSheet,
    pub custom_talk_sheet: CustomTalkSheet,
    pub tribe_sheet: TribeSheet,
    pub eobj_sheet: EObjSheet,
    pub switch_talk_sheet: SwitchTalkVariationSheet,
    pub param_grow_sheet: ParamGrowSheet,
    pub bnpc_base_sheet: BNpcBaseSheet,
    pub bnpc_customize_sheet: BNpcCustomizeSheet,
    pub item_level_sheet: ItemLevelSheet,
    pub gimmick_rect_sheet: GimmickRectSheet,
    pub base_param_sheet: BaseParamSheet,
    /// Statuses Esuna can remove (Status.CanDispel), read the first time it's needed.
    dispellable_statuses: Option<std::collections::HashSet<u16>>,
    /// Actions some other action continues from (another row's ActionCombo), filled in on first use.
    combo_starters: Option<std::collections::HashSet<u16>>,
    pub classjob_sheet: ClassJobSheet,
    pub battalion_sheet: BattalionSheet,
    pub enpc_base_sheet: ENpcBaseSheet,
    pub equip_slot_category_sheet: EquipSlotCategorySheet,
    pub content_finder_condition_sheet: ContentFinderConditionSheet,
    pub content_member_type_sheet: ContentMemberTypeSheet,
    pub instance_content_sheet: InstanceContentSheet,
    pub ikd_route_sheet: IKDRouteSheet,
    pub npc_equip_sheet: NpcEquipSheet,
    pub gathering_point_sheet: GatheringPointSheet,
    pub gathering_point_base_sheet: GatheringPointBaseSheet,
    pub gathering_item_sheet: GatheringItemSheet,
    pub gathering_item_level_convert_table_sheet: GatheringItemLevelConvertTableSheet,
    pub public_content_sheet: PublicContentSheet,
    pub fate_sheet: FateSheet,
    pub dawn_content_sheet: DawnContentSheet,
    pub fate_progress_ui_sheet: FateProgressUISheet,
    pub gold_saucer_content_sheet: GoldSaucerContentSheet,
    pub gimmick_accessor_sheet: GimmickAccessorSheet,
    pub trait_sheet: TraitSheet,
    pub gfate_sheet: GFATESheet,

    pub gimmick_rect_lookup: HashMap<u32, u32>,
    pub fate_event_range_lookup: HashMap<u32, u32>,
    /// Read the first time they're needed, see `achievement_rules`.
    achievement_rules: Option<Vec<AchievementRule>>,
    /// Each map's DiscoveryIndex, DiscoveryArrayByte and DiscoveryFlag, read the first time they're needed: the
    /// achievement check asks for hundreds at once, and reading the Map sheet each time took long enough (seconds)
    /// to desync the client's connection.
    map_discovery: Option<HashMap<u32, (i16, bool, u32)>>,
    /// The marketboard's items, read the first time it's used (see `marketboard`.)
    marketboard: Option<std::sync::Arc<MarketIndex>>,
}

/// The items on the marketboard: every item with an ItemSearchCategory, by ID and by category.
#[derive(Debug, Default)]
pub struct MarketIndex {
    pub items: HashMap<u32, crate::marketboard::MarketItem>,
    /// Each ItemSearchCategory's items in retail's order (ItemSortCategory's Param, SubcategorySort, ID.)
    pub by_category: HashMap<u16, Vec<u32>>,
}

impl Default for GameData {
    fn default() -> Self {
        Self::new()
    }
}

/// Struct detailing various information about an item, pulled from the Items sheet.
#[derive(Debug, Default, Clone)]
pub struct ItemRow {
    /// The item's textual name.
    pub name: String,
    /// The item's id number.
    pub id: u32,
    /// The item's price, when sold by an NPC.
    pub price_mid: u32,
    /// The item's price, when sold to an NPC by the player.
    pub price_low: u32,
    /// The item's equip category.
    pub equip_category: EquipSlotCategory,
    /// The item's equip restrictions.
    pub equip_restrictions: ItemEquipRestrictions,
    /// The item's primary model id.
    pub primary_model_id: u64,
    /// The item's sub model id.
    pub sub_model_id: u64,
    /// The item's max stack size.
    pub stack_size: u32,
    /// The item's item level.
    pub item_level: u16,
    /// The item's ClassJobCategory.
    pub classjob_category: u8,

    /// Stat modifier stuff
    pub base_param_ids: [u8; 6],
    pub base_param_values: [i16; 6],

    /// Physical defense.
    pub defense: u16,
    /// Magic defense;
    pub magic_defense: u16,

    /// A weapon's physical and magic damage, and its auto-attack delay in milliseconds.
    pub physical_damage: u16,
    pub magic_damage: u16,
    pub delay_ms: u16,
    /// The item's ItemUICategory.
    pub ui_category: u8,
}

/// ItemUICategory for materia.
const ITEM_UI_CATEGORY_MATERIA: u8 = 58;

impl ItemRow {
    /// What an NPC shop pays for one: PriceLow, and for materia a tenth more, rounded up (retail: exact for all 53
    /// sales in Bob's Chronofoil captures, 24 of them materia; PriceLow 90 sells for 99.)
    pub fn sell_price(&self) -> u32 {
        if self.ui_category == ITEM_UI_CATEGORY_MATERIA {
            self.price_low + self.price_low.div_ceil(10)
        } else {
            self.price_low
        }
    }
}

/// An equipped item's attributes under item level sync, see `GameData::synced_item_stats`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SyncedItemStats {
    /// BaseParam IDs and their (capped) values; ID 0 is an empty slot.
    pub params: [(u8, i32); 6],
    pub defense: u32,
    pub magic_defense: u32,
}

const BASE_PARAM_DEFENSE: u8 = 21;
const BASE_PARAM_MAGIC_DEFENSE: u8 = 24;

/// An attribute's cap under item level sync: ItemLevel × slot percent × MeldParam / 100000, rounded.
fn sync_cap(item_level_value: u16, slot_percent: f64, meld_param: f64) -> i32 {
    (item_level_value as f64 * slot_percent * meld_param / 100000.0).round() as i32
}

/// Defense under item level sync: scaled by the sync item level's value over the item's own.
fn scale_by_item_level(value: u32, synced: u16, own: u16) -> u32 {
    if own == 0 {
        value
    } else {
        (value as f64 * synced as f64 / own as f64).round() as u32
    }
}

/// The ItemLevel sheet's value for a BaseParam: its columns are in BaseParam ID order (1 Strength ... 73
/// Perception.) Only the ones gear has.
fn item_level_value(row: &ItemLevelRow, param: u8) -> Option<u16> {
    Some(match param {
        1 => row.Strength,
        2 => row.Dexterity,
        3 => row.Vitality,
        4 => row.Intelligence,
        5 => row.Mind,
        6 => row.Piety,
        7 => row.HP,
        8 => row.MP,
        10 => row.GP,
        11 => row.CP,
        12 => row.PhysicalDamage,
        13 => row.MagicalDamage,
        19 => row.Tenacity,
        20 => row.AttackPower,
        21 => row.Defense,
        22 => row.DirectHitRate,
        23 => row.Evasion,
        24 => row.MagicDefense,
        27 => row.CriticalHit,
        33 => row.AttackMagicPotency,
        34 => row.HealingMagicPotency,
        44 => row.Determination,
        45 => row.SkillSpeed,
        46 => row.SpellSpeed,
        70 => row.Craftsmanship,
        71 => row.Control,
        72 => row.Gathering,
        73 => row.Perception,
        _ => return None,
    })
}

/// A BaseParam's share for an EquipSlotCategory (percent of the ItemLevel value an item in that slot may have.)
fn slot_percent(row: &BaseParamRow, equip_slot_category: u8) -> u16 {
    match equip_slot_category {
        1 => row.OneHandWeaponPercent,
        2 => row.OffHandPercent,
        3 => row.HeadPercent,
        4 => row.ChestPercent,
        5 => row.HandsPercent,
        6 => row.WaistPercent,
        7 => row.LegsPercent,
        8 => row.FeetPercent,
        9 => row.EarringPercent,
        10 => row.NecklacePercent,
        11 => row.BraceletPercent,
        12 => row.RingPercent,
        13 => row.TwoHandWeaponPercent,
        15 => row.ChestHeadPercent,
        16 => row.ChestHeadLegsFeetPercent,
        18 => row.LegsFeetPercent,
        19 => row.HeadChestHandsLegsFeetPercent,
        20 => row.ChestLegsGlovesPercent,
        21 => row.ChestLegsFeetPercent,
        _ => 0,
    }
}

#[derive(Debug)]
pub enum ItemInfoQuery {
    ById(u32),
    ByName(String),
}

/// Struct detailing what slots, if any, this item blocks from equipping. Accessories are not considered here since they never block other items.
#[derive(Clone, Debug, Default)]
pub struct ItemEquipRestrictions {
    pub main_hand: i8,
    pub off_hand: i8,
    pub head: i8,
    pub body: i8,
    pub hands: i8,
    pub legs: i8,
    pub feet: i8,
}

#[derive(Debug, Clone, Copy)]
pub struct Recipe {
    pub id: u32,
    pub item_id: i32,
}

/// One of a recipe's ingredients.
#[derive(Debug, Clone, Copy)]
pub struct RecipeIngredient {
    pub item_id: u32,
    pub amount: u32,
    /// The ingredient's LevelItem, which shares out the quality HQ ingredients bring.
    pub item_level: u32,
    pub can_be_hq: bool,
}

/// What a synthesis needs from a recipe: its Recipe row and its RecipeLevelTable row (for a level-adjusted recipe,
/// the one GathererCrafterLvAdjustTable gives for the crafter's level.)
#[derive(Debug, Clone)]
pub struct RecipeInfo {
    pub id: u32,
    pub item_id: u32,
    pub amount_result: u32,
    /// The recipe level's ClassJobLevel.
    pub class_job_level: u8,
    pub max_progress: u32,
    pub max_quality: u32,
    pub max_durability: u32,
    pub conditions_flag: u16,
    pub progress_divider: u8,
    pub quality_divider: u8,
    pub progress_modifier: u8,
    pub quality_modifier: u8,
    pub material_quality_factor: u8,
    pub can_hq: bool,
    pub is_expert: bool,
    pub exp_rewarded: bool,
    /// The eight ingredient slots, in the sheet's order (crystals last); empty slots left out.
    pub ingredients: Vec<RecipeIngredient>,
}

/// The Teleport action (Action sheet), whose general action has to be unlocked before Teleport can be used.
const TELEPORT_ACTION_ID: u16 = 5;

/// Cost type used by the Action sheet for MP.
pub const ACTION_COST_TYPE_MP: u8 = 3;

/// Cost type used by the Action sheet for GP (the Miner's and Botanist's actions.)
pub const ACTION_COST_TYPE_GP: u8 = 7;

/// Cast type used by the Action sheet for circular area-of-effect actions.
pub const ACTION_CAST_TYPE_CIRCLE: u8 = 2;

/// Targeting and cost information for an action.
#[derive(Debug, Clone, Copy)]
pub struct ActionInfo {
    /// The shape of the action (1 = single target, 2 = circle, etc.)
    pub cast_type: u8,
    /// The radius of the action's area-of-effect, in yalms.
    pub effect_range: u8,
    /// How far away the target can be, in yalms. 0 means the action is centered on the caster.
    pub range: i8,
    /// Whether this action can be used on enemies.
    pub can_target_hostile: bool,
    /// What resource this action costs (3 = MP.)
    pub primary_cost_type: u8,
    /// How much of the resource this action costs. For MP, this is in hundreds.
    pub primary_cost_value: u16,
    /// The width of a line or rectangle area-of-effect, in yalms.
    pub x_axis_modifier: u8,
    /// How wide a cone area-of-effect is, in degrees, from its omen (e.g. "gl_fan090" is 90 degrees.)
    pub cone_angle: f32,
}

impl ActionInfo {
    /// Returns the resource cost of this action: MP (in the sheet in hundreds), or a gatherer's GP.
    pub fn mp_cost(&self) -> u16 {
        if self.primary_cost_type == ACTION_COST_TYPE_MP {
            self.primary_cost_value.saturating_mul(100)
        } else if self.primary_cost_type == ACTION_COST_TYPE_GP {
            self.primary_cost_value
        } else {
            0
        }
    }

    /// Returns true if this action hits multiple targets.
    pub fn is_area_of_effect(&self) -> bool {
        self.cast_type == ACTION_CAST_TYPE_CIRCLE && self.effect_range > 0
    }

    /// The shape this action covers on the ground, if it isn't single-target.
    pub fn area_shape(&self) -> Option<AreaShape> {
        let range = self.effect_range as f32;
        match self.cast_type {
            _ if self.effect_range == 0 => None,
            // A circle, centered on the target (or where it was aimed) if the action has a range, else the caster.
            2 => Some(AreaShape::Circle {
                radius: range,
                on_target: self.range > 0,
            }),
            // A circle around the caster.
            5 => Some(AreaShape::Circle {
                radius: range,
                on_target: false,
            }),
            // Cones in front of the caster.
            3 | 13 => Some(AreaShape::Cone {
                radius: range,
                angle: self.cone_angle,
            }),
            // Lines and rectangles in front of the caster.
            4 | 8 | 12 => Some(AreaShape::Line {
                length: range,
                width: (self.x_axis_modifier as f32).max(1.0),
            }),
            _ => None,
        }
    }
}

/// The area an action covers, see `ActionInfo::area_shape`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AreaShape {
    Circle { radius: f32, on_target: bool },
    Cone { radius: f32, angle: f32 },
    Line { length: f32, width: f32 },
}

/// Cones without an angle in their omen are 90 degrees wide, the most common.
const DEFAULT_CONE_ANGLE: f32 = 90.0;

/// Reads a cone's angle from its omen's path, e.g. "gl_fan090_1bf" or "er_gl_fan120_1bf".
fn cone_angle_from_omen(path: &str) -> Option<f32> {
    let start = path.find("fan")? + 3;
    let digits: String = path[start..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

#[derive(Debug, Clone, Copy)]
pub struct Modifiers {
    pub hp: u16,
    /// Whether the class is a tank (ClassJob's Role 1): more HP per Vitality.
    pub tank: bool,
    pub mp: u16,
    pub strength: u16,
    pub vitality: u16,
    pub dexterity: u16,
    pub intelligence: u16,
    pub mind: u16,
    pub piety: u16,
}

impl Modifiers {
    pub fn apply_to(&self, index: u8, value: u32) -> u32 {
        let modifier = match index {
            1 => self.strength,
            2 => self.dexterity,
            3 => self.vitality,
            4 => self.intelligence,
            5 => self.mind,
            6 => self.piety,
            _ => return value,
        };

        (value as f32 * (modifier as f32 / 100.0)).floor() as u32
    }
}

/// Helper enum for picking duties from roulettes.
#[repr(u8)]
#[derive(FromRepr)]
pub enum Roulette {
    Leveling = 1,
    HighLevel = 2,
    MSQ = 3,
    GuildHest = 4,
    Expert = 5,
    Trial = 6,
    DailyFrontline = 7,
    LevelCap = 8,
    Mentor = 9,
    Alliance = 15,
    NormalRaid = 17,
    CrystallineConflictCasual = 40,
    CrystallineConflictRanked = 41,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct GatheringPointItem {
    pub gathering_id: i32,
    pub item_id: i32,
    pub level: u8,
    pub hidden: bool,
    /// The GatheringItemLevelConvertTable row, which is also the ItemLevel row with the item's base Gathering and
    /// Perception (GatherBuddy Reborn's table matches ItemLevel's from row 151.)
    pub level_row: u16,
    /// The Perception it needs (GatheringItem.PerceptionReq.)
    pub perception_required: u16,
    /// No Gatherer's Boon (the window shows 255): GatheringItem.Unknown2 isn't 0xFFFF (retail: every such item had
    /// 255, Bob's capture of 2026-10-24.)
    pub no_boon: bool,
    /// The node's yield bonus applies to it (GatheringItem.Unknown5; retail gave the one item without it no bonus.)
    pub bonus_yield: bool,
}

/// A gathering point's bonus (GatheringPointBonus): what it needs and what it gives.
#[derive(Debug, Clone, Copy, Default)]
pub struct GatheringPointBonusInfo {
    /// GatheringCondition: 14 Gathering, 15 Perception, 19 max GP at least `condition_value`.
    pub condition: u8,
    pub condition_value: u32,
    /// GatheringPointBonusType: 14 and 17 yield, 18 and 19 integrity, 22 and 23 boon chance.
    pub bonus_type: u8,
    pub bonus_value: u16,
}

impl GameData {
    pub fn new() -> Self {
        let config = get_config();

        let sqpack_resource = SqPackResourceSpy::from(
            SqPackResource::from_existing(&config.filesystem.game_path()),
            &config.filesystem.unpack_path,
        );

        let mut resource_resolver = ResourceResolver::new();
        for path in config.filesystem.additional_search_paths {
            let unpacked_resource = UnpackedResource::from_existing(&path);
            resource_resolver.add_source(unpacked_resource);
        }
        resource_resolver.add_source(sqpack_resource);

        let mut classjob_exp_indexes = Vec::new();

        let classjob_sheet =
            ClassJobSheet::read_from(&mut resource_resolver, config.world.language())
                .expect("Failed to read ClassJobSheet, does the Excel files exist?");
        for (_, row) in classjob_sheet.into_iter().flatten_subrows() {
            classjob_exp_indexes.push(row.ExpArrayIndex);
        }

        let item_sheet = ItemSheet::read_from(&mut resource_resolver, config.world.language())
            .expect("Failed to read ItemSheet, does the Excel files exist?");

        let weather_rate_sheet =
            WeatherRateSheet::read_from(&mut resource_resolver, Language::None)
                .expect("Failed to read WeatherRateSheet, does the Excel files exist?");

        let quest_sheet = QuestSheet::read_from(&mut resource_resolver, config.world.language())
            .expect("Failed to read Quest, does the Excel files exist?");

        let territory_type_sheet =
            TerritoryTypeSheet::read_from(&mut resource_resolver, Language::None)
                .expect("Failed to read TerritoryTypeSheet, does the Excel files exist?");

        let warp_sheet = WarpSheet::read_from(&mut resource_resolver, config.world.language())
            .expect("Failed to read Warp, does the Excel files exist?");

        let action_sheet = ActionSheet::read_from(&mut resource_resolver, config.world.language())
            .expect("Failed to read Action, does the Excel files exist?");

        let place_name_sheet =
            PlaceNameSheet::read_from(&mut resource_resolver, config.world.language())
                .expect("Failed to read PlaceName, does the Excel files exist?");

        let custom_talk_sheet =
            CustomTalkSheet::read_from(&mut resource_resolver, config.world.language())
                .expect("Failed to read CustomTalk, does the Excel files exist?");

        let tribe_sheet = TribeSheet::read_from(&mut resource_resolver, config.world.language())
            .expect("Failed to read Tribe, does the Excel files exist?");

        let eobj_sheet = EObjSheet::read_from(&mut resource_resolver, Language::None)
            .expect("Failed to read EObj, does the Excel files exist?");

        let switch_talk_sheet =
            SwitchTalkVariationSheet::read_from(&mut resource_resolver, Language::None).unwrap();

        let param_grow_sheet =
            ParamGrowSheet::read_from(&mut resource_resolver, Language::None).unwrap();

        let bnpc_base_sheet =
            BNpcBaseSheet::read_from(&mut resource_resolver, Language::None).unwrap();

        let bnpc_customize_sheet =
            BNpcCustomizeSheet::read_from(&mut resource_resolver, Language::None).unwrap();

        let item_level_sheet =
            ItemLevelSheet::read_from(&mut resource_resolver, Language::None).unwrap();

        let gimmick_rect_sheet =
            GimmickRectSheet::read_from(&mut resource_resolver, Language::None).unwrap();

        let base_param_sheet =
            BaseParamSheet::read_from(&mut resource_resolver, config.world.language())
                .ok()
                .unwrap();

        let battalion_sheet = BattalionSheet::read_from(&mut resource_resolver, Language::None)
            .ok()
            .unwrap();

        let enpc_base_sheet =
            ENpcBaseSheet::read_from(&mut resource_resolver, Language::None).unwrap();

        let equip_slot_category_sheet =
            EquipSlotCategorySheet::read_from(&mut resource_resolver, Language::None).unwrap();

        let content_finder_condition_sheet =
            ContentFinderConditionSheet::read_from(&mut resource_resolver, config.world.language())
                .unwrap();

        let instance_content_sheet =
            InstanceContentSheet::read_from(&mut resource_resolver, Language::None).unwrap();

        let content_member_type_sheet =
            ContentMemberTypeSheet::read_from(&mut resource_resolver, Language::None).unwrap();

        let ikd_route_sheet =
            IKDRouteSheet::read_from(&mut resource_resolver, config.world.language()).unwrap();

        let npc_equip_sheet =
            NpcEquipSheet::read_from(&mut resource_resolver, Language::None).unwrap();

        let gathering_point_sheet =
            GatheringPointSheet::read_from(&mut resource_resolver, Language::None).unwrap();

        let gathering_point_base_sheet =
            GatheringPointBaseSheet::read_from(&mut resource_resolver, Language::None).unwrap();

        let gathering_item_sheet =
            GatheringItemSheet::read_from(&mut resource_resolver, Language::None).unwrap();

        let gathering_item_level_convert_table_sheet =
            GatheringItemLevelConvertTableSheet::read_from(&mut resource_resolver, Language::None)
                .unwrap();

        let public_content_sheet =
            PublicContentSheet::read_from(&mut resource_resolver, config.world.language()).unwrap();

        let fate_sheet =
            FateSheet::read_from(&mut resource_resolver, config.world.language()).unwrap();

        let dawn_content_sheet =
            DawnContentSheet::read_from(&mut resource_resolver, Language::None).unwrap();

        let fate_progress_ui_sheet =
            FateProgressUISheet::read_from(&mut resource_resolver, Language::None).unwrap();

        let gold_saucer_content_sheet =
            GoldSaucerContentSheet::read_from(&mut resource_resolver, Language::None).unwrap();

        let gimmick_accessor_sheet =
            GimmickAccessorSheet::read_from(&mut resource_resolver, Language::None).unwrap();

        let trait_sheet =
            TraitSheet::read_from(&mut resource_resolver, config.world.language()).unwrap();

        let gfate_sheet = GFATESheet::read_from(&mut resource_resolver, Language::None).unwrap();

        let mut gimmick_rect_lookup = HashMap::new();
        for (id, row) in gimmick_rect_sheet.into_iter().flatten_subrows() {
            gimmick_rect_lookup.insert(row.LayoutID, id);
        }

        let mut fate_event_range_lookup = HashMap::new();
        for (id, row) in fate_sheet.into_iter().flatten_subrows() {
            fate_event_range_lookup.insert(row.Location, id);
        }

        Self {
            resource: resource_resolver,
            item_sheet,
            classjob_exp_indexes,
            weather_rate_sheet,
            quest_sheet,
            territory_type_sheet,
            warp_sheet,
            action_sheet,
            place_name_sheet,
            custom_talk_sheet,
            tribe_sheet,
            eobj_sheet,
            switch_talk_sheet,
            param_grow_sheet,
            bnpc_base_sheet,
            bnpc_customize_sheet,
            item_level_sheet,
            gimmick_rect_sheet,
            base_param_sheet,
            dispellable_statuses: None,
            combo_starters: None,
            classjob_sheet,
            battalion_sheet,
            enpc_base_sheet,
            equip_slot_category_sheet,
            content_finder_condition_sheet,
            content_member_type_sheet,
            instance_content_sheet,
            ikd_route_sheet,
            npc_equip_sheet,
            gathering_point_sheet,
            gathering_point_base_sheet,
            gathering_item_sheet,
            gathering_item_level_convert_table_sheet,
            public_content_sheet,
            fate_sheet,
            gimmick_rect_lookup,
            fate_event_range_lookup,
            achievement_rules: None,
            map_discovery: None,
            marketboard: None,
            dawn_content_sheet,
            fate_progress_ui_sheet,
            gold_saucer_content_sheet,
            gimmick_accessor_sheet,
            trait_sheet,
            gfate_sheet,
        }
    }

    /// Gets the starting city-state from a given class/job id.
    pub fn get_citystate(&mut self, classjob_id: u16) -> Option<u8> {
        let row = self.classjob_sheet.row(classjob_id as u32)?;
        Some(row.StartingTown)
    }

    pub fn get_tribe(&mut self, tribe_id: u8) -> Option<TribeRow> {
        self.tribe_sheet.row(tribe_id as u32)
    }

    /// Gets various information from the Item sheet.
    pub fn get_item_info(&mut self, query: ItemInfoQuery) -> Option<ItemRow> {
        let mut result = None;
        match query {
            ItemInfoQuery::ById(ref query_item_id) => {
                if let Some(row) = self.item_sheet.row(*query_item_id) {
                    result = Some((row, *query_item_id));
                }
            }

            ItemInfoQuery::ByName(ref query_item_name) => {
                for (id, row) in self.item_sheet.into_iter().flatten_subrows() {
                    if row
                        .Name
                        .to_string() // Item names do not use SeString features
                        .to_lowercase()
                        .contains(&query_item_name.to_lowercase())
                    {
                        result = Some((row.clone(), id));
                        break;
                    }
                }
            }
        }

        if let Some((matched_row, item_id)) = result {
            let item_info = ItemRow {
                id: item_id,
                name: matched_row.Name.to_string(), // Item names do not use SeString features
                price_mid: matched_row.PriceMid,
                price_low: matched_row.PriceLow,
                equip_category: EquipSlotCategory::from_repr(matched_row.EquipSlotCategory)
                    .unwrap_or_default(),
                primary_model_id: matched_row.ModelMain,
                sub_model_id: matched_row.ModelSub,
                stack_size: matched_row.StackSize,
                item_level: matched_row.LevelItem,
                classjob_category: matched_row.ClassJobCategory,
                base_param_ids: matched_row.BaseParam,
                base_param_values: matched_row.BaseParamValue,
                defense: matched_row.DefensePhys,
                magic_defense: matched_row.DefenseMag,
                physical_damage: matched_row.DamagePhys,
                magic_damage: matched_row.DamageMag,
                delay_ms: matched_row.Delayms,
                ui_category: matched_row.ItemUICategory,
                equip_restrictions: self
                    .get_equipslot_restrictions(matched_row.EquipSlotCategory)
                    .unwrap(),
            };

            return Some(item_info);
        }

        None
    }

    /// Gets the primary model ID for a given item ID.
    pub fn get_primary_model_id(&mut self, item_id: u32) -> Option<u64> {
        if let Some(item_info) = self.get_item_info(ItemInfoQuery::ById(item_id)) {
            return Some(item_info.primary_model_id);
        }

        None
    }

    /// Gets the sub model ID for a given item ID.
    pub fn get_sub_model_id(&mut self, item_id: u32) -> Option<u64> {
        if let Some(item_info) = self.get_item_info(ItemInfoQuery::ById(item_id)) {
            // Only return an id if the item actually has a sub model.
            if item_info.sub_model_id != 0 {
                return Some(item_info.sub_model_id);
            }
        }

        None
    }

    /// Returns the pop range object id that's associated with the warp id
    pub fn get_warp(&mut self, warp_id: u32) -> Option<(u32, u16)> {
        let row = self.warp_sheet.row(warp_id)?;

        Some((row.PopRange, row.TerritoryType))
    }

    /// Returns the warp logic name (if any) for this Warp.
    pub fn get_warp_logic_name(&mut self, warp_id: u32) -> String {
        let row = self.warp_sheet.row(warp_id).unwrap();

        let warp_logic_id = row.WarpLogic;

        let config = get_config();
        let warp_logic_sheet =
            WarpLogicSheet::read_from(&mut self.resource, config.world.language()).unwrap();
        let warp_logic_row = warp_logic_sheet.row(warp_logic_id as u32).unwrap();

        warp_logic_row.WarpName.to_string() // This shouldn't use SeString features
    }

    /// What teleporting from this zone to this aetheryte costs in gil, as the client works it out (Telepo's
    /// GetTeleportCost; exact for all 87 paid teleports in Bob's Chronofoil captures): the distance between the two
    /// zones on TerritoryTypeTelepo's world map (through TelepoRelay's crossing point when they're in different
    /// regions, plus its cost), halved above 1000, halved again for a favourite destination. 999 if a zone has no row.
    pub fn teleport_cost(&mut self, from_zone: u16, aetheryte_id: u32, favourite: bool) -> u32 {
        let Some(cost) = self.teleport_cost_inner(from_zone, aetheryte_id) else {
            return 999;
        };
        let cost = if cost > 1000 {
            1000 + (cost - 1000) / 2
        } else {
            cost
        };
        if favourite { cost / 2 } else { cost }
    }

    fn teleport_cost_inner(&mut self, from_zone: u16, aetheryte_id: u32) -> Option<u32> {
        let language = get_config().world.language();
        let to_zone = AetheryteSheet::read_from(&mut self.resource, language)
            .ok()?
            .row(aetheryte_id)?
            .Territory;
        let telepo = icarus::TerritoryTypeTelepo::TerritoryTypeTelepoSheet::read_from(
            &mut self.resource,
            Language::None,
        )
        .ok()?;
        let relays =
            icarus::TelepoRelay::TelepoRelaySheet::read_from(&mut self.resource, Language::None)
                .ok()?;
        let point = |zone: u16| {
            telepo
                .row(zone as u32)
                .map(|row| (row.X as i64, row.Y as i64, row.Expansion as i64, row.Relay))
        };
        let a = point(from_zone)?;
        let b = point(to_zone)?;
        let relay_a = relays.row(a.3 as u32)?;
        let width_a = relay_a.Unknown_70 as i64;
        if a.3 == b.3 {
            return Some(teleport_leg(a, b, width_a));
        }
        let crossing = relay_a.Relays.get(b.3 as usize)?;
        let width_b = relays.row(b.3 as u32)?.Unknown_70 as i64;
        Some(
            crossing.Cost as u32
                + teleport_leg(a, point(crossing.EnterTerritory)?, width_a)
                + teleport_leg(point(crossing.ExitTerritory)?, b, width_b),
        )
    }

    pub fn get_aetheryte(
        &mut self,
        aetheryte_id: u32,
        housing_aethernet: bool,
    ) -> Option<(u32, u16)> {
        let config = get_config();

        if !housing_aethernet {
            let sheet =
                AetheryteSheet::read_from(&mut self.resource, config.world.language()).ok()?;
            let row = sheet.row(aetheryte_id)?;

            // Pick a random pop range to stop people from spawning in the same position
            let locations: Vec<u32> = row
                .Level
                .to_vec()
                .iter()
                .filter(|x| **x != 0)
                .copied()
                .collect();
            let pop_range_id = fastrand::choice(locations).unwrap();
            let zone_id = row.Territory;

            Some((pop_range_id, zone_id))
        } else {
            let sheet =
                HousingAethernetSheet::read_from(&mut self.resource, Language::None).ok()?;
            let row = sheet.row(aetheryte_id)?;

            // Note that the HousingAethernet sheet's Level column isn't an array.
            let pop_range_id = row.Level;
            let zone_id = row.TerritoryType;

            Some((pop_range_id, zone_id))
        }
    }

    /// The unlock link that lets a player use the Teleport general action (its UnlockLink in GeneralAction), if it has
    /// one. Teleport is the general action whose action is TELEPORT_ACTION_ID.
    pub fn get_teleport_unlock_link(&mut self) -> Option<u32> {
        let config = get_config();
        let sheet =
            GeneralActionSheet::read_from(&mut self.resource, config.world.language()).ok()?;
        sheet
            .into_iter()
            .flatten_subrows()
            .find(|(_, row)| row.Action == TELEPORT_ACTION_ID)
            .map(|(_, row)| row.UnlockLink as u32)
            .filter(|link| *link != 0)
    }

    /// How many big Aetherytes (not shards) `has_aetheryte` says the player has.
    pub fn count_main_aetherytes(&mut self, has_aetheryte: impl Fn(u32) -> bool) -> usize {
        let config = get_config();
        let Ok(sheet) = AetheryteSheet::read_from(&mut self.resource, config.world.language())
        else {
            return 0;
        };
        sheet
            .into_iter()
            .flatten_subrows()
            .filter(|(id, row)| row.IsAetheryte && has_aetheryte(*id))
            .count()
    }

    /// Checks if it's a big Aetheryte (true) or just a shard (false.)
    pub fn is_aetheryte(&mut self, aetheryte_id: u32) -> bool {
        let config = get_config();
        let sheet = AetheryteSheet::read_from(&mut self.resource, config.world.language()).unwrap();
        let row = sheet.row(aetheryte_id).unwrap();

        row.IsAetheryte
    }

    /// Retrieves a zone's internal name, place name or parent region name.
    pub fn get_territory_name(&mut self, zone_id: u32, which: TerritoryNameKind) -> Option<String> {
        let row = self.territory_type_sheet.row(zone_id)?;

        let offset = match which {
            TerritoryNameKind::Internal => {
                return Some(row.Name.to_string()); // This shouldn't use use SeString features
            }
            TerritoryNameKind::Region => row.PlaceNameRegion,
            TerritoryNameKind::Place => row.PlaceName,
        };

        let row = self.place_name_sheet.row(offset as u32)?;
        Some(row.Name.to_string()) // This shouldn't use use SeString features
    }

    /// Turn an equip slot category id into a slot for the equipped inventory
    pub fn get_equipslot_category(&mut self, equipslot_id: u8) -> Option<u16> {
        let row = self.equip_slot_category_sheet.row(equipslot_id as u32)?;

        let main_hand = row.MainHand;
        if main_hand == 1 {
            return Some(0);
        }

        let off_hand = row.OffHand;
        if off_hand == 1 {
            return Some(1);
        }

        let head = row.Head;
        if head == 1 {
            return Some(2);
        }

        let body = row.Body;
        if body == 1 {
            return Some(3);
        }

        let gloves = row.Gloves;
        if gloves == 1 {
            return Some(4);
        }

        let legs = row.Legs;
        if legs == 1 {
            return Some(6);
        }

        let feet = row.Feet;
        if feet == 1 {
            return Some(7);
        }

        let ears = row.Ears;
        if ears == 1 {
            return Some(8);
        }

        let neck = row.Neck;
        if neck == 1 {
            return Some(9);
        }

        let wrists = row.Wrists;
        if wrists == 1 {
            return Some(10);
        }

        let right_finger = row.FingerR;
        if right_finger == 1 {
            return Some(11);
        }

        let left_finger = row.FingerL;
        if left_finger == 1 {
            return Some(12);
        }

        let soul_crystal = row.SoulCrystal;
        if soul_crystal == 1 {
            return Some(13);
        }

        None
    }

    // Returns information on what item slots, if any, this equipslot configuration blocks.
    // For example, a two-handed weapon (MainHand = 1, OffHand = -1) will always block an off-hand from being equipped.
    fn get_equipslot_restrictions(&mut self, equipslot_id: u8) -> Option<ItemEquipRestrictions> {
        let row = self.equip_slot_category_sheet.row(equipslot_id as u32)?;

        Some(ItemEquipRestrictions {
            main_hand: row.MainHand,
            off_hand: row.OffHand,
            head: row.Head,
            body: row.Body,
            hands: row.Gloves,
            legs: row.Legs,
            feet: row.Feet,
        })
    }

    /// Returns the cast time in milliseconds.
    /// What a rotation needs to know about an action (see `server::rotation`): on the GCD (CooldownGroup 58, or 58 as
    /// its additional group), its recast in seconds, and the action it follows in a combo.
    pub fn get_action_mechanics(
        &mut self,
        action_id: u32,
    ) -> Option<crate::server::ActionMechanics> {
        let row = self.action_sheet.row(action_id)?;
        Some(crate::server::ActionMechanics {
            gcd: row.CooldownGroup == crate::server::GCD_GROUP
                || row.AdditionalCooldownGroup == crate::server::GCD_GROUP,
            recast: row.Recast100ms as f32 / 10.0,
            combo_from: row.ActionCombo as u32,
        })
    }

    /// A key item's EventItem row: the quest it belongs to (Quest row, e.g. 65843), its cast time in milliseconds and
    /// the Action row its use animates as.
    pub fn get_event_item(&mut self, item_id: u32) -> Option<EventItemInfo> {
        let sheet = icarus::EventItem::EventItemSheet::read_from(
            &mut self.resource,
            get_config().world.language(),
        )
        .ok()?;
        let row = sheet.row(item_id)?;
        Some(EventItemInfo {
            quest: row.Quest,
            cast_time_ms: row.CastTime as u32 * 1000,
            action: row.Action,
        })
    }

    pub fn get_casttime(&mut self, action_id: u32) -> Option<(ActionCategory, u32)> {
        let row = self.action_sheet.row(action_id)?;

        Some((
            ActionCategory::from_repr(row.ActionCategory)?,
            row.Cast100ms as u32 * 100,
        ))
    }

    /// Calculates the current weather at the current time
    pub fn get_weather_rate(&mut self, weather_rate_id: u32) -> Option<i32> {
        self.get_weather_rate_at(weather_rate_id, timestamp_secs())
    }

    /// The weather for this WeatherRate at this Unix time (see `weather_target`.)
    pub fn get_weather_rate_at(&mut self, weather_rate_id: u32, unix_seconds: u32) -> Option<i32> {
        let row = self.weather_rate_sheet.row(weather_rate_id)?;

        // sum up the rates
        let mut rates = row.Rate;
        let mut sum = 0;
        for rate in &mut rates {
            sum += *rate;
            *rate = sum;
        }

        let target = weather_target(unix_seconds);
        let weather_and_rates: Vec<(i32, i32)> = row
            .Weather
            .iter()
            .cloned()
            .zip(rates)
            .map(|(x, y)| (x, y as i32))
            .filter(|x| x.0 > 0) // don't take into account invalid weather ids
            .collect();

        Some(
            weather_and_rates
                .iter()
                .filter(|(_, rate)| target < *rate)
                .take(1)
                .collect::<Vec<&(i32, i32)>>()
                .first()?
                .0,
        )
    }

    /// Calculate target window for weather calculations
    /// Gets the current weather for the given zone id
    pub fn get_weather(&mut self, zone_id: u32) -> Option<i32> {
        self.get_weather_at(zone_id, timestamp_secs())
    }

    /// The weather in a zone at this Unix time.
    pub fn get_weather_at(&mut self, zone_id: u32, unix_seconds: u32) -> Option<i32> {
        let row = self.territory_type_sheet.row(zone_id)?;

        let weather_rate_id = row.WeatherRate;
        self.get_weather_rate_at(weather_rate_id as u32, unix_seconds)
    }

    /// Gets the array index used in EXP & levels.
    pub fn get_exp_array_index(&self, classjob_id: u16) -> Option<i8> {
        self.classjob_exp_indexes
            .get(classjob_id as usize)
            .copied()
            .filter(|x| *x != -1)
    }

    /// Gets the job index for a given class.
    pub fn get_job_index(&mut self, classjob_id: u16) -> Option<u8> {
        let row = self.classjob_sheet.row(classjob_id as u32)?;

        Some(row.JobIndex)
    }

    /// Gets the primary stat for a given class.
    pub fn get_job_primary_stat(&mut self, classjob_id: u16) -> Option<u8> {
        let row = self.classjob_sheet.row(classjob_id as u32)?;

        Some(row.PrimaryStat)
    }

    /// Gets the item and its cost from the specified shop.
    pub fn get_gilshop_item(&mut self, gilshop_id: u32, index: u16) -> Option<ItemRow> {
        let sheet = GilShopItemSheet::read_from(&mut self.resource, Language::None).ok()?;
        let row = sheet.subrow(gilshop_id, index)?;
        let item_id = row.Item;

        self.get_item_info(ItemInfoQuery::ById(item_id as u32))
    }

    /// Gets the item and its cost from the specified SpecialShop.
    pub fn get_specialshop_item(&mut self, gilshop_id: u32, index: u16) -> Option<ItemRow> {
        let config = get_config();
        let sheet =
            SpecialShopSheet::read_from(&mut self.resource, config.world.language()).ok()?;
        let row = sheet.row(gilshop_id)?;
        let item_id = row.Item[index as usize].Item[0]; // TODO: why are there two items?

        self.get_item_info(ItemInfoQuery::ById(item_id as u32))
    }

    /// Gets the zone id for the given ContentFinderCondition ID.
    pub fn find_zone_for_content(&mut self, content_id: u16) -> Option<u16> {
        let content_finder_row = self.content_finder_condition_sheet.row(content_id as u32)?;

        Some(content_finder_row.TerritoryType)
    }

    /// Grabs needed BattleNPC information such as their model id and more.
    pub fn find_bnpc(&mut self, id: u32) -> Option<(u16, u8, CustomizeData, u8, u16, u16)> {
        let bnpc_row = self.bnpc_base_sheet.row(id)?;
        let model_row_id = bnpc_row.ModelChara;
        let customize_row_id = bnpc_row.BNpcCustomize;
        let customize_row = self.bnpc_customize_sheet.row(customize_row_id as u32)?;

        let customize = CustomizeData {
            race: Race::from_repr(customize_row.Race).unwrap(),
            gender: Gender::from_repr(customize_row.Gender).unwrap(),
            age: customize_row.BodyType,
            height: customize_row.Height,
            tribe: Tribe::from_repr(customize_row.Tribe).unwrap(),
            face: customize_row.Face,
            hair: customize_row.HairStyle,
            enable_highlights: customize_row.HairHighlight == 1,
            skin_tone: customize_row.SkinColor,
            right_eye_color: customize_row.EyeColor,
            hair_tone: customize_row.HairColor,
            highlights: customize_row.HairHighlightColor,
            facial_features: customize_row.FacialFeature,
            facial_feature_color: customize_row.FacialFeatureColor,
            eyebrows: customize_row.Eyebrows,
            left_eye_color: customize_row.EyeHeterochromia,
            eyes: customize_row.EyeShape,
            nose: customize_row.Nose,
            jaw: customize_row.Jaw,
            mouth: customize_row.Mouth,
            lips_tone_fur_pattern: customize_row.LipColor,
            race_feature_size: customize_row.BustOrTone1,
            race_feature_type: customize_row.ExtraFeature1,
            bust: customize_row.ExtraFeature2OrBust,
            face_paint: customize_row.FacePaint,
            face_paint_color: customize_row.FacePaintColor,
        };

        Some((
            model_row_id,
            bnpc_row.Battalion,
            customize,
            bnpc_row.Rank,
            bnpc_row.NpcEquip,
            bnpc_row.Behavior,
        ))
    }

    /// Grabs needed EventNpc information such as their name, model id and more.
    pub fn find_enpc(&mut self, id: u32) -> Option<(u16, CustomizeData, u16, u16)> {
        let row = self.enpc_base_sheet.row(id)?;
        let model_row_id = row.ModelChara;

        let customize = CustomizeData {
            race: Race::from_repr(row.Race).unwrap(),
            gender: Gender::from_repr(row.Gender).unwrap(),
            age: row.BodyType,
            height: row.Height,
            tribe: Tribe::from_repr(row.Tribe).unwrap(),
            face: row.Face,
            hair: row.HairStyle,
            enable_highlights: row.HairHighlight == 1,
            skin_tone: row.SkinColor,
            right_eye_color: row.EyeColor,
            hair_tone: row.HairColor,
            highlights: row.HairHighlightColor,
            facial_features: row.FacialFeature,
            facial_feature_color: row.FacialFeatureColor,
            eyebrows: row.Eyebrows,
            left_eye_color: row.EyeHeterochromia,
            eyes: row.EyeShape,
            nose: row.Nose,
            jaw: row.Jaw,
            mouth: row.Mouth,
            lips_tone_fur_pattern: row.LipColor,
            race_feature_size: row.BustOrTone1,
            race_feature_type: row.ExtraFeature1,
            bust: row.ExtraFeature2OrBust,
            face_paint: row.FacePaint,
            face_paint_color: row.FacePaintColor,
        };

        Some((model_row_id, customize, row.NpcEquip, row.Behavior))
    }

    /// Gets the content type for the given InstanceContent.
    pub fn find_type_for_content(&mut self, content_id: u16) -> Option<InstanceContentType> {
        let instance_content_row = self.instance_content_sheet.row(content_id as u32)?;

        InstanceContentType::from_repr(instance_content_row.InstanceContentType)
    }

    /// Gets the order of the mount.
    pub fn find_mount_order(&mut self, mount_id: u32) -> Option<i16> {
        let config = get_config();
        let instance_content_sheet =
            MountSheet::read_from(&mut self.resource, config.world.language()).unwrap();
        let mount_row = instance_content_sheet.row(mount_id)?;

        Some(mount_row.Order)
    }

    /// Gets the Item ID of the Orchestrion Roll.
    pub fn find_orchestrion_item_id(&mut self, orchestrion_id: u32) -> Option<u32> {
        for (id, row) in self.item_sheet.into_iter().flatten_subrows() {
            // If filter_group is 32, then this item is an Orchestrion Roll...
            if row.FilterGroup != 32 {
                continue;
            }

            if row.AdditionalData == orchestrion_id {
                return Some(id);
            }
        }

        None
    }

    /// Gets the Set/Zone of the Aether Current
    pub fn find_aether_current_set(&mut self, aether_current_id: i32) -> Option<u32> {
        let sheet =
            AetherCurrentCompFlgSetSheet::read_from(&mut self.resource, Language::None).unwrap();

        // Start searching for Zone ID
        for (id, row) in sheet.into_iter().flatten_subrows() {
            let aether_currents = row.AetherCurrents;
            if aether_currents.contains(&aether_current_id) {
                return Some(id);
            }
        }

        None
    }

    /// Gets the Aether Currents needed for a zone.
    pub fn get_aether_currents_from_zone(
        &mut self,
        aether_current_comp_flg_set_id: u32,
    ) -> Option<Vec<i32>> {
        let aether_current_comp_flg_set_sheet =
            AetherCurrentCompFlgSetSheet::read_from(&mut self.resource, Language::None).unwrap();

        let row = aether_current_comp_flg_set_sheet.row(aether_current_comp_flg_set_id)?;

        let aether_currents_from_zone = row.AetherCurrents;
        Some(aether_currents_from_zone.to_vec())
    }

    pub fn get_screenimage_from_aether_current_comp_flg_set(
        &mut self,
        aether_current_comp_flg_set_id: u32,
    ) -> Option<u32> {
        let aether_current_comp_flg_set_to_screenimage =
            get_aether_current_comp_flg_set_to_screenimage();
        Some(aether_current_comp_flg_set_to_screenimage[&aether_current_comp_flg_set_id])
    }

    /// Returns the internal script name for this CustomTalk event.
    pub fn get_custom_talk_name(&mut self, custom_talk_id: u32) -> String {
        let row = self.custom_talk_sheet.row(custom_talk_id).unwrap();

        row.Name.to_string() // This shouldn't use use SeString features
    }

    /// Returns the internal script name for this Opening event.
    pub fn get_opening_name(&mut self, opening_id: u32) -> String {
        let sheet = OpeningSheet::read_from(&mut self.resource, Language::None).unwrap();
        let row = sheet.row(opening_id).unwrap();

        row.Name.to_string() // This shouldn't use use SeString features
    }

    /// Returns data useful for performing item actions.
    pub fn lookup_item_action_data(&mut self, item_id: u32) -> Option<(u16, [u16; 9], u32)> {
        if let Some(row) = self.item_sheet.row(item_id) {
            let additional_data = row.AdditionalData;

            let item_action_sheet =
                ItemActionSheet::read_from(&mut self.resource, Language::None).ok()?;
            let item_action_row = item_action_sheet.row(row.ItemAction as u32)?;

            return Some((
                item_action_row.Action,
                item_action_row.Data,
                additional_data,
            ));
        }

        None
    }

    /// Returns the target event for a given PreHandler event.
    pub fn get_pre_handler_target(&mut self, pre_handler_id: u32) -> Option<u32> {
        let config = get_config();
        let sheet = PreHandlerSheet::read_from(&mut self.resource, config.world.language()).ok()?;
        let row = sheet.row(pre_handler_id)?;

        Some(row.Target)
    }

    /// Returns a list of SwitchTalkRows for a given id.
    pub fn get_switch_talk_subrows(
        &mut self,
        switch_talk_id: u32,
    ) -> Vec<(u16, SwitchTalkVariationRow)> {
        let subrows = self
            .switch_talk_sheet
            .into_iter()
            .filter(|(row_id, _)| switch_talk_id == *row_id)
            .take(1)
            .next();
        subrows.map(|(_, subrows)| subrows).unwrap_or_default()
    }

    /// Returns the target Transform Row ID for a given selected NPC. (Only applicable to the Halloween Transform NPC.)
    pub fn get_halloween_npc_transform(&mut self, npc_id: u32) -> Option<u16> {
        let config = get_config();
        let sheet =
            HalloweenNpcSelectSheet::read_from(&mut self.resource, config.world.language()).ok()?;
        let row = sheet.row(npc_id)?;

        Some(row.Transformation)
    }

    /// Returns the internal script name for this Quest event.
    pub fn get_quest_name(&mut self, quest_id: u32) -> String {
        let row = self.quest_sheet.row(quest_id).unwrap();

        row.Id.to_string() // This shouldn't use use SeString features
    }

    /// Returns the target event of a TopicSelect event.
    pub fn get_topic_select_target(
        &mut self,
        topic_select_id: u32,
        selected_index: usize,
    ) -> Option<u32> {
        let config = get_config();
        let sheet =
            TopicSelectSheet::read_from(&mut self.resource, config.world.language()).ok()?;
        let row = sheet.row(topic_select_id)?;

        Some(row.Shop[selected_index])
    }

    /// Returns the rewards for this Quest, EXP and Gil respectively.
    /// A quest's EXP and gil rewards for a player of this level. EXP is its ExpFactor times ParamGrow's ScaledQuestXP
    /// and QuestExpModifier at the player's level, over 100 (Sapphire's formula, at the player's level rather than the
    /// quest's.) Checked against retail with Kawari Recorder: Eggs over Queasy (ExpFactor 400, a level 4 quest) gave
    /// 2240 EXP at level 7 (400 * 80 * 7 / 100), and Surveying the Damage 2720 at level 8 (400 * 85 * 8 / 100.)
    pub fn get_quest_rewards(&mut self, quest_id: u32, level: u16) -> (u32, u32) {
        let row = self.quest_sheet.row(quest_id).unwrap();

        // Retail scales a quest's EXP to the player's level only up to the quest's own (ClassJobLevel plus
        // QuestLevelOffset): exact on 30+ turn-ins in Bob's Chronofoil captures (a level 1 quest at level 2 gave 400,
        // not 660.)
        let quest_level = row.ClassJobLevel[0] as u32 + row.QuestLevelOffset as u32;
        let level = if quest_level > 0 {
            (level as u32).min(quest_level)
        } else {
            level as u32
        };
        let exp = self
            .param_grow_sheet
            .row(level)
            .map(|grow| {
                row.ExpFactor as u32 * grow.ScaledQuestXP as u32 * grow.QuestExpModifier as u32
                    / 100
            })
            .unwrap_or(0);
        tracing::info!(
            "Quest {quest_id} gives {exp} EXP (ExpFactor {} at level {level}) and {} gil",
            row.ExpFactor,
            row.GilReward
        );

        (exp, row.GilReward)
    }

    /// The items a quest always gives (not the ones you pick from): its reward items and crystals, as (item, count).
    /// Retail gave a Hi-Potion and 13 Wind Shards for Eggs over Queasy, which are these.
    pub fn get_quest_reward_items(&mut self, quest_id: u32) -> Vec<(u32, u32)> {
        let Some(row) = self.quest_sheet.row(quest_id) else {
            return Vec::new();
        };

        let items = row
            .Reward
            .iter()
            .zip(row.ItemCountReward.iter())
            .filter(|(item, count)| **item != 0 && **count != 0)
            .map(|(item, count)| (*item, *count as u32));
        let crystals = row
            .ItemCatalyst
            .iter()
            .zip(row.ItemCountCatalyst.iter())
            .filter(|(item, count)| **item != 0 && **count != 0)
            .map(|(item, count)| (*item as u32, *count as u32));
        items.chain(crystals).collect()
    }

    /// Returns how many seconds this EventAction takes to cast.
    pub fn get_event_action_cast_time(&mut self, id: u32) -> u8 {
        // It has a Name column, so it only exists per language: read without one it failed and every bar finished
        // after the fallback's 2 s, before the client's (Bob's 2026-10-20 bundle: aethernet shards, 551's destination.)
        EventActionSheet::read_from(&mut self.resource, get_config().world.language())
            .ok()
            .and_then(|sheet| sheet.row(id))
            .map(|row| row.CastTime)
            .unwrap_or(2)
    }

    /// Every achievement with a type, read from the Achievement sheet the first time.
    pub fn achievement_rules(&mut self) -> &[AchievementRule] {
        if self.achievement_rules.is_none() {
            let language = get_config().world.language();
            let rules = match AchievementSheet::read_from(&mut self.resource, language) {
                Ok(sheet) => sheet
                    .into_iter()
                    .filter_map(|(id, rows)| {
                        let (_, row) = rows.into_iter().next()?;
                        (row.Type != 0).then_some(AchievementRule {
                            id,
                            kind: row.Type,
                            key: row.Key,
                            data: row.Data,
                        })
                    })
                    .collect(),
                Err(err) => {
                    tracing::warn!("Couldn't read the Achievement sheet: {err:?}");
                    Vec::new()
                }
            };
            self.achievement_rules = Some(rules);
        }
        self.achievement_rules.as_deref().unwrap_or_default()
    }

    /// Returns the quest this QuestBattle belongs to, its time limit and its variables.
    pub fn get_quest_battle(&mut self, quest_battle_id: u32) -> Option<QuestBattleInfo> {
        let sheet = QuestBattleSheet::read_from(&mut self.resource, Language::None).ok()?;
        let row = sheet.row(quest_battle_id)?;

        let variables = row
            .QuestBattleParams
            .iter()
            .filter(|x| !x.ScriptInstruction.is_empty())
            .map(|x| (x.ScriptInstruction.to_string(), x.ScriptValue))
            .collect();

        Some(QuestBattleInfo {
            quest_id: row.Quest as u32,
            time_limit: row.TimeLimit,
            variables,
        })
    }

    /// Returns how many of `item_id` this Quest gives, if it's one of the optional rewards you choose from.
    pub fn get_quest_optional_reward(&mut self, quest_id: u32, item_id: u32) -> Option<u8> {
        let row = self.quest_sheet.row(quest_id)?;

        row.OptionalItemReward
            .iter()
            .position(|x| *x == item_id)
            .map(|i| row.OptionalItemCountReward[i])
    }

    /// Returns the max EXP or the exp "needed to grow" for a given level.
    /// The EXP for defeating a battle NPC of this level. Like Sapphire, this is ParamGrow's ScaledQuestXP for its level
    /// (the base that quest EXP rewards are scaled from.) Retail's own formula isn't known.
    pub fn get_bnpc_exp(&mut self, level: u8) -> i32 {
        self.param_grow_sheet
            .row(level as u32)
            .map(|row| row.ScaledQuestXP as i32)
            .unwrap_or(0)
    }

    pub fn get_max_exp(&mut self, level: u32) -> i32 {
        let row = self.param_grow_sheet.row(level).unwrap();

        row.ExpToNext
    }

    /// Gets the short name for a given content finder condition.
    pub fn get_content_short_name(&mut self, content_finder_row_id: u16) -> Option<String> {
        let content_finder_row = self
            .content_finder_condition_sheet
            .row(content_finder_row_id as u32)?;

        Some(content_finder_row.ShortCode.to_string()) // This shouldn't use use SeString features
    }

    /// Returns the DefaultTalk for a given FateShop and index.
    pub fn get_fate_default_talk(&mut self, fate_shop_id: u32, index: u8) -> u32 {
        let sheet = FateShopSheet::read_from(&mut self.resource, Language::None).unwrap();
        let row = sheet.row(fate_shop_id).unwrap();

        row.DefaultTalk[index as usize]
    }

    /// Returns the SpecialShop for a given FateShop and rank.
    pub fn get_fate_special_shop(&mut self, fate_shop_id: u32, index: u8) -> u32 {
        let sheet = FateShopSheet::read_from(&mut self.resource, Language::None).unwrap();
        let row = sheet.row(fate_shop_id).unwrap();

        row.SpecialShop[index as usize]
    }

    /// Returns the pop type for this EObj.
    pub fn get_eobj_extra_data(&mut self, eobj_id: u32) -> (u8, bool) {
        let row = self.eobj_sheet.row(eobj_id).unwrap();

        (row.PopType, row.Targetable)
    }

    /// Returns the InstanceContent for a given ContentFinderCondition id.
    pub fn find_content_for_content_finder_id(
        &mut self,
        content_finder_row_id: u16,
    ) -> Option<u16> {
        let content_finder_row = self
            .content_finder_condition_sheet
            .row(content_finder_row_id as u32)?;

        Some(content_finder_row.Content)
    }

    /// Checks if the duty recorder is allowed for this content.
    pub fn is_duty_recorder_allowed(&mut self, content_finder_row_id: u16) -> bool {
        let content_finder_row = self
            .content_finder_condition_sheet
            .row(content_finder_row_id as u32)
            .unwrap();

        content_finder_row.DutyRecorderAllowed
    }

    /// Returns the time limit in minutes for a given InstanceContent id.
    pub fn find_content_time_limit(&mut self, instance_content_id: u16) -> Option<u16> {
        let row = self
            .instance_content_sheet
            .row(instance_content_id as u32)?;

        Some(row.TimeLimitmin)
    }

    /// Returns information about a specific GimmickRect.
    pub fn get_gimmick_rect_info(&mut self, gimmick_rect_id: u32) -> Option<GimmickRectRow> {
        self.gimmick_rect_sheet.row(gimmick_rect_id)
    }

    /// Returns information about a specific GimmickRect based on the layout id.
    pub fn lookup_gimmick_rect(&mut self, layout_id: u32) -> Option<GimmickRectRow> {
        self.gimmick_rect_sheet
            .row(*self.gimmick_rect_lookup.get(&layout_id)?)
    }

    /// Returns the data associated with this EObj.
    pub fn get_eobj_data(&mut self, eobj_id: u32) -> u32 {
        let row = self.eobj_sheet.row(eobj_id).unwrap();
        row.Data
    }

    /// Returns the Map column value on the TerritoryType sheet. Used for revealing portions of ingame maps.
    /// Where a map's discovered parts are kept: its index into the discovery bitmasks, and whether it's one of the
    /// maps with up to 16 parts (two bytes each) rather than 32 (four bytes each.)
    pub fn get_map_discovery_index(&mut self, map_id: u32) -> Option<(i16, bool)> {
        let (index, two_bytes, _) = self.map_discovery(map_id)?;
        Some((index, two_bytes))
    }

    /// Every part of a map there is to discover, as the bits its discovered parts are kept in (Map's DiscoveryFlag:
    /// the Central Shroud's is 16382, parts 1 to 13.) Maps that share a discovery slot (DiscoveryIndex), like the
    /// floors of Alexander - The Cuff of the Father (map 238: parts 1-2, map 255: parts 3-4), are discovered together:
    /// retail completed its Mapping the Realm (Key 238) with part 4 (Chronofoil capture, 2026-09-23), so this is
    /// every part of every map in the slot.
    pub fn get_map_discovery_flag(&mut self, map_id: u32) -> Option<u32> {
        let (index, two_bytes, _) = self.map_discovery(map_id)?;
        if index < 0 {
            return Some(self.map_discovery(map_id)?.2);
        }
        Some(
            self.map_discovery
                .as_ref()?
                .values()
                .filter(|(other, other_two_bytes, _)| {
                    *other == index && *other_two_bytes == two_bytes
                })
                .fold(0, |all, (_, _, flag)| all | flag),
        )
    }

    fn map_discovery(&mut self, map_id: u32) -> Option<(i16, bool, u32)> {
        if self.map_discovery.is_none() {
            let maps = match MapSheet::read_from(&mut self.resource, Language::None) {
                Ok(sheet) => sheet
                    .into_iter()
                    .flatten_subrows()
                    .map(|(id, row)| {
                        (
                            id,
                            (
                                row.DiscoveryIndex,
                                row.DiscoveryArrayByte,
                                row.DiscoveryFlag,
                            ),
                        )
                    })
                    .collect(),
                Err(err) => {
                    tracing::warn!("Couldn't read the Map sheet: {err:?}");
                    HashMap::new()
                }
            };
            self.map_discovery = Some(maps);
        }
        self.map_discovery.as_ref()?.get(&map_id).copied()
    }

    /// The marketboard's items. Read from the Item and ItemSearchCategory sheets the first time.
    pub fn marketboard(&mut self) -> std::sync::Arc<MarketIndex> {
        if let Some(index) = &self.marketboard {
            return index.clone();
        }
        let tabs: HashMap<u32, u8> =
            // It has a Name column, so it only exists per language (Language::None fails to resolve, which left the
            // marketboard empty: 2026-10-17 bundle.)
            match ItemSearchCategorySheet::read_from(&mut self.resource, get_config().world.language()) {
                Ok(sheet) => sheet
                    .into_iter()
                    .flatten_subrows()
                    .map(|(id, row)| (id, row.Category))
                    .collect(),
                Err(err) => {
                    tracing::warn!("Couldn't read the ItemSearchCategory sheet: {err:?}");
                    HashMap::new()
                }
            };
        // Retail lists a category's items by their ItemSortCategory's Param, then their SubcategorySort, then their ID
        // (Bob's Chronofoil captures of 2026-09-15: 7 of the 8 categories browsed in exactly that order; the eighth
        // has two sort categories the other way round.)
        let sort_params: HashMap<u32, u8> =
            match ItemSortCategorySheet::read_from(&mut self.resource, Language::None) {
                Ok(sheet) => sheet
                    .into_iter()
                    .flatten_subrows()
                    .map(|(id, row)| (id, row.Param))
                    .collect(),
                Err(err) => {
                    tracing::warn!("Couldn't read the ItemSortCategory sheet: {err:?}");
                    HashMap::new()
                }
            };
        let mut sort_keys: HashMap<u32, (u8, u16)> = HashMap::new();
        let mut index = MarketIndex::default();
        for (id, row) in self.item_sheet.into_iter().flatten_subrows() {
            let category = row.ItemSearchCategory as u16;
            let Some(&tab) = tabs.get(&(category as u32)) else {
                continue;
            };
            if category == 0 || tab == 0 || row.IsUntradable {
                continue;
            }
            index.items.insert(
                id,
                crate::marketboard::MarketItem {
                    id,
                    category,
                    tab,
                    rarity: row.Rarity,
                    level_equip: row.LevelEquip,
                    level_item: row.LevelItem,
                    can_be_hq: row.CanBeHq,
                    stack_size: row.StackSize,
                },
            );
            index.by_category.entry(category).or_default().push(id);
            sort_keys.insert(
                id,
                (
                    sort_params
                        .get(&(row.ItemSortCategory as u32))
                        .copied()
                        .unwrap_or_default(),
                    row.SubcategorySort,
                ),
            );
        }
        for items in index.by_category.values_mut() {
            items.sort_unstable_by_key(|id| {
                let (param, subcategory) = sort_keys.get(id).copied().unwrap_or_default();
                (param, subcategory, *id)
            });
        }
        tracing::info!(
            "Marketboard: {} items in {} categories",
            index.items.len(),
            index.by_category.len()
        );
        let index = std::sync::Arc::new(index);
        // Kept only if the categories could be read, so a failure is tried again rather than kept until a restart.
        if !tabs.is_empty() {
            self.marketboard = Some(index.clone());
        }
        index
    }

    /// The style a pair of glasses (Glasses sheet row) unlocks, which is what the unlock bitmask counts.
    pub fn get_glasses_style(&mut self, glasses_id: u32) -> Option<u32> {
        // Per language, as it has a Name column (Language::None fails to resolve.)
        let sheet =
            GlassesSheet::read_from(&mut self.resource, get_config().world.language()).ok()?;
        u32::try_from(sheet.row(glasses_id)?.Style).ok()
    }

    pub fn get_territory_info_map_data(&mut self, zone_id: u16) -> Option<u16> {
        let row = self.territory_type_sheet.row(zone_id.into())?;
        Some(row.Map)
    }

    /// Returns the PlaceNameZone column value on the TerritoryType sheet. Used for determining if a zone is located in a certain region in the world.
    pub fn get_territory_placenamezone_data(&mut self, zone_id: u16) -> Option<u16> {
        let row = self.territory_type_sheet.row(zone_id.into())?;
        Some(row.PlaceNameZone)
    }

    /// Returns the entrance ID for this content finder condition.
    pub fn get_content_entrance_id(&mut self, content_finder_id: u16) -> Option<u32> {
        let content_finder_row = self
            .content_finder_condition_sheet
            .row(content_finder_id as u32)?;
        let instance_content_row = self
            .instance_content_sheet
            .row(content_finder_row.Content as u32)?;

        Some(instance_content_row.LGBEventRange)
    }

    /// Looks up a line of quest text by its key, e.g. TEXT_MANFST005_00445_PAPALYMO_BATTLETALK_01, which is in the
    /// quest's own text sheet (quest/004/ManFst005_00445.)
    pub fn get_quest_text(&mut self, key: &str) -> Option<SeString> {
        let config = get_config();

        // TEXT_<SCRIPT>_<ID>_...: the sheet is quest/<ID / 100>/<SCRIPT>_<ID>.
        let mut parts = key.split('_');
        let (Some("TEXT"), Some(script), Some(id)) = (parts.next(), parts.next(), parts.next())
        else {
            return None;
        };
        let folder = id.parse::<u32>().ok()? / 100;
        let sheet_name = format!("quest/{folder:03}/{script}_{id}");

        let exh = self.resource.read_excel_sheet_header(&sheet_name).ok()?;
        let sheet = self
            .resource
            .read_excel_sheet(&exh, &sheet_name, config.world.language())
            .ok()?;
        // Rows are keyed by their first column, e.g. TEXT_MANFST005_00445_PAPALYMO_BATTLETALK_01.
        sheet
            .pages
            .iter()
            .flat_map(|page| page.entries.iter())
            .filter_map(|entry| entry.subrows.first().map(|(_, row)| row))
            .find(|row| {
                row.columns
                    .first()
                    .and_then(|c| c.into_string())
                    .is_some_and(|k| k.to_string() == key)
            })
            .and_then(|row| row.columns.get(1)?.into_string().cloned())
    }

    /// A quest's battle talk lines (their keys contain "BATTLETALK"), as (key, text) sorted by key, e.g. for its quest
    /// battle's enemies to say. Logs what it found, so the order can be checked against a recording.
    pub fn get_quest_battle_talk(&mut self, quest_id: u32) -> Vec<(String, SeString)> {
        let config = get_config();
        let script = self.get_quest_name(quest_id);
        let Some(id) = script
            .rsplit('_')
            .next()
            .and_then(|id| id.parse::<u32>().ok())
        else {
            return Vec::new();
        };
        let sheet_name = format!("quest/{:03}/{script}", id / 100);
        let Ok(exh) = self.resource.read_excel_sheet_header(&sheet_name) else {
            return Vec::new();
        };
        let Ok(sheet) = self
            .resource
            .read_excel_sheet(&exh, &sheet_name, config.world.language())
        else {
            return Vec::new();
        };

        let mut all_keys = Vec::new();
        let mut lines: Vec<(String, SeString)> = sheet
            .pages
            .iter()
            .flat_map(|page| page.entries.iter())
            .filter_map(|entry| entry.subrows.first().map(|(_, row)| row))
            .filter_map(|row| {
                // Keys are plain text.
                let key = row.columns.first()?.into_string()?.to_string();
                all_keys.push(key.clone());
                let text = row.columns.get(1)?.into_string()?.clone();
                key.contains("BATTLETALK").then_some((key, text))
            })
            .collect();
        lines.sort_by(|a, b| a.0.cmp(&b.0));

        if lines.is_empty() {
            tracing::warn!(
                "{sheet_name} has no BATTLETALK lines, its keys are: {}",
                all_keys.join(", ")
            );
        } else {
            tracing::info!(
                "{sheet_name} battle talk: {}",
                lines
                    .iter()
                    .map(|(key, _)| key.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        lines
    }

    /// The (singular) name of a BNpc, from the BNpcName sheet.
    pub fn get_bnpc_name(&mut self, name_id: u32) -> Option<String> {
        let config = get_config();

        let exh = self.resource.read_excel_sheet_header("BNpcName").ok()?;
        let sheet = self
            .resource
            .read_excel_sheet(&exh, "BNpcName", config.world.language())
            .ok()?;
        sheet
            .row(name_id)?
            .columns
            .first()?
            .into_string()
            .map(|name| name.to_string()) // Names don't use SeString features
    }

    /// What the Scenario Guide should show: the main scenario quest that was last completed, the one to do next, and
    /// the layout ID of whoever offers that next one (for the map.) Quest IDs are adjusted (65575 is 39.)
    ///
    /// The main scenario is every quest in the ScenarioTree sheet. An accepted one comes first; otherwise the next one
    /// is the first that isn't done yet, follows on from a completed quest (PreviousQuest, all of them or any of them
    /// by PreviousQuestJoin), and suits the player's class and Grand Company, like Sapphire's quest availability check.
    pub fn get_scenario_guide(
        &mut self,
        is_completed: impl Fn(u32) -> bool,
        active: &[u16],
        classjob: u8,
        first_class: u8,
        grand_company: u8,
    ) -> Option<(u32, u32, u32)> {
        let config = get_config();
        let exh = self.resource.read_excel_sheet_header("ScenarioTree").ok()?;
        // Not every sheet is translated, and asking for a language it doesn't have fails.
        let language = if exh.languages.contains(&config.world.language()) {
            config.world.language()
        } else {
            Language::None
        };
        let sheet = match self
            .resource
            .read_excel_sheet(&exh, "ScenarioTree", language)
        {
            Ok(sheet) => sheet,
            Err(err) => {
                tracing::warn!("Couldn't read the ScenarioTree sheet: {err:?}");
                return None;
            }
        };
        let quest_ids: Vec<u32> = sheet
            .pages
            .iter()
            .flat_map(|page| page.entries.iter())
            .map(|entry| entry.id)
            .collect();

        // Some steps come in variants, one per starting class (Gridania has three Close to Home quests, all after
        // Coming to Gridania), and you only ever do one of them. Variants share a name and the quests before them, so
        // once one is done or accepted, the others are skipped.
        let variant_key = |row: &QuestRow| {
            let mut previous: Vec<u32> = row
                .PreviousQuest
                .iter()
                .copied()
                .filter(|x| *x != 0)
                .collect();
            previous.sort();
            (row.Name.to_string(), previous) // Names don't use SeString features
        };
        let done_variants: HashSet<(String, Vec<u32>)> = quest_ids
            .iter()
            .filter(|id| {
                let adjusted = adjust_quest_id(**id);
                is_completed(adjusted) || active.contains(&(adjusted as u16))
            })
            .filter_map(|id| self.quest_sheet.row(*id))
            .map(|row| variant_key(&row))
            .collect();

        let checked = quest_ids.len();
        let mut next = None;
        for quest_id in quest_ids {
            let adjusted = adjust_quest_id(quest_id);
            let Some(row) = self.quest_sheet.row(quest_id) else {
                continue;
            };
            let previous: Vec<u32> = row
                .PreviousQuest
                .iter()
                .copied()
                .filter(|x| *x != 0)
                .collect();

            if active.contains(&(adjusted as u16)) {
                next = Some((quest_id, row, previous));
                break;
            }
            if is_completed(adjusted) || previous.is_empty() {
                // Quests without one before them start a city's story, and are handed out by the opening.
                continue;
            }
            let follows_on = if row.PreviousQuestJoin == 2 {
                previous.iter().any(|x| is_completed(adjust_quest_id(*x)))
            } else {
                previous.iter().all(|x| is_completed(adjust_quest_id(*x)))
            };
            if !follows_on || done_variants.contains(&variant_key(&row)) {
                continue;
            }
            if row.GrandCompany != 0 && row.GrandCompany != grand_company {
                tracing::info!(
                    "Scenario Guide: skipping {adjusted}, it's for Grand Company {}",
                    row.GrandCompany
                );
                continue;
            }
            let mut suits_class = |category: u8| {
                category <= 1
                    || self
                        .get_applicable_classjobs(category as u16)
                        .contains(&classjob)
            };
            if !suits_class(row.ClassJobCategory0) || !suits_class(row.ClassJobCategory1) {
                tracing::info!(
                    "Scenario Guide: skipping {adjusted}, it's for classes {} / {}",
                    row.ClassJobCategory0,
                    row.ClassJobCategory1
                );
                continue;
            }

            // Between variants nobody has done yet, prefer the one for the class they started with. (Which of these two
            // fields says so isn't confirmed, so either counts.)
            let order = |row: &QuestRow| {
                let for_first_class =
                    row.ClassJobRequired == first_class || row.ClassJobUnlock == first_class;
                (row.ClassJobLevel[0], !for_first_class, row.SortKey)
            };
            let better = match &next {
                Some((_, best, _)) => order(&row) < order(best),
                None => true,
            };
            if better {
                next = Some((quest_id, row, previous));
            }
        }

        let Some((quest_id, row, previous)) = next else {
            tracing::info!(
                "Scenario Guide: checked {checked} main scenario quests, none are available"
            );
            return None;
        };
        let last_completed = previous
            .iter()
            .copied()
            .find(|x| is_completed(adjust_quest_id(*x)))
            .map(adjust_quest_id)
            .unwrap_or_default();
        let layout_id = LevelSheet::read_from(&mut self.resource, Language::None)
            .ok()
            .and_then(|sheet| sheet.row(row.IssuerLocation))
            .map(|level| level.Object)
            .unwrap_or_default();

        Some((last_completed, adjust_quest_id(quest_id), layout_id))
    }

    /// Returns the list of applicable classjob IDs based on the ClassJobCategory.
    pub fn get_applicable_classjobs(&mut self, classjob_category_id: u16) -> Vec<u8> {
        let config = get_config();

        let exh = self
            .resource
            .read_excel_sheet_header("ClassJobCategory")
            .unwrap();
        let sheet = self
            .resource
            .read_excel_sheet(&exh, "ClassJobCategory", config.world.language())
            .unwrap();

        let mut classjobs = Vec::new();
        let row = sheet.row(classjob_category_id as u32).unwrap();

        let applicable_classes = &row.columns[1..]; // First column is the label e.g. "Disciple of War"
        for (i, applicable) in applicable_classes.iter().enumerate() {
            if applicable.into_bool().copied().unwrap_or_default() {
                classjobs.push(i as u8);
            }
        }

        classjobs
    }

    /// Gets the soul crystal item ID for the classjob, if applicable.
    pub fn get_soul_crystal_item_id(&mut self, classjob_id: u16) -> Option<u32> {
        let row = self.classjob_sheet.row(classjob_id as u32)?;

        let item_id = row.ItemSoulCrystal;
        if item_id != 0 {
            return Some(item_id);
        }
        None
    }

    /// Gets the starting level for the classjob.
    pub fn get_starting_level(&mut self, classjob_id: u16) -> Option<u8> {
        let row = self.classjob_sheet.row(classjob_id as u32)?;

        Some(row.StartingLevel)
    }

    /// Returns information about the BaseParam.
    pub fn get_base_param(&mut self, base_param_id: u16) -> Option<BaseParamRow> {
        self.base_param_sheet.row(base_param_id as u32)
    }

    /// Returns the ParamGrow for this level.
    pub fn get_param_grow(&mut self, level: u32) -> Option<ParamGrowRow> {
        self.param_grow_sheet.row(level)
    }

    /// Returns the ParamGrow for this level.
    pub fn get_class_job_modifiers(&mut self, classjob_id: u32) -> Option<Modifiers> {
        let row = self.classjob_sheet.row(classjob_id)?;

        Some(Modifiers {
            hp: row.ModifierHitPoints,
            tank: row.Role == 1,
            mp: row.ModifierManaPoints,
            strength: row.ModifierStrength,
            vitality: row.ModifierVitality,
            dexterity: row.ModifierDexterity,
            intelligence: row.ModifierIntelligence,
            mind: row.ModifierMind,
            piety: row.ModifierPiety,
        })
    }

    /// Gets the classjob ID associated with this soul crystal item ID.
    pub fn get_applicable_classjob(&mut self, soul_crystal_id: u32) -> Option<u32> {
        for (id, row) in self.classjob_sheet.into_iter().flatten_subrows() {
            if row.ItemSoulCrystal == soul_crystal_id {
                return Some(id);
            }
        }

        None
    }

    /// Returns the layout IDs for the map effects of this InstanceContent.
    pub fn get_map_effects(&mut self, content_id: u32) -> Option<Vec<i32>> {
        let instance_content_row = self.instance_content_sheet.row(content_id)?;
        let content_id = instance_content_row.ContentDirectorManagedSG;

        let sheet =
            ContentDirectorManagedSGSheet::read_from(&mut self.resource, Language::None).ok()?;
        let subrows = sheet
            .into_iter()
            .find(|(row_id, _)| *row_id == content_id as u32)?;

        Some(subrows.1.iter().map(|(_, row)| row.LayoutId).collect())
    }

    /// Returns a list of variable name and value pairs for this Opening.
    pub fn get_opening_variables(&mut self, opening_id: u32) -> Vec<(String, u32)> {
        let sheet = OpeningSheet::read_from(&mut self.resource, Language::None).unwrap();
        let row = sheet.row(opening_id).unwrap();

        let mut translated_variables = Vec::new();
        for variable in row.Variables {
            let name = variable.Name;
            let value = variable.Value;

            if !name.is_empty() {
                translated_variables.push((name.to_string(), value));
            }
        }

        translated_variables
    }

    /// Returns a list of variable name and value pairs for this Quest.
    pub fn get_quest_variables(&mut self, quest_id: u32) -> Vec<(String, u32)> {
        let row = self.quest_sheet.row(quest_id).unwrap();

        let mut translated_variables = Vec::new();
        for variable in row.QuestParams {
            let name = variable.ScriptInstruction;
            let value = variable.ScriptArg;

            if !name.is_empty() {
                translated_variables.push((name.to_string(), value));
            }
        }

        translated_variables
    }

    /// Whether a quest (its Quest row, e.g. 66002) listens for emotes on this NPC or object (its base ID) in this
    /// sequence: a listener of event type 2 (Emote) in the Quest sheet, as kawari-questdump lists them.
    pub fn quest_listens_for_emote(&mut self, quest_id: u32, base_id: u32, sequence: u8) -> bool {
        const EMOTE: u8 = 2; // EventType::Emote
        let Some(row) = self.quest_sheet.row(quest_id) else {
            return false;
        };
        row.QuestListenerParams.iter().any(|listener| {
            listener.Listener == base_id
                && listener.QuestUInt8A == EMOTE
                && listener.ActorSpawnSeq == sequence
        })
    }

    /// The unlock links finishing this quest grants: its `REWARD…` parameters (e.g. Spirithold Broken's REWARD0 = 9,
    /// the Recommendations window, and REWARD_LEVE = 5; Close to Home's REWARD_DESION = 1, Return), except the ones
    /// that are amounts (REWARD_GIL…, REWARD_EXP…). The client checks these with IsUnlockLinkUnlocked, e.g.
    /// AgentRecommendList's IsActivatable checks 9.
    pub fn get_quest_unlock_links(&mut self, quest_id: u32) -> Vec<u32> {
        let Some(row) = self.quest_sheet.row(quest_id) else {
            return Vec::new();
        };
        row.QuestParams
            .iter()
            .filter(|variable| {
                let name = variable.ScriptInstruction.to_string();
                name.starts_with("REWARD")
                    && !name.starts_with("REWARD_GIL")
                    && !name.starts_with("REWARD_EXP")
            })
            .map(|variable| variable.ScriptArg)
            .filter(|link| *link != 0)
            .collect()
    }

    /// Returns a list of variable name and value pairs for this CustomTalk.
    pub fn get_custom_talk_variables(&mut self, custom_talk_id: u32) -> Vec<(String, u32)> {
        let row = self.custom_talk_sheet.row(custom_talk_id).unwrap();

        let mut translated_variables = Vec::new();
        for variable in row.Script {
            let name = variable.ScriptInstruction;
            let value = variable.ScriptArg;

            if !name.is_empty() {
                translated_variables.push((name.to_string(), value));
            }
        }

        translated_variables
    }

    /// Tries to guess the latest items for the Fitting Shop.
    /// Since this is server-controlled, we will never know - but we can guess!
    pub fn get_latest_fittingshop_display_ids(&mut self) -> [u8; 8] {
        let sheet =
            FittingShopCategoryItemSheet::read_from(&mut self.resource, Language::None).unwrap();

        let mut display_id_set = HashSet::new();

        // Assuming row 1 is "Latest Trends".
        let subrows = sheet
            .into_iter()
            .filter(|(row_id, _)| 1 == *row_id)
            .take(1)
            .next()
            .unwrap()
            .1;
        for (_, subrow) in subrows {
            // This is needed to weed out certain items that are invalid, but still in the Excel sheet.
            let item_id = subrow.Item;
            if item_id == 0 || (item_id < 1000000 && !self.is_item_valid(item_id as u32)) {
                continue;
            }

            display_id_set.insert(subrow.DisplayId);
        }

        // Sort so the highest display id is sent first.
        let mut display_id_vec: Vec<u8> = display_id_set.into_iter().collect();
        display_id_vec.resize(8, 0); // ensure we are at least eight items
        display_id_vec.sort();
        display_id_vec.reverse();

        let mut display_id_arr = [0u8; 8];
        display_id_arr.copy_from_slice(&display_id_vec);

        display_id_arr
    }

    /// Simple heurestic to determine if the Item is actually filled with useful/valid data.
    pub fn is_item_valid(&mut self, item_id: u32) -> bool {
        let Some(row) = self.item_sheet.row(item_id) else {
            return false;
        };

        !row.Singular.is_empty()
    }

    /// Returns a list of variable name and value pairs for all Aetherytes.
    pub fn get_aetheryte_variables(&mut self) -> Vec<(String, u32)> {
        let sheet =
            AetheryteSystemDefineSheet::read_from(&mut self.resource, Language::None).unwrap();

        let mut variables = Vec::new();
        for (_, row) in sheet.into_iter().flatten_subrows() {
            let name = row.Text;
            let value = row.DefineValue;

            if !name.is_empty() {
                variables.push((name.to_string(), value));
            }
        }

        variables
    }

    /// Returns the base id, level and count for a GatheringPoint.
    pub fn get_gathering_point(&mut self, id: u32) -> (i32, u8, u8, [u16; 2], u8) {
        let row = self.gathering_point_sheet.row(id).unwrap();

        let base_id = row.GatheringPointBase;
        let base_row = self.gathering_point_base_sheet.row(base_id as u32).unwrap();

        (
            base_id,
            base_row.GatheringLevel,
            row.Count,
            row.GatheringPointBonus,
            row.Type,
        )
    }

    /// A gathering point's GatheringPointBase GatheringType: 0 mining, 1 quarrying, 2 logging, 3 harvesting.
    pub fn get_gathering_type(&mut self, id: u32) -> u8 {
        let Some(row) = self.gathering_point_sheet.row(id) else {
            return 0;
        };
        self.gathering_point_base_sheet
            .row(row.GatheringPointBase as u32)
            .map(|base| base.GatheringType as u8)
            .unwrap_or_default()
    }

    /// A GatheringPointBonus row (a point's `gathering_point_bonus`; 0 is none.)
    pub fn get_gathering_point_bonus(&mut self, id: u16) -> Option<GatheringPointBonusInfo> {
        if id == 0 {
            return None;
        }
        let row = GatheringPointBonusSheet::read_from(&mut self.resource, Language::None)
            .ok()?
            .row(id as u32)?;
        Some(GatheringPointBonusInfo {
            condition: row.Condition,
            condition_value: row.ConditionValue,
            bonus_type: row.BonusType,
            bonus_value: row.BonusValue,
        })
    }

    /// Returns the item list for a gathering point.
    pub fn get_gathering_point_items(&mut self, id: u32) -> [GatheringPointItem; 8] {
        let row = self.gathering_point_sheet.row(id).unwrap();

        let base_id = row.GatheringPointBase;
        let base_row = self.gathering_point_base_sheet.row(base_id as u32).unwrap();

        let mut new_items = [GatheringPointItem::default(); 8];
        for (i, new_item) in new_items.iter_mut().enumerate() {
            let row = self
                .gathering_item_sheet
                .row(base_row.Item[i] as u32)
                .unwrap();

            *new_item = GatheringPointItem {
                gathering_id: base_row.Item[i],
                item_id: row.Item,
                level: self
                    .gathering_item_level_convert_table_sheet
                    .row(row.GatheringItemLevel as u32)
                    .unwrap()
                    .GatheringItemLevel,
                hidden: row.IsHidden,
                level_row: row.GatheringItemLevel,
                perception_required: row.PerceptionReq,
                no_boon: row.Unknown2 != u16::MAX,
                bonus_yield: row.Unknown5,
            };
        }

        new_items
    }

    /// An item level's base Gathering and Perception, what the gathering formulas measure against and the node's
    /// window shows: GatherBuddy Reborn's table (`resources/data/gatherbuddy/`), which retail's window matched, and
    /// ItemLevel's for a level it hasn't got.
    pub fn gathering_base_stats(&mut self, level_row: u16) -> (u32, u32) {
        const TABLE: &str = include_str!("../../../resources/data/gatherbuddy/IlvConvertTable.csv");
        let found = TABLE.lines().skip(1).find_map(|line| {
            let mut fields = line
                .split(',')
                .map(|field| field.trim().parse::<u32>().ok());
            match (fields.next()??, fields.next()??, fields.next()??) {
                (level, gathering, perception) if level == level_row as u32 => {
                    Some((gathering, perception))
                }
                _ => None,
            }
        });
        if let Some(found) = found {
            return found;
        }
        self.item_level_sheet
            .row(level_row as u32)
            .map(|row| (row.Gathering as u32, row.Perception as u32))
            .unwrap_or_default()
    }

    /// When a timed gathering point is up, in Eorzea minutes of the day (start, end): GatheringPointTransient's
    /// ephemeral window, or GatheringRarePopTimeTable's for unspoiled and legendary points. Its Duration 160 is two
    /// Eorzea hours and 300 three (GatherBuddy and Teamcraft read the column differently but agree on that.) Empty if
    /// the point isn't timed.
    pub fn get_gathering_point_windows(&mut self, id: u32) -> Vec<(u32, u32)> {
        let Some(transient) =
            GatheringPointTransientSheet::read_from(&mut self.resource, Language::None)
                .ok()
                .and_then(|sheet| sheet.row(id))
        else {
            return Vec::new();
        };
        let hhmm = |value: u16| (value as u32 / 100) * 60 + value as u32 % 100;

        let mut windows = Vec::new();
        if transient.EphemeralStartTime != u16::MAX
            && transient.EphemeralStartTime != transient.EphemeralEndTime
        {
            let end = if transient.EphemeralEndTime == 0 {
                1440
            } else {
                hhmm(transient.EphemeralEndTime)
            };
            windows.push((hhmm(transient.EphemeralStartTime), end % 1440));
        }
        if transient.GatheringRarePopTimeTable > 0
            && let Some(table) =
                GatheringRarePopTimeTableSheet::read_from(&mut self.resource, Language::None)
                    .ok()
                    .and_then(|sheet| sheet.row(transient.GatheringRarePopTimeTable as u32))
        {
            for (start, duration) in table.StartTime.iter().zip(table.Duration.iter()) {
                if *start == u16::MAX || *duration == 0 {
                    continue;
                }
                let minutes = match duration {
                    160 => 120,
                    300 => 180,
                    other => *other as u32,
                };
                let start = hhmm(*start);
                windows.push((start, (start + minutes) % 1440));
            }
        }
        windows
    }

    /// Whether one can only have one of this item (the Item sheet's IsUnique): a coffer's LootItem says so.
    pub fn item_is_unique(&mut self, item_id: u32) -> bool {
        self.item_sheet
            .row(item_id)
            .map(|row| row.IsUnique)
            .unwrap_or_default()
    }

    /// Returns the ClassJobCategory for this item.
    pub fn get_item_classjobcategory(&mut self, item_id: u32) -> u8 {
        let row = self.item_sheet.row(item_id).unwrap();
        row.ClassJobCategory
    }

    /// Returns a Recipe.
    pub fn get_recipe(&mut self, id: u32) -> Recipe {
        let sheet = RecipeSheet::read_from(&mut self.resource, Language::None).unwrap();
        let row = sheet.row(id).unwrap();

        Recipe {
            id,
            item_id: row.ItemResult,
        }
    }

    /// A recipe's numbers for a crafter of `crafter_level`, or None if there's no such recipe.
    pub fn get_recipe_info(&mut self, id: u32, crafter_level: u8) -> Option<RecipeInfo> {
        let sheet = RecipeSheet::read_from(&mut self.resource, Language::None).ok()?;
        let row = sheet.row(id)?;

        let mut rlvl = row.RecipeLevelTable as u32;
        let mut max_durability = None;
        // Level-adjusted recipes: the crafter's level (up to MaxAdjustableJobLevel) picks the recipe level, and the
        // durability is always 80 (raphael's `get_game_settings`.)
        if row.MaxAdjustableJobLevel != 0 {
            let adjust =
                GathererCrafterLvAdjustTableSheet::read_from(&mut self.resource, Language::None)
                    .ok()?;
            let level = (crafter_level as u16).min(row.MaxAdjustableJobLevel);
            rlvl = adjust.row(level as u32)?.RecipeLevel as u32;
            max_durability = Some(80);
        }
        let levels = RecipeLevelTableSheet::read_from(&mut self.resource, Language::None).ok()?;
        let level = levels.row(rlvl)?;

        let mut ingredients = Vec::new();
        for (item_id, amount) in row.Ingredient.iter().zip(row.AmountIngredient.iter()) {
            if *item_id <= 0 || *amount == 0 {
                continue;
            }
            let item = self.item_sheet.row(*item_id as u32);
            ingredients.push(RecipeIngredient {
                item_id: *item_id as u32,
                amount: *amount as u32,
                item_level: item.as_ref().map(|item| item.LevelItem as u32).unwrap_or(0),
                can_be_hq: item.as_ref().map(|item| item.CanBeHq).unwrap_or(false),
            });
        }

        // The client works the maxima out the same way when it opens the Synthesis window (7.57, 0x140EA7960.)
        Some(RecipeInfo {
            id,
            item_id: row.ItemResult.max(0) as u32,
            amount_result: row.AmountResult.max(1) as u32,
            class_job_level: level.ClassJobLevel,
            max_progress: level.Difficulty as u32 * row.DifficultyFactor as u32 / 100,
            max_quality: (level.Quality as u64 * row.QualityFactor as u64 / 100) as u32,
            max_durability: max_durability
                .unwrap_or(level.Durability as u32 * row.DurabilityFactor as u32 / 100),
            conditions_flag: level.ConditionsFlag,
            progress_divider: level.ProgressDivider,
            quality_divider: level.QualityDivider,
            progress_modifier: level.ProgressModifier,
            quality_modifier: level.QualityModifier,
            material_quality_factor: row.MaterialQualityFactor,
            can_hq: row.CanHq,
            is_expert: row.IsExpert,
            exp_rewarded: row.ExpRewarded,
            ingredients,
        })
    }

    /// A class's row of a CraftAction, from its Carpenter's row (the sheet's CRP…CUL columns; ClassJob 8–15.)
    pub fn craft_action_for_class(&mut self, base_id: u32, class_job: u8) -> u32 {
        let language = get_config().world.language();
        let Ok(sheet) = CraftActionSheet::read_from(&mut self.resource, language) else {
            return base_id;
        };
        let Some(row) = sheet.row(base_id) else {
            return base_id;
        };
        let id = match class_job {
            9 => row.BSM,
            10 => row.ARM,
            11 => row.GSM,
            12 => row.LTW,
            13 => row.WVR,
            14 => row.ALC,
            15 => row.CUL,
            _ => row.CRP,
        };
        if id > 0 { id as u32 } else { base_id }
    }

    /// GatheringExp's EXP for a level (0 if there's none.)
    pub fn get_gathering_exp(&mut self, level: u32) -> i32 {
        GatheringExpSheet::read_from(&mut self.resource, Language::None)
            .ok()
            .and_then(|sheet| sheet.row(level))
            .map(|row| row.Exp)
            .unwrap_or(0)
    }

    /// Returns a CraftAction's animation start/end.
    pub fn get_craft_action_animations(&mut self, id: u32) -> (u16, u16) {
        let config = get_config();
        let sheet =
            CraftActionSheet::read_from(&mut self.resource, config.world.language()).unwrap();
        let row = sheet.row(id).unwrap();

        (row.AnimationStart, row.AnimationEnd)
    }

    /// Returns a list of priorities for each online status.
    pub fn online_status_priorities(&mut self) -> Vec<u8> {
        let mut priorities = Vec::new();
        let config = get_config();

        let sheet =
            OnlineStatusSheet::read_from(&mut self.resource, config.world.language()).unwrap();
        for (_, row) in sheet.into_iter().flatten_subrows() {
            priorities.push(row.Priority);
        }

        priorities
    }

    /// Whether an action takes part in combos: it continues one (its ActionCombo is set) or starts one (another action's
    /// ActionCombo is it.) Only these change a player's combo on retail; other spells and weaponskills in between don't
    /// break it (Bob's Chronofoil captures: 263 combo steps after other GCDs kept full potency.)
    pub fn takes_part_in_combos(&mut self, action_id: u32) -> bool {
        if self.combo_starters.is_none() {
            self.combo_starters = Some(
                self.action_sheet
                    .into_iter()
                    .flatten_subrows()
                    .map(|(_, row)| row.ActionCombo)
                    .filter(|combo| *combo != 0)
                    .collect(),
            );
        }
        self.get_combo_action(action_id) != 0
            || self
                .combo_starters
                .as_ref()
                .is_some_and(|starters| starters.contains(&(action_id as u16)))
    }

    /// Whether a status can be removed by Esuna and the like (Status.CanDispel.)
    pub fn is_dispellable(&mut self, status_id: u16) -> bool {
        if self.dispellable_statuses.is_none() {
            let sheet = icarus::Status::StatusSheet::read_from(
                &mut self.resource,
                get_config().world.language(),
            );
            self.dispellable_statuses = Some(
                sheet
                    .map(|sheet| {
                        sheet
                            .into_iter()
                            .flatten_subrows()
                            .filter(|(_, row)| row.CanDispel)
                            .map(|(id, _)| id as u16)
                            .collect()
                    })
                    .unwrap_or_default(),
            );
        }
        self.dispellable_statuses
            .as_ref()
            .is_some_and(|statuses| statuses.contains(&status_id))
    }

    /// Returns the item level this content syncs to, if it has its own (ContentFinderCondition.ItemLevelSync: 420,
    /// 440, 470, 505 and 60 on retail in Bob's captures); otherwise it's the synced level's ParamGrow.ItemLevelSync.
    pub fn find_content_synced_item_level(&mut self, content_finder_row_id: u16) -> Option<u16> {
        let content_finder_row = self
            .content_finder_condition_sheet
            .row(content_finder_row_id as u32)?;

        Some(content_finder_row.ItemLevelSync).filter(|x| *x != 0)
    }

    /// The roles one party of this duty is made of (ContentFinderCondition's ContentMemberType), for duties of a single
    /// party of two to eight: Sastasha's is a tank, a healer and two DPS. None for alliances and solo duties.
    pub fn content_party_roles(&mut self, content_finder_row_id: u16) -> Option<PartyRoles> {
        let content_finder_row = self
            .content_finder_condition_sheet
            .row(content_finder_row_id as u32)?;
        let member_type = self
            .content_member_type_sheet
            .row(content_finder_row.ContentMemberType as u32)?;
        if member_type.PartyCount > 1
            || member_type.AlliancePartyCount > 1
            || !(2..=8).contains(&member_type.MembersPerParty)
        {
            return None;
        }
        Some(PartyRoles {
            members: member_type.MembersPerParty,
            tanks: member_type.TanksPerParty,
            healers: member_type.HealersPerParty,
            melee: member_type.MeleesPerParty,
            ranged: member_type.RangedPerParty,
        })
    }

    /// Returns the synced level for this content.
    pub fn find_content_synced_level(&mut self, content_finder_row_id: u16) -> Option<u8> {
        let content_finder_row = self
            .content_finder_condition_sheet
            .row(content_finder_row_id as u32)?;

        Some(content_finder_row.ClassJobLevelSync).filter(|x| *x != 0)
    }

    /// Returns the attributes for a given item level;
    pub fn get_item_level_attributes(&mut self, item_level: u16) -> [u16; 6] {
        let row = self.item_level_sheet.row(item_level as u32).unwrap();

        [
            row.Strength,
            row.Dexterity,
            row.Vitality,
            row.Intelligence,
            row.Mind,
            row.Piety,
        ]
    }

    /// What an equipped item gives: its attributes (with the HQ bonuses, BaseParamValueSpecial, when `hq`), defense
    /// and magic defense, capped under item level sync `sync_item_level` (docs/src/fork/level-sync.md). Fitted to Bob's
    /// Chronofoil captures of retail (a White Mage at 86, and synced to 50 and to 18): each attribute is capped at
    /// `round(ItemLevel[sync][attribute] × BaseParam[attribute].<slot>Percent × BaseParam[attribute].MeldParam[item's
    /// BaseParamModifier] / 100000)`, and defense and magic defense scale by ItemLevel[sync] / ItemLevel[item's level];
    /// items at or below the sync item level are unchanged. `None` if the item isn't in the sheet.
    pub fn equipped_item_stats(
        &mut self,
        item_id: u32,
        hq: bool,
        sync_item_level: Option<u16>,
    ) -> Option<SyncedItemStats> {
        let item = self.item_sheet.row(item_id)?;
        let mut stats = SyncedItemStats {
            params: std::array::from_fn(|i| (item.BaseParam[i], item.BaseParamValue[i] as i32)),
            defense: item.DefensePhys as u32,
            magic_defense: item.DefenseMag as u32,
        };
        if hq {
            for (param, bonus) in item.BaseParamSpecial.iter().zip(item.BaseParamValueSpecial) {
                match *param {
                    0 => {}
                    BASE_PARAM_DEFENSE => {
                        stats.defense = stats.defense.saturating_add_signed(bonus as i32)
                    }
                    BASE_PARAM_MAGIC_DEFENSE => {
                        stats.magic_defense =
                            stats.magic_defense.saturating_add_signed(bonus as i32)
                    }
                    _ => {
                        // Only attributes the item already has (weapon damage bonuses aren't counted here.)
                        if let Some((_, value)) =
                            stats.params.iter_mut().find(|(id, _)| id == param)
                        {
                            *value += bonus as i32;
                        }
                    }
                }
            }
        }
        let Some(sync_item_level) = sync_item_level.filter(|sync| item.LevelItem > *sync) else {
            return Some(stats);
        };
        let synced = self.item_level_sheet.row(sync_item_level as u32)?;
        let own = self.item_level_sheet.row(item.LevelItem as u32)?;
        for (param, value) in &mut stats.params {
            if *param == 0 {
                continue;
            }
            let Some(base) = item_level_value(&synced, *param) else {
                continue;
            };
            let Some(base_param) = self.base_param_sheet.row(*param as u32) else {
                continue;
            };
            let percent = slot_percent(&base_param, item.EquipSlotCategory) as f64;
            let meld = base_param
                .MeldParam
                .get(item.BaseParamModifier as usize)
                .copied()
                .unwrap_or(100) as f64;
            *value = (*value).min(sync_cap(base, percent, meld));
        }
        stats.defense = scale_by_item_level(stats.defense, synced.Defense, own.Defense);
        stats.magic_defense =
            scale_by_item_level(stats.magic_defense, synced.MagicDefense, own.MagicDefense);
        Some(stats)
    }

    /// An action's recast: (CooldownGroup, Recast100ms, MaxCharges.)
    pub fn get_action_recast(&mut self, id: u32) -> Option<(u8, u16, u8)> {
        let row = self.action_sheet.row(id)?;
        Some((row.CooldownGroup, row.Recast100ms, row.MaxCharges))
    }

    pub fn get_action_cooldown_group(&mut self, id: u32) -> u8 {
        let row = self.action_sheet.row(id).unwrap();

        row.CooldownGroup
    }

    /// Returns the targeting and cost information for an action, used for area-of-effect and MP costs.
    pub fn get_action_info(&mut self, id: u32) -> Option<ActionInfo> {
        let row = self.action_sheet.row(id)?;

        // Only cones need their omen, for their angle.
        let cone_angle = if matches!(row.CastType, 3 | 13) {
            OmenSheet::read_from(&mut self.resource, Language::None)
                .ok()
                .and_then(|sheet| sheet.row(row.Omen as u32))
                .and_then(|omen| cone_angle_from_omen(&omen.Path.to_string()))
                .unwrap_or(DEFAULT_CONE_ANGLE)
        } else {
            DEFAULT_CONE_ANGLE
        };

        Some(ActionInfo {
            cast_type: row.CastType,
            effect_range: row.EffectRange,
            range: row.Range,
            can_target_hostile: row.CanTargetHostile,
            primary_cost_type: row.PrimaryCostType,
            primary_cost_value: row.PrimaryCostValue,
            x_axis_modifier: row.XAxisModifier,
            cone_angle,
        })
    }

    /// Checks if this zone is associated with a ContentFinderCondition.
    pub fn is_zone_associated_with_content(&mut self, zone_id: u16) -> bool {
        let Some(row) = self.territory_type_sheet.row(zone_id.into()) else {
            return false;
        };
        row.ContentFinderCondition != 0
    }

    /// Tells us if this item belongs to the Seasonal Miscellany or Miscellany categories.
    pub fn item_is_misc(&mut self, id: u32) -> bool {
        let Some(row) = self.item_sheet.row(id) else {
            return false;
        };

        let is_seasonal_misc = row.ItemUICategory == 85;
        let is_misc = row.ItemUICategory == 61;

        is_seasonal_misc || is_misc
    }

    /// Checks if this zone is valid.
    pub fn is_zone_valid(&mut self, zone_id: u16) -> bool {
        let Some(row) = self.territory_type_sheet.row(zone_id.into()) else {
            return false;
        };
        !row.Bg.is_empty()
    }

    /// Returns the emote mode (if any), really only relevant for persistent/loopable emotes.
    pub fn get_emote_mode(&mut self, emote_id: u32) -> Option<u8> {
        let config = get_config();
        let sheet = EmoteSheet::read_from(&mut self.resource, config.world.language()).ok()?;
        let row = sheet.row(emote_id)?;

        let mode = row.EmoteMode;
        if mode != 0 {
            return Some(mode);
        }
        None
    }

    pub fn get_mount_id_from_name(&mut self, mount_name: String) -> Option<u16> {
        let config = get_config();
        let sheet = MountSheet::read_from(&mut self.resource, config.world.language()).ok()?;
        for (id, row) in sheet.into_iter().flatten_subrows() {
            if row
                .Singular
                .to_string() // This shouldn't use use SeString features
                .to_lowercase()
                .contains(&mount_name.to_lowercase())
            {
                return Some(id as u16);
            }
        }

        None
    }

    /// Returns the list of battalions that should be considered an enemy to this battalion.
    pub fn get_battalion_enemies(&mut self, battalion_id: u32) -> Vec<bool> {
        let row = self.battalion_sheet.row(battalion_id).unwrap();
        row.IsEnemyTo.to_vec()
    }

    /// Returns a ContentFinderCondition for a given roulette.
    pub fn pick_roulette_duty(&mut self, roulette: Roulette) -> u32 {
        let rows: Vec<u32> = self
            .content_finder_condition_sheet
            .into_iter()
            .flatten_subrows()
            .filter(|(_, row)| match roulette {
                Roulette::Leveling => row.LevelingRoulette,
                Roulette::HighLevel => row.HighLevelRoulette,
                Roulette::MSQ => row.MSQRoulette,
                Roulette::GuildHest => row.GuildHestRoulette,
                Roulette::Expert => row.ExpertRoulette,
                Roulette::Trial => row.TrialRoulette,
                Roulette::DailyFrontline => row.DailyFrontlineChallenge,
                Roulette::LevelCap => row.LevelCapRoulette,
                Roulette::Mentor => row.MentorRoulette,
                Roulette::Alliance => row.AllianceRoulette,
                Roulette::NormalRaid => row.NormalRaidRoulette,
                Roulette::CrystallineConflictCasual => row.CrystallineConflictCasualRoulette,
                Roulette::CrystallineConflictRanked => row.CrystallineConflictRankedRoulette,
            })
            .map(|(id, _)| id)
            .collect();

        fastrand::choice(rows).unwrap()
    }

    /// Returns the name ID for a given NpcYell.
    pub fn get_npc_yell_name_id(&mut self, npc_yell_id: u32) -> Option<u32> {
        let config = get_config();
        let sheet = NpcYellSheet::read_from(&mut self.resource, config.world.language()).ok()?;
        let row = sheet.row(npc_yell_id)?;

        Some(row.Name)
    }

    /// Whether discovering this zone's map reveals all of it at once: on retail, A Realm Reborn's field zones
    /// (ExVersion 0, TerritoryIntendedUse Overworld). Exactly these 17 zones' "Mapping the Realm" achievements say
    /// "Visit ... and unlock the area map"; every other one says "Discover every location" (XIVAPI, 2026-10-17).
    pub fn reveals_whole_map(&mut self, zone_id: u16) -> bool {
        self.territory_type_sheet
            .row(zone_id.into())
            .is_some_and(|row| {
                row.ExVersion == 0
                    && row.TerritoryIntendedUse == TerritoryIntendedUse::Overworld as u8
            })
    }

    /// The expansion a zone belongs to (TerritoryType's ExVersion: 0 A Realm Reborn, 1 Heavensward...)
    pub fn get_ex_version(&mut self, zone_id: u16) -> Option<u8> {
        Some(self.territory_type_sheet.row(zone_id.into())?.ExVersion)
    }

    /// Returns the TerritoryIntendedUse for a given zone.
    pub fn get_intended_use(&mut self, zone_id: u32) -> Option<TerritoryIntendedUse> {
        let row = self.territory_type_sheet.row(zone_id)?;
        TerritoryIntendedUse::from_repr(row.TerritoryIntendedUse)
    }

    /// Returns the plot size for a given land set in a housing district.
    pub fn get_land_set_size(&mut self, housing_id: u32, plot_index: usize) -> Option<PlotSize> {
        let sheet = HousingLandSetSheet::read_from(&mut self.resource, Language::None).ok()?;
        let row = sheet.row(housing_id)?;

        PlotSize::from_repr(row.LandSet[plot_index].PlotSize)
    }

    /// Returns the land sets in a housing district.
    pub fn get_land_sets(&mut self, housing_id: u32) -> Option<Vec<LandSetElement>> {
        let sheet = HousingLandSetSheet::read_from(&mut self.resource, Language::None).ok()?;
        let row = sheet.row(housing_id)?;

        Some(row.LandSet.to_vec())
    }

    /// Returns a CommonSpawn with the NPC's equipment.
    pub fn get_npc_equip(&mut self, equip_id: u32) -> Option<CommonSpawn> {
        let row = self.npc_equip_sheet.row(equip_id)?;

        // TODO: support dyes
        Some(CommonSpawn {
            main_weapon_model: WeaponModelId::from(row.ModelMainHand),
            sec_weapon_model: WeaponModelId::from(row.ModelOffHand),
            models: [
                LegacyEquipmentModelId::from(row.ModelHead),
                LegacyEquipmentModelId::from(row.ModelBody),
                LegacyEquipmentModelId::from(row.ModelHands),
                LegacyEquipmentModelId::from(row.ModelLegs),
                LegacyEquipmentModelId::from(row.ModelFeet),
                LegacyEquipmentModelId::from(row.ModelEars),
                LegacyEquipmentModelId::from(row.ModelNeck),
                LegacyEquipmentModelId::from(row.ModelWrists),
                LegacyEquipmentModelId::from(row.ModelLeftRing),
                LegacyEquipmentModelId::from(row.ModelRightRing),
            ],
            glasses_ids: row.Glasses,
            ..Default::default()
        })
    }

    /// Returns a piece of furniture's catalog id from its item id. In this context, "catalog id" means the low 12 bits of the furniture's row number on the HousingFurniture/HousingYardObject sheet, which is what we send to the client when placing furniture.
    pub fn get_furniture_item_id(&mut self, item_id: u32) -> Option<u16> {
        // First, we need the item's AdditionalData column to point us to where we need to look on the HousingFurniture sheet.
        let row = self.item_sheet.row(item_id)?;
        let item_ui_category = row.ItemUICategory;

        // TODO: A better way to do this is checking if the link in AdditionalData leads to the HousingFurniture/HousingYardObject sheet or not.
        // In order: Furnishing, Outdoor Furnishing, Table, Tabletop, Wall-mounted, Rug
        let acceptable_ui_categories = [57, 76, 77, 78, 79, 80];
        let next_row = row.AdditionalData;

        // If the item is a piece of furniture, return the low 12 bits of the row number.
        if acceptable_ui_categories.contains(&item_ui_category) && next_row != 0 {
            return Some((next_row & 0x0FFF) as u16);
        }

        None
    }

    /// Returns the damage element of this action.
    pub fn get_action_damage_element(&mut self, id: u32) -> DamageElement {
        let row = self.action_sheet.row(id).unwrap();
        DamageElement::from_repr(row.Aspect).unwrap() // Should never fail under normal circumstances
    }

    /// Returns the id of the action this action comboes with.
    pub fn get_combo_action(&mut self, id: u32) -> u16 {
        let row = self.action_sheet.row(id).unwrap();
        row.ActionCombo
    }

    /// Returns the ContentFinderCondition for a given IKDRoute.
    pub fn lookup_ikd_route_content(&mut self, id: u32) -> u32 {
        let row = self.ikd_route_sheet.row(id).unwrap();

        row.Instance
    }

    /// Returns the IKDRoute's spots for a given ContentFinderCondition id.
    pub fn lookup_ikd_route_spots_via_content(&mut self, id: u32) -> Option<[u32; 3]> {
        self.ikd_route_sheet
            .into_iter()
            .flatten_subrows()
            .find(|x| x.1.Instance == id)
            .map(|x| x.1.Spot)
    }

    /// Returns the EObj base ID for a given event ID, if none returns 0.
    pub fn find_eobj_base_id(&mut self, id: u32) -> u32 {
        self.eobj_sheet
            .into_iter()
            .flatten_subrows()
            .find(|x| x.1.Data == id)
            .map(|x| x.0)
            .unwrap_or_default()
    }

    /// Returns the pop range ID for this public content, if one is defined.
    pub fn find_public_content_pop_range(&mut self, id: u32) -> Option<u32> {
        Some(self.public_content_sheet.row(id)?.LGBPopRange).filter(|x| *x != 0)
    }

    /// Returns the type of PublicContent this is.
    pub fn find_public_content_type(&mut self, id: u32) -> Option<PublicContentType> {
        PublicContentType::from_repr(self.public_content_sheet.row(id)?.Type)
    }

    /// A FATE's rewards at gold rank, as retail gave them (Bob's Chronofoil captures: FATE 222, level 13, 1320 EXP and
    /// 26 gil; FATE 1409, level 50, 11700 EXP and 100 gil): EXP is ParamGrow's ScaledQuestXP × QuestExpModifier at the
    /// FATE's level, gil twice its level. Returns (EXP, gil.)
    pub fn get_fate_rewards(&mut self, id: u32) -> Option<(u32, u32)> {
        let level = self.fate_sheet.row(id)?.ClassJobLevel as u32;
        let grow = self.param_grow_sheet.row(level)?;
        Some((
            grow.ScaledQuestXP as u32 * grow.QuestExpModifier as u32,
            level * 2,
        ))
    }

    /// Returns the max level for a FATE.
    pub fn get_fate_max_level(&mut self, id: u32) -> Option<u8> {
        let row = self.fate_sheet.row(id)?;
        Some(row.ClassJobLevelMax)
    }

    /// Returns the max level for a FATE
    pub fn find_fate_by_event_range(&mut self, id: u32) -> Option<u32> {
        self.fate_event_range_lookup.get(&id).copied()
    }

    /// Returns the FateRule for a FATE.
    pub fn get_fate_rule(&mut self, id: u32) -> Option<FateRule> {
        let row = self.fate_sheet.row(id)?;
        FateRule::from_repr(row.Rule)
    }

    /// Returns the list of ContentFinderCondition IDs in DawnContent.
    pub fn list_dawn_content_content_finder_conditions(&mut self) -> Vec<(u32, u32)> {
        self.dawn_content_sheet
            .into_iter()
            .flatten_subrows()
            .map(|x| (x.0, x.1.Content))
            .skip(1) // Don't consider the first empty row
            .collect()
    }

    /// Returns the ContentFinderCondition for this DawnContent.
    pub fn get_dawn_content_cfc_id(&mut self, id: u32) -> Option<u32> {
        Some(self.dawn_content_sheet.row(id)?.Content)
    }

    /// Returns the FateChain and special content information for this FATE.
    pub fn get_fate_chain_special(&mut self, id: u32) -> Option<(u32, bool)> {
        let row = self.fate_sheet.row(id)?;
        // TODO: Unknown8 seems to be a marker for special fates, despite there being another column for that?!
        Some((row.FATEChain, row.Unknown8))
    }

    /// Returns the FateProgressUI row for a given TerritoryType ID, if available.
    pub fn get_fate_progress_ui_row(&mut self, territorytype_id: i32) -> Option<u32> {
        self.fate_progress_ui_sheet
            .into_iter()
            .flatten_subrows()
            .find(|x| x.1.Location == territorytype_id)
            .map(|x| x.0)
    }

    /// Returns the maximum number of FATEs needed for this FateProgressUI row.
    pub fn get_fate_progress_maximum_count(&mut self, row: u32) -> Option<u8> {
        let row = self.fate_progress_ui_sheet.row(row)?;
        Some(row.ReqFatesToRank2 + row.ReqFatesToRank3 + row.ReqFatesToRank4)
    }

    /// Determines the current FATE rank based on the FateProgressUI row and number of FATEs completed.
    pub fn get_fate_rank(&mut self, row: u32, count: u8) -> Option<u8> {
        let row = self.fate_progress_ui_sheet.row(row)?;
        if count > row.ReqFatesToRank4 {
            return Some(4);
        }
        if count > row.ReqFatesToRank3 {
            return Some(3);
        }
        if count > row.ReqFatesToRank2 {
            return Some(2);
        }
        if count > 0 {
            return Some(1);
        }
        Some(0)
    }

    /// Returns the ContentFinderCondition for a given GoldSaucerContent.
    pub fn lookup_gold_saucer_content(&mut self, id: u32) -> u32 {
        let row = self.gold_saucer_content_sheet.row(id).unwrap();

        row.ContentFinderCondition as u32
    }

    /// Returns information about a GimmickAccessor.
    pub fn lookup_gimmick_accessor(
        &mut self,
        id: u32,
    ) -> Option<(GimmickAccessorType, u32, u32, u32)> {
        let row = self.gimmick_accessor_sheet.row(id)?;
        Some((
            GimmickAccessorType::from_repr(row.Param0)?, // TODO: Rename to Type upstream
            row.Param1,                                  // TODO: Rename to Param0
            row.Param2,                                  // TODO: Rename to Param1
            row.Type,                                    // TODO: Rename to Param2
        ))
    }

    /// Look up the requirements for a given Trait.
    pub fn get_trait_requirements(&mut self, id: u32) -> Option<(u8, u32)> {
        let row = self.trait_sheet.row(id)?;
        Some((row.Level, row.Quest))
    }

    /// Returns this GATE's information.
    pub fn get_gate(&mut self, id: u32) -> Option<GFATERow> {
        self.gfate_sheet.row(id)
    }
}

impl mlua::UserData for GameData {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("get_pre_handler_target", |_, this, pre_handler_id: u32| {
            Ok(this.get_pre_handler_target(pre_handler_id))
        });
        methods.add_method_mut("get_halloween_npc_transform", |_, this, npc_id: u32| {
            Ok(this.get_halloween_npc_transform(npc_id))
        });
        methods.add_method_mut(
            "get_topic_select_target",
            |_, this, (topic_select_id, selected_topic): (u32, usize)| {
                Ok(this.get_topic_select_target(topic_select_id, selected_topic))
            },
        );
        methods.add_method_mut(
            "get_fate_default_talk",
            |_, this, (fate_shop_id, index): (u32, u8)| {
                Ok(this.get_fate_default_talk(fate_shop_id, index))
            },
        );
        methods.add_method_mut(
            "get_fate_special_shop",
            |_, this, (fate_shop_id, index): (u32, u8)| {
                Ok(this.get_fate_special_shop(fate_shop_id, index))
            },
        );
        methods.add_method_mut("lookup_ikd_route_content", |_, this, id: u32| {
            Ok(this.lookup_ikd_route_content(id))
        });
        methods.add_method_mut("find_eobj_base_id", |_, this, id: u32| {
            Ok(this.find_eobj_base_id(id))
        });
        methods.add_method_mut("lookup_gold_saucer_content", |_, this, id: u32| {
            Ok(this.lookup_gold_saucer_content(id))
        });
    }
}

/// Simple enum for GameData::get_territory_name
pub enum TerritoryNameKind {
    Internal,
    Region,
    Place,
}

/// Wrapper around SqPackResource to let us spy when it reads files
#[derive(Clone)]
struct SqPackResourceSpy {
    sqpack_resource: SqPackResource,
    output_directory: String,
}

impl SqPackResourceSpy {
    pub fn from(sqpack_resource: SqPackResource, output_directory: &str) -> Self {
        Self {
            sqpack_resource,
            output_directory: output_directory.to_string(),
        }
    }
}

impl Resource for SqPackResourceSpy {
    fn read(&mut self, path: &str) -> physis::Result<physis::ByteBuffer> {
        if let Ok(buffer) = self.sqpack_resource.read(path) {
            if !self.output_directory.is_empty() {
                let mut new_path = PathBuf::from(&self.output_directory);
                new_path.push(path.to_lowercase());

                if !std::fs::exists(&new_path).unwrap_or_default() {
                    // create directory if it doesn't exist'
                    let parent_directory = new_path.parent().unwrap();
                    if !std::fs::exists(parent_directory).unwrap_or_default() {
                        std::fs::create_dir_all(parent_directory)
                            .expect("Couldn't create directory for extraction?!");
                    }

                    std::fs::write(new_path, &buffer).expect("Couldn't extract file!!");
                }
            }

            return Ok(buffer);
        }

        Err(physis::Error::FileNotFound {
            path: path.to_string(),
        })
    }

    fn exists(&mut self, path: &str) -> bool {
        self.sqpack_resource.exists(path)
    }
}

#[cfg(test)]
mod area_tests {
    use super::{cone_angle_from_omen, scale_by_item_level, sync_cap};

    #[test]
    fn cone_angles() {
        assert_eq!(cone_angle_from_omen("gl_fan090_1bf"), Some(90.0));
        assert_eq!(cone_angle_from_omen("er_gl_fan120_1bf"), Some(120.0));
        assert_eq!(cone_angle_from_omen("general_1bf"), None);
    }

    /// The level sync caps with the sheets' numbers: a body piece synced to item level 130 (level 50's ParamGrow), Vitality
    /// (ItemLevel 414, ChestPercent 135, MeldParam 90 for a healer's item) and Mind (385, 135, 100); defense 1000 on an
    /// item level 530 piece (ItemLevel Defense 3467 there, 796 at 130.)
    #[test]
    fn level_sync_caps() {
        assert_eq!(sync_cap(414, 135.0, 90.0), 50);
        assert_eq!(sync_cap(385, 135.0, 100.0), 52);
        assert_eq!(scale_by_item_level(1000, 796, 3467), 230);
        assert_eq!(scale_by_item_level(1000, 796, 0), 1000);
    }
}

/// The forecast's roll (0-99) for the weather window this Unix time is in: windows are 8 Eorzean hours (1400 s.) In
/// integers with wrapping shifts, as the client computes it: the float version Kawari had was only exact to 128 s at
/// today's times, and chose `<=` where retail uses `<`, so it disagreed with retail 8.6 % of the time (Bob's Chronofoil
/// captures: this matched 261 of 262 zone-ins and 423 of 425 changes, the rest scripted duty weather.)
pub fn weather_target(unix_seconds: u32) -> i32 {
    let eorzean_hours = unix_seconds / 175;
    let eorzean_days = unix_seconds / 4200;

    let time_chunk = (eorzean_hours % 24) - (eorzean_hours % 8);
    let time_chunk = (time_chunk + 8) % 24;
    let calc_base = eorzean_days.wrapping_mul(100).wrapping_add(time_chunk);

    let step1 = calc_base.wrapping_shl(11) ^ calc_base;
    let step2 = (step1 >> 8) ^ step1;

    (step2 % 100) as i32
}

#[cfg(test)]
mod weather_tests {
    use super::weather_target;

    #[test]
    fn the_target_changes_only_at_window_boundaries() {
        let start = 1_760_000_000 / 1400 * 1400;
        assert_eq!(weather_target(start), weather_target(start + 1399));
        assert!((0..100).contains(&weather_target(start)));
        // Exact integers: neighbouring windows give their own rolls.
        let rolls: Vec<i32> = (0..20).map(|i| weather_target(start + i * 1400)).collect();
        assert!(rolls.windows(2).any(|pair| pair[0] != pair[1]));
    }
}

/// One leg of a teleport's cost between two TerritoryTypeTelepo points (x, y, expansion, relay): the map distance (x
/// wrapping around a region `width` wide), scaled up for later expansions, / 5 + 100.
fn teleport_leg(a: (i64, i64, i64, u8), b: (i64, i64, i64, u8), width: i64) -> u32 {
    let mut dx = (a.0 - b.0).abs();
    if width > 0 && dx > width && dx < 2 * width {
        dx = 2 * width - dx;
    }
    let dy = a.1 - b.1;
    let distance = ((dx * dx + dy * dy) as f64).sqrt().floor() as i64;
    let expansions = a.2 + b.2;
    let multiplier = if expansions < 400 {
        1000
    } else {
        expansions + 600
    };
    ((distance * multiplier / 1000 + 500) / 5) as u32
}

#[cfg(test)]
mod teleport_tests {
    use super::teleport_leg;

    #[test]
    fn legs() {
        // Same point: the 100 gil minimum.
        assert_eq!(teleport_leg((10, 10, 0, 0), (10, 10, 0, 0), 0), 100);
        // 300 apart: 300 / 5 + 100.
        assert_eq!(teleport_leg((0, 0, 0, 0), (300, 0, 0, 0), 0), 160);
        // Wrapping around a 1000-wide region: 1700 apart is 300.
        assert_eq!(teleport_leg((0, 0, 0, 0), (1700, 0, 0, 0), 1000), 160);
        // Later expansions cost more per distance.
        assert_eq!(teleport_leg((0, 0, 300, 0), (300, 0, 300, 0), 0), 172);
    }
}
