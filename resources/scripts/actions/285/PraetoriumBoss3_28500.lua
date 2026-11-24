-- The Praetorium's last boss's 28500 (Boss Mod's enrage for the fight): as the gauge fills with any of the four of
-- BNpcBase 14455 still standing, it wipes the party (Harmony's visual guide to the dungeon: "If all four phantoms haven't
-- been destroyed before the blade energy gauge has reached 100% ... this attack will instantly wipe the party"). None
-- of the five retail runs let the gauge fill, so its amount isn't recorded: the timeline's `recorded_amounts` give it
-- more than anyone's HP. Its area is the Action sheet's, as 28499's (unverified: not seen.)

local POTENCY = 500

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
