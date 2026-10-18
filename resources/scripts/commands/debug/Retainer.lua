required_rank = GM_RANK_DEBUG
command_sender = "[retainer] "

-- Retainers (docs/src/fork/retainers.md) until hiring them at a retainer vocate works:
--   !retainer hire <name> [class/job ID] [level]   hires one who looks like your character
--   !retainer list                                 lists them
--   !retainer release <name>                       lets one go, with everything they hold
function onCommand(player, args, name)
    local sub = args[1]
    if sub == "hire" and args[2] then
        player:hire_retainer(args[2], tonumber(args[3]), tonumber(args[4]))
    elseif sub == "list" then
        player:list_retainers()
    elseif sub == "release" and args[2] then
        player:release_retainer(args[2])
    else
        printf(player, "Usage: !retainer hire <name> [class/job] [level], !retainer list, !retainer release <name>")
    end
end
