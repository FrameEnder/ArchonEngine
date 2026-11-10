-- This file should only be used for globally useful constants and functions.
-- Please put new events, actions, items, etc. in their respective 'main' Lua files.

function split(input, separator)
    if separator == nil then
        separator = '%s'
    end

    local t = {}
    for str in string.gmatch(input, '([^'..separator..']+)') do
        table.insert(t, str)
    end

    return t
end

function getTableSize(tbl)
    local count = 0

    for _, _ in pairs(tbl) do
        count = count + 1
    end

    return count
end

function printf(player, fmt_str, ...)
    -- Sender would be defined elsewhere, if at all
    if command_sender == nil then
        command_sender = ""
    end

    if ... ~= nil then
        player:send_message(command_sender..fmt_str:format(...))
    else
        player:send_message(command_sender..fmt_str)
    end
end

function has_value(tab, val)
    for index, value in ipairs(tab) do
        if value == val then
            return true
        end
    end

    return false
end

-- Constants, if two or more scripts share the same global they should be placed here
EFFECT_TRANSFIGURATION = 565
EFFECT_SILKEN_SYMMETRY = 2693
EFFECT_SILKEN_FLOW = 2694
TRAIT_DYNAMIC_DANCER = 662

-- Role actions
EFFECT_STUN = 2
EFFECT_SLEEP = 3
EFFECT_SURECAST = 160
EFFECT_SWIFTCAST = 167
EFFECT_LUCID_DREAMING = 1204

-- Conjurer/White Mage
EFFECT_AERO = 143
EFFECT_AERO_II = 144
EFFECT_MEDICA_II = 150
EFFECT_FREECURE = 155
EFFECT_PRESENCE_OF_MIND = 157
EFFECT_REGEN = 158
EFFECT_THIN_AIR = 1217
EFFECT_DIVINE_BENISON = 1218
EFFECT_CONFESSION = 1219
EFFECT_DIA = 1871
EFFECT_TEMPERANCE = 1872
EFFECT_ASYLUM = 1911
EFFECT_AQUAVEIL = 2708
EFFECT_LITURGY_OF_THE_BELL = 2709
EFFECT_SACRED_SIGHT = 3879
EFFECT_MEDICA_III = 3880
EFFECT_DIVINE_GRACE = 3881
EFFECT_DIVINE_CARESS = 3903

TRAIT_MAIM_AND_MEND = 23
TRAIT_FREECURE = 25
TRAIT_MAIM_AND_MEND_II = 26
TRAIT_ENHANCED_HEALING_MAGIC = 489
TRAIT_ENHANCED_PRESENCE_OF_MIND = 623
TRAIT_WHITE_MAGIC_MASTERY = 651
TRAIT_ENHANCED_TEMPERANCE = 626

-- Applies the Conjurer/White Mage "Maim and Mend" traits (+10% at level 20, +30% at level 40) to a potency.
function whmPotency(player, potency)
    local multiplier = 1.0
    if player:has_trait(TRAIT_MAIM_AND_MEND_II) then
        multiplier = 1.3
    elseif player:has_trait(TRAIT_MAIM_AND_MEND) then
        multiplier = 1.1
    end
    return math.floor(potency * multiplier)
end

-- Picks a potency based on whether a trait is unlocked.
function traitPotency(player, trait, base_potency, enhanced_potency)
    if player:has_trait(trait) then
        return enhanced_potency
    end
    return base_potency
end

-- Pugilist/Monk forms (status effects), changed by each step of the combo.
STATUS_OPO_OPO_FORM = 107
STATUS_RAPTOR_FORM = 108
STATUS_COEURL_FORM = 109

-- Other classes' damage traits. has_trait only checks the level, so each class names its own: {trait, multiplier},
-- highest first. Maim and Mend (Thaumaturge 29/31, Arcanist 66/69: +10% at 20, +30% at 40), Increased Action Damage
-- (Archer 18/20: +10% at 20, +20% at 40.)
TRAITS_THM_MAIM_AND_MEND = { { 31, 1.3 }, { 29, 1.1 } }
TRAITS_ACN_MAIM_AND_MEND = { { 69, 1.3 }, { 66, 1.1 } }
TRAITS_ARC_ACTION_DAMAGE = { { 20, 1.2 }, { 18, 1.1 } }

-- Applies a class's damage traits (one of the TRAITS_ tables above) to a potency, like whmPotency.
function classPotency(player, potency, traits)
    for _, entry in ipairs(traits) do
        if player:has_trait(entry[1]) then
            return math.floor(potency * entry[2])
        end
    end
    return potency
end

function physicalDamage(player, potency)
    return player.parameters:calc_physical_damage(potency)
end

function magicalDamage(player, potency)
    return player.parameters:calc_magical_damage(potency)
end

function healAmount(player, potency)
    return player.parameters:calc_heal_amount(potency)
end

-- Whether a player (not an NPC) is using the action. Some action IDs are also used by NPCs in quest battles, whose damage
-- is tuned to recordings of retail: their scripts keep the NPC numbers unless this says a player cast it.
function isPlayerCaster(player)
    return player.is_player == true
end

-- Toggles a stance (Iron Will, Defiance): on with no time limit, or off if it's on.
function toggleStance(player, effects, effect_id)
    if player:get_effect(effect_id) ~= nil then
        effects:consume_effect(effect_id)
    else
        effects:gain_effect_self(effect_id, 0, 0.0)
    end
end

-- Shorthands for Conjurer/White Mage damage and healing amounts.
function whmDamage(player, potency)
    return player.parameters:calc_magical_damage(whmPotency(player, potency))
end

function whmHeal(player, potency)
    return player.parameters:calc_heal_amount(whmPotency(player, potency))
end

-- As seen on retail
INITIAL_CUTSCENE_FLAGS = NO_DEFAULT_CAMERA | INVIS_ENPC | CONDITION_CUTSCENE | HIDE_UI | HIDE_HOTBAR | SILENT_ENTER_TERRI_ENV | SILENT_ENTER_TERRI_BGM | SILENT_ENTER_TERRI_SE | DISABLE_SKIP | DISABLE_STEALTH

-- For housing
TERRITORY_S1T2 = 129 -- Limsa Lominsa Lower Decks
TERRITORY_W1T1 = 130 -- Ul'dah - Steps of Nald
TERRITORY_F1T1 = 132 -- New Gridania
TERRITORY_S1H1 = 339 -- Mist
TERRITORY_F1H1 = 340 -- The Lavender Beds
TERRITORY_W1H1 = 341 -- The Goblet
TERRITORY_R2T1 = 418 -- Foundation
TERRITORY_E3T1 = 628 -- Kugane
TERRITORY_E1H1 = 641 -- Shirogane
TERRITORY_R1H1 = 979 -- Empyreum

-- As seen in the client
OPENING_SEQ_0 = 0 -- Hasn't seen the cutscene
OPENING_SEQ_1 = 1 -- Seen the cutscene
OPENING_SEQ_2 = 2 -- Accepted first quest from questgiver

-- Opening quests for the check below
OPENING_QUEST_GRIDANIA = 39 -- Coming to Gridania
OPENING_QUEST_LIMSA = 107 -- Coming to Limsa Lominsa
OPENING_QUEST_ULDAH = 594 -- Coming to Ul'dah

-- Determine the opening sequence of this player
function determineSequence(player, cutscene)
    if not player:has_seen_cutscene(cutscene) then
        return OPENING_SEQ_0
    end

    local quest
    if player.city_state == 1 then
        quest = OPENING_QUEST_LIMSA
    elseif player.city_state == 2 then
        quest = OPENING_QUEST_GRIDANIA
    elseif player.city_state == 3 then
        quest = OPENING_QUEST_ULDAH
    end

    if not player:has_quest(quest) then
       return OPENING_SEQ_1
    end

    return OPENING_SEQ_2
end

-- Black Mage (and Thaumaturge): Astral Fire and Umbral Ice, the job gauge's "element" (1 to 3 Astral Fire, -1 to -3
-- Umbral Ice, 0 neither; `server/job_gauge.rs`). As the official job guide gives them for 7.x: under Astral Fire, Fire
-- spells' potency is 40 % higher at one stack and 80 % at three (60 % at two), they cost twice the MP (without Umbral
-- Hearts) and MP doesn't come back over time; under Umbral Ice, Ice spells cost nothing and landing one gives back 2,500
-- MP at one stack, 10,000 at three (5,000 at two), and Fire spells cost nothing; the other element's spells are 10 %
-- weaker a stack. Changing element (or gaining one from none) grants Thunderhead. One stack before Aspect Mastery II
-- (20), two before Aspect Mastery III (35).
BLM_EFFECT_THUNDERHEAD = 3870
BLM_ICE_MP = { 2500, 5000, 10000 }

function blmElement(player)
    return player:gauge("element")
end

function blmMaxStacks(player)
    if player.level >= 35 then
        return 3
    elseif player.level >= 20 then
        return 2
    end
    return 1
end

function blmFirePotency(player, potency)
    local element = blmElement(player)
    if element > 0 then
        return math.floor(potency * (1.2 + 0.2 * element))
    elseif element < 0 then
        return math.floor(potency * (1 + 0.1 * element))
    end
    return potency
end

function blmIcePotency(player, potency)
    local element = blmElement(player)
    if element > 0 then
        return math.floor(potency * (1 - 0.1 * element))
    end
    return potency
end

-- A Fire spell's MP cost (its sheet's `cost`) under the element.
function blmFireCost(player, cost)
    local element = blmElement(player)
    if element < 0 then
        return 0
    elseif element > 0 and player:gauge("umbral_hearts") == 0 then
        return cost * 2
    end
    return cost
end

function blmIceCost(player, cost)
    if blmElement(player) < 0 then
        return 0
    end
    return cost
end

-- Moves the element to `new` (kept to the stacks it may have), granting Thunderhead on a change of element, and for
-- an Ice spell landing under Umbral Ice its MP.
function blmSetElement(effects, player, new, ice_spell)
    local old = blmElement(player)
    local max = blmMaxStacks(player)
    if new > max then new = max end
    if new < -max then new = -max end
    effects:gauge_set("element", new)
    if new ~= 0 and (old == 0 or (old > 0) ~= (new > 0)) and player:get_effect(BLM_EFFECT_THUNDERHEAD) == nil then
        effects:gain_effect_self(BLM_EFFECT_THUNDERHEAD, 0, 0.0)
    end
    if ice_spell and new < 0 then
        effects:restore_mp(BLM_ICE_MP[-new])
    end
end

-- A Fire spell's step (Fire, Fire II): one more stack of Astral Fire, or Umbral Ice gone.
function blmFireStep(effects, player)
    local element = blmElement(player)
    if element < 0 then
        blmSetElement(effects, player, 0, false)
    else
        blmSetElement(effects, player, element + 1, false)
    end
end

-- An Ice spell's step (Blizzard, Blizzard II): one more stack of Umbral Ice, or Astral Fire gone.
function blmIceStep(effects, player)
    local element = blmElement(player)
    if element > 0 then
        blmSetElement(effects, player, 0, true)
    else
        blmSetElement(effects, player, element - 1, true)
    end
end

-- Warrior: the Beast Gauge ("beast", 0-100; `server/job_gauge.rs`). The Beast Within (35) fills it: Maim 10, Storm's
-- Path 20, Storm's Eye 10, Mythril Tempest 20 from Mastering the Beast (74), Infuriate 50; Inner Beast, Steel Cyclone,
-- Fell Cleave and Decimate spend 50, nothing under Inner Release (the job guide, 7.x).
WAR_BEAST_WITHIN_LEVEL = 35
WAR_EFFECT_INNER_RELEASE = 1177

function warGainBeast(effects, player, amount)
    if player.level >= WAR_BEAST_WITHIN_LEVEL then
        effects:gauge_add("beast", amount)
    end
end

function warSpendBeast(effects, player)
    if player:get_effect(WAR_EFFECT_INNER_RELEASE) == nil then
        effects:gauge_add("beast", -50)
    end
end

-- Monk: Perfect Balance (110) lets three weaponskills ignore form; each uses up one of its stacks (the job guide, 7.x).
MNK_EFFECT_PERFECT_BALANCE = 110

function mnkUsePerfectBalance(effects, player)
    if player:get_effect(MNK_EFFECT_PERFECT_BALANCE) ~= nil then
        effects:consume_effect_stack(MNK_EFFECT_PERFECT_BALANCE)
    end
end
