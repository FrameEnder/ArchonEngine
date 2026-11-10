-- Precious Brilliance (Lv. 26): the attuned element's area spell round the target: fire (Ruby Outburst, Disaster from 74, Catastrophe from 82), earth, wind; a stack of attunement.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local level = player.level
    local fire = level >= 82 and 210 or level >= 74 and 190 or 160
    local earth = level >= 82 and 140 or level >= 74 and 130 or 110
    local wind = level >= 82 and 100 or level >= 74 and 90 or 70
    local kind = player:gauge("attunement_type")
    if smnAttuned(effects, player, { fire, earth, wind }) and kind == 2 and level >= 86 then
        effects:gain_effect_self(SMN_EFFECT_TITANS_FAVOR, 0, 30.0)
    end

    return effects
end
