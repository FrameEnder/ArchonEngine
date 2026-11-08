-- History Repeating's quest battle (86): the thaumaturges' 2.2 s cast round themselves (BNpcBase 1648): a magic hit
-- (84-132, the timeline's recorded amount) and Bind (13) on whoever it reaches. Retail's Bind ended 2.5-5.7 s later
-- (four times; it may end early when hit, which isn't scripted): 5 s here. Bob's Chronofoil captures of 2026-11-08.
local POTENCY = 100
local BIND = 13
local BIND_SECONDS = 5.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))
    effects:gain_effect(BIND, 0, BIND_SECONDS)

    return effects
end
