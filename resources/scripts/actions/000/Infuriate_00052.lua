-- Infuriate (Lv. 50): Beast Gauge +50 (Global.lua's warGainBeast), and from Nascent Chaos (72) that status for 30 s
-- (it turns the next Fell Cleave or Decimate into Inner Chaos or Chaotic Cyclone).
local NASCENT_CHAOS_LEVEL = 72
local EFFECT_NASCENT_CHAOS = 1897
local DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    warGainBeast(effects, player, 50)
    if player.level >= NASCENT_CHAOS_LEVEL then
        effects:gain_effect_self(EFFECT_NASCENT_CHAOS, 0, DURATION)
    end

    return effects
end
