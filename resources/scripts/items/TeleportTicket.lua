-- Aetheryte tickets: after the 5 s cast (the ItemAction's own action, 9994), one is used up and the player is
-- teleported to where the ticket goes: the territory and pop range retail sent them to (ZoneInit's pop range).
--
-- From Bob's Chronofoil capture b8432613 (2026-10-27, five uses during the Ul'dah story): retail's ActionEffect, then
-- the ticket's slot one fewer, then PrepareZoning (warp type 6) into Western Thanalan, landing just outside Vesper
-- Bay's gates on pop range 8366917 every time.

-- { [item ID] = { territory, pop range } }
TICKETS = {
    [30362] = { territory = 140, pop_range = 8366917 }, -- Vesper Bay Aetheryte Ticket
}

function doAction(player, item_id)
    local effects = EffectsBuilder()
    local ticket = TICKETS[item_id]
    if ticket == nil then
        player:send_message("Where aetheryte ticket " .. item_id .. " goes isn't known yet")
        return effects
    end
    player:teleport_by_ticket(item_id, ticket.territory, ticket.pop_range)
    return effects
end
