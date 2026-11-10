-- Jin (Lv. 45), as the client sends it after the first mudra (retail's Ninjas in Bob's captures send 18807 for the second
-- and third): a mudra, 6 s; adds itself (3) to the mudras so far (Global.lua's ninMudra).
function doAction(player, in_combo)
    local effects = EffectsBuilder()
    ninMudra(effects, player, 3)

    return effects
end
