-- Apex Arrow (Lv. 80): 140 up to 700 with the Soul Voice spent (20 at least); at 80 or more Blast Arrow Ready (10 s) from 86.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local voice = player:gauge("soul_voice")
    local potency = math.floor(140 + (700 - 140) * math.max(voice - 20, 0) / 80)
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, classPotency(player, potency, TRAITS_ARC_ACTION_DAMAGE)))
    effects:gauge_set("soul_voice", 0)
    if voice >= 80 and player.level >= 86 then
        effects:gain_effect_self(BRD_EFFECT_BLAST_ARROW_READY, 0, 10.0)
    end

    return effects
end
