-- Celestial Revolution (Lv. 60): 600; three Beast Chakra; opens the Lunar Nadi, or the Solar one if the Lunar is open; Formless Fist.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, 600))
    -- The Lunar Nadi, or the Solar one if the Lunar is already open.
    local nadi = 1
    if player:gauge("nadi") & 1 ~= 0 then
        nadi = 2
    end
    mnkBlitz(effects, player, nadi)

    return effects
end
