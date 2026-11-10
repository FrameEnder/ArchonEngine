-- Rook Overdrive (Lv. 40): the turret's Rook Overload at once: its hits left land now.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local left = player:gauge("pet_hits")
    if left > 0 then
        effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, left * player:gauge("pet_potency")))
        effects:gauge_set("pet_hits", 0)
    end

    return effects
end
