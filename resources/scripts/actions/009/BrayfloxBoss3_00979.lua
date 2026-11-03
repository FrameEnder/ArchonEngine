-- An action of Brayflox's Longstop's third boss (BNpcBase 427) (territory 1041): a magic hit on one, every 31.2 s
-- (98 to the player, 54-57 to the tank); the director brings a 1064 where its target stood. From Bob's Chronofoil
-- capture 44df02fc (2026-11-03, visit 0.20: one run with Duty Support). Its area and cast are the Action sheet's.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_magical_damage(POTENCY))

    return effects
end
