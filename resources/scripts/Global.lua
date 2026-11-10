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

-- A Fire spell's potency under the element (and Enochian, `blmEnochian`, from 56.)
function blmFirePotency(player, potency)
    local element = blmElement(player)
    local multiplier = 1
    if element > 0 then
        multiplier = 1.2 + 0.2 * element
    elseif element < 0 then
        multiplier = 1 + 0.1 * element
    end
    return math.floor(potency * multiplier * blmEnochian(player))
end

function blmIcePotency(player, potency)
    local element = blmElement(player)
    local multiplier = 1
    if element > 0 then
        multiplier = 1 - 0.1 * element
    end
    return math.floor(potency * multiplier * blmEnochian(player))
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
    blmParadoxOnSwitch(effects, player, new)
    -- Astral Soul ends with Astral Fire.
    if new <= 0 and player:gauge("astral_soul") > 0 then
        effects:gauge_set("astral_soul", 0)
    end
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

-- Paladin (and Gladiator): the Oath Gauge ("oath", 0-100). From Oath Mastery (35) each landed auto-attack adds 5;
-- Sheltron, Holy Sheltron, Intervention and Cover spend 50 (the job guide, 7.x).
PLD_OATH_MASTERY_LEVEL = 35

function pldGainOath(effects, player)
    if (player.classjob == 19 or player.classjob == 1) and player.level >= PLD_OATH_MASTERY_LEVEL then
        effects:gauge_add("oath", 5)
    end
end

-- Levels 51-100 of the A Realm Reborn jobs (the official job guides, 7.x; potencies at level 100, as the scripts give
-- them elsewhere).

-- Paladin.
PLD_EFFECT_FIGHT_OR_FLIGHT = 76
PLD_EFFECT_GORING_BLADE_READY = 3847
PLD_EFFECT_DIVINE_MIGHT = 2673
PLD_EFFECT_ATONEMENT_READY = 1902
PLD_EFFECT_SUPPLICATION_READY = 3827
PLD_EFFECT_SEPULCHRE_READY = 3828
PLD_EFFECT_REQUIESCAT = 1368
PLD_EFFECT_CONFITEOR_READY = 3019
PLD_EFFECT_BLADE_OF_HONOR_READY = 3831

-- A Paladin spell's potency: Requiescat's (using up a stack) over Divine Might's over its own; Divine Might first when
-- both are up (it goes first).
function pldMagicPotency(effects, player, base, might, requiescat)
    if might ~= nil and player:get_effect(PLD_EFFECT_DIVINE_MIGHT) ~= nil then
        effects:consume_effect(PLD_EFFECT_DIVINE_MIGHT)
        return might
    end
    if player:get_effect(PLD_EFFECT_REQUIESCAT) ~= nil then
        effects:consume_effect_stack(PLD_EFFECT_REQUIESCAT)
        return requiescat
    end
    return base
end

-- Divine Magic Mastery II (84): Holy Spirit, Holy Circle and the Confiteor combo heal 400.
function pldSpellHeal(effects, player)
    if player.level >= 84 then
        effects:heal_self(healAmount(player, 400))
    end
end

-- Warrior.
WAR_EFFECT_SURGING_TEMPEST = 2677
WAR_EFFECT_NASCENT_CHAOS = 1897
WAR_EFFECT_BERSERK = 86
WAR_EFFECT_INNER_STRENGTH = 2663
WAR_EFFECT_PRIMAL_REND_READY = 2624
WAR_EFFECT_BURGEONING_FURY = 3833
WAR_EFFECT_WRATHFUL = 3901
WAR_EFFECT_PRIMAL_RUINATION_READY = 3834

-- Fell Cleave, Decimate and their Chaos forms under Inner Release: free, surely a critical direct hit, one of its
-- stacks gone, and from 96 a stack of Burgeoning Fury (three make Wrathful.)
function warReleaseHit(effects, player)
    local release = player:get_effect(WAR_EFFECT_INNER_RELEASE)
    if release == nil then
        return false
    end
    effects:guarantee_critical()
    effects:guarantee_direct_hit()
    effects:consume_effect_stack(WAR_EFFECT_INNER_RELEASE)
    if player.level >= 96 then
        local fury = player:get_effect(WAR_EFFECT_BURGEONING_FURY)
        local stacks = (fury and fury.param or 0) + 1
        if stacks >= 3 then
            effects:consume_effect(WAR_EFFECT_BURGEONING_FURY)
            effects:gain_effect_self(WAR_EFFECT_WRATHFUL, 0, 30.0)
        else
            effects:gain_effect_self(WAR_EFFECT_BURGEONING_FURY, stacks, 30.0)
        end
    end
    return true
end

-- Berserk (86): a Warrior's weaponskills are critical direct hits while its three stacks last, one each.
function warBerserk(effects, player)
    if player:get_effect(WAR_EFFECT_BERSERK) ~= nil then
        effects:guarantee_critical()
        effects:guarantee_direct_hit()
        effects:consume_effect_stack(WAR_EFFECT_BERSERK)
    end
end

-- Monk: the Furies, Beast Chakra and Nadi.
MNK_EFFECT_FORMLESS_FIST = 2513
MNK_EFFECT_RIDDLE_OF_FIRE = 1181
MNK_EFFECT_FIRES_RUMINATION = 3843
MNK_EFFECT_WINDS_RUMINATION = 3842
MNK_EFFECT_EARTHS_RUMINATION = 3841
MNK_EFFECT_BROTHERHOOD = 1185
MNK_EFFECT_MEDITATIVE_BROTHERHOOD = 1182
MNK_EFFECT_RIDDLE_OF_WIND = 2687
MNK_EFFECT_RIDDLE_OF_EARTH = 1179
MNK_EFFECT_SIX_SIDED_STAR = 2514

-- Under Perfect Balance from Enhanced Perfect Balance (60) each weaponskill opens a Beast Chakra of its form (1
-- Opo-opo, 2 Raptor, 3 Coeurl), three at most.
function mnkBeastChakra(effects, player, kind)
    if player.level < 60 or player:get_effect(MNK_EFFECT_PERFECT_BALANCE) == nil then
        return
    end
    local chakra = player:gauge("beast_chakra")
    for slot = 0, 2 do
        if (chakra >> (slot * 2)) & 3 == 0 then
            effects:gauge_set("beast_chakra", chakra | (kind << (slot * 2)))
            return
        end
    end
end

-- The Beast Chakra opened: how many, and how many different.
function mnkChakraKinds(player)
    local chakra = player:gauge("beast_chakra")
    local seen, count, kinds = {}, 0, 0
    for slot = 0, 2 do
        local kind = (chakra >> (slot * 2)) & 3
        if kind ~= 0 then
            count = count + 1
            if not seen[kind] then
                seen[kind] = true
                kinds = kinds + 1
            end
        end
    end
    return count, kinds
end

-- A Masterful Blitz: the Beast Chakra closed, a Nadi opened (`nadi`: 1 Lunar, 2 Solar, 3 for Phantom Rush, which closes
-- both), Formless Fist.
function mnkBlitz(effects, player, nadi)
    effects:gauge_set("beast_chakra", 0)
    if nadi == 3 then
        effects:gauge_set("nadi", 0)
    else
        effects:gauge_set("nadi", player:gauge("nadi") | nadi)
    end
    effects:gain_effect_self(MNK_EFFECT_FORMLESS_FIST, 0, 30.0)
end

-- Shifts to a form: the other forms and Formless Fist go (retail shows one form at a time.)
function mnkShift(effects, player, form)
    for _, other in ipairs({ STATUS_OPO_OPO_FORM, STATUS_RAPTOR_FORM, STATUS_COEURL_FORM, MNK_EFFECT_FORMLESS_FIST }) do
        if other ~= form and player:get_effect(other) ~= nil then
            effects:consume_effect(other)
        end
    end
    effects:gain_effect_self(form, 0, 30.0)
end

-- In Opo-opo form (or formless, or under Perfect Balance).
function mnkInOpoOpo(player)
    return player:get_effect(STATUS_OPO_OPO_FORM) ~= nil or player:get_effect(MNK_EFFECT_FORMLESS_FIST) ~= nil
        or player:get_effect(MNK_EFFECT_PERFECT_BALANCE) ~= nil
        or (player:get_effect(STATUS_RAPTOR_FORM) == nil and player:get_effect(STATUS_COEURL_FORM) == nil)
end

-- A Fury's bonus, the Fury spent.
function mnkSpendFury(effects, player, name)
    if player:gauge(name) > 0 then
        effects:gauge_add(name, -1)
        return true
    end
    return false
end

-- Dragoon: Life of the Dragon (the gauge's "lotd", tenths of a second left) is 15 % more damage.
DRG_EFFECT_POWER_SURGE = 2720
DRG_EFFECT_LIFE_SURGE = 116
DRG_EFFECT_DIVE_READY = 1243
DRG_EFFECT_NASTROND_READY = 3844
DRG_EFFECT_DRAGONS_FLIGHT = 3845
DRG_EFFECT_STARCROSS_READY = 3846
DRG_EFFECT_DRACONIAN_FIRE = 1863
DRG_EFFECT_BATTLE_LITANY = 786
DRG_EFFECT_CHAOTIC_SPRING = 2719

function drgDamage(player, potency)
    local damage = physicalDamage(player, potency)
    if player:gauge("lotd") > 0 then
        damage = math.floor(damage * 1.15)
    end
    return damage
end

-- A Dragoon weaponskill: Life Surge's sure critical (it goes with it.)
function drgWeaponskill(effects, player)
    if player:get_effect(DRG_EFFECT_LIFE_SURGE) ~= nil then
        effects:guarantee_critical()
        effects:consume_effect(DRG_EFFECT_LIFE_SURGE)
    end
end

-- Bard: Repertoire while singing (retail: 80 % a DoT tick, every 3 s; here with each weaponskill, about as often),
-- Soul Voice 5 with each from 80; the Codas.
BRD_EFFECT_MAGES_BALLAD = 2217
BRD_EFFECT_ARMYS_PAEON = 2218
BRD_EFFECT_WANDERERS_MINUET = 2216
BRD_EFFECT_HAWKS_EYE = 3861
BRD_EFFECT_BARRAGE = 128
BRD_EFFECT_BLAST_ARROW_READY = 2692
BRD_EFFECT_RESONANT_ARROW_READY = 3862
BRD_EFFECT_RADIANT_ENCORE_READY = 3863

function brdRepertoire(effects, player, chance)
    if math.random() >= (chance or 0.67) then
        return
    end
    local max = 0
    if player:get_effect(BRD_EFFECT_WANDERERS_MINUET) ~= nil then
        max = 3
    elseif player:get_effect(BRD_EFFECT_ARMYS_PAEON) ~= nil then
        max = 4
    elseif player:get_effect(BRD_EFFECT_MAGES_BALLAD) ~= nil then
        max = -1
    end
    if max == 0 then
        return
    end
    if max > 0 and player:gauge("repertoire") < max then
        effects:gauge_add("repertoire", 1)
    end
    if player.level >= 80 then
        effects:gauge_add("soul_voice", 5)
    end
end

-- A Bard weaponskill that may give Hawk's Eye (35 % from Bite Mastery II, 76; Heavy Shot's 20 %.)
function brdHawksEye(effects, chance)
    if math.random() < chance then
        effects:gain_effect_self(BRD_EFFECT_HAWKS_EYE, 0, 30.0)
    end
end

-- Ninja: Ninki and the mudras. A mudra adds its digit (1 Ten, 2 Chi, 3 Jin) to "mudras"; Ninjutsu reads the sequence.
NIN_EFFECT_MUDRA = 496
NIN_EFFECT_KASSATSU = 497
NIN_EFFECT_HIDDEN = 614
NIN_EFFECT_TEN_CHI_JIN = 1186
NIN_EFFECT_RAIJU_READY = 2690
NIN_EFFECT_SHADOW_WALKER = 3848
NIN_EFFECT_MEISUI = 2689
NIN_EFFECT_BUNSHIN = 1954
NIN_EFFECT_PHANTOM_KAMAITACHI_READY = 2723
NIN_EFFECT_HIGI = 3850
NIN_EFFECT_TENRI_JINDO_READY = 3851
NIN_EFFECT_DOKUMORI = 3849
NIN_EFFECT_TRICK_ATTACK = 3254
NIN_EFFECT_KUNAIS_BANE = 3906
NIN_EFFECT_DOTON = 501

function ninGainNinki(effects, player, amount)
    if player.level >= 62 then
        effects:gauge_add("ninki", amount)
    end
end

function ninMudra(effects, player, digit)
    local mudras = player:gauge("mudras")
    local count = 0
    while (mudras >> (count * 2)) & 3 ~= 0 and count < 3 do
        count = count + 1
    end
    if count < 3 then
        effects:gauge_set("mudras", mudras | (digit << (count * 2)))
    end
    effects:gain_effect_self(NIN_EFFECT_MUDRA, 0, 6.0)
end

-- The Ninjutsu the mudras so far make (the job guide): one mudra Fuma Shuriken; two ending in Ten Katon, in Chi Raiton,
-- in Jin Hyoton; three ending in Ten Huton, in Chi Doton, in Jin Suiton; a mudra repeated (but Fuma's) is Rabbit Medium.
-- Under Kassatsu from 76 Katon and Hyoton are Goka Mekkyaku and Hyosho Ranryu.
function ninNinjutsuOf(player)
    local mudras = player:gauge("mudras")
    local digits = {}
    for i = 0, 2 do
        local digit = (mudras >> (i * 2)) & 3
        if digit ~= 0 then
            digits[#digits + 1] = digit
        end
    end
    if #digits == 0 then
        return nil
    end
    local seen = {}
    for _, digit in ipairs(digits) do
        if seen[digit] then
            return "rabbit"
        end
        seen[digit] = true
    end
    local last = digits[#digits]
    local kassatsu = player.level >= 76 and player:get_effect(NIN_EFFECT_KASSATSU) ~= nil
    if #digits == 1 then
        return "fuma"
    elseif #digits == 2 then
        if last == 1 then return kassatsu and "goka" or "katon" end
        if last == 2 then return "raiton" end
        return kassatsu and "hyosho" or "hyoton"
    end
    if last == 1 then return "huton" end
    if last == 2 then return "doton" end
    return "suiton"
end

-- Each Ninjutsu's damage and what it leaves (the job guide, 7.x).
NIN_NINJUTSU = {
    fuma = { potency = 500 },
    katon = { potency = 350, falloff = 0 },
    raiton = { potency = 740, raiju = true },
    hyoton = { potency = 350 },
    huton = { potency = 240, falloff = 0, shadow_walker = true },
    doton = { potency = 0, doton = true },
    suiton = { potency = 580, shadow_walker = true },
    goka = { potency = 850, falloff = 0 },
    hyosho = { potency = 1300 },
    rabbit = { potency = 0 },
}

-- Casts a Ninjutsu: its damage (Kassatsu's 30 % more), what it leaves, the mudras and Kassatsu gone.
function ninCastNinjutsu(effects, player, which)
    local spell = NIN_NINJUTSU[which]
    if spell == nil then
        return effects
    end
    local kassatsu = player:get_effect(NIN_EFFECT_KASSATSU) ~= nil
    if spell.potency > 0 then
        local potency = spell.potency
        if kassatsu then
            potency = math.floor(potency * 1.3)
        end
        effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, potency))
    end
    if spell.raiju and player.level >= 90 then
        local raiju = player:get_effect(NIN_EFFECT_RAIJU_READY)
        effects:gain_effect_self(NIN_EFFECT_RAIJU_READY, math.min((raiju and raiju.param or 0) + 1, 3), 30.0)
    end
    if spell.shadow_walker then
        effects:gain_effect_self(NIN_EFFECT_SHADOW_WALKER, 0, 20.0)
    end
    if spell.doton then
        -- The patch on the ground: 80 a second to who stays in it, here as its damage over time on the target.
        effects:damage_over_time(NIN_EFFECT_DOTON, 18.0, magicalDamage(player, 80))
    end
    effects:gauge_set("mudras", 0)
    if player:get_effect(NIN_EFFECT_MUDRA) ~= nil then
        effects:consume_effect(NIN_EFFECT_MUDRA)
    end
    if kassatsu then
        effects:consume_effect(NIN_EFFECT_KASSATSU)
    end
    return effects
end

-- Summoner: the arcana (1 Ruby, 2 Topaz, 4 Emerald) and attunement (1 fire, 2 earth, 3 wind; stacks.)
SMN_EFFECT_FURTHER_RUIN = 2701
SMN_EFFECT_SEARING_LIGHT = 2703
SMN_EFFECT_TITANS_FAVOR = 2853
SMN_EFFECT_IFRITS_FAVOR = 2724
SMN_EFFECT_GARUDAS_FAVOR = 2725
SMN_EFFECT_CRIMSON_STRIKE_READY = 4403
SMN_EFFECT_RUBYS_GLIMMER = 3873
SMN_EFFECT_REFULGENT_LUX = 3874
SMN_EFFECT_DREADWYRM_TRANCE = 3228
SMN_EFFECT_FIREBIRD_TRANCE = 3229
SMN_EFFECT_REKINDLE = 2704
-- Lightwyrm Trance has no status of its own in Boss Mod's list; the Dreadwyrm one stands in for it.
SMN_ARCANA_ALL = 7

-- Summons an egi (or carbuncle) of `arcanum` (1, 2 or 4): its attack, the attunement it gives, the arcanum gone.
function smnSummon(effects, player, arcanum, kind, stacks, potency)
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, potency, TRAITS_ACN_MAIM_AND_MEND)))
    effects:gauge_set("arcana", player:gauge("arcana") & (7 - arcanum))
    effects:gauge_set("attunement_type", kind)
    effects:gauge_set("attunement", stacks)
end

-- Gemshine and Precious Brilliance: the attuned element's spell, a stack of attunement spent.
function smnAttuned(effects, player, potencies)
    local kind = player:gauge("attunement_type")
    local potency = potencies[kind]
    if potency == nil or player:gauge("attunement") <= 0 then
        return false
    end
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, potency, TRAITS_ACN_MAIM_AND_MEND)))
    effects:gauge_add("attunement", -1)
    if player:gauge("attunement") <= 1 then
        effects:gauge_set("attunement_type", 0)
    end
    return true
end

-- A gem spell (Ruby, Topaz or Emerald Ruin, Rite, Outburst, Disaster, Catastrophe: what the client sends for Gemshine
-- and Precious Brilliance, retail's Summoners in Bob's captures): its damage while attuned to its element (`kind`: 1 fire,
-- 2 earth, 3 wind), a stack of attunement spent.
function smnGem(effects, player, kind, potency)
    if player:gauge("attunement_type") ~= kind or player:gauge("attunement") <= 0 then
        return false
    end
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, potency, TRAITS_ACN_MAIM_AND_MEND)))
    effects:gauge_add("attunement", -1)
    if player:gauge("attunement") <= 1 then
        effects:gauge_set("attunement_type", 0)
    end
    return true
end

-- A demi-summon's own attacks (Wyrmwave, Scarlet Flame, Luxwave: one with each of the Summoner's actions while it's
-- out) are dealt with the Summoner's spells while the trance lasts: there are no pets on the server yet.
function smnDemiHit(effects, player)
    if player:get_effect(SMN_EFFECT_DREADWYRM_TRANCE) ~= nil then
        effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, 150, TRAITS_ACN_MAIM_AND_MEND)))
    elseif player:get_effect(SMN_EFFECT_FIREBIRD_TRANCE) ~= nil then
        effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, 150, TRAITS_ACN_MAIM_AND_MEND)))
    end
end

-- Scholar.
SCH_EFFECT_BIOLYSIS = 1895
SCH_EFFECT_GALVANIZE = 297
SCH_EFFECT_CATALYZE = 1918
SCH_EFFECT_RECITATION = 1896
SCH_EFFECT_IMPACT_IMMINENT = 3882
SCH_EFFECT_BANEFUL_IMPACTION = 3883
SCH_EFFECT_CHAIN_STRATAGEM = 1221
SCH_EFFECT_SERAPHISM = 3884
SCH_EFFECT_SERAPHISM_REGEN = 3885
SCH_EFFECT_PROTRACTION = 2710
SCH_EFFECT_DESPERATE_MEASURES = 2711
SCH_EFFECT_DISSIPATION = 791
SCH_EFFECT_EXCOGITATION = 1220
SCH_EFFECT_EMERGENCY_TACTICS = 792
SCH_EFFECT_FEY_UNION = 1223

-- A Scholar's Aetherflow action: one stack (none under Recitation for those it covers) and 10 for the Faerie Gauge.
function schSpendAetherflow(effects, player, recitable)
    if recitable and player:get_effect(SCH_EFFECT_RECITATION) ~= nil then
        effects:consume_effect(SCH_EFFECT_RECITATION)
    else
        effects:spend_aetherflow()
    end
    if player.level >= 70 then
        effects:gauge_add("fairy", 10)
    end
end

-- Black Mage from 51: Enochian (56) is more damage while Astral Fire or Umbral Ice lasts: 5 %, 10 % from 70, 15 % from
-- 78, 22 % from 86, 27 % from 96.
function blmEnochian(player)
    if player.level < 56 or blmElement(player) == 0 then
        return 1.0
    end
    if player.level >= 96 then return 1.27 end
    if player.level >= 86 then return 1.22 end
    if player.level >= 78 then return 1.15 end
    if player.level >= 70 then return 1.10 end
    return 1.05
end

function blmDamage(player, potency)
    return math.floor(magicalDamage(player, classPotency(player, potency, TRAITS_THM_MAIM_AND_MEND)) * blmEnochian(player))
end

-- Fire spells in Astral Fire use up an Umbral Heart (it spares them the doubled cost.)
function blmUseHeart(effects, player)
    if blmElement(player) > 0 and player:gauge("umbral_hearts") > 0 then
        effects:gauge_add("umbral_hearts", -1)
    end
end

-- Aspect Mastery V (90): reaching Astral Fire III or Umbral Ice III (with three Umbral Hearts) and then changing
-- element readies Paradox.
function blmParadoxOnSwitch(effects, player, new)
    if player.level < 90 then
        return
    end
    local old = blmElement(player)
    if (old == 3 and new < 0) or (old == -3 and player:gauge("umbral_hearts") >= 3 and new > 0) then
        effects:gauge_set("paradox", 1)
    end
end

-- Starts a Bard song: the others end, Repertoire starts again, and from 90 the song's Coda (1 Mage's, 2 Army's, 4
-- Wanderer's.)
function brdStartSong(effects, player, song, coda)
    for _, other in ipairs({ BRD_EFFECT_MAGES_BALLAD, BRD_EFFECT_ARMYS_PAEON, BRD_EFFECT_WANDERERS_MINUET }) do
        if other ~= song and player:get_effect(other) ~= nil then
            effects:consume_effect(other)
        end
    end
    effects:gauge_set("repertoire", 0)
    effects:gain_effect(song, 0, 45.0)
    if player.level >= 90 then
        effects:gauge_set("coda", player:gauge("coda") | coda)
    end
end

-- Hawk's Eye (or Barrage) used up.
function brdUseHawksEye(effects, player)
    if player:get_effect(BRD_EFFECT_HAWKS_EYE) ~= nil then
        effects:consume_effect(BRD_EFFECT_HAWKS_EYE)
    elseif player:get_effect(BRD_EFFECT_BARRAGE) ~= nil then
        effects:consume_effect(BRD_EFFECT_BARRAGE)
    end
end

-- Trick Attack and Kunai's Bane need Hidden or Shadow Walker (7.x), which they end.
function ninLeaveHiding(effects, player)
    if player:get_effect(NIN_EFFECT_HIDDEN) ~= nil then
        effects:consume_effect(NIN_EFFECT_HIDDEN)
    elseif player:get_effect(NIN_EFFECT_SHADOW_WALKER) ~= nil then
        effects:consume_effect(NIN_EFFECT_SHADOW_WALKER)
    end
end

-- A Ninja weaponskill under Bunshin: a shadow's hit too (160, 80 for area ones), 5 Ninki, a stack spent.
function ninBunshin(effects, player, area)
    if player:get_effect(NIN_EFFECT_BUNSHIN) ~= nil then
        effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, area and 80 or 160))
        effects:consume_effect_stack(NIN_EFFECT_BUNSHIN)
        ninGainNinki(effects, player, 5)
    end
end

-- The jobs from Heavensward on (stage 3, 2026-11-10). Their damage traits go by level (the job guide, 7.x): Maim and
-- Mend for the casters and healers (+10 % at 20, +30 % at 40), Increased Action Damage for the Machinist (20, 40) and
-- the Dancer (50, 60).
LEVEL_MAIM_AND_MEND = { { 40, 1.3 }, { 20, 1.1 } }
LEVEL_MCH_ACTION_DAMAGE = { { 40, 1.2 }, { 20, 1.1 } }
LEVEL_DNC_ACTION_DAMAGE = { { 60, 1.2 }, { 50, 1.1 } }

-- A potency the job's traits raise: `tiers` is { { level, potency }, ... } from the highest level down (the job guide's
-- trait text, 7.x); below the lowest level listed the lowest potency listed stands (the guide gives none under it.)
function tierPotency(player, tiers)
    for _, tier in ipairs(tiers) do
        if player.level >= tier[1] then
            return tier[2]
        end
    end
    return tiers[#tiers][2]
end

function levelPotency(player, potency, steps)
    for _, step in ipairs(steps) do
        if player.level >= step[1] then
            return math.floor(potency * step[2])
        end
    end
    return potency
end

-- Dark Knight. Darkside (the gauge's "darkside", tenths of a second, up to 60 s) is 10 % more damage.
DRK_EFFECT_BLOOD_WEAPON = 742
DRK_EFFECT_GRIT = 743
DRK_EFFECT_SHADOW_WALL = 747
DRK_EFFECT_DARK_MIND = 746
DRK_EFFECT_LIVING_DEAD = 810
DRK_EFFECT_SALTED_EARTH = 749
DRK_EFFECT_DELIRIUM = 1972
DRK_EFFECT_ENHANCED_DELIRIUM = 3836
DRK_EFFECT_BLACKEST_NIGHT = 1178
DRK_EFFECT_DARK_MISSIONARY = 1894
DRK_EFFECT_OBLATION = 2682
DRK_EFFECT_SCORN = 3837

function drkDamage(effects, player, potency, kind)
    if player:gauge("darkside") > 0 then
        potency = math.floor(potency * 1.1)
    end
    effects:damage(kind or DAMAGE_TYPE_SLASHING, physicalDamage(player, potency))
end

-- A weaponskill or spell under Blood Weapon (35; Delirium's from 68): a stack spent for 600 MP, and 10 Blood from 62
-- (Blackblood). The simulacrum's attacks come with them while it's out (`drkShadowHit`.)
function drkWeaponskill(effects, player)
    if player:get_effect(DRK_EFFECT_BLOOD_WEAPON) ~= nil then
        effects:consume_effect_stack(DRK_EFFECT_BLOOD_WEAPON)
        effects:restore_mp(600)
        if player.level >= 62 then
            effects:gauge_add("blood", 10)
        end
    end
    drkShadowHit(effects, player)
end

-- Living Shadow's simulacrum (80) attacks six times in its 22 s (Abyssal Drain, Plunge, Shadowbringer from 90, Edge of
-- Shadow, Bloodspiller, Carve and Spit, Disesteem from 100: 420 each, the job guide's numbers for its stronger ones
-- aside); with no pets on the server they land with the Dark Knight's next six weaponskills ("shadow_hits").
function drkShadowHit(effects, player)
    local left = player:gauge("shadow_hits")
    if left <= 0 then
        return
    end
    local potency = 420
    if left == 4 and player.level >= 90 then
        potency = 570
    elseif left == 1 and player.level >= 100 then
        potency = 620
    end
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, potency))
    effects:gauge_add("shadow_hits", -1)
end

-- Edge and Flood of Darkness / Shadow: Darkside 30 s more (up to 60), 3,000 MP or Dark Arts.
function drkDarkside(effects, player)
    effects:gauge_add("darkside", 300)
    if player:gauge("dark_arts") > 0 then
        effects:gauge_set("dark_arts", 0)
        effects:set_mp_cost(0)
    end
end

-- Gunbreaker: Cartridges up to two, three from 88 (Cartridge Charge II), six under Bloodfest.
GNB_EFFECT_NO_MERCY = 1831
GNB_EFFECT_READY_TO_BREAK = 3886
GNB_EFFECT_BRUTAL_SHELL = 1898
GNB_EFFECT_CAMOUFLAGE = 1832
GNB_EFFECT_ROYAL_GUARD = 1833
GNB_EFFECT_NEBULA = 1834
GNB_EFFECT_GREAT_NEBULA = 3838
GNB_EFFECT_AURORA = 1835
GNB_EFFECT_SUPERBOLIDE = 1836
GNB_EFFECT_SONIC_BREAK = 1837
GNB_EFFECT_BOW_SHOCK = 1838
GNB_EFFECT_HEART_OF_LIGHT = 1839
GNB_EFFECT_HEART_OF_STONE = 1840
GNB_EFFECT_HEART_OF_CORUNDUM = 2683
GNB_EFFECT_CLARITY_OF_CORUNDUM = 2684
GNB_EFFECT_CATHARSIS_OF_CORUNDUM = 2685
GNB_EFFECT_READY_TO_RIP = 1842
GNB_EFFECT_READY_TO_TEAR = 1843
GNB_EFFECT_READY_TO_GOUGE = 1844
GNB_EFFECT_READY_TO_BLAST = 2686
GNB_EFFECT_READY_TO_RAZE = 3839
GNB_EFFECT_READY_TO_REIGN = 3840
GNB_EFFECT_BLOODFEST = 5051

function gnbMaxCartridges(player)
    if player:get_effect(GNB_EFFECT_BLOODFEST) ~= nil then
        return 6
    end
    return player.level >= 88 and 3 or 2
end

function gnbAddCartridges(effects, player, count)
    local now = player:gauge("cartridges")
    effects:gauge_set("cartridges", math.min(now + count, gnbMaxCartridges(player)))
end

function gnbDamage(effects, player, potency)
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, potency))
end

-- A Continuation (Jugular Rip, Abdomen Tear, Eye Gouge, Hypervelocity, Fated Brand): its Ready status spent.
function gnbContinuation(effects, player, ready, potency, falloff)
    if falloff then
        effects:aoe_falloff(falloff)
    end
    gnbDamage(effects, player, potency)
    effects:consume_effect(ready)
end

-- Machinist. Overheated (Hypercharge's five stacks) is 20 more potency for its single-target weaponskills.
MCH_EFFECT_REASSEMBLED = 851
MCH_EFFECT_OVERHEATED = 2688
MCH_EFFECT_WILDFIRE = 1946
MCH_EFFECT_WILDFIRE_TARGET = 861
MCH_EFFECT_DISMANTLED = 860
MCH_EFFECT_HYPERCHARGED = 3864
MCH_EFFECT_FLAMETHROWER = 1205
MCH_EFFECT_EXCAVATOR_READY = 3865
MCH_EFFECT_FULL_METAL_MACHINIST = 3866
MCH_EFFECT_TACTICIAN = 1951
MCH_EFFECT_BIOBLASTER = 1866

-- A Machinist weaponskill: its damage (Increased Action Damage by level, Overheated's 20 for single-target ones),
-- Reassemble's sure critical direct hit, Wildfire's 240 for each (up to six; here as they land, since nothing marks
-- its end), and the turret's or queen's attacks while one is out (`petHit`.)
function mchShot(effects, player, potency, area)
    if not area and player:get_effect(MCH_EFFECT_OVERHEATED) ~= nil then
        potency = potency + 20
    end
    if player:get_effect(MCH_EFFECT_REASSEMBLED) ~= nil then
        effects:guarantee_critical()
        effects:guarantee_direct_hit()
        effects:consume_effect(MCH_EFFECT_REASSEMBLED)
    end
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, levelPotency(player, potency, LEVEL_MCH_ACTION_DAMAGE)))
    local wildfire = player:get_effect(MCH_EFFECT_WILDFIRE)
    if wildfire ~= nil and wildfire.param > 0 then
        effects:damage(DAMAGE_TYPE_MAGIC, physicalDamage(player, 240))
        effects:consume_effect_stack(MCH_EFFECT_WILDFIRE)
    end
    petHit(effects, player)
end

function mchHeat(effects, player, amount, battery)
    effects:gauge_add("heat", amount)
    if battery then
        effects:gauge_add("battery", battery)
    end
end

-- A pet's attacks (no pets on the server): `count` hits of `potency` with the next weaponskills.
function petSummon(effects, count, potency)
    effects:gauge_set("pet_hits", count)
    effects:gauge_set("pet_potency", potency)
end

function petHit(effects, player)
    if player:gauge("pet_hits") > 0 then
        effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, player:gauge("pet_potency")))
        effects:gauge_add("pet_hits", -1)
    end
end

-- Samurai. Kenki from 52 (Kenki Mastery; every weaponskill's from 62.)
SAM_EFFECT_FUGETSU = 1298
SAM_EFFECT_FUKA = 1299
SAM_EFFECT_HIGANBANA = 1228
SAM_EFFECT_MEIKYO_SHISUI = 1233
SAM_EFFECT_THIRD_EYE = 1232
SAM_EFFECT_ENHANCED_ENPI = 1236
SAM_EFFECT_OGI_NAMIKIRI_READY = 2959
SAM_EFFECT_TSUBAME_READY = 4216
SAM_EFFECT_TENGENTSU = 3853
SAM_EFFECT_ZANSHIN_READY = 3855
SAM_EFFECT_TENDO = 3856

function samDamage(effects, player, potency)
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, potency))
end

function samKenki(effects, player, amount)
    if player.level >= 52 then
        effects:gauge_add("kenki", amount)
    end
end

-- A combo step that finishes or builds (Jinpu, Shifu, Gekko, Kasha, Yukikaze, Mangetsu, Oka): its combo potency, Sen,
-- Kenki and Fugetsu / Fuka in combo or under Meikyo Shisui (a stack spent; then Gekko gives Fugetsu and Kasha Fuka.)
function samStep(effects, player, in_combo, base, combo, sen, kenki, buff)
    local meikyo = player:get_effect(SAM_EFFECT_MEIKYO_SHISUI) ~= nil and sen ~= 0
    if not (in_combo or meikyo) then
        samDamage(effects, player, base)
        return
    end
    samDamage(effects, player, combo)
    if sen ~= 0 then
        effects:gauge_set("sen", player:gauge("sen") | sen)
    end
    samKenki(effects, player, kenki)
    if buff ~= nil then
        effects:gain_effect_self(buff, 0, 40.0)
    end
    if meikyo then
        effects:consume_effect_stack(SAM_EFFECT_MEIKYO_SHISUI)
    end
end

-- An Iaijutsu: the Sen spent, a Meditation stack, and from 74 Tsubame-gaeshi ready to repeat it (`kaeshi`: 2 Goken, 3
-- Setsugekka.)
function samIaijutsu(effects, player, kaeshi)
    effects:gauge_set("sen", 0)
    effects:gauge_add("meditation", 1)
    if kaeshi ~= nil and player.level >= 74 then
        effects:gain_effect_self(SAM_EFFECT_TSUBAME_READY, 0, 30.0)
        effects:gauge_set("kaeshi", kaeshi)
    end
end

-- Dancer. The dance's steps are the gauge's ("dance_steps", three bits each; "step_index" how many are done.)
DNC_EFFECT_SILKEN_SYMMETRY = 2693
DNC_EFFECT_SILKEN_FLOW = 2694
DNC_EFFECT_FLOURISHING_SYMMETRY = 3017
DNC_EFFECT_FLOURISHING_FLOW = 3018
DNC_EFFECT_THREEFOLD_FAN_DANCE = 1820
DNC_EFFECT_FOURFOLD_FAN_DANCE = 2699
DNC_EFFECT_STANDARD_STEP = 1818
DNC_EFFECT_TECHNICAL_STEP = 1819
DNC_EFFECT_STANDARD_FINISH = 1821
DNC_EFFECT_TECHNICAL_FINISH = 1822
DNC_EFFECT_STANDARD_ESPRIT = 1847
DNC_EFFECT_TECHNICAL_ESPRIT = 1848
DNC_EFFECT_FLOURISHING_FINISH = 2698
DNC_EFFECT_FLOURISHING_STARFALL = 2700
DNC_EFFECT_DEVILMENT = 1825
DNC_EFFECT_SHIELD_SAMBA = 1826
DNC_EFFECT_IMPROVISATION = 1827
DNC_EFFECT_CLOSED_POSITION = 1823
DNC_EFFECT_DANCE_PARTNER = 1824
DNC_EFFECT_LAST_DANCE_READY = 3867
DNC_EFFECT_FINISHING_MOVE_READY = 3868
DNC_EFFECT_DANCE_OF_THE_DAWN_READY = 3869

function dncDamage(effects, player, potency)
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, levelPotency(player, potency, LEVEL_DNC_ACTION_DAMAGE)))
end

-- Esprit from the Dancer's own weaponskills (76: 5; the procs' 10 from 84.)
function dncEsprit(effects, player, proc)
    if player.level >= 76 then
        effects:gauge_add("esprit", (proc and player.level >= 84) and 10 or 5)
    end
end

function dncChance(effects, effect_id, chance)
    if math.random() < chance then
        effects:gain_effect_self(effect_id, 0, 30.0)
    end
end

-- A proc spent (Reverse Cascade, Fountainfall, Rising Windmill, Bloodshower): the Silken one, or the Flourishing one;
-- a Fourfold Feather half the time from 30.
function dncProc(effects, player, silken, flourishing)
    if player:get_effect(silken) ~= nil then
        effects:consume_effect(silken)
    elseif player:get_effect(flourishing) ~= nil then
        effects:consume_effect(flourishing)
    end
    if player.level >= 30 and math.random() < 0.5 then
        effects:gauge_add("feathers", 1)
    end
    dncEsprit(effects, player, true)
end

-- Starts a dance: `count` steps drawn at random (as the client shows them), none done yet.
function dncStartDance(effects, status, count)
    local steps = 0
    for i = 0, count - 1 do
        steps = steps | (math.random(1, 4) << (i * 3))
    end
    effects:gauge_set("dance_steps", steps)
    effects:gauge_set("step_index", 0)
    effects:gain_effect_self(status, 0, 15.0)
end

-- A step: the one due is done.
function dncStep(effects, player, step)
    local index = player:gauge("step_index")
    local due = (player:gauge("dance_steps") >> (index * 3)) & 7
    if due == step then
        effects:gauge_add("step_index", 1)
    end
end

-- A dance's finish: its damage, the dance over.
function dncFinish(effects, player, status, potency)
    effects:aoe_falloff(60)
    dncDamage(effects, player, potency)
    if player:get_effect(status) ~= nil then
        effects:consume_effect(status)
    end
    effects:gauge_set("dance_steps", 0)
    effects:gauge_set("step_index", 0)
end

-- Reaper.
RPR_EFFECT_DEATHS_DESIGN = 2586
RPR_EFFECT_SOUL_REAVER = 2587
RPR_EFFECT_ENHANCED_GIBBET = 2588
RPR_EFFECT_ENHANCED_GALLOWS = 2589
RPR_EFFECT_ENHANCED_VOID_REAPING = 2590
RPR_EFFECT_ENHANCED_CROSS_REAPING = 2591
RPR_EFFECT_IMMORTAL_SACRIFICE = 2592
RPR_EFFECT_ENSHROUDED = 2593
RPR_EFFECT_SOULSOW = 2594
RPR_EFFECT_THRESHOLD = 2595
RPR_EFFECT_CREST_OF_TIME_BORROWED = 2596
RPR_EFFECT_ARCANE_CIRCLE = 2599
RPR_EFFECT_CIRCLE_OF_SACRIFICE = 2600
RPR_EFFECT_BLOODSOWN_CIRCLE = 2972
RPR_EFFECT_ENHANCED_HARPE = 2845
RPR_EFFECT_IDEAL_HOST = 3905
RPR_EFFECT_OBLATIO = 3857
RPR_EFFECT_EXECUTIONER = 3858
RPR_EFFECT_PERFECTIO_OCCULTA = 3859
RPR_EFFECT_PERFECTIO_PARATA = 3860

function rprDamage(effects, player, potency, magic)
    effects:damage(magic and DAMAGE_TYPE_MAGIC or DAMAGE_TYPE_SLASHING, physicalDamage(player, potency))
end

-- Soul from 50 (the Soul Gauge trait.)
function rprSoul(effects, player, amount)
    if player.level >= 50 then
        effects:gauge_add("soul", amount)
    end
end

-- Gibbet, Gallows, Guillotine and their Executioner's: Soul Reaver's (or Executioner's) stack, the Enhanced one spent
-- for 60 more and the other given, 10 Shroud from 80.
function rprReaver(effects, player, base, enhanced, mine, other)
    local potency = base
    if mine ~= nil and player:get_effect(mine) ~= nil then
        potency = base + 60
        effects:consume_effect(mine)
    end
    rprDamage(effects, player, potency)
    if other ~= nil then
        effects:gain_effect_self(other, 0, 60.0)
    end
    if player:get_effect(RPR_EFFECT_EXECUTIONER) ~= nil then
        effects:consume_effect_stack(RPR_EFFECT_EXECUTIONER)
    else
        effects:consume_effect_stack(RPR_EFFECT_SOUL_REAVER)
    end
    if player.level >= 80 then
        effects:gauge_add("shroud", 10)
    end
end

-- Void, Cross and Grim Reaping: a Lemure Shroud for a Void Shroud (86.)
function rprReaping(effects, player)
    effects:gauge_add("lemure", -1)
    if player.level >= 86 then
        effects:gauge_add("void", 1)
    end
end

-- Viper.
VPR_EFFECT_HUNTERS_INSTINCT = 3668
VPR_EFFECT_SWIFTSCALED = 3669
VPR_EFFECT_HONED_STEEL = 3672
VPR_EFFECT_HONED_REAVERS = 3772
VPR_EFFECT_FLANKSTUNG_VENOM = 3645
VPR_EFFECT_FLANKSBANE_VENOM = 3646
VPR_EFFECT_HINDSTUNG_VENOM = 3647
VPR_EFFECT_HINDSBANE_VENOM = 3648
VPR_EFFECT_GRIMHUNTERS_VENOM = 3649
VPR_EFFECT_GRIMSKINS_VENOM = 3650
VPR_EFFECT_HUNTERS_VENOM = 3657
VPR_EFFECT_SWIFTSKINS_VENOM = 3658
VPR_EFFECT_FELLHUNTERS_VENOM = 3659
VPR_EFFECT_FELLSKINS_VENOM = 3660
VPR_EFFECT_POISED_FOR_TWINFANG = 3665
VPR_EFFECT_POISED_FOR_TWINBLOOD = 3666
VPR_EFFECT_REAWAKENED = 3670
VPR_EFFECT_READY_TO_REAWAKEN = 3671
VPR_VENOMS = { 3645, 3646, 3647, 3648, 3649, 3650 }

function vprDamage(effects, player, potency)
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, potency))
end

-- A status that adds to this action: spent, its bonus added.
function vprBonus(effects, player, status, bonus)
    if status ~= nil and player:get_effect(status) ~= nil then
        effects:consume_effect(status)
        return bonus
    end
    return 0
end

-- A venom given: the other venoms go (one at a time.)
function vprVenom(effects, player, status)
    for _, other in ipairs(VPR_VENOMS) do
        if other ~= status and player:get_effect(other) ~= nil then
            effects:consume_effect(other)
        end
    end
    effects:gain_effect_self(status, 0, 60.0)
end

-- Serpent Offerings from 90 (Serpent's Lineage.)
function vprOfferings(effects, player, amount)
    if player.level >= 90 then
        effects:gauge_add("offerings", amount)
    end
end

-- Red Mage. Dualcast: a spell cast with its cast time gives it (the next is instant, `INSTANT_CASTS` in action.rs);
-- one cast with it spends it.
RDM_EFFECT_VERFIRE_READY = 1234
RDM_EFFECT_VERSTONE_READY = 1235
RDM_EFFECT_ACCELERATION = 1238
RDM_EFFECT_EMBOLDEN_SELF = 1239
RDM_EFFECT_DUALCAST = 1249
RDM_EFFECT_EMBOLDEN = 1297
RDM_EFFECT_MANAFICATION = 1971
RDM_EFFECT_MAGICKED_SWORDPLAY = 3875
RDM_EFFECT_THORNED_FLOURISH = 3876
RDM_EFFECT_GRAND_IMPACT_READY = 3877
RDM_EFFECT_PREFULGENCE_READY = 3878
RDM_EFFECT_SWIFTCAST = 167

function rdmDualcast(effects, player, accelerated)
    if player:get_effect(RDM_EFFECT_DUALCAST) ~= nil then
        effects:consume_effect(RDM_EFFECT_DUALCAST)
    elseif accelerated or player:get_effect(RDM_EFFECT_SWIFTCAST) ~= nil then
        return
    else
        effects:gain_effect_self(RDM_EFFECT_DUALCAST, 0, 15.0)
    end
end

function rdmMagic(effects, player, potency)
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, levelPotency(player, potency, LEVEL_MAIM_AND_MEND)))
end

function rdmMana(effects, white, black)
    if white > 0 then
        effects:gauge_add("white_mana", white)
    end
    if black > 0 then
        effects:gauge_add("black_mana", black)
    end
end

-- An enchanted sword skill: its White and Black Mana (or a Magicked Swordplay stack), a Mana Stack from 68.
function rdmEnchanted(effects, player, cost, potency)
    if player:get_effect(RDM_EFFECT_MAGICKED_SWORDPLAY) ~= nil then
        effects:consume_effect_stack(RDM_EFFECT_MAGICKED_SWORDPLAY)
    else
        effects:gauge_add("white_mana", -cost)
        effects:gauge_add("black_mana", -cost)
    end
    effects:damage(DAMAGE_TYPE_MAGIC, physicalDamage(player, potency))
    if player.level >= 68 then
        effects:gauge_add("mana_stacks", 1)
    end
end

-- Verflare / Verholy: three Mana Stacks, the other colour's proc on 20 % (always if it's behind.)
function rdmFinisher(effects, player, white, black, proc, behind)
    effects:aoe_falloff(55)
    rdmMagic(effects, player, 650)
    rdmMana(effects, white, black)
    effects:gauge_set("mana_stacks", 0)
    if behind or math.random() < 0.2 then
        effects:gain_effect_self(proc, 0, 30.0)
    end
end

-- Pictomancer. The motifs painted are the gauge's "canvas" (1 Pom, 2 Wing, 4 Claw, 8 Maw, 16 hammer, 32 starry sky),
-- the depictions and portraits its "creature" (1 Pom, 2 Wings, 4 Claw; 16 Moogle, 32 Madeen.)
PCT_EFFECT_AETHERHUES = 3675
PCT_EFFECT_AETHERHUES_II = 3676
PCT_EFFECT_SUBTRACTIVE_PALETTE = 3674
PCT_EFFECT_MONOCHROME_TONES = 3691
PCT_EFFECT_HAMMER_TIME = 3680
PCT_EFFECT_STARRY_MUSE = 3685
PCT_EFFECT_SUBTRACTIVE_SPECTRUM = 3690
PCT_EFFECT_HYPERPHANTASIA = 3688
PCT_EFFECT_RAINBOW_BRIGHT = 3679
PCT_EFFECT_STARSTRUCK = 3681
PCT_EFFECT_TEMPERA_COAT = 3686
PCT_EFFECT_TEMPERA_GRASSA = 3687
PCT_EFFECT_SMUDGE = 3684

function pctMagic(effects, player, potency, falloff)
    if falloff ~= nil then
        effects:aoe_falloff(falloff)
    end
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, levelPotency(player, potency, LEVEL_MAIM_AND_MEND)))
end

-- An aetherhue spell (the Fire, Aero and Water ones, or with Subtractive Palette the Blizzard, Stone and Thunder ones):
-- the hue it needs spent and the next given; a Hyperphantasia stack under Starry Muse, Rainbow Bright when they're gone.
function pctHue(effects, player, needs, gives)
    if needs ~= nil and player:get_effect(needs) ~= nil then
        effects:consume_effect(needs)
    end
    if gives ~= nil then
        effects:gain_effect_self(gives, 0, 30.0)
    end
    local hyper = player:get_effect(PCT_EFFECT_HYPERPHANTASIA)
    if hyper ~= nil then
        effects:consume_effect_stack(PCT_EFFECT_HYPERPHANTASIA)
        if hyper.param <= 1 and player.level >= 92 then
            effects:gain_effect_self(PCT_EFFECT_RAINBOW_BRIGHT, 0, 30.0)
        end
    end
end

function pctWhitePaint(effects, player)
    if player.level >= 80 then
        effects:gauge_add("white_paint", 1)
    end
end

-- A creature muse: its motif off the canvas, its depiction added; Wings after Pom (and Fangs after the other three) a
-- portrait.
function pctMuse(effects, player, motif, depiction, potency)
    pctMagic(effects, player, potency or 800, 70)
    local creature = player:gauge("creature")
    effects:gauge_set("canvas", player:gauge("canvas") & (63 - motif))
    if motif == 2 and creature & 1 ~= 0 then
        creature = creature | 16
    end
    if motif == 8 then
        creature = (creature & 48) | 32
    else
        creature = creature | depiction
    end
    effects:gauge_set("creature", creature)
end

-- Astrologian. The cards drawn are the gauge's "ast_cards" (four bits each: 1 Balance, 2 Bole, 3 Arrow, 4 Spear, 5
-- Ewer, 6 Spire), the minor arcanum "ast_arcana" (7 Lord, 8 Lady), the next draw "ast_draw" (0 Astral, 1 Umbral.)
AST_EFFECT_COMBUST = 838
AST_EFFECT_COMBUST_II = 843
AST_EFFECT_COMBUST_III = 1881
AST_EFFECT_ASPECTED_BENEFIC = 835
AST_EFFECT_ASPECTED_HELIOS = 836
AST_EFFECT_HELIOS_CONJUNCTION = 3894
AST_EFFECT_LIGHTSPEED = 841
AST_EFFECT_DIVINATION = 1878
AST_EFFECT_DIVINING = 3893
AST_EFFECT_EARTHLY_DOMINANCE = 1224
AST_EFFECT_COLLECTIVE_UNCONSCIOUS = 849
AST_EFFECT_SYNASTRY = 845
AST_EFFECT_HOROSCOPE = 1890
AST_EFFECT_NEUTRAL_SECT = 1892
AST_EFFECT_SUNTOUCHED = 3895
AST_EFFECT_SUN_SIGN = 3896
AST_EFFECT_MACROCOSMOS = 2718
AST_EFFECT_THE_BALANCE = 3887
AST_EFFECT_THE_ARROW = 3888
AST_EFFECT_THE_SPEAR = 3889
AST_EFFECT_THE_BOLE = 3890
AST_EFFECT_THE_EWER = 3891
AST_EFFECT_THE_SPIRE = 3892

function astMagic(effects, player, potency, falloff)
    if falloff ~= nil then
        effects:aoe_falloff(falloff)
    end
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, levelPotency(player, potency, LEVEL_MAIM_AND_MEND)))
end

function astHeal(effects, player, potency)
    if player:get_effect(AST_EFFECT_NEUTRAL_SECT) ~= nil then
        potency = math.floor(potency * 1.2)
    end
    effects:heal(healAmount(player, levelPotency(player, potency, LEVEL_MAIM_AND_MEND)))
end

-- A card played: its slot emptied.
function astPlay(effects, player, card)
    local cards = player:gauge("ast_cards")
    for slot = 0, 2 do
        if (cards >> (slot * 4)) & 15 == card then
            effects:gauge_set("ast_cards", cards & ~(15 << (slot * 4)))
            return
        end
    end
end

-- Sage.
SGE_EFFECT_KARDIA = 2604
SGE_EFFECT_KARDION = 2605
SGE_EFFECT_EUKRASIA = 2606
SGE_EFFECT_EUKRASIAN_DIAGNOSIS = 2607
SGE_EFFECT_EUKRASIAN_PROGNOSIS = 2609
SGE_EFFECT_SOTERIA = 2610
SGE_EFFECT_ZOE = 2611
SGE_EFFECT_HAIMA = 2612
SGE_EFFECT_PANHAIMA = 2613
SGE_EFFECT_EUKRASIAN_DOSIS = 2614
SGE_EFFECT_EUKRASIAN_DOSIS_II = 2615
SGE_EFFECT_EUKRASIAN_DOSIS_III = 2616
SGE_EFFECT_PHYSIS = 2617
SGE_EFFECT_KERACHOLE = 2618
SGE_EFFECT_TAUROCHOLE = 2619
SGE_EFFECT_PHYSIS_II = 2620
SGE_EFFECT_KRASIS = 2622
SGE_EFFECT_KERAKEIA = 2938
SGE_EFFECT_HOLOS = 3003
SGE_EFFECT_HOLOSAKOS = 3365
SGE_EFFECT_EUKRASIAN_DYSKRASIA = 3897

function sgeMagic(effects, player, potency, falloff)
    if falloff ~= nil then
        effects:aoe_falloff(falloff)
    end
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, levelPotency(player, potency, LEVEL_MAIM_AND_MEND)))
end

-- A heal, Zoe's 50 % more on the next spell's (`spell`.)
function sgeHeal(effects, player, potency, spell)
    if spell and player:get_effect(SGE_EFFECT_ZOE) ~= nil then
        potency = math.floor(potency * 1.5)
        effects:consume_effect(SGE_EFFECT_ZOE)
    end
    effects:heal(healAmount(player, levelPotency(player, potency, LEVEL_MAIM_AND_MEND)))
end

-- An Addersgall action: one spent, 7 % of the Sage's MP back.
function sgeAddersgall(effects, player)
    effects:gauge_add("addersgall", -1)
    effects:restore_mp(math.floor(player.max_mp * 7 / 100))
end

-- A Eukrasian spell: Eukrasia spent.
function sgeEukrasian(effects, player)
    effects:gauge_set("eukrasia", 0)
    if player:get_effect(SGE_EFFECT_EUKRASIA) ~= nil then
        effects:consume_effect(SGE_EFFECT_EUKRASIA)
    end
end
