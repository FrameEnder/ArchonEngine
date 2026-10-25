-- An action of the old pugilist's kit that quest battle NPCs still use (Into a Copper Hell's pugilist, BNpcBase 960):
-- a heal on itself, 51-78 (median 52.5) when it was at 33-46 % HP (Bob's Chronofoil capture of 2026-10-25.) Retail's
-- median, as a fixed amount.
local AMOUNT = 52

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal_self(AMOUNT)

    return effects
end
