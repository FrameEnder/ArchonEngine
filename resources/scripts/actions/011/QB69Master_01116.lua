-- Quest battle 69's master's action at the helper (BNpcBase 1561: a 3.5 s cast, an area of 8 yalms where it was
-- aimed, once in the run, as the script has it: content/questbattle/69.lua): 2959 and 3501 on the helper and the
-- ally beside it (Bob's Chronofoil capture 36c7d119 of 2026-11-17). Its timeline's `recorded_amounts` scales it.
local POTENCY = 600

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_magical_damage(POTENCY))

    return effects
end
