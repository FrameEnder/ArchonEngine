-- Rotten Meat (BNpcBase 1746, quest battle 61's boss): a 2.7 s cast on itself as the fight begins; retail's
-- ActionEffect had only an effect of kind 61 on itself, which Kawari doesn't know (Bob's Chronofoil captures 130088ec
-- and 3c56e09f of 2026-11-06: four runs.)
function doAction(player, in_combo)
    return EffectsBuilder()
end
