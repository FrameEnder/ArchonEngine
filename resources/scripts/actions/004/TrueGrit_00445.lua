-- True Grit: Spirithold Broken's clay golem. A 120 degree cone, 6 yalms (the Action sheet and its omen.)
-- On retail it hit a level 9 conjurer for 8 (about 2.25x the golem's auto-attack) and Blinded them (status 15),
-- as recorded with Kawari Recorder. How long the Blind lasts wasn't recorded.
-- Potency is relative to the enemy auto-attack (100).
POTENCY = 225
EFFECT_BLIND = 15
-- Retail's Blind lasted 7 s (Bob's Chronofoil captures, 2 samples.)
BLIND_DURATION = 7.0

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_BREATH, player.parameters:calc_physical_damage(POTENCY))
    effects:gain_effect(EFFECT_BLIND, 0, BLIND_DURATION)

    return effects
end
