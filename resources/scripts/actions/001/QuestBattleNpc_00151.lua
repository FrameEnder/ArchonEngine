-- An action of the old thaumaturge's kit that quest battle NPCs still use (Brotherly Love's Papalymo, BNpcBase 1587):
-- on an enemy, statuses 9 at 20 and 14 (Bob's Chronofoil capture 5fcc970c, 2026-10-30: its effect, once, on the
-- beast). How long they last wasn't seen: 10 s is a guess. What they do isn't scripted.
local STATUSES = { { 9, 20 }, { 14, 0 } }
local DURATION = 10.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    for _, status in ipairs(STATUSES) do
        effects:gain_effect(status[1], status[2], DURATION)
    end

    return effects
end
