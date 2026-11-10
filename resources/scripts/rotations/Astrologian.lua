-- Astrologian: its damage, 1-100 (its healing is the party bots' own, `party_bot.rs`, which uses its GCD first).
-- Priorities from Boss Mod's standard Astrologian rotation (awgil/ffxiv_bossmod, BSD-3), The Balance's Astrologian
-- guide and the official job guide (7.x): Combust (II, III) kept on the target, Malefic (to Fall Malefic) otherwise,
-- Gravity on three or more round it; Divination and Oracle, the draws on cooldown with The Balance and The Spear played
-- (on the Astrologian: it has no say over the party's buffs), the other cards when hurt, Lord of Crowns with enemies
-- near; Earthly Star and its Stellar Detonation when it's a Giant. Retail's Astrologians at 30-39 (one player, 449 s)
-- cast Malefic 105 times, Benefic 43, Essential Dignity 7.

local MALEFIC, MALEFIC_II, MALEFIC_III, MALEFIC_IV, FALL_MALEFIC = 3596, 3598, 7442, 16555, 25871
local GRAVITY, GRAVITY_II = 3615, 25872
local COMBUST, COMBUST_II, COMBUST_III = 3599, 3608, 16554
local ASTRAL_DRAW, UMBRAL_DRAW, DIVINATION, ORACLE = 37017, 37018, 16552, 37029
local THE_BALANCE, THE_ARROW, THE_SPIRE, THE_SPEAR, THE_BOLE, THE_EWER = 37023, 37024, 37025, 37026, 37027, 37028
local LORD_OF_CROWNS, LADY_OF_CROWNS, EARTHLY_STAR, STELLAR_DETONATION = 7444, 7445, 7439, 8324

local S_COMBUST, S_COMBUST_II, S_COMBUST_III, S_DIVINING, S_EARTHLY_DOMINANCE = 838, 843, 1881, 3893, 1224
local CARD_ACTION = { THE_BALANCE, THE_BOLE, THE_ARROW, THE_SPEAR, THE_EWER, THE_SPIRE }

local function gcd(b)
    local dot = b:best(COMBUST_III, COMBUST_II, COMBUST)
    local left = math.max(b:dot(S_COMBUST), b:dot(S_COMBUST_II), b:dot(S_COMBUST_III))
    if b:aoe(3) then
        local gravity = b:best(GRAVITY_II, GRAVITY)
        if gravity and b:affords(gravity) then return gravity end
    end
    if dot and left < 3 and b.target_hp > 10 and b:affords(dot) then return dot end
    local malefic = b:best(FALL_MALEFIC, MALEFIC_IV, MALEFIC_III, MALEFIC_II, MALEFIC)
    if malefic and b:affords(malefic) then return malefic end
    return nil
end

local function ogcd(b)
    if b:ready(DIVINATION) then return DIVINATION, "self" end
    if b:has_status(S_DIVINING) and b:ready(ORACLE) then return ORACLE end
    local cards = b:gauge_of("ast_cards")
    if cards == 0 then
        local draw = b:gauge_of("ast_draw") == 0 and ASTRAL_DRAW or UMBRAL_DRAW
        if b:ready(draw) then return draw, "self" end
    end
    for slot = 0, 2 do
        local card = (cards >> (slot * 4)) & 15
        local action = CARD_ACTION[card]
        if action and b:ready(action) then
            -- The damage cards at once, the others when they'd help (or before the next draw.)
            local damage = card == 1 or card == 4
            local draw_soon = b:cd(ASTRAL_DRAW) < 3
            if damage or b.hp < 90 or draw_soon then return action, "self" end
        end
    end
    local arcana = b:gauge_of("ast_arcana")
    if arcana == 7 and b:ready(LORD_OF_CROWNS) and b.near_self >= 1 then return LORD_OF_CROWNS, "self" end
    if arcana == 8 and b:ready(LADY_OF_CROWNS) and b.hp < 90 then return LADY_OF_CROWNS, "self" end
    if b:ready(EARTHLY_STAR) and b.near_self >= 1 then return EARTHLY_STAR, "self" end
    local star = b:status(S_EARTHLY_DOMINANCE)
    if star > 0 and star < 9 and b:ready(STELLAR_DETONATION) then return STELLAR_DETONATION, "self" end
    return nil
end

return {
    classjobs = { 33 },
    choose = function(b)
        if b:gcd_ready() then
            local action, on = gcd(b)
            if action then return action, on end
        end
        -- Abilities between GCD actions (or with the GCD free when there's nothing to cast on it.)
        if b:can_weave() or b:gcd_ready() then return ogcd(b) end
    end,
}
