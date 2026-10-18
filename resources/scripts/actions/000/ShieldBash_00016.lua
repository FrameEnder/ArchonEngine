-- Shield Bash (Lv. 10): stuns.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 100
local STUN_DURATION = 6.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))
    effects:gain_effect(EFFECT_STUN, 0, STUN_DURATION)

    return effects
end
