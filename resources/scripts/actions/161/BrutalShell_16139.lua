-- Brutal Shell (Lv. 4): 240, 380 after Keen Edge with 200 healed and Brutal Shell (a barrier, 30 s) from 52.
-- Potencies: the official job guide (7.x), at level 100; its traits' lower ones below (`tierPotency`).

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if in_combo then
        gnbDamage(effects, player, 380)
        effects:heal_self(healAmount(player, 200))
        if player.level >= 52 then
            effects:gain_effect_self(GNB_EFFECT_BRUTAL_SHELL, 0, 30.0)
        end
    else
        gnbDamage(effects, player, tierPotency(player, { { 94, 240 }, { 84, 160 } }))
    end

    return effects
end
