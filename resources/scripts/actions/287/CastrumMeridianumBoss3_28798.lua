-- Castrum Meridianum's last boss's 28798: a 4.7 s cast on its target: 827 to Duty Support's tank (Connor's Chronofoil
-- capture af05916c of 2026-11-09 and Bob's 9ba7d981 and 50e72631 of 2026-11-17, three runs.) The timeline's
-- `recorded_amounts` scale it. To a player: 1202 to 1620 to Bob's Paladin (capture 6320ae56 of 2026-11-18, four
-- hits; 2255 to the members it turned on), taken as 2112 to anyone else.

local POTENCY = 450

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
