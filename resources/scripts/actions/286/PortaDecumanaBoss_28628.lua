-- The Porta Decumana's first boss's 28628: no cast, no target: it goes, and SnapNextMove brings it out 1.7 s later
-- (both bosses use it) (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, one run
-- each.) It does nothing itself: what goes with it is the director's (content/w1d5_battle.lua).

function doAction(player, in_combo)
    return EffectsBuilder()
end
