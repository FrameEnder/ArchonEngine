-- Bearclaw, the Allagan mirrorknights' (BNpcBase 1948, Allagan Attire): a physical hit on its target. From Bob's Kawari
-- Recorder export of 2026-11-12 (three fights): 167-208 where their auto-attacks did 150-165, so about 1.2 times one.
local POTENCY = 120

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
