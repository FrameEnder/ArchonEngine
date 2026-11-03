-- The Lominsan Way's quest battle (59): its bomb's (BNpcBase 1772, layout 4295473) blast round itself (the Action
-- sheet's area, no cast), 32 s after it came out unless it was killed first: 2290 to the NPC it was after (BNpcBase
-- 1771) and 1826 to the player beside it, both fatal (Bob's Chronofoil captures 44df02fc, 2026-11-03, two of seven
-- runs). The bomb itself falls 2 s later (the director's `kill_bnpc`, content/questbattle/59.lua). Its timeline's
-- `recorded_amounts` scales the hit.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_magical_damage(POTENCY))

    return effects
end
