-- The Praetorium's second boss's 28479: a 2.7 s cast on itself: one of BNpcBase 14452 comes 0.9 s after it lands
-- (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981, 50e72631 and 6320ae56 of 2026-11-17 and 18,
-- five runs.) It does nothing itself: what goes with it is the director's (content/w1d5_re.lua).

function doAction(player, in_combo)
    return EffectsBuilder()
end
