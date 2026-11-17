-- The Ultimate Weapon's last battle's boss's 29336: no cast, no target: he goes, and SnapNextMove puts him at his
-- next spot 1.8 s later (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 of 2026-11-17, two
-- runs.) It does nothing itself: what follows is the director's (content/questbattle/5061.lua).
function doAction(player, in_combo)
    return EffectsBuilder()
end
