-- Intervention (Lv. 62): the target takes 10 % less for 8 s (Knight's Resolve, Knight's Benediction from 82); 50 Oath.
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:gain_effect(1174, 0, 8.0)
    if player.level >= 82 then
        effects:gain_effect(2675, 0, 4.0)
        effects:gain_effect(2676, 0, 12.0)
    end
    effects:gauge_add("oath", -50)

    return effects
end
