-- The Ultimate Weapon's last battle's boss's 29787: no cast, on himself, as the gauge fills (Connor's Chronofoil
-- capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, two runs.) It does nothing itself: what follows is
-- the director's (content/questbattle/5061.lua).
-- (Retail's effect on himself: the action's own number as a combo's, `1B 00 00 00 00 80` and it, every time in
-- Bob's Chronofoil captures 9ba7d981, 6320ae56 and 50e72631.)
function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:execute_combo(0)
    return effects
end
