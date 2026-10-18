-- Mug (Lv. 15).
-- NOTE: Dokumori (damage taken +5%) comes at a higher level, so it isn't given here.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 150

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, POTENCY))

    return effects
end
