function doAction(player, in_combo)
    -- get the aetheryte they requested
    local id = player.teleport_query.aetheryte_id

    -- pay for it (retail takes the gil as the cast lands) and warp there
    player:pay_for_teleport(id)
    player:warp_aetheryte(id)

    return EffectsBuilder()
end
