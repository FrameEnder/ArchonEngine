-- Jump (Lv. 30).
-- NOTE: Dive Ready comes with Mirage Dive (Lv. 68), so it isn't given here.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 320

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, POTENCY))

    return effects
end
