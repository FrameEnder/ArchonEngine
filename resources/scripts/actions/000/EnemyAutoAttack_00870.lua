-- Enemy auto-attack (the slashing kind): retail's 870 hits were slashing, 0x71 in the damage's second byte, from every
-- enemy that used it in Bob's Copperbell Mines captures (2026-10-25: 282 hits by six kinds; 872's were all blunt). It
-- was a link to 872, blunt.
POTENCY = 100

function doAction(player, in_combo)
    effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_SLASHING, player.parameters:calc_physical_damage(POTENCY))

    -- A Paladin's lands give it Oath (Global.lua).
    if isPlayerCaster(player) then
        pldGainOath(effects, player)
    end

    return effects
end
