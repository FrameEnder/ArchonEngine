-- Fell Cleave (Lv. 54): 580; 50 Beast Gauge, nothing under Inner Release, where it is surely a critical direct hit.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if isPlayerCaster(player) then
        warBerserk(effects, player)
    end
    if not warReleaseHit(effects, player) then
        warSpendBeast(effects, player)
    end
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, 580))

    return effects
end
