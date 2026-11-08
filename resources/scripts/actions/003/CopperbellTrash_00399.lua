-- An NPC action, a 2.2 s cast round itself: the Copperbell Mines trash (BNpcBase 1421, every 27.8 s, hit nobody in any
-- of its four uses: Bob's Chronofoil captures of the Copperbell Mines, 2026-10-25) and The Curious Case of Giggity's
-- enemy (quest 3870, BNpcBase 1894: a blunt hit of 507 and 763 on the player, Bob's Chronofoil captures of 2026-11-08).
-- The hit is scaled to the NPC's timeline's recorded amount.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))

    return effects
end
