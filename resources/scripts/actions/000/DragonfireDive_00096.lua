-- Dragonfire Dive (Lv. 50): the target and enemies near it, 50% less after the first.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 500
local FALLOFF = 50

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(FALLOFF)
    effects:damage(DAMAGE_TYPE_PIERCING, drgDamage(player, POTENCY))

    -- Dragon's Flight (30 s) for Rise of the Dragon from 92.
    if player.level >= 92 then
        effects:gain_effect_self(DRG_EFFECT_DRAGONS_FLIGHT, 0, 30.0)
    end

    return effects
end
