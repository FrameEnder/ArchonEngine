-- Battle: the shared framework quest battles (and later duties) describe their fight with.
--
-- On retail a battle's director runs it by an outline: who is on which side, what each ally is there to do (Yda, a
-- pugilist, goes for the boss up close; Papalymo, a thaumaturge, takes the adds from range, stays near everyone and
-- heals you when you're low), and how that changes as the fight does (new adds arrive, the boss is gone, someone else
-- joins.) A battle script gives that outline here, and this does the rest every update: who attacks whom, from how
-- far, who stays near whom, who heals whom, and the timed steps (director sequence, battle talk, waves.)
--
-- Loaded by the server before each quest battle's script (servers/world/src/server/mod.rs). Pure Lua, so questkit
-- can run it. See docs/src/fork/quest-battles.md ("The battle framework").
--
--   BATTLE = Battle.new()
--   BATTLE:ally(BNPC_YDA, { focus = { "boss", "adds" }, order = "highest_hp", range = 3 })
--   BATTLE:ally(BNPC_PAPALYMO, { focus = { "adds", "boss" }, order = "oldest", range = 15,
--       keep_near = { within = 15 }, heals = { { action = 120, targets = { "player" }, below = 50, every = 4 } } })
--   BATTLE:enemy(BNPC_BOSS, { group = "boss", attacks = BNPC_YDA })
--   ...
--   function onUpdate(director, health, player_health, info) BATTLE:update(director, health, player_health, info) end
--   function onActorDeath(director, layout_id) BATTLE:died(director, layout_id) end

Battle = {}
Battle.__index = Battle

-- How much an enemy hates whoever it's sent after at first (damage adds to it, so the player can pull it away.)
Battle.STARTING_ENMITY = 300
-- How much more an ally hates each new target it's given, so it switches to it and stays on it.
Battle.ALLY_ENMITY = 10000
-- How often (in updates, half a second each) targets are looked at again without anything happening, for orders that
-- change by themselves (highest_hp, nearest.)
Battle.RETARGET_EVERY = 4
-- The player, wherever a target list takes NPC layout IDs.
Battle.PLAYER = "player"
-- How long (in updates) no other healer picks a target someone has just started healing: a Cure's cast (1.2 s) and
-- its landing, so the others see its health after the heal instead of all healing it at once (three healers on one
-- target in the same update overhealed it by twice its Cure in Bob's 2026-10-20 runs of Duty, Honor, Country.)
Battle.HEAL_PENDING_UPDATES = 3
-- The least time (seconds) between two heals by one NPC, whichever of its rules they come from (a role's `heal_gap`
-- overrides it): one heal's cast and the global cooldown after it. Each rule kept its own clock, so an enemy conjurer
-- Cured and then Cure II'd 1.5 s later (Bob's bundle 20261110-091329); retail's never healed twice within 5.6 s (In
-- the Eyes of Gods and Men, three runs) and no NPC casts twice within a 2.5 s global cooldown.
Battle.HEAL_GAP_SECONDS = 2.5

-- ---- Roles ----
-- Every class belongs to one of three roles, and the role says most of what an NPC is there to do (enemies follow it
-- too, even unique ones with no class.) An NPC given `role = "tank"`, "dps" or "support" starts from these; anything
-- else in its table overrides them.
--
-- `role` is the part it plays in this fight; `class_role` the one its class has, when they differ. In Chasing Shadows
-- Yda's class is a DPS (a pugilist) but she plays the tank, taking the toughest enemy on; Papalymo's is a DPS (a
-- thaumaturge) and he plays a ranged sub-DPS with some support (he heals), staying near Yda and the player.
--
-- * Tank: leads the charge (the others hold back a moment), keeps the enemies on itself and off everyone else
--   (`protect`: every `provoke_every` seconds it pulls an enemy that's hitting someone else onto itself, a support
--   before a DPS before the player), with its big health pool. Up close.
-- * DPS: the most versatile role (a DPS can carry heals and buffs too, as Papalymo's Cure, and still be a DPS):
--   goes for the enemies with the most health; two (or more) DPS split them (`split`: the first takes the
--   toughest, the second the next...), then clean up the adds. Can heal itself (a heal on "self".) Up close unless
--   it's a ranged class (give it a `range`.)
-- * Support: the backline: casts from range, stays near the tank and the player but not straight behind the tank
--   (`stand_behind`: behind the enemy the tank is fighting, out of its cones and cleaves, since a support has the
--   least health), buffs the DPS first and then the tank (`buffs`), heals the tank first and then whoever else is in
--   trouble (heals on "party": the tank below `tank_below`, the others below `below`.)
--
-- keep_near = { anchors = { ... }, within = yalms }: anchors are layout IDs, Battle.PLAYER, or a role ("tank": its
-- first living one); it stays within that of every one of them.
Battle.ROLES = {
    tank = { focus = { "boss", "adds" }, order = "highest_hp", range = 3, protect = true, provoke_every = 10 },
    dps = { focus = { "boss", "adds" }, order = "highest_hp", split = true },
    support = {
        focus = { "adds", "boss" },
        order = "oldest",
        range = 15,
        keep_near = { anchors = { "tank", "player" }, within = 20 },
        stand_behind = { role = "tank", beyond = 6 },
    },
}
-- How long the others hold back after a tank charges in (in updates.)
Battle.FOLLOW_DELAY = 4
-- How much enmity a tank's pull gives it on the enemy it pulls (more each time, so it holds.)
Battle.TANK_ENMITY = 50000
-- Whom a tank protects first, then next...
Battle.PROTECT_ORDER = { support = 1, dps = 2, player = 3 }
-- How long (seconds) enemies that arrive once the fight is on stand where they appear before they set off (they act
-- if someone comes into reach): retail's quest battle waves stood 4.8-5.0 s, Notorious Biggs' second and third waves
-- in all four of Bob's runs (Chronofoil captures 689b15da, da424662, 898dd0b9), quest battle 61's adds and To Guard a
-- Guardian's gargoyle alike. A role's `stands_for` overrides it (0: off at once.)
Battle.ARRIVAL_STAND_SECONDS = 4.9

function Battle.new()
    local self = setmetatable({}, Battle)
    self.roles = {}      -- layout ID -> its role (side, group, focus, order, range, keep_near, heals, attacks)
    self.alive = {}      -- layout ID -> true while it's spawned and standing
    self.arrived = {}    -- enemies, in the order they arrived
    self.arrival = {}    -- layout ID -> its place in that order
    self.targets = {}    -- ally layout ID -> who it was last sent after
    self.priority = 0
    self.pending = {}    -- timed steps: { updates left, step }
    self.triggers = {}   -- { kind, args..., fn, done }
    self.health = {}
    self.info = {}
    self.player_health = nil
    self.updates = 0
    self.context = "start"
    self.hold = {}       -- ally layout ID -> the update it may start fighting (others wait for the tank)
    self.away = {}       -- enemy layout ID -> the update until which allies leave it be (on its way in: Battle:walk)
    self.tank_priority = 0
    self.heal_pending = {} -- target -> the update until which it's being healed by someone
    return self
end

-- ---- The outline ----

local function merge(into, from)
    for k, v in pairs(from or {}) do
        into[k] = v
    end
    return into
end

-- An NPC on the player's side. role: focus (the groups it goes for, in order; "any" is every enemy, "mine" the ones
-- after it), order (which
-- one of a group: "oldest", "newest", "highest_hp", "lowest_hp", "nearest"), range (how close it gets, in yalms),
-- keep_near ({ anchor = layout ID or nil for the player, within = yalms }), heals (see Battle:heal), still (true: it
-- stands where it's put and does nothing: it's held (director:hold_bnpc) as it comes out and never sent after anyone,
-- while the enemies can still be sent after it. Retail's Magitek Reaper in Every Little Thing She Does Is Magitek
-- never moved or acted in three runs while the enemies wore it down; as a tank it walked up to the boss, Bob's mark of
-- 2026-11-14. `questkit battle-sim` asks for it wherever retail's NPC did neither in any run.)
function Battle:ally(layout, role)
    local preset = role and role.role and Battle.ROLES[role.role] or {}
    self.roles[layout] = merge(merge({ side = "ally", focus = { "any" }, order = "oldest" }, preset), role)
end

-- An NPC on the other side. role: group (e.g. "boss", "adds"; several groups can be given as a list), attacks (who
-- it goes after first: an ally's layout ID, or nil for the player), enmity (how much), heals, range, passive (true:
-- it goes after no one until someone hits it, as To Guard a Guardian's masked mage stands by until attacked),
-- unaware (true: it doesn't notice anyone who comes close either, until Battle:engage; Victory in Peril's boss and
-- second pair, who wait for the script.)
-- `role` (tank, dps, support) matters for enemies too: whom their supports heal first, whom a tank protects.
function Battle:enemy(layout, role)
    self.roles[layout] = merge({ side = "enemy", group = "adds" }, role)
end

-- The living NPCs of a side with a role, in the order they were described (stable.)
function Battle:withRole(side, role_name)
    local list = {}
    for layout, role in pairs(self.roles) do
        if role.side == side and role.role == role_name and self.alive[layout] then
            table.insert(list, layout)
        end
    end
    table.sort(list)
    return list
end

-- Changes roles from now on, e.g. when the boss is gone or new allies join: changes = { [layout] = { focus = ... } }.
-- `name` is just for the log of what the battle is doing (BATTLE.context.)
function Battle:change(director, name, changes)
    self.context = name or self.context
    for layout, change in pairs(changes or {}) do
        self.roles[layout] = merge(self.roles[layout] or { side = "ally", focus = { "any" }, order = "oldest" }, change)
        if self.alive[layout] then
            self:applyPlacement(director, layout)
        end
    end
    self:retarget(director)
end

local function inGroup(role, group)
    if group == "any" then
        return true
    end
    if type(role.group) == "table" then
        for _, g in ipairs(role.group) do
            if g == group then
                return true
            end
        end
        return false
    end
    return role.group == group
end

-- ---- Bringing NPCs in ----

function Battle:applyPlacement(director, layout)
    local role = self.roles[layout]
    if role == nil then
        return
    end
    if role.range ~= nil then
        director:set_attack_range(layout, role.range)
    end
    if role.keep_near ~= nil then
        local near = role.keep_near
        local anchors = {}
        -- anchors: layout IDs, the player, or roles (a support near the tank: its first living one.)
        for _, anchor in ipairs(near.anchors or { near.anchor or (near.role or Battle.PLAYER) }) do
            if anchor == Battle.PLAYER or type(anchor) == "number" then
                table.insert(anchors, anchor)
            else
                local first = self:withRole(role.side, anchor)[1]
                if first ~= nil and first ~= layout then
                    table.insert(anchors, first)
                end
            end
        end
        if #anchors == 0 then
            anchors = { Battle.PLAYER }
        end
        director:keep_near(layout, anchors, near.within)
    end
    if role.stand_behind ~= nil then
        local tank = role.stand_behind.tank or self:withRole(role.side, role.stand_behind.role or "tank")[1]
        if tank ~= nil and tank ~= layout then
            director:stand_behind(layout, tank, role.stand_behind.beyond or 6)
        end
    end
end

-- Spawns NPCs (allies or enemies), each as its role says, and points everyone at their targets. `at` ({ x, y, z },
-- optional) spawns them there instead of their layout's spot: quest battle 61's boss turns into another NPC where he
-- stands (retail's spawned 0-5.5 yalms from its layout's spot, wherever he was.)
function Battle:spawn(director, layouts, at)
    for _, layout in ipairs(layouts) do
        local role = self.roles[layout] or { side = "enemy", group = "adds" }
        self.roles[layout] = role
        if at ~= nil then
            director:spawn_bnpc(layout, at.x, at.y, at.z, at.rotation)
        else
            director:spawn_bnpc(layout)
        end
        self.alive[layout] = true
        if role.still then
            director:hold_bnpc(layout)
        end
        if role.side == "enemy" then
            table.insert(self.arrived, layout)
            self.arrival[layout] = #self.arrived
            -- Their health isn't known until the next update: look at everyone's targets again then (an ally that
            -- goes for the weakest settles on it half a second after they come, as retail's did at once.)
            self.retarget_soon = true
            if not role.passive then
                director:set_hate(layout, role.attacks, role.enmity or Battle.STARTING_ENMITY) -- nil is the player
            end
            -- Arriving once the fight is on, it stands a moment first (Battle.ARRIVAL_STAND_SECONDS.)
            local stands = role.stands_for or Battle.ARRIVAL_STAND_SECONDS
            if self.updates > 0 and stands > 0 and not role.passive and not role.unaware then
                director:root_bnpc(layout, stands)
            end
            if role.unaware then
                director:set_notice(layout, false)
            end
        elseif role.role ~= "tank" and #self:withRole("ally", "tank") > 0 then
            -- The tank goes in first; everyone else follows a moment later (or later still, if held: Battle:holdBack.)
            self.hold[layout] = math.max(self.hold[layout] or 0, self.updates + Battle.FOLLOW_DELAY)
        end
        self:applyPlacement(director, layout)
    end
    -- Anyone placed by a role (near the tank, behind its target) is placed again when an ally arrives (it may be
    -- the tank.)
    local ally_arrived = false
    for _, layout in ipairs(layouts) do
        if self.roles[layout].side == "ally" then
            ally_arrived = true
        end
    end
    for other, other_role in pairs(ally_arrived and self.roles or {}) do
        if self.alive[other] and (other_role.stand_behind ~= nil
                or (other_role.keep_near ~= nil and (other_role.keep_near.role ~= nil or other_role.keep_near.anchors ~= nil))) then
            local placed_now = false
            for _, layout in ipairs(layouts) do
                if layout == other then
                    placed_now = true
                end
            end
            if not placed_now then
                self:applyPlacement(director, other)
            end
        end
    end
    self:retarget(director)
end

-- Sends enemies after someone (an ally's layout ID, or nil for the player), and lets unaware ones notice people again.
function Battle:engage(director, layouts, target, enmity)
    for _, layout in ipairs(layouts) do
        if self.alive[layout] then
            local role = self.roles[layout]
            if role ~= nil and role.unaware then
                director:set_notice(layout, true)
            end
            director:set_hate(layout, target, enmity or Battle.STARTING_ENMITY)
        end
    end
end

-- Keeps allies out of the fight, spawned or not, until Battle:release (e.g. through a cutscene, as retail waits for
-- it to end.)
function Battle:holdBack(layouts)
    for _, layout in ipairs(layouts) do
        self.hold[layout] = math.huge
    end
end

-- Lets allies held back fight: the tank at once, everyone else a moment later.
function Battle:release(director, layouts)
    local has_tank = #self:withRole("ally", "tank") > 0
    for _, layout in ipairs(layouts) do
        local role = self.roles[layout]
        if role ~= nil and role.role ~= "tank" and has_tank then
            self.hold[layout] = self.updates + Battle.FOLLOW_DELAY
        else
            self.hold[layout] = nil
        end
    end
    self:retarget(director)
end

-- Removes NPCs without them dying (e.g. everyone from the first half before a cutscene.)
function Battle:remove(director, layouts)
    for _, layout in ipairs(layouts) do
        if self.alive[layout] then
            director:despawn_bnpc(layout)
        end
        self.alive[layout] = nil
        self.targets[layout] = nil
    end
end

-- ---- Who's where ----

-- Whom an enemy is after (an ally's layout ID, or Battle.PLAYER): as the server last said, or, for one that has only
-- just arrived, whom it was sent after (its `attacks`).
function Battle:afterWhom(enemy)
    local info = self.info[enemy]
    if info ~= nil and info.target ~= nil then
        return info.target
    end
    local role = self.roles[enemy]
    if role ~= nil and not role.passive then
        return role.attacks or Battle.PLAYER
    end
    return nil
end

-- The living enemies of a group, in the order they came; `engageable`: only those allies may go for (not `away`.)
-- The group "mine" is the enemies after `ally` (Battle:afterWhom): an ally whose focus starts with it meets whoever
-- comes for it, as Notorious Biggs' escort and conjurer ran out to each wave's enemies that came for them, the moment
-- they appeared (Bob's Chronofoil captures 689b15da, da424662, 898dd0b9: four runs.)
function Battle:living(group, engageable, ally)
    local list = {}
    for _, layout in ipairs(self.arrived) do
        local role = self.roles[layout]
        local in_group
        if group == "mine" then
            in_group = ally ~= nil and self:afterWhom(layout) == ally
        else
            in_group = inGroup(role, group)
        end
        if self.alive[layout] and role.side == "enemy" and in_group
                and (not engageable or (self.away[layout] or 0) <= self.updates) then
            table.insert(list, layout)
        end
    end
    return list
end

function Battle:enemiesLeft()
    return #self:living("any")
end

local function hpOf(self, layout)
    local info = self.info[layout]
    if info ~= nil and info.hp ~= nil then
        return info.hp
    end
    return self.health[layout] or 0
end

local function distance(self, a, b)
    local p, q = self.info[a], self.info[b]
    if p == nil or q == nil or p.x == nil or q.x == nil then
        return math.huge
    end
    return math.sqrt((p.x - q.x) ^ 2 + (p.z - q.z) ^ 2)
end

-- The one of `candidates` that `order` picks for `ally`.
function Battle:pick(ally, candidates, order)
    local best, best_value = nil, nil
    for _, layout in ipairs(candidates) do
        local value
        if order == "newest" then
            value = -self.arrival[layout]
        elseif order == "highest_hp" then
            value = -hpOf(self, layout)
        elseif order == "lowest_hp" then
            value = hpOf(self, layout)
        elseif order == "nearest" then
            value = distance(self, ally, layout)
        else -- oldest
            value = self.arrival[layout]
        end
        if best_value == nil or value < best_value then
            best, best_value = layout, value
        end
    end
    return best
end

-- Every candidate across an ally's focus groups, each group in its order (for DPS splitting the enemies.)
function Battle:pool(ally, role)
    local pool, seen = {}, {}
    for _, group in ipairs(role.focus or { "any" }) do
        local candidates = self:living(group, true, ally)
        while #candidates > 0 do
            local best = self:pick(ally, candidates, role.order)
            for i, layout in ipairs(candidates) do
                if layout == best then
                    table.remove(candidates, i)
                    break
                end
            end
            if not seen[best] then
                seen[best] = true
                table.insert(pool, best)
            end
        end
    end
    return pool
end

-- What an ally should be fighting now: the first of its focus groups with anyone left, by its order. DPS that split
-- take the toughest, the next toughest... between them, by their place among the DPS that split.
function Battle:targetFor(ally)
    local role = self.roles[ally]
    if role.split then
        local splitters = {}
        for layout, other in pairs(self.roles) do
            if other.side == role.side and other.split and self.alive[layout] then
                table.insert(splitters, layout)
            end
        end
        table.sort(splitters)
        if #splitters > 1 then
            local rank = 1
            for i, layout in ipairs(splitters) do
                if layout == ally then
                    rank = i
                end
            end
            local pool = self:pool(ally, role)
            return pool[rank] or pool[1]
        end
    end
    for _, group in ipairs(role.focus or { "any" }) do
        local candidates = self:living(group, true, ally)
        if #candidates > 0 then
            return self:pick(ally, candidates, role.order)
        end
    end
    return nil
end

-- Points every ally at its target, changing only those whose target changed (so enmity doesn't pile up.)
function Battle:retarget(director)
    for layout, role in pairs(self.roles) do
        if role.side == "ally" and not role.still and self.alive[layout]
            and (self.hold[layout] or 0) <= self.updates then
            local target = self:targetFor(layout)
            if target ~= nil and target ~= self.targets[layout] then
                self.priority = self.priority + Battle.ALLY_ENMITY
                director:set_hate(layout, target, self.priority)
                self.targets[layout] = target
            end
        end
    end
end

-- ---- Heals ----

-- A heal rule (in a role's `heals`): action (the heal), targets (a list of layout IDs and/or "player", or "allies"
-- for its whole side), below (health % that calls for it), every (seconds between heals at most), say (a function
-- called with the director when it heals, e.g. for a line.) It heals whoever of its targets is lowest, leaving one
-- another healer has just started on (Battle.HEAL_PENDING_UPDATES). The rules are tried in order (put the bigger heal
-- with the lower mark first), and after any heal none of them heals again for the role's `heal_gap` seconds
-- (Battle.HEAL_GAP_SECONDS.)
function Battle:runHeals(director)
    for layout, role in pairs(self.roles) do
        if role.heals ~= nil and self.alive[layout] then
            role.heal_gap_wait = (role.heal_gap_wait or 0) - 1
            for _, heal in ipairs(role.heals) do
                heal.wait = (heal.wait or 0) - 1
                if heal.wait <= 0 and role.heal_gap_wait <= 0 then
                    local targets = heal.targets
                    local first = {}
                    if targets == "allies" or targets == "party" then
                        targets = role.side == "ally" and { Battle.PLAYER } or {}
                        for other, other_role in pairs(self.roles) do
                            if other_role.side == role.side and self.alive[other] then
                                -- "party": the tank first (or, for enemies, their boss), the rest after.
                                if heal.targets == "party" and (other_role.role == "tank"
                                        or (role.side == "enemy" and inGroup(other_role, "boss"))) then
                                    table.insert(first, other)
                                else
                                    table.insert(targets, other)
                                end
                            end
                        end
                    end
                    local best, best_health = nil, nil
                    -- The tank first, below its own threshold (higher: it takes the hits.)
                    if #first > 0 then
                        best_health = heal.tank_below or 70
                        for _, target in ipairs(first) do
                            local current = self.health[target]
                            if current ~= nil and current > 0 and current < best_health
                                and (self.heal_pending[target] or 0) <= self.updates then
                                best, best_health = target, current
                            end
                        end
                    end
                    if best == nil then
                        best_health = heal.below or 50
                    end
                    for _, target in ipairs(best == nil and targets or {}) do
                        if target == "self" then
                            target = layout
                        end
                        local current
                        if target == Battle.PLAYER then
                            current = self.player_health
                        else
                            current = self.alive[target] and self.health[target] or nil
                        end
                        if current ~= nil and current > 0 and current < best_health
                            and (self.heal_pending[target] or 0) <= self.updates then
                            best, best_health = target, current
                        end
                    end
                    if best ~= nil then
                        self.heal_pending[best] = self.updates + Battle.HEAL_PENDING_UPDATES
                        if best == Battle.PLAYER then
                            director:use_action(layout, heal.action)
                        else
                            director:use_action(layout, heal.action, best)
                        end
                        heal.wait = math.floor((heal.every or 4) * 2)
                        role.heal_gap_wait = math.floor((role.heal_gap or Battle.HEAL_GAP_SECONDS) * 2)
                        if heal.say ~= nil then
                            heal.say(director)
                        end
                    end
                end
            end
        end
    end
end

-- ---- Tanks and supports ----

-- A tank pulls an enemy that's hitting someone else onto itself, every so often: one hitting a support first, then a
-- DPS, then the player.
function Battle:runTanks(director)
    for _, tank in ipairs(self:withRole("ally", "tank")) do
        local role = self.roles[tank]
        if role.protect then
            role.provoke_wait = (role.provoke_wait or 0) - 1
            if role.provoke_wait <= 0 then
                local best, best_rank = nil, nil
                for _, enemy in ipairs(self:living("any", true)) do
                    local info = self.info[enemy]
                    local target = info and info.target
                    local rank = nil
                    if target == Battle.PLAYER then
                        rank = Battle.PROTECT_ORDER.player
                    elseif target ~= nil and target ~= tank and self.roles[target] ~= nil
                        and self.roles[target].side == "ally" then
                        rank = Battle.PROTECT_ORDER[self.roles[target].role or "dps"] or Battle.PROTECT_ORDER.dps
                    end
                    if rank ~= nil and (best_rank == nil or rank < best_rank) then
                        best, best_rank = enemy, rank
                    end
                end
                if best ~= nil then
                    self:provoke(director, tank, best)
                    role.provoke_wait = math.floor((role.provoke_every or 10) * 2)
                end
            end
        end
    end
end

-- A tank takes an enemy onto itself at once (above everyone else's enmity), as a Provoke: `action` (optional) is used
-- on it for the show. For a tank that provoked once, at a set moment, rather than every so often (`protect`).
function Battle:provoke(director, tank, enemy, action)
    if action ~= nil then
        director:use_action(tank, action, enemy)
    end
    self.tank_priority = self.tank_priority + Battle.TANK_ENMITY
    director:set_hate(enemy, tank, self.tank_priority)
end

-- Walks an NPC along a set route, held (not fighting, Battle leaving it alone) until the route's end, then lets it go
-- (unless `release` is false.) For an entrance retail plays the same way every time where the navmesh doesn't lead:
-- quest battle 61's adds stood about 5 s where they appeared, walked to the edge of their ledge and dropped 15 yalms
-- to the arena (five runs, to a yalm), a drop no navmesh path takes. `from` is where it stands as the route begins;
-- each leg is { wait = seconds } or { to = { x, y, z }, speed = yalms a second over the ground } (a drop is a leg
-- whose speed covers the little ground it crosses as it falls, `director:sweep_bnpc` going straight from the one
-- height to the other.) Returns how many seconds it takes.
--
-- Allies leave it be until it's there (`away`): quest battle 61's archer kept on the boss until the first add had
-- dropped to the arena (every run), where Kawari's went after it on the ledge, walked into the cliff below and stopped
-- fighting for 40 s (Bob's bundle 20261110-131148.) A script can keep one away longer (`BATTLE.away[layout]`).
function Battle:walk(director, layout, from, legs, release)
    director:hold_bnpc(layout)
    local t, at = 0, from
    for _, leg in ipairs(legs) do
        if leg.wait ~= nil then
            t = t + leg.wait
        else
            director:delay(t)
            director:sweep_bnpc(layout, leg.to.x, leg.to.y, leg.to.z, leg.speed)
            local dx, dz = leg.to.x - at.x, leg.to.z - at.z
            t = t + math.sqrt(dx * dx + dz * dz) / leg.speed
            at = leg.to
        end
    end
    if release ~= false then
        director:delay(t)
        director:hold_bnpc(layout, false)
        self.away[layout] = self.updates + math.ceil(t * 2)
    else
        self.away[layout] = math.huge
    end
    director:delay(0)
    return t
end

-- A support's buffs ({ action, every (s) } each): on the DPS first, then the tank, one after another.
function Battle:runBuffs(director)
    for layout, role in pairs(self.roles) do
        if role.buffs ~= nil and self.alive[layout] then
            local order = {}
            for _, name in ipairs({ "dps", "tank" }) do
                for _, other in ipairs(self:withRole(role.side, name)) do
                    table.insert(order, other)
                end
            end
            for _, buff in ipairs(role.buffs) do
                buff.wait = (buff.wait or 0) - 1
                if buff.wait <= 0 and #order > 0 then
                    buff.next = (buff.next or 0) % #order + 1
                    director:use_action(layout, buff.action, order[buff.next])
                    buff.wait = math.floor((buff.every or 30) * 2 / #order)
                end
            end
        end
    end
end

-- ---- What happens when ----

-- Timed steps, one after another: { updates after the one before, kind, ... }. Kinds: "sequence" n, "talk" speaker
-- text_id (speaker = { layout, name, icon }), "say" layout key, "yell" layout npc_yell_id, "spawn" { layouts },
-- "remove" { layouts }, "action" layout action_id [target], "change" name changes, "call" function(director).
function Battle:steps(steps)
    local at = 0
    for _, queued in ipairs(self.pending) do
        at = math.max(at, queued[1])
    end
    for _, step in ipairs(steps) do
        at = at + step[1]
        table.insert(self.pending, { at, step })
    end
end

function Battle:runStep(director, step)
    local kind = step[2]
    if kind == "sequence" then
        director:set_sequence(step[3])
    elseif kind == "talk" then
        director:battle_talk(step[3].layout, step[3].name, step[4], step[3].icon, nil, step[3].kind)
    elseif kind == "say" then
        director:say(step[3], step[4])
    elseif kind == "yell" then
        director:yell(step[3], step[4])
    elseif kind == "spawn" then
        self:spawn(director, step[3])
    elseif kind == "remove" then
        self:remove(director, step[3])
    elseif kind == "action" then
        director:use_action(step[3], step[4], step[5])
    elseif kind == "change" then
        self:change(director, step[3], step[4])
    elseif kind == "call" then
        step[3](director)
    end
end

function Battle:runPending(director)
    local still = {}
    for _, queued in ipairs(self.pending) do
        if queued[1] <= 0 then
            self:runStep(director, queued[2])
        else
            queued[1] = queued[1] - 1
            table.insert(still, queued)
        end
    end
    self.pending = still
end

function Battle:clearSteps()
    self.pending = {}
end

-- Calls fn(director) once `layout`'s health is at or below `percent` (or it's gone after being there.)
function Battle:onHealth(layout, percent, fn)
    table.insert(self.triggers, { kind = "health", layout = layout, percent = percent, fn = fn })
end

-- Calls fn(director) once everyone in `group` (a group name, or a list of layout IDs) who arrived has fallen.
function Battle:onCleared(group, fn)
    table.insert(self.triggers, { kind = "cleared", group = group, fn = fn })
end

function Battle:fire(director, trigger)
    trigger.done = true
    trigger.fn(director)
end

function Battle:checkTriggers(director)
    for _, trigger in ipairs(self.triggers) do
        if not trigger.done then
            if trigger.kind == "health" then
                local health = self.health[trigger.layout]
                -- Gone since the last update without passing the mark (killed in one go) counts too.
                if (health ~= nil and health <= trigger.percent)
                    or (health == nil and self.wasAlive ~= nil and self.wasAlive[trigger.layout]) then
                    self:fire(director, trigger)
                end
            elseif trigger.kind == "cleared" then
                local members, arrived_any = {}, false
                if type(trigger.group) == "table" then
                    members = trigger.group
                else
                    for _, layout in ipairs(self.arrived) do
                        if inGroup(self.roles[layout], trigger.group) then
                            table.insert(members, layout)
                        end
                    end
                end
                local standing = false
                for _, layout in ipairs(members) do
                    if self.arrival[layout] ~= nil then
                        arrived_any = true
                    end
                    if self.alive[layout] or self.arrival[layout] == nil then
                        standing = true
                    end
                end
                if arrived_any and not standing then
                    self:fire(director, trigger)
                end
            end
        end
    end
end

-- ---- Every update ----

function Battle:update(director, health, player_health, info)
    self.updates = self.updates + 1
    self.health = health or {}
    self.player_health = player_health
    self.info = info or {}
    self:runPending(director)
    self:runHeals(director)
    self:runTanks(director)
    self:runBuffs(director)
    self:checkTriggers(director)
    if self.retarget_soon or self.updates % Battle.RETARGET_EVERY == 0 then
        self.retarget_soon = false
        self:retarget(director)
    end
    self.wasAlive = {}
    for layout, _ in pairs(self.alive) do
        self.wasAlive[layout] = true
    end
end

-- An NPC died: whoever was after it moves on. Returns true if it was one of the enemies.
function Battle:died(director, layout)
    local was_enemy = self.alive[layout] and self.roles[layout] ~= nil and self.roles[layout].side == "enemy"
    self.alive[layout] = nil
    -- Its health phases it never reached happen now (as the fight would have brought them.)
    self.health[layout] = nil
    for ally, target in pairs(self.targets) do
        if target == layout then
            self.targets[ally] = nil
        end
    end
    self:checkTriggers(director)
    self:retarget(director)
    return was_enemy
end
