-- Flaming Crush, Ifrit-Egi's in Austerities of Flame (quest battle 79, BNpcBase 1513): a 2 s cast, 5 yalms round
-- itself (the Action sheet's, with its omen). Bob's Kawari Recorder run of 2026-11-12 saw it twice, dodged both times,
-- so how hard it hits isn't recorded: an auto-attack's worth.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
