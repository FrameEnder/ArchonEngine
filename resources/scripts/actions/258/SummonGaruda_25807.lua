-- Summon Garuda (Lv. 45): Garuda-Egi's Aerial Blast, 600 (50 % less round it); four wind attunement; Garuda's Favor from 86.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:aoe_falloff(50)
    smnSummon(effects, player, 4, 3, 4, 600)
    if player.level >= 86 then
        effects:gain_effect_self(SMN_EFFECT_GARUDAS_FAVOR, 0, 30.0)
    end

    return effects
end
