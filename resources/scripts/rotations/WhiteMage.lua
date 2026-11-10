-- Conjurer and White Mage: their damage, 1-100 (their healing is the party bots' own, `party_bot.rs`, which uses their
-- GCD first).
-- Priorities from Boss Mod's standard White Mage rotation (awgil/ffxiv_bossmod, BSD-3) and The Balance's White Mage
-- guide (7.x): Aero / Aero II / Dia kept on the target, Stone (II-IV) / Glare (III) otherwise, Holy (III) on three or
-- more, Afflatus Misery when the Blood Lily is full, Glare IV under Sacred Sight; Presence of Mind and Assize on
-- cooldown. Retail's White Mages at 50-59 (five players, 2172 s) cast Stone II 11.6 a minute, Aero II 3.2 (DoT on each
-- enemy), Regen 2.1, Cure II 1.7, Presence of Mind 0.3; its Conjurers at 20-29 Stone II and Aero likewise.

local STONE, STONE_II, STONE_III, STONE_IV, GLARE, GLARE_III, GLARE_IV = 119, 127, 3568, 7431, 16533, 25859, 37009
local AERO, AERO_II, DIA = 121, 132, 16532
local HOLY, HOLY_III = 139, 25860
local AFFLATUS_MISERY = 16535
local PRESENCE_OF_MIND, ASSIZE = 136, 3571

local S_AERO, S_AERO_II, S_DIA, S_SACRED_SIGHT = 143, 144, 1871, 3879

local function gcd(b)
    if b:gauge_of("blood_lily") >= 3 and b:has(AFFLATUS_MISERY) then return AFFLATUS_MISERY end
    if b:has_status(S_SACRED_SIGHT) and b:has(GLARE_IV) then return GLARE_IV end
    local dot = b:best(DIA, AERO_II, AERO)
    local left = math.max(b:dot(S_AERO), b:dot(S_AERO_II), b:dot(S_DIA))
    -- Holy is a circle round the caster.
    if b.near_self >= 3 then
        local holy = b:best(HOLY_III, HOLY)
        if holy and b:affords(holy) then return holy end
    end
    if dot and left < 3 and b.target_hp > 10 and b:affords(dot) then return dot end
    local stone = b:best(GLARE_III, GLARE, STONE_IV, STONE_III, STONE_II, STONE)
    if stone and b:affords(stone) then return stone end
    return nil
end

local function ogcd(b)
    if b:ready(PRESENCE_OF_MIND) then return PRESENCE_OF_MIND, "self" end
    if b:ready(ASSIZE) and b.near_self >= 1 then return ASSIZE, "self" end
    return nil
end

return {
    classjobs = { 6, 24 },
    choose = function(b)
        if b:gcd_ready() then
            local action, on = gcd(b)
            if action then return action, on end
        end
        -- Abilities between GCD actions (or with the GCD free when there's nothing to cast on it.)
        if b:can_weave() or b:gcd_ready() then return ogcd(b) end
    end,
}
