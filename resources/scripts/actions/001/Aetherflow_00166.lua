-- Aetherflow (Lv. 45): restores 20% of max MP.
-- NOTE: Scholar's Aetherflow and Faerie gauges and the faerie aren't implemented: no Aetherflow stacks are given; restores 2000 MP (20% of 10000).
local MP = 2000

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:restore_mp(MP)

    return effects
end
