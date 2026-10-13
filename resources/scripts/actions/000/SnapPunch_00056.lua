-- Snap Punch (Lv. 6), the last step of the Pugilist combo.
POTENCY = 230
STATUS_OPO_OPO_FORM = 107

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, player.parameters:calc_physical_damage(POTENCY))
    effects:gain_effect_self(STATUS_OPO_OPO_FORM, 0, 30.0)

    return effects
end
