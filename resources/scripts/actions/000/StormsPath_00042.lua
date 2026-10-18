-- Storm's Path (Lv. 26), after Maim. Combo bonus: restores own HP.
-- NOTE: the job gauge isn't implemented, so the combo's Beast Gauge (+20) isn't given.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 220
local COMBO_POTENCY = 500
local HEAL_POTENCY = 250

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local potency = POTENCY
    if in_combo then
        potency = COMBO_POTENCY
    end
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, potency))
    if in_combo then
        effects:heal_self(healAmount(player, HEAL_POTENCY))
    end

    return effects
end
