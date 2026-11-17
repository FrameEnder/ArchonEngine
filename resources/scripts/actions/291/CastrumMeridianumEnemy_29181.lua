-- Castrum Meridianum's enemies' 29181 (BNpcBase 14578, 14663): a 2.7 s cast; base 14578: 218 to npcs; base 14663: 364
-- to npcs (Connor's Chronofoil capture af05916c of 2026-11-09 and Bob's 9ba7d981 and 50e72631 of 2026-11-17, three
-- runs.) Its area is the Action sheet's; the enemies' timelines' `recorded_amounts` scale it (the potency only has to
-- make some damage).

local POTENCY = 250

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, POTENCY))

    return effects
end
