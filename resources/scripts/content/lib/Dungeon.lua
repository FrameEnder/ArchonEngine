-- Dungeon: the shared framework a duty's script describes its way through with.
--
-- Every dungeon is made of the same things: packs whose fall opens the way (map effects, a gate, who comes out next),
-- places the party's steps set something off, boss rooms (a wall that goes up at the pull, a line, who is in the room
-- and keeps to it, the boss's hidden helpers, its talks and its waves), and what a wipe puts back. Each script wrote
-- them all by hand, so a fix found in one (Castrum Meridianum's boss re-pulled at once after a wipe, its room's four
-- walking out after the party) had to be found again in the others. A script now gives its duty's facts here, and
-- this does the rest the same way in every duty; what one duty does that no other does stays in its own script, in
-- the hooks each piece takes (`on_pull`, `on_cast`, `on_death`, `on_wipe`) or around the calls below.
--
-- Loaded by the server before each duty's script (servers/world/src/server/mod.rs) and by questkit's duty-sim. Pure
-- Lua. See docs/src/fork/dungeons.md ("The dungeon framework").
--
--   DUNGEON = Dungeon.new()
--   DUNGEON:pack({ layouts = { 9088382, 9088383 }, effects = { { 0, 2, 1 } }, spawns = { BOSS } })
--   DUNGEON:step({ name = "boss1_room", where = function(m) return Dungeon.within(m, MIDDLE, 26) end,
--       run = function(d) d:objective(0, 1) end })
--   DUNGEON:boss({ id = 1, layout = BOSS, wall = WALL, line = LINE, place = 434, middle = MIDDLE,
--       room = { members = { BOSS, 9169252 }, radius = 24 }, helpers = { layout = 0, count = 3 },
--       talker = { name = 557, pull = 33523, at = { { 14.6, 33524 } } },
--       waves = { action = 28774, after = 4.7, list = { { 9169258 }, { 9169264 } } },
--       on_death = function(d) d:objective(1, 2) end })
--   function onSetup(director) DUNGEON:setup(director) end
--   function onUpdate(director, health, player_health, info) DUNGEON:update(director, health, player_health, info) end
--   function onActorDeath(director, layout_id, position) DUNGEON:died(director, layout_id) end
--   function onActorCast(director, layout_id, action_id, actor_id, x, y, z) DUNGEON:cast(director, layout_id, ...) end
--   function onPartyWipe(director) DUNGEON:wipe(director) end
--   function onDutyRecommence(director) DUNGEON:recommence(director) end

Dungeon = {}
Dungeon.__index = Dungeon

-- An update is half a second.
Dungeon.UPDATES_PER_SECOND = 2
-- Retail's music in a fight and in the duty (BGM rows), unless a boss says otherwise.
Dungeon.BGM_BATTLE = 37
-- How long a battle talk shows (ms), unless a talker says otherwise.
Dungeon.TALK_MS = 6000
-- How someone in hiding comes out (ActionTimeline, and after how many seconds): Castrum Meridianum's three and two.
Dungeon.APPEAR_TIMELINE = 142
Dungeon.APPEAR_AFTER = 0.7

function Dungeon.new()
    local self = setmetatable({}, Dungeon)
    self.ticks = 0
    self.updating = false
    self.later = {}
    self.dead = {}
    self.done = {}
    self.party = {}
    -- The boss being fought (its `id`), nil between fights, "over" once the last has fallen.
    self.fight = nil
    -- Bosses on their way back home after a wipe (not pulled again until they're there.)
    self.resetting = {}
    self.packs = {}
    self.steps = {}
    self.bosses = {}
    self.helpers_of = {}
    self.helpers = {}
    self.next_helper = 0
    self.health = {}
    self.info = nil
    -- The duty's own clock in seconds (half a second an update), and what's due on it (`inSeconds`.)
    self.clock = 0
    self.timed = {}
    return self
end

-- ---- Time ----

-- Runs fn(director) so many seconds from now (a call made outside an update waits for the next.)
function Dungeon:after(seconds, fn)
    local updates = math.max(1, math.floor(seconds * Dungeon.UPDATES_PER_SECOND + 0.5))
    if not self.updating then
        updates = updates + 1
    end
    table.insert(self.later, { at = self.ticks + updates, fn = fn })
end

function Dungeon:runLater(director)
    local due, waiting = {}, {}
    for _, entry in ipairs(self.later) do
        if entry.at <= self.ticks then
            table.insert(due, entry)
        else
            table.insert(waiting, entry)
        end
    end
    self.later = waiting
    table.sort(due, function(a, b) return a.at < b.at end)
    for _, entry in ipairs(due) do
        entry.fn(director)
    end
end

-- A clock table's rows ({ seconds, ... }) at their times from now, each run as `run(director, row)` if `still()` holds
-- then (a row under a quarter second at once): a trial's fight that keeps to one clock from its pull (the Porta
-- Decumana's two, to 0.3 s in every run.)
function Dungeon:schedule(director, rows, still, run)
    for _, row in ipairs(rows) do
        local go = function(d)
            if still() then
                run(d, row)
            end
        end
        if row[1] < 0.25 then
            go(director)
        else
            self:after(row[1], go)
        end
    end
end

-- Runs fn(director, health, info) once the clock is `seconds` on, at the start of an update (before the party, the
-- steps and the pulls are looked at), with that update's numbers: for a duty that keeps its own clock in seconds (the
-- Stone Vigil's dives and Swinges). `after` counts updates and runs at an update's end.
function Dungeon:inSeconds(seconds, fn, keep)
    -- `keep`: it runs on through a wipe too (the Copperbell Mines' rock coming down 10 s after its blast.)
    table.insert(self.timed, { at = self.clock + seconds, fn = fn, keep = keep })
end

function Dungeon:runTimed(director, health, info)
    local due, rest = {}, {}
    for _, task in ipairs(self.timed) do
        if task.at <= self.clock then
            table.insert(due, task)
        else
            table.insert(rest, task)
        end
    end
    self.timed = rest
    for _, task in ipairs(due) do
        task.fn(director, health, info)
    end
end

-- True the first time it's asked for `name`, never again.
function Dungeon:once(name)
    if self.done[name] then
        return false
    end
    self.done[name] = true
    return true
end

-- ---- The party and the fallen ----

function Dungeon.within(member, at, yalms)
    local dx, dz = member.x - at.x, member.z - at.z
    return dx * dx + dz * dz <= yalms * yalms
end

function Dungeon:anyone(test)
    for _, member in ipairs(self.party) do
        if test(member) then
            return true
        end
    end
    return false
end

function Dungeon:allDead(layouts)
    for _, layout in ipairs(layouts) do
        if not self.dead[layout] then
            return false
        end
    end
    return true
end

function Dungeon:countDead(layouts)
    local count = 0
    for _, layout in ipairs(layouts) do
        if self.dead[layout] then
            count = count + 1
        end
    end
    return count
end

-- Those in hiding come out (spawned, then their appearing timeline.)
function Dungeon:comeOut(director, layouts, timeline, after)
    for _, layout in ipairs(layouts) do
        director:spawn_bnpc(layout)
        director:appear_bnpc(layout, timeline or Dungeon.APPEAR_TIMELINE, after or Dungeon.APPEAR_AFTER)
    end
end

-- ---- Packs ----

-- A pack whose fall opens the way: once every one of `layouts` is dead (and `ready()`, if given), its map effects
-- ({ index, timeline, state }) play, `spawns` are spawned (`come_out`: they come out of hiding instead), and
-- `on_fall(director)` runs. `name` is what `done` calls it (`pack1`, `pack2`... if not given.)
function Dungeon:pack(pack)
    table.insert(self.packs, pack)
    pack.index = #self.packs
    pack.name = pack.name or ("pack" .. pack.index)
    return pack
end

function Dungeon:checkPacks(director)
    for _, pack in ipairs(self.packs) do
        if not self.done[pack.name] and (pack.ready == nil or pack.ready()) and self:allDead(pack.layouts) then
            self:once(pack.name)
            for _, effect in ipairs(pack.effects or {}) do
                director:map_effect(effect[1], effect[2], effect[3])
            end
            if pack.come_out then
                self:comeOut(director, pack.spawns or {})
            else
                for _, layout in ipairs(pack.spawns or {}) do
                    director:spawn_bnpc(layout)
                end
            end
            if pack.on_fall then
                pack.on_fall(director)
            end
        end
    end
end

-- ---- Steps ----

-- Somewhere the party's steps set something off, once: when `ready()` (if given) and anyone of the party passes
-- `where(member)`, `run(director)`. Steps are looked at in the order they're given, every update.
function Dungeon:step(step)
    table.insert(self.steps, step)
    return step
end

function Dungeon:checkSteps(director)
    for _, step in ipairs(self.steps) do
        if not self.done[step.name] and (step.ready == nil or step.ready()) and self:anyone(step.where) then
            self:once(step.name)
            step.run(director)
        end
    end
end

-- ---- Bosses ----

-- A boss and its room. `id` names its fight (`self.fight`); `layout` the boss; `wall`, `line` and `place` its room's
-- (`director:boss_room`, or `director:spawn_boss` for one that comes later: `comes_later`); `middle` its room's
-- middle; `room` = { members, radius }: they keep to it (`director:keep_in_room`); `helpers` = { layout, count }: its
-- hidden helpers, brought with it and held (`helperAt`, `helpersCast`); `bgm` its fight's music (`after_bgm` the
-- duty's after it falls); `talker` = { name, pull, at = { { seconds, text }... }, ms, until_fn }: its talks, at its
-- pull and so many seconds into its fight; `waves` = { action, after, list }: each of its casts of `action` brings
-- the next wave of `list` `after` seconds later (gone with its fall or a wipe); `ready()` whether it can be pulled
-- yet; `group` = { layouts }: several that are the boss together (pulled with any, fallen with the last; `layout`
-- the first of them); `pulled_by` = { layouts }: one of them after someone pulls it too; `arena` its room's range;
-- `last`: its fall ends the duty's fights. Hooks: `on_pull(director)`, `on_cast[action](director, x, y, z)` (or
-- `on_cast(director, action, x, y, z)`), `on_death(director, layout, position)` (the one that fell last, and where),
-- `on_wipe(director)`, `on_recommence(director)`.
function Dungeon:boss(boss)
    table.insert(self.bosses, boss)
    self.helpers_of[boss.id] = {}
    return boss
end

function Dungeon:bossById(id)
    for _, boss in ipairs(self.bosses) do
        if boss.id == id then
            return boss
        end
    end
    return nil
end

function Dungeon:bossByLayout(layout)
    for _, boss in ipairs(self.bosses) do
        if boss.layout == layout then
            return boss
        end
        for _, member in ipairs(boss.group or {}) do
            if member == layout then
                return boss
            end
        end
    end
    return nil
end

-- Its helpers come (held where its room's middle is.)
function Dungeon:bringHelpers(director, boss)
    local helpers = boss.helpers
    self.helpers_of[boss.id] = {}
    if helpers == nil then
        return
    end
    local at = boss.middle
    for i = 1, helpers.count do
        -- (The simulator's spawns return nothing: the layout ID stands in there.)
        self.helpers_of[boss.id][i] = director:spawn_bnpc(helpers.layout, at.x, at.y, at.z, 0.0) or helpers.layout
        director:hold_bnpc(self.helpers_of[boss.id][i])
    end
end

function Dungeon:dropHelpers(director, boss)
    for _, helper in ipairs(self.helpers_of[boss.id] or {}) do
        -- `helpers.release`: let go first (ActorControl 417 with no one: the Copperbell Mines' first and last.)
        if boss.helpers and boss.helpers.release then
            director:release_bnpc(helper)
        end
        director:despawn_bnpc(helper)
    end
    self.helpers_of[boss.id] = {}
    self.helpers = {}
end

-- The next helper of the boss fought, put at a spot.
function Dungeon:helperAt(director, x, y, z, facing)
    if #self.helpers == 0 then
        return nil
    end
    self.next_helper = self.next_helper % #self.helpers + 1
    local helper = self.helpers[self.next_helper]
    director:warp_bnpc(helper, x, y, z, facing or 0.0)
    return helper
end

-- A helper casts `action` at each spot ({ x, z, facing }) at height `y`.
function Dungeon:helpersCast(director, action, spots, y)
    for _, spot in ipairs(spots) do
        local helper = self:helperAt(director, spot[1], y, spot[2], spot[3])
        if helper ~= nil then
            director:actor_use_action(helper, action)
        end
    end
end

-- A helper's cast on each of the party, from where they stand (aimed at them.)
function Dungeon:helpersOnEach(director, action, y)
    for _, member in ipairs(self.party) do
        local helper = self:helperAt(director, member.x, y, member.z)
        if helper ~= nil then
            director:actor_use_action(helper, action, member.id)
        end
    end
end

-- Its room comes as the boss does: the wall open, the line shown, its helpers held, its NPCs kept to it.
function Dungeon:setUpRoom(director, boss)
    if boss.room then
        local middle = boss.middle
        director:keep_in_room(boss.room.members, middle.x, middle.y, middle.z, boss.room.radius)
    end
end

-- The last boss and its room, when it comes later than the start (a cutscene, a wipe.)
function Dungeon:bringBoss(director, boss)
    director:spawn_boss(boss.layout, boss.wall, boss.line, boss.place, boss.arena)
    self:bringHelpers(director, boss)
end

-- Whether `boss` is pulled: hurt, or after someone. After a wipe a boss isn't pulled again until it has been back home,
-- whole and with no one to fight: it was hurt and still had a target as the party went back, and the fight began again
-- at once, its talk with it (Bob's marks of 2026-11-20 at Castrum Meridianum: "the dialogue also played for the black
-- eft after a reset".)
function Dungeon:pulled(boss)
    local layout = boss.layout
    local it = self.info and self.info[layout]
    if it == nil and boss.group then
        -- A group's first may have fallen: whichever of it is there.
        for _, member in ipairs(boss.group) do
            it = it or (self.info and self.info[member])
        end
    end
    -- One of `pulled_by` (or of its `group`) after someone pulls it too (Brayflox's first boss with its three adds,
    -- Haukke Manor's pair), even before the boss is out (the Tam-Tara Deepcroft's rooms: the pack's pull is the room's
    -- fight, its boss called by the pack's fall.)
    local by, others_there = false, false
    local others = {}
    for _, other in ipairs(boss.pulled_by or {}) do
        table.insert(others, other)
    end
    for _, other in ipairs(boss.group or {}) do
        table.insert(others, other)
    end
    for _, other in ipairs(others) do
        local them = self.info and self.info[other]
        if them ~= nil then
            others_there = true
            if them.target ~= nil then
                by = true
            end
        end
    end
    if it == nil and not (others_there and boss.pulled_by) then
        return false
    end
    local health = self.health[layout]
    if self.resetting[layout] then
        if (it == nil or it.target == nil) and not by and (health == nil or health >= 100) then
            self.resetting[layout] = nil
        end
        return false
    end
    return by or (it ~= nil and it.target ~= nil) or (health ~= nil and health < 100)
end

function Dungeon:pull(director, boss)
    self.fight = boss.id
    self.waves_brought = 0
    if boss.bgm ~= false then
        director:set_bgm(boss.bgm or Dungeon.BGM_BATTLE)
    end
    local talker = boss.talker
    if talker then
        local ms = talker.ms or Dungeon.TALK_MS
        if talker.pull then
            director:battle_talk(boss.layout, talker.name, talker.pull, 0, ms)
        end
        for _, row in ipairs(talker.at or {}) do
            self:after(row[1], function(d)
                if self.fight == boss.id and (talker.still == nil or talker.still()) then
                    d:battle_talk(boss.layout, talker.name, row[2], 0, ms)
                end
            end)
        end
    end
    self.helpers = self.helpers_of[boss.id] or {}
    self.next_helper = 0
    if boss.on_pull then
        boss.on_pull(director)
    end
end

function Dungeon:dropWaves(director, boss)
    for _, wave in ipairs(boss.waves and boss.waves.list or {}) do
        for _, layout in ipairs(wave) do
            director:despawn_bnpc(layout)
        end
    end
end

-- ---- What the server calls ----

function Dungeon:setup(director)
    for _, boss in ipairs(self.bosses) do
        if not boss.comes_later then
            for _, layout in ipairs(boss.group or { boss.layout }) do
                director:boss_room(layout, boss.wall, boss.line, boss.place, boss.triggers or {}, boss.arena)
            end
        end
        self:setUpRoom(director, boss)
    end
    -- The walls stand open until a pull; the rooms' lines wait (`eobjs_at_setup = false`: its room's are left to
    -- `spawn_boss` as it comes.)
    for _, boss in ipairs(self.bosses) do
        if boss.wall and boss.eobjs_at_setup ~= false then
            director:hide_eobj(boss.wall)
        end
    end
    for _, boss in ipairs(self.bosses) do
        if boss.line and boss.eobjs_at_setup ~= false then
            director:show_eobj(boss.line)
        end
    end
end

-- Brings each boss's helpers that's there from the start (after the script's own setup, as it wants.)
function Dungeon:bringStartingHelpers(director)
    for _, boss in ipairs(self.bosses) do
        if not boss.comes_later and boss.helpers then
            self:bringHelpers(director, boss)
        end
    end
end

-- Every player of the instance ("player1", "player2"...; "player" alone when that's all there is), then the party's
-- NPCs ("party1"...): before 2026-11-21 only "player", one of them, so a Duty Finder run's other players set nothing
-- off (the Stone Vigil's ranges took them all.)
local function members(info)
    local keys = {}
    local index = 1
    while info["player" .. index] ~= nil do
        table.insert(keys, "player" .. index)
        index = index + 1
    end
    if #keys == 0 and info.player ~= nil then
        table.insert(keys, "player")
    end
    for key, _ in pairs(info) do
        if type(key) == "string" and key:sub(1, 5) == "party" then
            table.insert(keys, key)
        end
    end
    return keys
end

-- Every update: the clock's due (`inSeconds`), the party, the steps, the pulls; then `between(director)` (the
-- script's own, if given), then what's due (`after`).
function Dungeon:update(director, health, player_health, info, between)
    self.ticks = self.ticks + 1
    self.clock = self.clock + 1 / Dungeon.UPDATES_PER_SECOND
    self.updating = true
    self.health = health or {}
    self:runTimed(director, health, info)
    if info ~= nil then
        self.info = info
        self.party = {}
        for _, key in ipairs(members(info)) do
            local actor = info[key]
            -- (Someone with no place yet, at the origin, isn't anywhere; `in_the_party(actor)`, if a script gives it,
            -- says who else isn't: the Porta Decumana's, whoever is out of the arena.)
            if not (actor.x == 0 and actor.z == 0) and (self.in_the_party == nil or self.in_the_party(actor)) then
                table.insert(self.party, { x = actor.x, y = actor.y or 0, z = actor.z, id = actor.id,
                                           player = key:sub(1, 6) == "player" })
            end
        end
        self:checkSteps(director)
        if self.fight == nil then
            for _, boss in ipairs(self.bosses) do
                local fallen = boss.group and self:allDead(boss.group) or (not boss.group and self.dead[boss.layout])
                -- (A fight ended without a death, by `fell`, is over too: Sastasha's captains, who run off hurt.)
                fallen = fallen or self.done["fell" .. boss.id]
                if not fallen and (boss.ready == nil or boss.ready()) and self:pulled(boss) then
                    self:pull(director, boss)
                    break
                end
            end
        end
        if between then
            between(director)
        end
    end
    self:runLater(director)
    self.updating = false
end

function Dungeon:cast(director, layout_id, action_id, actor_id, x, y, z)
    local boss = self:bossByLayout(layout_id)
    if boss == nil or self.fight ~= boss.id then
        return
    end
    local waves = boss.waves
    if waves and action_id == waves.action then
        self.waves_brought = (self.waves_brought or 0) + 1
        local wave = waves.list[self.waves_brought]
        if wave ~= nil then
            self:after(waves.after, function(d)
                if self.fight == boss.id then
                    for _, layout in ipairs(wave) do
                        d:spawn_bnpc(layout)
                    end
                end
            end)
        end
    end
    if type(boss.on_cast) == "table" and boss.on_cast[action_id] then
        boss.on_cast[action_id](director, x, y, z, actor_id)
    elseif type(boss.on_cast) == "function" then
        boss.on_cast(director, action_id, x, y, z, actor_id)
    end
end

function Dungeon:died(director, layout_id, position)
    self.dead[layout_id] = true
    self:checkPacks(director)
    local boss = self:bossByLayout(layout_id)
    if boss == nil then
        return
    end
    -- A group falls with the last of it.
    if boss.group and not self:allDead(boss.group) then
        return
    end
    self:fell(director, boss, layout_id, position)
end

-- A boss's fight is over: by its death (`died`: the last of its group, and where it fell, go to `on_death`), or as a
-- script says (one that ends at 1 HP.) Once only.
function Dungeon:fell(director, boss, layout_id, position)
    if self.fight == "over" or self.done["fell" .. boss.id] then
        return
    end
    self.done["fell" .. boss.id] = true
    self.fight = boss.last and "over" or nil
    -- What was due in its fight goes (`after`); the clock's (`inSeconds`) stays: what a duty set off on it runs on
    -- after a boss falls (Brayflox's goblin's lines, the dragon flying off.)
    self.later = {}
    self:dropHelpers(director, boss)
    self:dropWaves(director, boss)
    if boss.after_bgm then
        director:set_bgm(boss.after_bgm)
    end
    if boss.on_death then
        boss.on_death(director, layout_id, position)
    end
end

-- Everyone down: the fight they fell in is forgotten (the bosses' rooms reset themselves, and none is pulled again
-- till it's home and whole), its waves go, and the boss's `on_wipe` runs.
function Dungeon:wipe(director)
    if self.fight == "over" then
        return
    end
    for _, boss in ipairs(self.bosses) do
        self.resetting[boss.layout] = true
    end
    local was = self:bossById(self.fight)
    self.fight = nil
    self.later = {}
    local kept = {}
    for _, task in ipairs(self.timed) do
        if task.keep then
            table.insert(kept, task)
        end
    end
    self.timed = kept
    self.helpers = {}
    if was then
        self:dropWaves(director, was)
        if was.on_wipe then
            was.on_wipe(director)
        end
    end
end

function Dungeon:recommence(director)
    for _, boss in ipairs(self.bosses) do
        if boss.on_recommence then
            boss.on_recommence(director)
        end
    end
end
