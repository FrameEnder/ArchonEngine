-- Quest: The Greatest Story Never Told
--
-- Written from the client's quest script (kawari-questdump, 2026-11-18) and checked against Bob's Chronofoil capture
-- dba9b575 of 2026-11-18 (the whole quest on one character: its fixture is tools/questkit/fixtures/1437.json).
--
-- Four puzzles, each one of nine: three by the player's Grand Company and three picked at random.
-- * The Grand Company is the client's business: every scene asks which of The Company You Keep (66216, 66217,
--   66218) is done, shows that company's clue, and cancels itself at an object or explorer that is another
--   company's. A cancelled scene comes back with error code 50 (retail's, capture dba9b575: the statue with no ring
--   worn, the last ??? with no gorget), and nothing more happens.
-- * The random pick is the server's: UI8CL, 1-3, set as Valiant Hart hands the next puzzle out (retail's were 2, 2
--   and 3). The client only lets the three objects of that group be touched (its isInRandomSelectTable) and shows
--   that group's clue.
-- * The travel log (ITEM0) is held in UI8BH between puzzles and in UI8CH during one (the client's GetEventItems).
--
-- Sequence 0:   ACTOR0 (Valiant Hart) offers it (scene 0, a yield), scene 1: accepted, sequence 1.
-- Sequence 1:   EOBJECT0 (Winebaud's message) shows the first clue (7), ACTOR4 (Yayaroku) the order to read it in (5),
--               ACTOR0 a line (6). The novice explorer where the clue leads (ACTOR1-3: scenes 2-4) gives the travel
--               log: UI8BH 1, notice 0 [1, 1, 26002], sequence 2.
-- Sequence 2:   ACTOR0 (8): notice 1, the group rolled, sequence 3. EOBJECT0 (9), the explorers (10-12).
-- Sequence 3:   EOBJECT0 shows the second clue (34), ACTOR5-7 how to read it (31-33), ACTOR0 a line (35). The place it
--               names (EOBJECT1-9): scene 13 + 2n, the bar (EVENT_ACTION_SEARCH_MIDDLE), scene 14 + 2n: notice 2
--               [1, 1], sequence 4.
-- Sequence 4:   ACTOR0 (36): notice 3, the group rolled, sequence 5. EOBJECT0 (37).
-- Sequence 5:   EOBJECT0 shows the third clue (57), EOBJECT19 how to read it (56), ACTOR0 a line (58). The place it
--               names (EOBJECT10-18): talked to, scene 38 + 2n; the emote the clue spells used on it, scene 39 + 2n:
--               notice 4 [1, 1], sequence 6.
-- Sequence 6:   ACTOR0 (59): notice 5, the group rolled, sequence 7. EOBJECT0 (60).
-- Sequence 7:   The group's statue (EOBJECT0, 20 or 21: scene 61, 62 or 63) shows the fourth clue and, with the piece
--               of jewellery it names worn, asks for it: notice 6 [1, 1], sequence 8. ACTOR8-10 give Winebaud's list
--               (64-66), ACTOR0 a line (67).
-- Sequence 8:   ACTOR0 (68): notice 7, sequence 255. EOBJECT0 (69).
-- Sequence 255: EOBJECT0 shows the poem (72), ACTOR0 talks it over (73). EOBJECT22 with the gorget worn (70), then the
--               cutscene (71, 0xF8406FFF, a yield with the reward): completed.
--
-- Seen on retail: scenes 0, 1, 2, 5, 7, 8, 34, 19 and 20, 36, 57, 44 and 45 (emote 21), 59, 63 (cancelled, then not),
-- 65, 67, 68, 72, 70 (cancelled, then not), 71. The others go by the client's script: which scene is whose follows
-- from what each says and who may be talked to in which sequence (its IsAcceptEvent), and their flags from the seen
-- scenes of the same kind.

-- What retail sets UI8A to when a reward window is closed (414, 680, 840, 855, 960.)
REWARD_CLOSED = 0x11
-- The travel log's icon in notice 0 (retail's, capture dba9b575.)
ICON_TRAVEL_LOG = 26002

ACTOR_FLAGS = HIDE_HOTBAR
OBJECT_FLAGS = NO_DEFAULT_CAMERA | HIDE_HOTBAR
CUTSCENE_FLAGS = SET_BASE | INVIS_ENPC | INVIS_AETHERYTE | HIDE_HOTBAR | INVINCIBLE -- 0xF8406FFF on retail

-- The third clues' lines in the quest's text sheet, 140-148: the client's scene 57 reads 140 + 3 * (company - 1) +
-- (group - 1), and each names its emote with its capital letters (`GAME_DATA:emote_spelled_by`).
THIRD_CLUE = "TEXT_SUBFST999_01437_IBUN01437_000_%d"

-- Only the scene we last played may finish, in case the client reports a scene through both yield and return.
current_scene = nil
-- What the scene that is playing does when it ends without being cancelled, and what follows a progress bar: each
-- is given the callback's own `player`.
after_scene = nil
after_bar = nil

function play(player, scene, flags, after)
    current_scene = scene
    after_scene = after
    player:play_scene(scene, flags, {})
end

-- The group of three the server picked for this puzzle (UI8CL), 0 between puzzles.
function group(player)
    return player:get_quest_var(EVENT_ID, 2) & 0x0F
end

-- Between puzzles: the travel log in UI8BH, no group.
function puzzleSolved(player, notice, sequence)
    player:event_notice(EVENT_ID, notice, { 1, 1 })
    player:set_quest_var(EVENT_ID, 2, 0x00)
    player:set_quest_var(EVENT_ID, 1, 0x10)
    player:quest_sequence(EVENT_ID, sequence)
end

-- A puzzle handed out: the travel log in UI8CH, one of its three groups in UI8CL.
function puzzleGiven(player, notice, sequence)
    player:event_notice(EVENT_ID, notice, {})
    player:set_quest_var(EVENT_ID, 1, 0x00)
    player:set_quest_var(EVENT_ID, 2, 0x10 | math.random(3))
    player:quest_sequence(EVENT_ID, sequence)
end

-- Which of `first`, `first + 1`... `BASE_ID` is, from 0, or nil: EOBJECT1-9 and EOBJECT10-18 are in the order of
-- the client's tables, three to a group.
function indexAmong(objects)
    for index, object in ipairs(objects) do
        if BASE_ID == object then
            return index - 1
        end
    end
    return nil
end

SECOND_PLACES = { EOBJECT1, EOBJECT2, EOBJECT3, EOBJECT4, EOBJECT5, EOBJECT6, EOBJECT7, EOBJECT8, EOBJECT9 }
THIRD_PLACES = { EOBJECT10, EOBJECT11, EOBJECT12, EOBJECT13, EOBJECT14, EOBJECT15, EOBJECT16, EOBJECT17, EOBJECT18 }
STATUES = { EOBJECT0, EOBJECT20, EOBJECT21 }

-- The place's index if it is one of the puzzle's group, as the client's isInRandomSelectTable has it.
function placeInGroup(player, places, size)
    local index = indexAmong(places)
    if index ~= nil and index // size + 1 == group(player) then
        return index
    end
    return nil
end

-- A scene that only shows something.
function show(player, scene, flags)
    play(player, scene, flags, nil)
end

function onTalk(target, player)
    local sequence = player:get_quest_sequence(EVENT_ID)

    if sequence == nil then
        if BASE_ID == ACTOR0 then
            play(player, 0, ACTOR_FLAGS, "offer")
            return
        end
    elseif sequence == 1 then
        local explorer = indexAmong({ ACTOR1, ACTOR2, ACTOR3 })
        if explorer ~= nil then
            -- The client's scene gives the log only to the company the explorer waits for, and cancels otherwise.
            play(player, 2 + explorer, ACTOR_FLAGS, function(player)
                player:event_notice(EVENT_ID, 0, { 1, 1, ICON_TRAVEL_LOG })
                player:set_quest_var(EVENT_ID, 1, 0x10)
                player:quest_sequence(EVENT_ID, 2)
            end)
            return
        elseif BASE_ID == ACTOR4 then
            return show(player, 5, ACTOR_FLAGS)
        elseif BASE_ID == ACTOR0 then
            return show(player, 6, ACTOR_FLAGS)
        elseif BASE_ID == EOBJECT0 then
            return show(player, 7, OBJECT_FLAGS)
        end
    elseif sequence == 2 then
        local explorer = indexAmong({ ACTOR1, ACTOR2, ACTOR3 })
        if BASE_ID == ACTOR0 then
            play(player, 8, ACTOR_FLAGS, function(player)
                puzzleGiven(player, 1, 3)
            end)
            return
        elseif BASE_ID == EOBJECT0 then
            return show(player, 9, OBJECT_FLAGS)
        elseif explorer ~= nil then
            return show(player, 10 + explorer, ACTOR_FLAGS)
        end
    elseif sequence == 3 then
        local place = placeInGroup(player, SECOND_PLACES, 3)
        local helper = indexAmong({ ACTOR5, ACTOR6, ACTOR7 })
        if place ~= nil then
            play(player, 13 + 2 * place, OBJECT_FLAGS, function(player)
                after_bar = function(player)
                    play(player, 14 + 2 * place, OBJECT_FLAGS, function(player)
                        puzzleSolved(player, 2, 4)
                    end)
                end
                player:event_action(EVENT_ACTION_SEARCH_MIDDLE)
                return true
            end)
            return
        elseif helper ~= nil then
            return show(player, 31 + helper, ACTOR_FLAGS)
        elseif BASE_ID == EOBJECT0 then
            return show(player, 34, OBJECT_FLAGS)
        elseif BASE_ID == ACTOR0 then
            return show(player, 35, ACTOR_FLAGS)
        end
    elseif sequence == 4 then
        if BASE_ID == ACTOR0 then
            play(player, 36, ACTOR_FLAGS, function(player)
                puzzleGiven(player, 3, 5)
            end)
            return
        elseif BASE_ID == EOBJECT0 then
            return show(player, 37, OBJECT_FLAGS)
        end
    elseif sequence == 5 then
        local place = placeInGroup(player, THIRD_PLACES, 3)
        if place ~= nil then
            -- Retail's flags (scene 44): an object's talk without the object's camera.
            return show(player, 38 + 2 * place, ACTOR_FLAGS)
        elseif BASE_ID == EOBJECT19 then
            return show(player, 56, OBJECT_FLAGS)
        elseif BASE_ID == EOBJECT0 then
            return show(player, 57, OBJECT_FLAGS)
        elseif BASE_ID == ACTOR0 then
            return show(player, 58, ACTOR_FLAGS)
        end
    elseif sequence == 6 then
        if BASE_ID == ACTOR0 then
            play(player, 59, ACTOR_FLAGS, function(player)
                puzzleGiven(player, 5, 7)
            end)
            return
        elseif BASE_ID == EOBJECT0 then
            return show(player, 60, OBJECT_FLAGS)
        end
    elseif sequence == 7 then
        local statue = placeInGroup(player, STATUES, 1)
        local master = indexAmong({ ACTOR8, ACTOR9, ACTOR10 })
        if statue ~= nil then
            -- The client's scene checks what is worn and asks; without the piece, or on "no", it cancels.
            play(player, 61 + statue, OBJECT_FLAGS, function(player)
                puzzleSolved(player, 6, 8)
            end)
            return
        elseif master ~= nil then
            return show(player, 64 + master, ACTOR_FLAGS)
        elseif BASE_ID == ACTOR0 then
            return show(player, 67, ACTOR_FLAGS)
        end
    elseif sequence == 8 then
        if BASE_ID == ACTOR0 then
            play(player, 68, ACTOR_FLAGS, function(player)
                player:event_notice(EVENT_ID, 7, {})
                player:quest_sequence(EVENT_ID, 255)
            end)
            return
        elseif BASE_ID == EOBJECT0 then
            return show(player, 69, OBJECT_FLAGS)
        end
    elseif sequence == 255 then
        if BASE_ID == EOBJECT22 then
            -- The client's scene cancels unless the gorget is worn.
            play(player, 70, OBJECT_FLAGS, function(player)
                play(player, 71, CUTSCENE_FLAGS, "reward")
                return true
            end)
            return
        elseif BASE_ID == EOBJECT0 then
            return show(player, 72, OBJECT_FLAGS)
        elseif BASE_ID == ACTOR0 then
            return show(player, 73, ACTOR_FLAGS)
        end
    end

    player:finish_event()
end

function onEmote(emote, target, player)
    local place = nil
    if player:get_quest_sequence(EVENT_ID) == 5 then
        place = placeInGroup(player, THIRD_PLACES, 3)
    end
    if place ~= nil then
        -- The object's clue: its group's line of its company's three. A line that spells no emote (it shouldn't
        -- happen: the server logs it) lets any emote through, so the quest can still be finished.
        local company, clue_group = place % 3, place // 3
        local wanted = GAME_DATA:emote_spelled_by(string.format(THIRD_CLUE, 140 + 3 * company + clue_group))
        if wanted == nil or emote == wanted then
            -- The client's scene cancels for another company's object.
            play(player, 39 + 2 * place, OBJECT_FLAGS, function(player)
                puzzleSolved(player, 4, 6)
            end)
            return
        end
    end
    player:finish_event()
end

function onEventActionCast(player)
    if after_bar ~= nil then
        local continue = after_bar
        after_bar = nil
        continue(player)
        return
    end
    player:finish_event()
end

function sceneFinished(scene, results, player)
    if scene ~= current_scene then
        return
    end
    current_scene = nil
    local after = after_scene
    after_scene = nil

    if after == "offer" then
        if results[1] == 1 then
            play(player, 1, ACTOR_FLAGS, function(player)
                player:accept_quest(EVENT_ID)
                player:quest_sequence(EVENT_ID, 1)
            end)
            return
        end
    elseif after == "reward" then
        if results[1] == 1 then
            player:finish_event()
            player:finish_quest(EVENT_ID, results[2] or 0)
            return
        end
        -- The reward window closed: the quest stays open.
        if player:get_quest_var(EVENT_ID, 0) ~= REWARD_CLOSED then
            player:set_quest_var(EVENT_ID, 0, REWARD_CLOSED)
        end
    elseif after ~= nil and player.scene_error_code == 0 then
        -- True when it went on to something else (a bar, the next scene) and the event isn't over.
        if after(player) then
            return
        end
    end

    player:finish_event()
end

function onYield(scene, id, results, player)
    sceneFinished(scene, results, player)
end

function onReturn(scene, results, player)
    sceneFinished(scene, results, player)
end
