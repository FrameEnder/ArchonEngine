required_rank = GM_RANK_DEBUG
command_sender = "[achievement] "

-- !achievement <id>        completes an achievement (with the pop-up)
-- !achievement clear <id>  takes it away (reopen the Achievements window to see it)
-- !achievement check       completes every achievement whose requirements are met (quests, levels, other achievements)
function onCommand(player, args, name)
    local argc = #args
    if argc == 1 and args[1] == "check" then
        player:check_achievements()
    elseif argc == 1 and tonumber(args[1]) ~= nil then
        player:complete_achievement(tonumber(args[1]))
    elseif argc == 2 and args[1] == "clear" and tonumber(args[2]) ~= nil then
        player:clear_achievement(tonumber(args[2]))
    else
        printf(player, "Usage: !achievement <id> | clear <id> | check")
    end
end
