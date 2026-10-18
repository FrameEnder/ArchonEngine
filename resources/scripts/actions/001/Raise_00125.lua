-- Raise (Lv. 12): offers a dead party member a revive (status 148, 60 s), which they accept to get up where they lie
-- with 20 % of their HP and MP and Weakness, as retail (Bob's Chronofoil captures; the server side is AcceptRevive.)
function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:offer_revive()

    return effects
end
