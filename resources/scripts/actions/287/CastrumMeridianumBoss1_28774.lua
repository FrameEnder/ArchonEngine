-- Castrum Meridianum's first boss's 28774: a 3.7 s cast on itself: a wave of its soldiers comes a second after it
-- lands (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 and 50e72631 of 2026-11-17, three
-- runs.) It does nothing itself: what goes with it is the director's (content/w1d4_re.lua).

function doAction(player, in_combo)
    return EffectsBuilder()
end
