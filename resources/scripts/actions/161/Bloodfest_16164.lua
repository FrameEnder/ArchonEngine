-- Bloodfest (Lv. 76): three Cartridges (up to six for 30 s); Ready to Reign (30 s) from 100.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(GNB_EFFECT_BLOODFEST, 0, 30.0)
    effects:gauge_set("cartridges", math.min(player:gauge("cartridges") + 3, 6))
    if player.level >= 100 then
        effects:gain_effect_self(GNB_EFFECT_READY_TO_REIGN, 0, 30.0)
    end

    return effects
end
