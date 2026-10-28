-- Sastasha

EOBJ_BLOODY_MEMO_BLUE = 2000212
EOBJ_BLOODY_MEMO_RED = 2001548
EOBJ_BLOODY_MEMO_GREEN = 2001549
EOBJ_INCONSPICUOUS_SWITCH = 2000216
EOBJ_BLUE_CORAL_FORMATION = 2000213
EOBJ_RED_CORAL_FORMATION = 2000214
EOBJ_GREEN_CORAL_FORMATION = 2000215
EOBJ_HIDDEN_DOOR = 2000217
EOBJ_NEXT_DOOR1 = 2001506 -- TODO: what is a better name for this?
EOBJ_RAMBADE_DOOR1 = 2000225
EOBJ_CAPTAINS_QUARTERS_DOOR = 2000227
EOBJ_WAVERIDER_GATE = 2000231
EOBJ_THE_HOLE_DOOR = 2000232
EOBJ_CAPTAINS_QUARTERS_KEY = 2000250
EOBJ_WAVERIDER_GATE_KEY = 2000255
EOBJ_KEY_TO_THE_HOLE = 2000256
EOBJ_NEXT_DOOR2 = 2001539
EOBJ_RAMBADE_DOOR2 = 2000236
EOBJ_UNNATURAL_RIPPLES1 = 2000405
EOBJ_UNNATURAL_RIPPLES2 = 2000406
EOBJ_UNNATURAL_RIPPLES3 = 2000407
EOBJ_UNNATURAL_RIPPLES4 = 2000408

EOBJ_CORAL_IDS = {
    EOBJ_BLUE_CORAL_FORMATION,
    EOBJ_RED_CORAL_FORMATION,
    EOBJ_GREEN_CORAL_FORMATION
}

-- Boss rooms
PLACE_CATTERY = 662
EOBJ_CATTERY_BOSS_WALL = 2001504
EOBJ_CATTERY_BOSS_LINE = 2001505

PLACE_FIRST_RAMBADE = 663
EOBJ_FIRST_RAMBADE_BOSS_WALL = 2001506
EOBJ_FIRST_RAMBADE_BOSS_LINE = 2001507

PLACE_SECOND_RAMBADE = 670
EOBJ_SECOND_RAMBADE_BOSS_WALL = 2001539
EOBJ_SECOND_RAMBADE_BOSS_LINE = 2001540

-- TODO: unsure if its actually called Mistbeard Cove, need to double check
PLACE_NAME_MISTBEARD_COVE = 658
EOBJ_MISTBEARD_COVE_BOSS_WALL = 2001508
EOBJ_MISTBEARD_COVE_BOSS_LINE = 2001509

EVENT_RANGE_BOSS = 4069552

-- Sequence 0
BNPC_GIANT_CLAM1 = 3637470
BNPC_GIANT_CLAM2 = 3637472
BNPC_GIANT_CLAM3 = 3637475
BNPC_GIANT_CLAM4 = 3637473
BNPC_GIANT_CLAM5 = 3637474
BNPC_GIANT_CLAM6 = 3637476

BNPC_RED_CORAL = 4217967
BNPC_BLUE_CORAL = 4217968
BNPC_GREEN_CORAL = 0 -- TODO: figure out layout id
BNPC_CHOPPER = 4035011

-- Sequence 2
BNPC_REAVER1 = 3981887
BNPC_REAVER2 = 3981888
BNPC_CAPTAIN1 = 3988325

-- Sequence 3
BNPC_KEY_HOLDER_REAVER = 3981878
BNPC_CAPTAINS_QUARTERS_REAVER = 3282344

-- Sequence 4
BNPC_REAVER3 = 3978797
BNPC_REAVER4 = 3988324
BNPC_CAPTAIN2 = 4035056
-- The four behind the gate by the second captain (base 414, three of them, and 415): on retail they stand behind it
-- from the moment you come near and join his fight as he falls below half his health: he plays animation 980, the gate
-- (2000235) opens 0.6 s later, he yells 136 and they come (all five of Bob's runs of 2026-10-22, the boss at 30-49 %
-- by then, 0.7-5 s after 50 %; in one he fell before they reached him.)
BNPC_GATE_CORSAIRS = { 4087815, 3988322, 4087814, 3988323 }
EOBJ_CAPTAIN2_GATE = 2000235
CAPTAIN2_GATE_HEALTH = 50
CAPTAIN2_CALL_TIMELINE = 980
CAPTAIN2_CALL_YELL = 136

-- Sequence 5
BNPC_DENN = 3978771
-- His adds come in two waves in the four corners of his arena, the first 47 s into his fight (three of them), the
-- second 16 s later (all four, the first wave's again beside any still standing). Bob's Chronofoil captures: Sastasha
-- with Duty Support (2026-10-23), whose Denn fight lasted 105 s, saw both (at 47.3 and 63.0 s after the tank's pull,
-- Denn at 41 % and 28 %); of five Duty Finder runs (2026-10-22), two lasted long enough for one wave (four, 47 s after
-- Denn appeared in one), three killed him first.
BNPC_DENN_ADDS_FIRST = { 4064938, 4064937, 3978761 }
BNPC_DENN_ADDS_SECOND = { 4064938, 4064937, 3978770, 3978761 }
DENN_FIRST_WAVE = 47.3
DENN_SECOND_WAVE = 63.0
-- The director's onUpdate comes every server tick (0.5 s).
UPDATES_PER_SECOND = 2

-- The Duty Support members' story lines (NpcYell, `director:party_npc_yell(role, line, delay, other_role,
-- other_line)`), when retail's said them (Bob's Chronofoil capture of 2026-10-23, the tank, Lancer and Thaumaturge with
-- a healer): the Lancer 43.6 s after the duty began, the tank as Chopper came, the Lancer 2 s after the first captain
-- fled (his line 134), the Thaumaturge 2 s after the second (137), the tank 15.5 s after Denn's cutscene began. With
-- the Conjurer in the party (an arcanist's: the tank, the Conjurer, the Thaumaturge; Bob's capture cc62ad94 of
-- 2026-10-28) the lines were others': the tank's 14294 at the start (no Lancer; 9.7 s after the opening cutscene, where
-- the Lancer's came 27.1 s after it), the Conjurer's 14299 as Chopper came (the tank there too), the Conjurer's 14301 2
-- s after the first captain fled; the second captain's and Denn's the same. Nobody says them without Duty Support.
PARTY_NPC_LINE_START = 14296
PARTY_NPC_LINE_START_TANK = 14294
PARTY_NPC_LINE_START_AFTER = 43.6
PARTY_NPC_LINE_CHOPPER = 14298
PARTY_NPC_LINE_CHOPPER_HEALER = 14299
PARTY_NPC_LINE_CAPTAIN1 = 14300
PARTY_NPC_LINE_CAPTAIN1_HEALER = 14301
PARTY_NPC_LINE_CAPTAIN2 = 14302
PARTY_NPC_LINE_DENN = 14303
PARTY_NPC_LINE_DENN_AFTER = 15.5
PARTY_NPC_LINE_AFTER_CAPTAIN = 2.0

-- Shortcuts
SHORTCUT_BEFORE_CATTERY = 11432969
SHORTCUT_AFTER_CATTERY = 4033741

SHORTCUT_BEFORE_CAPTAIN1 = 11432970
SHORTCUT_AFTER_CAPTAIN1 = 4033745

SHORTCUT_BEFORE_CAPTAIN2 = 11432971
SHORTCUT_AFTER_CAPTAIN2 = 4165184

SHORTCUT_BEFORE_DENN = 11432972
SHORTCUT_AFTER_DENN = 7372594

EVENT_ACTION_INTERACT = 24

LOG_MESSAGE_SEQ0 = 2034 -- You hear something move in the distance
LOG_MESSAGE_POSION = 2035 -- You are doused with poison!

EFFECT_POSION = 18

SEQ0 = 0 -- Activate the coral trigger
SEQ1 = 1 -- Open the hidden door
SEQ2 = 2 -- Discover the pirate captain
SEQ3 = 4 -- Obtain the Waverider Gate key
SEQ4 = 8 -- Defeat final boss
SEQ5 = 16 -- Dungeon completed

-- Randomized coral color
local coral_color
-- Whether the party has the key to The Hole
local has_hole
-- Whether the party has the key to Captain's Quarters
local has_captains_quarters_key
-- Whether the party has the key to Waverider Gate
local has_waverider_gate_key
-- Whether the Chopper boss was defeated
local chopper_defeated
-- Whether the final boss cutscene played
local seen_final_cutscene
-- Whether Chopper was spawned
local chopper_spawned
-- The second captain's fight, and whether he has called his men through the gate yet
captain2_fighting = false
gate_opened = false
-- Denn's fight: updates since he first went for someone (nil until then), and the waves of adds sent so far
denn_updates = nil
denn_waves = 0

-- What the coffers may hold besides what was recorded (`director:treasure_pool`): the FFXIV Console Games Wiki's loot
-- lists for the duty (each boss's coffer and each "Treasure Coffer" at its map position, matched to these base IDs by
-- the recorded items and by the coffers' positions in Bob's Chronofoil captures), as item IDs from the Item sheet. The
-- starting point: a recording adds what the wiki misses (`server/loot.rs`). The wiki's fixed-rate extras (a Triple
-- Triad card, an orchestrion material) aren't in these.
HEAD_HANDS = { 2684, 2683, 2679, 3556, 3554, 3555 }
ACCESSORIES = { 4206, 4207, 4310, 4096 }
BODY_LEGS = { 3033, 3034, 3035, 3331, 3329, 3330 }
WEAPONS = { 38467, 31422, 31423, 31421, 31425, 31424, 31427, 31428, 31426 }
FEET = { 3781, 3786, 3787 }
-- The bronze coffer by the weapons' list also has these (older pieces of the same levels.)
OLDER_PIECES = { 2235, 2682, 3031, 3030, 3032, 3783, 3782 }

function joined(...)
    local all = {}
    for _, list in ipairs({ ... }) do
        for _, item in ipairs(list) do
            table.insert(all, item)
        end
    end
    return all
end

function onSetup(director)
    coral_color = math.random(0, #EOBJ_CORAL_IDS - 1)

    -- Take down initial boss walls
    director:hide_eobj(EOBJ_CATTERY_BOSS_WALL)
    director:hide_eobj(EOBJ_FIRST_RAMBADE_BOSS_WALL)
    director:hide_eobj(EOBJ_SECOND_RAMBADE_BOSS_WALL)
    director:hide_eobj(EOBJ_MISTBEARD_COVE_BOSS_WALL)

    -- What the coffers held on retail, one list per opening (item IDs; Bob's Chronofoil captures of five runs,
    -- 2026-10-22, a sixth, c99fe374 of 2026-10-28, and a seventh with Duty Support, cc62ad94 of the same day, their
    -- last openings of 93, 94 and 95.) A coffer without a recording (98, 100) holds one of these. See server/loot.rs.
    director:treasure_loot(94, {
        { 3554, 4096 }, { 2684, 4096 }, { 2683, 4207 }, { 2679, 4096 }, { 2679, 4310 }, { 4310 },
    })
    director:treasure_loot(95, { { 3034 }, { 3331 }, { 3035 }, { 3331 }, { 3329 } })
    director:treasure_loot(99, { { 3555 } })
    director:treasure_loot(101, { { 31422 } })
    director:treasure_loot(93, { { 3786, 31424 }, { 31425 }, { 31426, 36374 }, { 3786, 31423 }, { 38467 } })
    -- The wiki's lists: Chopper's (94), Captain Madison's (95), Denn's (93), and Treasure Coffers 1-4 (98 at its
    -- X 13.8 Y 9.2, 99, 100, 101.)
    director:treasure_pool(94, joined(HEAD_HANDS, ACCESSORIES))
    director:treasure_pool(95, BODY_LEGS)
    director:treasure_pool(93, joined(WEAPONS, FEET, { 36374 }))
    director:treasure_pool(98, joined(FEET, { 7036 }))
    director:treasure_pool(99, HEAD_HANDS)
    director:treasure_pool(100, BODY_LEGS)
    director:treasure_pool(101, joined(WEAPONS, OLDER_PIECES))

    -- The coral room's coffer (retail: 98 in all five runs, never 97)
    director:spawn_treasure(98)

    -- The coffers in The Hole (retail: 99 and 101, both every run)
    director:spawn_treasure(99)
    director:spawn_treasure(101)

    beginSequence0(director)

    director:party_npc_yell("melee", PARTY_NPC_LINE_START, PARTY_NPC_LINE_START_AFTER, "tank",
        PARTY_NPC_LINE_START_TANK)
end

function setSequence(director, sequence)
    director:set_data(0, sequence)
end

-- TODO: This should be handled using the existing boss system
function beginBattleMusic(director)
    director:set_bgm(37)
end

function endBattleMusic(director)
    director:set_bgm(0)
end

function onGimmickAccessor(director, actor_id, id, params)
    -- -1 = will not touch the coral
    -- 0 = will touch the coral
    if has_value(EOBJ_CORAL_IDS, id) then
        director:event_action(EVENT_ACTION_INTERACT, actor_id, id)
        return
    elseif id == EOBJ_INCONSPICUOUS_SWITCH then
        if not chopper_defeated then
            if not chopper_spawned then
                chopper_spawned = true

                beginBattleMusic(director)
                director:spawn_boss(BNPC_CHOPPER, EOBJ_CATTERY_BOSS_WALL, EOBJ_CATTERY_BOSS_LINE, PLACE_CATTERY)
                director:party_npc_yell("healer", PARTY_NPC_LINE_CHOPPER_HEALER, 0, "tank", PARTY_NPC_LINE_CHOPPER)
            end
        else
            director:event_action(EVENT_ACTION_INTERACT, actor_id, id)
            return
        end
    elseif id == EOBJ_KEY_TO_THE_HOLE or id == EOBJ_CAPTAINS_QUARTERS_KEY or id == EOBJ_WAVERIDER_GATE_KEY then
        director:event_action(EVENT_ACTION_INTERACT, actor_id, id)
        return
    elseif id == EOBJ_THE_HOLE_DOOR then
        director:event_action(EVENT_ACTION_INTERACT, actor_id, id)
        return
    elseif id == EOBJ_CAPTAINS_QUARTERS_DOOR then
        director:event_action(EVENT_ACTION_INTERACT, actor_id, id)
        return
    elseif id == EOBJ_WAVERIDER_GATE then
        director:event_action(EVENT_ACTION_INTERACT, actor_id, id)
        return
    end

    director:finish_gimmick(actor_id)
end

function isGimmickFunctional(id)
    if id == EOBJ_INCONSPICUOUS_SWITCH then
        if not chopper_defeated and chopper_spawned then
            return false -- Switch is non-functional if you attempt to use it while Chopper is still alive
        else
            return true
        end
    elseif id == EOBJ_THE_HOLE_DOOR then
        return has_hole_key
    elseif id == EOBJ_CAPTAINS_QUARTERS_DOOR then
        return has_captains_quarters_key
    elseif id == EOBJ_WAVERIDER_GATE then
        return has_waverider_gate_key
    elseif id == EOBJ_UNNATURAL_RIPPLES1 or id == EOBJ_UNNATURAL_RIPPLES2 or id == EOBJ_UNNATURAL_RIPPLES3 or id == EOBJ_UNNATURAL_RIPPLES4 then
        return false -- Currently not used
    end

    return true
end

function onGimmickRect(director, target)
    if target == EVENT_RANGE_BOSS and not seen_final_cutscene then
        director:play_cutscene(175) -- Denn's cutscene
        seen_final_cutscene = true
        director:party_npc_yell("tank", PARTY_NPC_LINE_DENN, PARTY_NPC_LINE_DENN_AFTER)
    end
end

function onEventActionCast(director, actor_id, target)
    -- Finish up and hide the event object
    director:finish_gimmick(actor_id)

    if has_value(EOBJ_CORAL_IDS, target) then
        director:hide_eobj(target)

        local coral_gimmick_id = EOBJ_CORAL_IDS[coral_color + 1]
        if target == coral_gimmick_id then
            beginSequence1(director)
        else
            spawnCoralEnemy(director, actor_id, target)
        end
    elseif target == EOBJ_INCONSPICUOUS_SWITCH then
        beginSequence2(director)
    elseif target == EOBJ_KEY_TO_THE_HOLE then
        has_hole_key = true

        director:hide_eobj(target)
    elseif target == EOBJ_WAVERIDER_GATE_KEY then
        has_waverider_gate_key = true

        director:hide_eobj(target)

        beginSequence4(director)
    elseif target == EOBJ_CAPTAINS_QUARTERS_KEY then
        has_captains_quarters_key = true

        director:hide_eobj(target)
    elseif target == EOBJ_CAPTAINS_QUARTERS_DOOR then
        -- TODO: when does this guy spawn? when you open the door?
        director:spawn_bnpc(BNPC_CAPTAINS_QUARTERS_REAVER)

        director:hide_eobj(target)
    elseif target == EOBJ_THE_HOLE_DOOR or target == EOBJ_WAVERIDER_GATE then
        director:hide_eobj(target)
    end
end

-- The second captain below half his health: he calls his men through the gate. Denn's adds come in waves.
function onUpdate(director, health, player_health, info)
    updateDenn(director, health, info)
    if captain2_fighting and not gate_opened and health[BNPC_CAPTAIN2] ~= nil
        and health[BNPC_CAPTAIN2] < CAPTAIN2_GATE_HEALTH then
        gate_opened = true
        director:play_action_timeline(BNPC_CAPTAIN2, CAPTAIN2_CALL_TIMELINE)
        director:hide_eobj(EOBJ_CAPTAIN2_GATE)
        director:yell(BNPC_CAPTAIN2, CAPTAIN2_CALL_YELL)
        for _, id in ipairs(BNPC_GATE_CORSAIRS) do
            director:set_notice(id, true)
            director:set_hate(id, nil, 300) -- the player
        end
    end
end

function onActorDeath(director, bnpc_id, position)
    if bnpc_id == BNPC_CHOPPER then
        director:spawn_treasure(94) -- Treasure for this boss
        endBattleMusic(director)
        chopper_defeated = true

        -- Update shortcut
        director:update_shortcut(SHORTCUT_AFTER_CATTERY)
    elseif bnpc_id == BNPC_CAPTAIN1 then
        -- Two coffers for this boss (retail: 95 and 100 together, every run)
        director:spawn_treasure(95)
        director:spawn_treasure(100)
        endBattleMusic(director)
        beginSequence3(director)
        director:party_npc_yell("melee", PARTY_NPC_LINE_CAPTAIN1, PARTY_NPC_LINE_AFTER_CAPTAIN, "healer",
            PARTY_NPC_LINE_CAPTAIN1_HEALER)

        -- Update shortcut
        director:update_shortcut(SHORTCUT_AFTER_CAPTAIN1)
    elseif bnpc_id == BNPC_KEY_HOLDER_REAVER then
        director:spawn_eobj(EOBJ_CAPTAINS_QUARTERS_KEY, { x = position.x, y = position.y, z = position.z })
    elseif bnpc_id == BNPC_CAPTAINS_QUARTERS_REAVER then
        director:spawn_eobj(EOBJ_WAVERIDER_GATE_KEY, { x = position.x, y = position.y, z = position.z })
    elseif bnpc_id == BNPC_CAPTAIN2 then
        -- No coffer for this boss on retail (no 96 in five runs.)
        endBattleMusic(director)
        director:hide_eobj(EOBJ_RAMBADE_DOOR2)
        director:party_npc_yell("ranged", PARTY_NPC_LINE_CAPTAIN2, PARTY_NPC_LINE_AFTER_CAPTAIN)

        beginSequence5(director)

        -- Update shortcut
        director:update_shortcut(SHORTCUT_AFTER_CAPTAIN2)
    elseif bnpc_id == BNPC_DENN then
        director:spawn_treasure(93) -- Treasure for this boss
        endBattleMusic(director)

        -- Update shortcut
        director:update_shortcut(SHORTCUT_AFTER_DENN)

        -- Play defeat cutscene, with retail's parameters for it (Bob's Chronofoil capture of 2026-10-23: 176, then
        -- 1, 18, 0, 0); with the arrival's (1, 38, 1, 0) the client played Denn's arrival dialogue again after it.
        director:play_cutscene(176, { 1, 18, 0, 0 })

        director:complete_duty()
    end
end

-- Denn's adds: counted from when he first goes for someone, a wave at DENN_FIRST_WAVE and one at DENN_SECOND_WAVE.
function updateDenn(director, health, info)
    if health[BNPC_DENN] == nil then
        return
    end
    if denn_updates == nil then
        if info ~= nil and info[BNPC_DENN] ~= nil and info[BNPC_DENN].target ~= nil then
            denn_updates = 0
        end
        return
    end
    denn_updates = denn_updates + 1
    local seconds = denn_updates / UPDATES_PER_SECOND
    if denn_waves == 0 and seconds >= DENN_FIRST_WAVE then
        denn_waves = 1
        spawnDennAdds(director, BNPC_DENN_ADDS_FIRST)
    elseif denn_waves == 1 and seconds >= DENN_SECOND_WAVE then
        denn_waves = 2
        spawnDennAdds(director, BNPC_DENN_ADDS_SECOND)
    end
end

-- His adds go for the party at once.
function spawnDennAdds(director, adds)
    for _, id in ipairs(adds) do
        director:spawn_bnpc(id)
        director:set_hate(id, nil, 300) -- the player
    end
end

-- Spawns an enemy for getting the coral selection wrong, and douse the player in posion.
function spawnCoralEnemy(director, actor_id, target)
    director:log_message(LOG_MESSAGE_POSION, {})

    director:gain_effect(actor_id, EFFECT_POSION, 0, 120.0)

    if target == EOBJ_BLUE_CORAL_FORMATION then
        director:spawn_bnpc(BNPC_BLUE_CORAL)
    elseif target == EOBJ_RED_CORAL_FORMATION then
        director:spawn_bnpc(BNPC_RED_CORAL)
    elseif target == EOBJ_GREEN_CORAL_FORMATION then
        director:spawn_bnpc(BNPC_GREEN_CORAL)
    end
end

function beginSequence0(director)
    setSequence(director, SEQ0)

    hideBloodyMemos(director)
    director:hide_eobj(EOBJ_INCONSPICUOUS_SWITCH)

    -- spawn them clams
    director:spawn_bnpc(BNPC_GIANT_CLAM1)
    director:spawn_bnpc(BNPC_GIANT_CLAM2)
    director:spawn_bnpc(BNPC_GIANT_CLAM3)
    director:spawn_bnpc(BNPC_GIANT_CLAM4)
    director:spawn_bnpc(BNPC_GIANT_CLAM5)
    director:spawn_bnpc(BNPC_GIANT_CLAM6)
end

function hideBloodyMemos(director)
    if coral_color ~= 0 then
        director:hide_eobj(EOBJ_BLOODY_MEMO_BLUE)
    end
    if coral_color ~= 1 then
        director:hide_eobj(EOBJ_BLOODY_MEMO_RED)
    end
    if coral_color ~= 2 then
        director:hide_eobj(EOBJ_BLOODY_MEMO_GREEN)
    end
end

function beginSequence1(director)
    setSequence(director, SEQ1)

    -- Show log message to help indicate to the player
    director:log_message(LOG_MESSAGE_SEQ0, {})

    -- Hide and deactivate all coral
    director:hide_eobj(EOBJ_BLUE_CORAL_FORMATION)
    director:hide_eobj(EOBJ_RED_CORAL_FORMATION)
    director:hide_eobj(EOBJ_GREEN_CORAL_FORMATION)

    director:show_eobj(EOBJ_INCONSPICUOUS_SWITCH)
end

function beginSequence2(director)
    setSequence(director, SEQ1 | SEQ2) -- TODO: lol this looks awkward

    director:hide_eobj(EOBJ_INCONSPICUOUS_SWITCH)
    director:hide_eobj(EOBJ_HIDDEN_DOOR)

    director:hide_eobj(EOBJ_NEXT_DOOR1)

    -- Update shortcut
    director:update_shortcut(SHORTCUT_BEFORE_CAPTAIN1)

    -- Spawn captain and his goons
    director:spawn_bnpc(BNPC_REAVER1)
    director:spawn_bnpc(BNPC_REAVER2)

    beginBattleMusic(director)
    director:spawn_boss(BNPC_CAPTAIN1, EOBJ_FIRST_RAMBADE_BOSS_WALL, EOBJ_FIRST_RAMBADE_BOSS_LINE, PLACE_FIRST_RAMBADE)
end

function beginSequence3(director)
    setSequence(director, SEQ1 | SEQ2 | SEQ3)

    director:hide_eobj(EOBJ_RAMBADE_DOOR1)

    -- TODO: is this the same BNPC every playthrough?
    director:spawn_bnpc(BNPC_KEY_HOLDER_REAVER)
end

function beginSequence4(director)
    setSequence(director, SEQ1 | SEQ2 | SEQ3 | SEQ4)

    director:hide_eobj(EOBJ_NEXT_DOOR2)

    -- Spawn captain and his goons *AGAIN*
    director:spawn_bnpc(BNPC_REAVER3)
    director:spawn_bnpc(BNPC_REAVER4)

    -- His men behind the gate, who don't notice anyone through it until he calls them
    for _, id in ipairs(BNPC_GATE_CORSAIRS) do
        director:spawn_bnpc(id)
        director:set_notice(id, false)
    end
    captain2_fighting = true

    -- TODO: spawn them dogs

    -- Update shortcut
    director:update_shortcut(SHORTCUT_BEFORE_CAPTAIN2)

    beginBattleMusic(director)
    director:spawn_boss(BNPC_CAPTAIN2, EOBJ_SECOND_RAMBADE_BOSS_WALL, EOBJ_SECOND_RAMBADE_BOSS_LINE, PLACE_SECOND_RAMBADE)
end

-- TODO: why is this "sequence 5"? shouldn't it be 4?
function beginSequence5(director)
    -- Update shortcut
    director:update_shortcut(SHORTCUT_BEFORE_DENN)

    director:spawn_boss(BNPC_DENN, EOBJ_MISTBEARD_COVE_BOSS_WALL, EOBJ_MISTBEARD_COVE_BOSS_LINE, PLACE_NAME_MISTBEARD_COVE)
end

function getDebugShortcut(id)
    if id == 0 then
        return SHORTCUT_BEFORE_CATTERY
    elseif id == 1 then
        return SHORTCUT_BEFORE_CAPTAIN1
    elseif id == 2 then
        return SHORTCUT_BEFORE_CAPTAIN2
    elseif id == 3 then
        return SHORTCUT_BEFORE_DENN
    end

    return 0
end
