-- A battle NPC's cast on an area (the Action sheet's), 2.2 s: 174 on the player the once it landed (Helm Me You Love
-- Me's enemy, BNpcBase 1892; Looking for Trouble's 331 missed with it; Bob's Chronofoil capture 651981e7 of
-- 2026-11-16.) What it does to players comes from the NPC's timeline (recorded_amounts), not from this potency; it
-- isn't verified for players who use it.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
