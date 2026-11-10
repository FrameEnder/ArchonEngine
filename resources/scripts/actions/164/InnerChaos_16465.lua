-- Inner Chaos (Lv. 80): 700, a critical direct hit; 50 Beast Gauge; only with Nascent Chaos, which it uses up.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if not warReleaseHit(effects, player) then
        effects:guarantee_critical()
        effects:guarantee_direct_hit()
        warSpendBeast(effects, player)
    end
    effects:consume_effect(WAR_EFFECT_NASCENT_CHAOS)
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, 700))

    return effects
end
