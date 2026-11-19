-- Castrum Meridianum's enemies' 29178 (BNpcBase 14574): a 1.2 s cast on one of its own side: a heal. Three uses in
-- five retail runs: 736, 747 and 715 (Bob's Chronofoil captures 9ba7d981 of 2026-11-17, 6320ae56 of 2026-11-18 and
-- deea324c of 2026-11-19: on BNpcBase 14571s at 31, 0.05 and 19 % of 7663 as it landed). Its timeline says when and
-- on whom (`ally_hp_below`, `not_before`).
local HEAL = 736 -- the middle one of the three

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal(HEAL)
    return effects
end
