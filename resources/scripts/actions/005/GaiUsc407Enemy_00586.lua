-- Monumental Hopes' (982) quest enemies' cast (BNpcBase 305, the three at the monument): a 2.2 s cast about every
-- 10.5 s, alongside their auto-attack (872). Bob's Chronofoil capture 689b15da (2026-11-09): 20 uses, 179-194 damage
-- against about 103 for an auto-attack, so its potency is about 180 of the auto-attack's 100. The timeline's recorded
-- amounts (GaiUsc407Enemy_305.json) come first where they apply.
POTENCY = 180

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, player.parameters:calc_physical_damage(POTENCY))

    return effects
end
