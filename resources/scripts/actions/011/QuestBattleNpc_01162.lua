-- Dragon's Blood (BNpcBase 1746, quest battle 61's boss): a 3.7 s cast on itself 11-13 s after the director's sequence
-- 4, whatever its health (31-46 %); retail's ActionEffect had nothing in it (its effect byte 1 on itself), and the
-- director's sequence 5 and his turning followed it (Bob's Chronofoil captures 130088ec and 3c56e09f of 2026-11-06:
-- four runs; content/questbattle/61.lua casts it.)
function doAction(player, in_combo)
    return EffectsBuilder()
end
