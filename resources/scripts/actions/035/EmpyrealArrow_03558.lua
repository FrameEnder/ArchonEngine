-- Empyreal Arrow (Lv. 54): 260; a Repertoire of the song being sung.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, classPotency(player, 260, TRAITS_ARC_ACTION_DAMAGE)))
    if isPlayerCaster(player) then
        brdRepertoire(effects, player, 1.0)
    end

    return effects
end
