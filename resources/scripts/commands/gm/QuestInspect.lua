required_rank = GM_RANK_DEBUG
command_sender = "[questinspect] "

-- Prints a quest's sequence and variables, e.g. //gm quest inspect 124
function onCommand(player, args, name)
    local id <const> = args[1]
    local event_id <const> = 65536 + id

    local sequence = player:get_quest_sequence(event_id)
    if sequence == nil then
        if player:has_quest(id) then
            printf(player, "Quest %d is completed.", id)
        else
            printf(player, "Quest %d isn't active.", id)
        end
        return
    end

    local names = { "A", "B", "C", "D", "E", "F" }
    local parts = {}
    for index = 0, 5 do
        local value = player:get_quest_var(event_id, index)
        table.insert(parts, string.format("UI8%s=%d (H=%d L=%d)", names[index + 1], value, value >> 4, value & 0x0F))
    end

    printf(player, "Quest %d: sequence %d\n%s", id, sequence, table.concat(parts, "\n"))
end
