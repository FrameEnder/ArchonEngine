-- An action quest battle NPCs use (the Akoman in quest battle 17, BNpcBase 301; the QuestBattle sheet's
-- DEF_YODOMI_DORAIN): on one of the yodomi on its side, a drain. It hurts the yodomi (61-97, its timeline's recorded
-- amount), heals the Akoman 30-48 and gives it Regen (158) for 21 s, about 90 a tick (Bob's Chronofoil capture
-- df6f5af9, three runs: 12 drains, the Regen lost 21.1-21.2 s after each, its ticks 86-95). Retail's numbers, as fixed
-- amounts.
local POTENCY = 100
local DRAINED = 32
local REGEN = 158
local REGEN_SECONDS = 21.0
local REGEN_TICK = 90

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_magical_damage(POTENCY))
    effects:heal_self(DRAINED)
    effects:heal_over_time_self(REGEN, REGEN_SECONDS, REGEN_TICK)

    return effects
end
