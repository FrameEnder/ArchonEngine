-- Transpose (Lv. 4): swaps Astral Fire and Umbral Ice.
-- Astral Fire and Umbral Ice: the job gauge's element (Global.lua's blm* functions).
local EFFECT_THUNDERHEAD = 3870
local THUNDERHEAD_DURATION = 0.0 -- no time limit on retail (Bob's Chronofoil captures; the Status sheet's permanent)

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    -- Astral Fire becomes one stack of Umbral Ice, and the other way round.
    local element = blmElement(player)
    if element > 0 then
        blmSetElement(effects, player, -1, false)
    elseif element < 0 then
        blmSetElement(effects, player, 1, false)
    end

    return effects
end
