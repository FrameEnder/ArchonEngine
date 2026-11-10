-- Jump (Lv. 30).
-- NOTE: Dive Ready comes with Mirage Dive (Lv. 68), so it isn't given here.
-- Potencies: retail's for levels 1-50, measured from level-synced party members in Bob's Chronofoil captures
-- (docs/src/fork/combat.md); higher levels' traits raise them.
local POTENCY = 250

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PIERCING, drgDamage(player, POTENCY))

    -- Dive Ready (15 s) for Mirage Dive from 68.
    if player.level >= 68 then
        effects:gain_effect_self(DRG_EFFECT_DIVE_READY, 0, 15.0)
    end

    return effects
end
