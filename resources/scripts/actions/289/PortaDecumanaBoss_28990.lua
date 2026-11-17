-- The Porta Decumana's first boss's 28990: a 31.7 s cast on itself with twelve of the helpers' 28991; stopped 21.9 s
-- in (ActorControl 15 [537, 1, 28990]) in both runs (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's
-- 9ba7d981 of 2026-11-17, one run each.) It does nothing itself: what goes with it is the director's
-- (content/w1d5_battle.lua).

function doAction(player, in_combo)
    return EffectsBuilder()
end
