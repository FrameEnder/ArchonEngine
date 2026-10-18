-- Storm's Eye (Lv. 50), after Maim. Combo bonus: Surging Tempest (+10% damage, 30 s).
-- NOTE: the job gauge isn't implemented, so the combo's Beast Gauge (+10) isn't given.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 220
local COMBO_POTENCY = 500
local EFFECT_SURGING_TEMPEST = 2677
local DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local potency = POTENCY
    if in_combo then
        potency = COMBO_POTENCY
    end
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, potency))
    if in_combo then
        effects:gain_effect_self(EFFECT_SURGING_TEMPEST, 0, DURATION)
    end

    return effects
end
