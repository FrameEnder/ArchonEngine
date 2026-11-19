-- Castrum Meridianum's enemies' 29174 (BNpcBase 14573): a 2.7 s cast; base 14573: 184 to npcs (Connor's Chronofoil
-- capture af05916c of 2026-11-09 and Bob's 9ba7d981 and 50e72631 of 2026-11-17, three runs.) Its area is the Action
-- sheet's; the enemies' timelines' `recorded_amounts` scale it (the potency only has to make some damage).
-- 524 to a player (Bob's White Mage, Chronofoil capture deea324c of 2026-11-19).

local POTENCY = 250

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, POTENCY))

    return effects
end
