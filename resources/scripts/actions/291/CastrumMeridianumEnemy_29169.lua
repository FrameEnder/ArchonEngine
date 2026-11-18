-- Castrum Meridianum's enemies' 29169 (BNpcBase 14575): no cast seen; base 14575: 112 to npcs (Connor's Chronofoil
-- capture af05916c of 2026-11-09 and Bob's 9ba7d981 and 50e72631 of 2026-11-17, three runs.) Its area is the Action
-- sheet's; the enemies' timelines' `recorded_amounts` scale it (the potency only has to make some damage). To a
-- player: 136 to 188 to Bob's Paladin (capture 6320ae56 of 2026-11-18, three hits), taken as 298 to anyone else.

local POTENCY = 250

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, POTENCY))

    return effects
end
