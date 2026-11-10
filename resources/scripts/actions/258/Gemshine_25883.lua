-- Gemshine (Lv. 6): the attuned element's spell: fire (Ruby Ruin, II from 30, III from 54, Ruby Rite from 72), earth (Topaz, Titan's Favor from Topaz Rite at 86), wind (Emerald); a stack of attunement.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local level = player.level
    local fire = level >= 72 and 620 or level >= 54 and 410 or level >= 30 and 380 or 340
    local earth = level >= 72 and 340 or level >= 54 and 300 or level >= 30 and 270 or 240
    local wind = level >= 72 and 280 or level >= 54 and 180 or level >= 30 and 170 or 160
    local kind = player:gauge("attunement_type")
    if smnAttuned(effects, player, { fire, earth, wind }) and kind == 2 and level >= 86 then
        effects:gain_effect_self(SMN_EFFECT_TITANS_FAVOR, 0, 30.0)
    end

    return effects
end
