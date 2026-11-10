-- Retail auto-attacks: a level 3-6 conjurer hits for about 6, which is what a potency of 90 gives with the weapon's
-- delay taken into account.
POTENCY = 90

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_PHYSICAL, player.parameters:calc_auto_attack_damage(POTENCY))

    -- A Paladin's lands give it Oath (Global.lua).
    if isPlayerCaster(player) then
        pldGainOath(effects, player)
    end

    return effects
end
