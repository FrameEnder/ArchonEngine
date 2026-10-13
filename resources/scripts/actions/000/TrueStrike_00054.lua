-- True Strike (Lv. 4), the second step of the Pugilist combo.
POTENCY = 260
STATUS_COEURL_FORM = 109

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, player.parameters:calc_physical_damage(POTENCY))
    effects:gain_effect_self(STATUS_COEURL_FORM, 0, 30.0)

    return effects
end
