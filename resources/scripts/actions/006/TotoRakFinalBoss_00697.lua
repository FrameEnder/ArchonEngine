-- The Thousand Maws of Toto-Rak's last boss (BNpcBase 259): instant, every 27.6 s, on its target and those in front
-- of it (Bob's Chronofoil captures of the Thousand Maws of Toto-Rak, 2026-10-30 (caa5e908 and 1fac2416): 121 to players, 45.5 to Duty Support members.) Its timeline's `recorded_amounts` scales it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PHYSICAL, physicalDamage(player, POTENCY))

    return effects
end
