-- An action of Brayflox's Longstop's second boss's add (BNpcBase 991) (territory 1041): a lightning attack round
-- itself (297-304 on the goblin 422), twice 29.4 s apart. From Bob's Chronofoil capture 44df02fc (2026-11-03, visit
-- 0.20: one run with Duty Support). Its area and cast are the Action sheet's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_magical_damage(POTENCY))

    return effects
end
