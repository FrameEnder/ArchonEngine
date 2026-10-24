-- Every Miner and Botanist action (registered in Actions.lua): what each does at a node, or away from one for
-- Prospect, Truth and Sneak, is the server's (servers/world/src/zone_connection/gathering.rs); the action pipeline has
-- already charged its GP.
function doAction(player, in_combo, action_id)
    player:gathering_action(action_id)
    return EffectsBuilder()
end
