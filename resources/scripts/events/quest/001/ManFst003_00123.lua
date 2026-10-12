-- Quest: Close to Home (Gridania), for Archers
-- NOTE: These quests are so similar, ensure changes are synced between all of them!
-- The class-specific parts (ACTOR2 is the guild to visit, ITEM0 is the delivery) come from the Quest sheet.
-- Checked against the client's quest script (objectives in UI8AL/BH/BL, ITEM0 held in UI8CH during sequence 1.)

-- Sequence 0: accept from ACTOR0 in the opening, then get moved into New Gridania (TERRITORYTYPE0.)
-- Sequence 1, in any order:
--   * Attune to the aetheryte (ACTOR1)                        -> UI8AL = 1
--   * Visit your guild (ACTOR2)                               -> UI8BH = 1
--   * Hand the letter (ITEM0, held as UI8CH) to ACTOR3        -> UI8BL = 1, UI8CH = 0
-- Sequence 255: report to ACTOR4 to finish.

function onTalk(target, player)
    local sequence = questSequence(player)

    if sequence == nil then
        -- Show the quest prompt
        questPlay(player, SEQ_0_ACTOR0, HIDE_HOTBAR)
    elseif sequence == 1 then
        if BASE_ID == ACTOR2 then
            questPlay(player, SEQ_1_ACTOR2, HIDE_HOTBAR)
        elseif BASE_ID == ACTOR3 then
            questPlay(player, SEQ_1_ACTOR3, HIDE_HOTBAR)
        elseif BASE_ID == ACTOR0 then
            questPlay(player, SEQ_1_ACTOR0, HIDE_HOTBAR)
        else
            -- Anything else the client routes to us during this sequence is the aetheryte.
            attuneToAetheryte(player)
        end
    elseif sequence == SEQ_FINISH and BASE_ID == ACTOR4 then
        questPlay(player, SEQ_2_ACTOR4, FADE_OUT | CONDITION_CUTSCENE | HIDE_UI)
    else
        player:finish_event()
    end
end

function attuneToAetheryte(player)
    if not player:has_aetheryte(ACTOR1) then
        player:unlock_aetheryte(1, ACTOR1)
    end
    player:unlock(UNLOCK_RETURN) -- attuning here teaches Return
    questPlay(player, SEQ_1_ACTOR1_WAIT, HIDE_HOTBAR)
end

-- Moves to turn-in once every objective is done.
function checkCompletion(player)
    if questVar(player, "AL") == 1 and questVar(player, "BH") == 1 and questVar(player, "BL") == 1 then
        questAdvance(player, SEQ_FINISH)
    end
end

function onSceneFinished(scene, results, player)
    if scene == SEQ_0_ACTOR0 then
        -- Accepted?
        if results[1] == 1 then
            -- Play the introductory text (this has to be played from Momodi)
            questPlay(player, SEQ_0_ACTOR0_LQ, FADE_OUT | CONDITION_CUTSCENE | HIDE_UI)
            return true
        end
    elseif scene == SEQ_0_ACTOR0_LQ then
        player:accept_quest(EVENT_ID, 1)
        setQuestVar(player, "CH", 1) -- receive the letter

        -- Just like in retail, "seamlessly" transition them to the real zone:
        local old_position = player.position
        local old_rotation = player.rotation
        player:change_territory(TERRITORYTYPE0, { x = old_position.x, y = old_position.y, z = old_position.z }, old_rotation)
    elseif scene == SEQ_1_ACTOR1_WAIT then
        questPlay(player, SEQ_1_ACTOR1, SET_EOBJ_BASE | HIDE_HOTBAR | INVIS_EOBJ)
        return true
    elseif scene == SEQ_1_ACTOR1 then
        setQuestVar(player, "AL", 1)
        checkCompletion(player)
    elseif scene == SEQ_1_ACTOR2 then
        setQuestVar(player, "BH", 1)
        checkCompletion(player)
    elseif scene == SEQ_1_ACTOR3 then
        -- Did they hand over the letter?
        if results[1] == 1 then
            questPlay(player, SEQ_1_ACTOR3_NPCTRADEOK, SET_EOBJ_BASE | HIDE_HOTBAR | INVIS_EOBJ)
        else
            questPlay(player, SEQ_1_ACTOR3_NPCTRADENO, HIDE_HOTBAR)
        end
        return true
    elseif scene == SEQ_1_ACTOR3_NPCTRADEOK then
        setQuestVar(player, "CH", 0) -- the letter was handed over
        setQuestVar(player, "BL", 1)
        checkCompletion(player)
    elseif scene == SEQ_1_ACTOR3_NPCTRADENO then
        questPlay(player, SEQ_1_ACTOR0, HIDE_HOTBAR)
        return true
    elseif scene == SEQ_2_ACTOR4 then
        if results[1] == 1 then
            player:finish_quest(EVENT_ID)
        end
    end

    return false
end

onYield = questOnYield
onReturn = questOnReturn
