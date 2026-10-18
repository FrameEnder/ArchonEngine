-- Riot Blade (Lv. 4), after Fast Blade. Combo bonus: restores MP.
-- Potencies: retail's for levels 1-50, measured from level-synced party members in Bob's Chronofoil captures
-- (docs/src/fork/combat.md); higher levels' traits raise them.
local POTENCY = 170
local COMBO_POTENCY = 260
local COMBO_MP = 1000

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local potency = POTENCY
    if in_combo then
        potency = COMBO_POTENCY
    end
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, potency))
    if in_combo then
        effects:restore_mp(COMBO_MP)
    end

    return effects
end
