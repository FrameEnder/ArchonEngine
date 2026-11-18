-- Castrum Meridianum's enemies' 29178 (BNpcBase 14574): a 1.2 s cast on one of its own side: a heal, 747 (Bob's
-- Chronofoil capture 6320ae56 of 2026-11-18, its one use in four runs: on a BNpcBase 14571 at 4 % of 7663.) Its
-- timeline uses it on the most hurt of its side (`ally_hp_below`).
local HEAL = 747

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal(HEAL)
    return effects
end
