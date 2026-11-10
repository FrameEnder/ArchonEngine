-- Delirium (Lv. 68): three stacks of Delirium (Enhanced Delirium from 96: Scarlet Delirium's combo) and Blood Weapon, 15 s.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect_self(DRK_EFFECT_BLOOD_WEAPON, 3, 15.0)
    if player.level >= 96 then
        effects:gain_effect_self(DRK_EFFECT_ENHANCED_DELIRIUM, 3, 15.0)
    else
        effects:gain_effect_self(DRK_EFFECT_DELIRIUM, 3, 15.0)
    end

    return effects
end
