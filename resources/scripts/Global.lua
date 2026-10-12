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

-- Quests

-- The sequence a quest is in when it's ready to be turned in.
SEQ_FINISH = 255

-- Scene flags used by many quest cutscenes that aren't defined in Kawari's SceneFlags.
SET_EOBJ_BASE = 0xF8400EF3

-- Quest variables are six bytes, UI8A through UI8F. Each byte is split into a high (H) and low (L) nibble,
-- and the last byte (UI8F) doubles as eight bit flags (BitFlag8). These all work on the quest running the
-- current event (EVENT_ID), and use the same names as Sapphire and the client's quest scripts.
local QUEST_VAR_BYTE = { A = 0, B = 1, C = 2, D = 3, E = 4, F = 5 }

local function questVarLocation(name)
    local byte = QUEST_VAR_BYTE[string.sub(name, 1, 1)]
    if byte == nil then
        error("Unknown quest variable " .. tostring(name))
    end
    return byte, string.sub(name, 2, 2)
end

-- Returns a variable of any quest (by its event ID, 65536 + quest ID.)
function questVarOf(player, quest_id, name)
    local byte, half = questVarLocation(name)
    local value = player:get_quest_var(quest_id, byte)
    if half == "H" then
        return value >> 4
    elseif half == "L" then
        return value & 0x0F
    end
    return value
end

function setQuestVarOf(player, quest_id, name, value)
    local byte, half = questVarLocation(name)
    local current = player:get_quest_var(quest_id, byte)
    if half == "H" then
        value = (current & 0x0F) | ((value & 0x0F) << 4)
    elseif half == "L" then
        value = (current & 0xF0) | (value & 0x0F)
    end
    player:set_quest_var(quest_id, byte, value & 0xFF)
end

-- Returns a quest variable, e.g. questVar(player, "A") for UI8A or questVar(player, "AL") for UI8AL.
function questVar(player, name)
    return questVarOf(player, EVENT_ID, name)
end

-- Sets a quest variable, e.g. setQuestVar(player, "BH", 1).
function setQuestVar(player, name, value)
    setQuestVarOf(player, EVENT_ID, name, value)
end

-- BitFlag8 flags 1-8, stored in UI8F (flag 1 is the highest bit, like Sapphire.)
function questBitFlag8(player, index)
    return (questVar(player, "F") & (1 << (8 - index))) ~= 0
end

function setQuestBitFlag8(player, index, enabled)
    local value = questVar(player, "F")
    local bit = 1 << (8 - index)
    if enabled then
        value = value | bit
    else
        value = value & ~bit
    end
    setQuestVar(player, "F", value)
end

-- Returns the current quest's sequence, or nil if it hasn't been accepted.
function questSequence(player)
    return player:get_quest_sequence(EVENT_ID)
end

function setQuestSequence(player, sequence)
    player:set_quest_sequence(EVENT_ID, sequence)
end

-- Quest scene flow
--
-- Quest scripts play scenes with questPlay() and implement onSceneFinished(scene, results, player), which
-- returns true if it started another scene. Scenes can finish through either onYield or onReturn (prompts
-- usually yield), so quest scripts point both at questOnYield/questOnReturn, which ignore any stale callback
-- for a scene we've already moved on from and end the event once there's nothing left to play.
function questPlay(player, scene, flags, params)
    QUEST_CURRENT_SCENE = scene
    player:play_scene(scene, flags or HIDE_HOTBAR, params or {})
end

local function questSceneDone(scene, results, player)
    if scene ~= QUEST_CURRENT_SCENE then
        return
    end
    QUEST_CURRENT_SCENE = nil

    local continued = false
    if onSceneFinished ~= nil then
        continued = onSceneFinished(scene, results, player)
    end

    if not continued then
        player:finish_event()
    end
end

function questOnYield(scene, id, results, player)
    questSceneDone(scene, results, player)
end

function questOnReturn(scene, results, player)
    questSceneDone(scene, results, player)
end

-- Close to Home (all class variants) asks you to attune to New Gridania's aetheryte. If that happens through
-- the aetheryte's own event instead of the quest's, mark the objective here.
CLOSE_TO_HOME_QUESTS = { 65621, 65659, 65660 }
AETHERYTE_NEW_GRIDANIA = 2

function onAetheryteAttuned(player, aetheryte_id)
    if aetheryte_id ~= AETHERYTE_NEW_GRIDANIA then
        return
    end

    for _, quest_id in ipairs(CLOSE_TO_HOME_QUESTS) do
        if player:get_quest_sequence(quest_id) == 1 then
            setQuestVarOf(player, quest_id, "AL", 1)
            if questVarOf(player, quest_id, "BH") == 1 and questVarOf(player, quest_id, "BL") == 1 then
                player:set_quest_sequence(quest_id, SEQ_FINISH)
            end
        end
    end
end
