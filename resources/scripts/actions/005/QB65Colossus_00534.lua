-- Grand Sword, the Magitek Colossus's in Escape from Castrum Centri (quest battle 65, BNpcBase 2178): a 2.5 s cast, a
-- cone 6 yalms in front of it (the Action sheet's, with its omen), twice in Bob's Kawari Recorder run of 2026-11-12,
-- dodged both times: how hard it hits isn't recorded, an attack's worth.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, POTENCY))

    return effects
end
