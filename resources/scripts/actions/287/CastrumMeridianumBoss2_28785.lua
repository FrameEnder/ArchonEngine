-- Castrum Meridianum's second boss's 28785: a 4.7 s cast on its target: 713 to Duty Support's tank (Connor's
-- Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 and 50e72631 of 2026-11-17, three runs.) The
-- timeline's `recorded_amounts` scale it. To a player: 939 and 1044 to Bob's Paladin (capture 6320ae56 of
-- 2026-11-18), taken as 1488 to anyone else (a Paladin took 1/1.5 of what Bob's other jobs did from the duty's spells).

local POTENCY = 450

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
