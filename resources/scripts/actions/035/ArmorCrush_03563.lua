-- Armor Crush (Lv. 54): 240, 440 after Gust Slash: two Kazematoi and 15 Ninki (flank bonuses aren't done: no positionals).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local potency = 240
    if in_combo then
        potency = 440
        effects:gauge_add("kazematoi", 2)
        ninGainNinki(effects, player, 15)
    end
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, potency))

    return effects
end
