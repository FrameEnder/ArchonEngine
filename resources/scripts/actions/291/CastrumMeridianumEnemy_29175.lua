-- Castrum Meridianum's enemies' 29175 (BNpcBase 14577): a 2.7 s cast; base 14577: 177 to npcs (Connor's Chronofoil
-- capture af05916c of 2026-11-09 and Bob's 9ba7d981 and 50e72631 of 2026-11-17, three runs.) Its area is the Action
-- sheet's; the enemies' timelines' `recorded_amounts` scale it (the potency only has to make some damage).
-- Its damage is blunt (retail's damage byte 0x73, on its hits in the five Praetorium runs but parried ones; it was slashing until 2026-11-24.)

local POTENCY = 250

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))

    return effects
end
