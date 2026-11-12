-- Magitek Ray, the Magitek Colossus's in Escape from Castrum Centri (quest battle 65, BNpcBase 2178): a 3 s cast on the
-- ground, 6 yalms round (the Action sheet's, with its omen), once in Bob's Kawari Recorder run of 2026-11-12, dodged:
-- how hard it hits isn't recorded, an attack's worth.
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
