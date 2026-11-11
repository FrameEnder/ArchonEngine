-- Garuda's helper's Feather Rain (664, Eye of the Storm in Boss Mod's T03GarudaN; the Howling Eye; BNpcBase 1570, 2.7 s,
-- every 5.2 s from 16.1 s after Mistral Song began until the end): from the middle, a ring 12-25 yalms out
-- (`action_areas.json`'s `rings`), the arena's edge. It hit no one in any of its 32 uses (Bob's Chronofoil captures of
-- 2026-11-08: everyone stayed within 10.9 yalms of the middle); whoever is caught takes a magic hit, scaled to the
-- helper's timeline's amount (TrialGarudaHelper_1570: a guess, as Mistral Song's.)
local POTENCY = 100

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, POTENCY))

    return effects
end
