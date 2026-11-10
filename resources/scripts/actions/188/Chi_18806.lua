-- Chi (Lv. 35), as the client sends it after the first mudra (retail's Ninjas in Bob's captures send 18806 for the second
-- and third): a mudra, 6 s; adds itself (2) to the mudras so far (Global.lua's ninMudra).
function doAction(player, in_combo)
    local effects = EffectsBuilder()
    ninMudra(effects, player, 2)

    return effects
end
