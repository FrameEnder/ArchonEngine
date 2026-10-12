required_rank = GM_RANK_DEBUG

-- Sets one byte of a quest's variables directly, for testing what the client sees.
-- Usage: !questvar <quest id> <byte 0-5 (0 = UI8A)> <value 0-255>, e.g. !questvar 124 0 16
function onCommand(player, args, name)
    local quest = tonumber(args[1])
    local index = tonumber(args[2])
    local value = tonumber(args[3])
    if quest == nil or index == nil or value == nil or index < 0 or index > 5 or value < 0 or value > 255 then
        printf(player, "Usage: !questvar <quest id> <byte 0-5> <value 0-255>")
        return
    end

    player:set_quest_var(65536 + quest, index, value)
    printf(player, "Set byte %d of quest %d to %d.", index, quest, value)
end
