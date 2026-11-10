-- What a party bot's rotation script sees (server/bot_rotation.rs builds the table each time the bot may act; BotView
-- gives it these methods). A job's script returns { classjobs = {...}, choose = function(b) ... end }; `choose`
-- returns the action to use next (and "self" for one used on the bot itself), or nil to wait.
--
-- b.level, b.mp, b.max_mp, b.hp and b.target_hp (%), b.distance (yalms to its target), b.near_target and b.near_self
-- (living enemies within 5 yalms of its target / of itself), b.combo (its last GCD action while a combo could follow
-- it, else 0), b.gcd_left (seconds until its GCD is back), b.since_gcd, b.weaves (abilities since its last GCD action),
-- b.in_combat (seconds into the fight), b.gauge (its job gauge by name), b.mem (a table kept for the bot).

local View = {}
View.__index = View

function BotView(t)
    return setmetatable(t, View)
end

-- It knows the action at its level, and the server can do it.
function View:has(action)
    return self.known[action] ~= nil
end

-- Seconds until the action can be used again (huge if it isn't known.)
function View:cd(action)
    return self.known[action] or math.huge
end

-- Known and off cooldown (a GCD action's own recast is the GCD's: see gcd_ready.)
function View:ready(action)
    local left = self.known[action]
    return left ~= nil and left <= 0.25
end

function View:gcd_ready()
    return self.gcd_left <= 0.25
end

-- An ability fits now: its GCD is far enough off, and no more than two abilities go between two GCD actions.
function View:can_weave()
    return self.gcd_left >= 0.6 and self.weaves < 2 and self.since_gcd >= 0.5
end

-- Seconds left on one of its own statuses (0 if it hasn't got it; 999 for one that never runs out.)
function View:status(id)
    return self.statuses[id] or 0
end

function View:has_status(id)
    return self.statuses[id] ~= nil
end

-- A status's param (its stacks, for those that count.)
function View:param(id)
    return self.params[id] or 0
end

-- Seconds left on a status it put on its target (a damage over time), or anyone's.
function View:dot(id)
    return self.target_mine[id] or 0
end

function View:debuff(id)
    return self.target_any[id] or 0
end

function View:gauge_of(name)
    return self.gauge[name] or 0
end

-- The first of `actions` it knows (an upgraded action listed before what it replaces.)
function View:best(...)
    for _, action in ipairs({ ... }) do
        if self:has(action) then
            return action
        end
    end
    return nil
end

-- Its damage over time wants putting (back) on: not on the target, or about to wear off, and the target won't die first.
function View:needs_dot(status, refresh_at)
    return self:dot(status) <= (refresh_at or 3) and self.target_hp > 5
end

-- Enough enemies close together for its area attacks to beat its single-target ones.
function View:aoe(count)
    return self.near_target >= (count or 3)
end

-- In melee range of its target (3 yalms, edges; middle to middle allows the target's hitbox a little.)
function View:in_melee()
    return self.distance <= 6
end

-- Its MP covers the action's cost as the sheet has it (`cost` instead, where its gauge changes it.)
function View:affords(action, cost)
    return self.mp >= (cost or self.costs[action] or 0)
end
