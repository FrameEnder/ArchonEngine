-- An action quest battle NPCs use (quest battle 62's boss, BNpcBase 1652): 200 on an ally, once (Bob's Chronofoil
-- capture 689b15da), scaled to the timeline's recorded amount. Not verified for players who use it.
local POTENCY = 150

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))

    return effects
end
