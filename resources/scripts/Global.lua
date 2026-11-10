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
