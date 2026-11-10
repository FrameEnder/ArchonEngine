-- Chaos Thrust (Lv. 50), after Disembowel. Combo bonus: damage over time.
-- NOTE: the rear bonuses (140, 260) aren't implemented.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 100
local COMBO_POTENCY = 220
local DOT_POTENCY = 40
local DURATION = 24.0
local EFFECT_CHAOS_THRUST = 118

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    drgWeaponskill(effects, player)
    local potency = POTENCY
    if in_combo then
        potency = COMBO_POTENCY
    end
    effects:damage(DAMAGE_TYPE_PIERCING, drgDamage(player, potency))
    if in_combo then
        effects:damage_over_time(EFFECT_CHAOS_THRUST, DURATION, drgDamage(player, DOT_POTENCY))
    end

    return effects
end
