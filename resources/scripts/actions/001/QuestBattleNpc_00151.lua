-- Lethargy, of the old thaumaturge's kit that quest battle NPCs still use (Brotherly Love's Papalymo, BNpcBase 1587;
-- Escape from Castrum Centri's signifers, 2171): Slow (9) and Heavy (14, at 20) on its target. Retail's effect has 20
-- in the first status's second byte and as the second's parameter (`0e 14 00 0000 00 0900`, `0e 00 00 1400 00 0e00`:
-- 13 of 13 in Bob's Chronofoil captures da424662 and d1c46dff), and the player then had status 9 with parameter 0 and
-- 14 with 20, for 12 s (the first on them; 6 s the second time and 3 s the third, which isn't done here). Until
-- 2026-11-15 the 20 was given to status 9 and Heavy came with 0, for a guessed 10 s. What they do isn't scripted.
local STATUSES = { { 9, 0 }, { 14, 20 } }
local DURATION = 12.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    for _, status in ipairs(STATUSES) do
        effects:gain_effect(status[1], status[2], DURATION)
    end

    return effects
end
