-- Burst Shot (Lv. 76, replaces Heavy Shot): 220; 35 % chance of Hawk's Eye (30 s); Repertoire and Soul Voice in a song
-- (Global.lua's brdRepertoire).
-- Potencies: the official job guide (7.x), at level 100.
function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PIERCING, physicalDamage(player, classPotency(player, 220, TRAITS_ARC_ACTION_DAMAGE)))
    brdHawksEye(effects, 0.35)
    brdRepertoire(effects, player)
    return effects
end
