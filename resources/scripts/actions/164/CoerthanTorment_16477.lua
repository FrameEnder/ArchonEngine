-- Coerthan Torment (Lv. 72): 100 to the enemies in a line, 150 after Sonic Thrust; the combo gives Draconian Fire (30 s) from 82.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    drgWeaponskill(effects, player)
    local potency = 100
    if in_combo then
        potency = 150
    end
    effects:damage(DAMAGE_TYPE_PIERCING, drgDamage(player, potency))
    if in_combo and player.level >= 82 then
        effects:gain_effect_self(DRG_EFFECT_DRACONIAN_FIRE, 0, 30.0)
    end

    return effects
end
