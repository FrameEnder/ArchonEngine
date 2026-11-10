-- Aeolian Edge (Lv. 26), after Gust Slash.
-- NOTE: the rear bonuses (280, 460) aren't implemented.
-- Potencies: retail's for levels 1-50, measured from level-synced party members in Bob's Chronofoil captures
-- (docs/src/fork/combat.md); higher levels' traits raise them.
local POTENCY = 220
local COMBO_POTENCY = 280

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local potency = POTENCY
    if in_combo then
        potency = COMBO_POTENCY
        ninGainNinki(effects, player, 15)
    end
    -- Each Kazematoi (Armor Crush's) adds 100 and is spent.
    if player:gauge("kazematoi") > 0 then
        potency = potency + 100
        effects:gauge_add("kazematoi", -1)
    end
    effects:damage(DAMAGE_TYPE_SLASHING, physicalDamage(player, potency))
    ninBunshin(effects, player, false)

    return effects
end
