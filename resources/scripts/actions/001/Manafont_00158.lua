-- Manafont (Lv. 30): fully restores MP.
-- With Astral Fire (the job guide, 7.x): Astral Fire III, three Umbral Hearts (from Blizzard IV's 58) and Paradox (from
-- 90) too (Global.lua's blm* functions); Thunderhead as before.
local MP = 10000
local EFFECT_THUNDERHEAD = 3870
local THUNDERHEAD_DURATION = 0.0 -- no time limit on retail (Bob's Chronofoil captures; the Status sheet's permanent)

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:restore_mp(MP)
    if blmElement(player) > 0 then
        blmSetElement(effects, player, 3, false)
        if player.level >= 58 then
            effects:gauge_set("umbral_hearts", 3)
        end
        if player.level >= 90 then
            effects:gauge_set("paradox", 1)
        end
    end
    -- Granted when it isn't already up (retail doesn't refresh it.)
    if player:get_effect(EFFECT_THUNDERHEAD) == nil then
        effects:gain_effect_self(EFFECT_THUNDERHEAD, 0, THUNDERHEAD_DURATION)
    end

    return effects
end
