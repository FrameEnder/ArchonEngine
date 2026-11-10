-- Archer and Bard: their rotation, 1-100.
-- Priorities from Boss Mod's standard Bard rotation (awgil/ffxiv_bossmod, BSD-3) and The Balance's Bard guide (7.x):
-- both damage over time effects kept up (Venomous Bite / Caustic Bite, Windbite / Stormbite, refreshed together by
-- Iron Jaws), Straight Shot / Refulgent Arrow on Hawk's Eye, Burst Shot / Heavy Shot otherwise, Quick Nock / Ladonsbite
-- on three or more; the songs in turn (the Wanderer's Minuet, Mage's Ballad, Army's Paeon), Raging Strikes, Battle
-- Voice and Barrage on cooldown, Bloodletter / Rain of Death, Empyreal Arrow and Sidewinder as they come up; Pitch Perfect
-- at three Repertoire in the Minuet (or before it ends), Apex Arrow at 80 Soul Voice (100 outside Raging Strikes) and
-- Blast Arrow after it, Resonant Arrow and Radiant Encore after Barrage and Radiant Finale (the gauge's "repertoire",
-- "soul_voice", "coda"). Retail's
-- Archers at 10-19 (two players, 857 s) used Heavy Shot 10.0 a minute, Bloodletter 4.7, Venomous Bite 4.7, Quick Nock
-- 3.8, Straight Shot 1.6; its Bards at 30-39 Quick Nock 8.9, Heavy Shot 7.5, Bloodletter 5.4, Venomous Bite 1.5,
-- Windbite 1.2, Straight Shot 1.0, Raging Strikes 0.6; at 80-89 (one player, area pulls) Quick Nock 5.6, Burst Shot
-- 4.8, Pitch Perfect 2.6, Bloodletter 2.5, Rain of Death 2.4, Shadowbite 2.1, Empyreal Arrow 1.8.

local HEAVY_SHOT, BURST_SHOT, STRAIGHT_SHOT, REFULGENT_ARROW = 97, 16495, 98, 7409
local VENOMOUS_BITE, CAUSTIC_BITE, WINDBITE, STORMBITE, IRON_JAWS = 100, 7406, 113, 7407, 3560
local QUICK_NOCK, LADONSBITE, WIDE_VOLLEY, SHADOWBITE = 106, 25783, 36974, 16494
local APEX_ARROW, BLAST_ARROW, RESONANT_ARROW, RADIANT_ENCORE = 16496, 25784, 36976, 36977
local RAGING_STRIKES, BATTLE_VOICE, BARRAGE, RADIANT_FINALE = 101, 118, 107, 25785
local BLOODLETTER, HEARTBREAK_SHOT, RAIN_OF_DEATH, EMPYREAL_ARROW, SIDEWINDER, PITCH_PERFECT = 110, 36975, 117, 3558, 3562, 7404
local WANDERERS_MINUET, MAGES_BALLAD, ARMYS_PAEON = 3559, 114, 116

local S_HAWKS_EYE, S_RAGING_STRIKES, S_BARRAGE = 3861, 125, 128
local S_VENOMOUS_BITE, S_CAUSTIC_BITE, S_WINDBITE, S_STORMBITE = 124, 1200, 129, 1201
local S_BLAST_ARROW_READY, S_RESONANT_ARROW_READY, S_RADIANT_ENCORE_READY = 2692, 3862, 3863
-- The songs, on the Bard (Status rows; the server gives each to the party.)
local S_WANDERERS_MINUET, S_MAGES_BALLAD, S_ARMYS_PAEON = 2216, 2217, 2218

local function poison(b) return math.max(b:dot(S_VENOMOUS_BITE), b:dot(S_CAUSTIC_BITE)) end
local function wind(b) return math.max(b:dot(S_WINDBITE), b:dot(S_STORMBITE)) end

local function gcd(b)
    if b:has_status(S_BLAST_ARROW_READY) and b:has(BLAST_ARROW) then return BLAST_ARROW end
    if b:has_status(S_RESONANT_ARROW_READY) and b:has(RESONANT_ARROW) then return RESONANT_ARROW end
    if b:has_status(S_RADIANT_ENCORE_READY) and b:has(RADIANT_ENCORE) then return RADIANT_ENCORE end
    -- Both bites up: Iron Jaws refreshes them together once either runs low.
    if b:has(IRON_JAWS) and poison(b) > 0 and wind(b) > 0 and math.min(poison(b), wind(b)) < 6 then
        return IRON_JAWS
    end
    if b.target_hp > 10 and not b:aoe(3) then
        if b:has(VENOMOUS_BITE) and poison(b) < 3 then return b:best(CAUSTIC_BITE, VENOMOUS_BITE) end
        if b:has(WINDBITE) and wind(b) < 3 then return b:best(STORMBITE, WINDBITE) end
    end
    -- Apex Arrow with the Soul Voice built (at 80 under Raging Strikes, else full.)
    local voice = b:gauge_of("soul_voice")
    if b:has(APEX_ARROW) and (voice >= 100 or (voice >= 80 and b:has_status(S_RAGING_STRIKES))) then return APEX_ARROW end
    if b:has_status(S_HAWKS_EYE) or b:has_status(S_BARRAGE) then
        if b:aoe(3) then
            local area = b:best(SHADOWBITE)
            if area then return area end
        end
        local straight = b:best(REFULGENT_ARROW, STRAIGHT_SHOT)
        if straight then return straight end
    end
    if b:aoe(3) then
        local area = b:best(LADONSBITE, QUICK_NOCK)
        if area then return area end
    end
    if b:aoe(2) and b:has(WIDE_VOLLEY) and b:has_status(S_HAWKS_EYE) then return WIDE_VOLLEY end
    return b:best(BURST_SHOT, HEAVY_SHOT)
end

local function song(b)
    if b:has_status(S_WANDERERS_MINUET) or b:has_status(S_MAGES_BALLAD) or b:has_status(S_ARMYS_PAEON) then
        return nil
    end
    for _, s in ipairs({ WANDERERS_MINUET, MAGES_BALLAD, ARMYS_PAEON }) do
        if b:ready(s) then return s end
    end
    return nil
end

local function ogcd(b)
    local s = song(b)
    if s then return s end
    -- Pitch Perfect at three Repertoire, or with what there is as the Minuet ends.
    if b:has(PITCH_PERFECT) and b:has_status(S_WANDERERS_MINUET) and b:ready(PITCH_PERFECT) then
        local repertoire = b:gauge_of("repertoire")
        if repertoire >= 3 or (repertoire > 0 and b:status(S_WANDERERS_MINUET) < 3) then return PITCH_PERFECT end
    end
    if b:ready(RAGING_STRIKES) then return RAGING_STRIKES, "self" end
    if b:ready(BATTLE_VOICE) and b:has_status(S_RAGING_STRIKES) then return BATTLE_VOICE, "self" end
    if b:ready(RADIANT_FINALE) and b:has_status(S_RAGING_STRIKES) then return RADIANT_FINALE, "self" end
    if b:ready(BARRAGE) and b:has_status(S_RAGING_STRIKES) and not b:has_status(S_HAWKS_EYE) then
        return BARRAGE, "self"
    end
    if b:ready(EMPYREAL_ARROW) then return EMPYREAL_ARROW end
    if b:ready(SIDEWINDER) then return SIDEWINDER end
    local letter = b:aoe(3) and b:has(RAIN_OF_DEATH) and RAIN_OF_DEATH or b:best(HEARTBREAK_SHOT, BLOODLETTER)
    if letter and b:ready(letter) then return letter end
    return nil
end

return {
    classjobs = { 5, 23 },
    choose = function(b)
        if b:gcd_ready() then
            local action, on = gcd(b)
            if action then return action, on end
        end
        -- Abilities between GCD actions (or with the GCD free when there's nothing to cast on it.)
        if b:can_weave() or b:gcd_ready() then return ogcd(b) end
    end,
}
