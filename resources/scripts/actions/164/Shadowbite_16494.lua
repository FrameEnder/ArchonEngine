-- Shadowbite (Lv. 72): 200 to the target and round it, 300 under Barrage; only with Hawk's Eye or Barrage.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local potency = 200
    if player:get_effect(BRD_EFFECT_BARRAGE) ~= nil then
        potency = 300
    end
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, classPotency(player, potency, TRAITS_ARC_ACTION_DAMAGE)))
    brdUseHawksEye(effects, player)

    return effects
end
