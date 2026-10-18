-- Rage of Halone (Lv. 26), after Riot Blade.
-- Also used by the handsome stranger in Underneath the Sultantree's quest battle: NPCs never have a combo, so they keep
-- POTENCY, and what it does there comes from the recording (the timeline's recorded_amounts.)
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 100
local COMBO_POTENCY = 330

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local potency = POTENCY
    if in_combo then
        potency = COMBO_POTENCY
    end
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, potency))

    return effects
end
