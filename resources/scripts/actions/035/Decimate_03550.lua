-- Decimate (Lv. 60): 180 round the Warrior; 50 Beast Gauge, nothing under Inner Release (a critical direct hit there).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if isPlayerCaster(player) then
        warBerserk(effects, player)
    end
    if not warReleaseHit(effects, player) then
        warSpendBeast(effects, player)
    end
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, 180))

    return effects
end
