-- Arm of the Destroyer (Lv. 26): all nearby enemies, changes to raptor form.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 110
local FORM_DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if isPlayerCaster(player) then
        mnkBeastChakra(effects, player, 1)
    end
    mnkUsePerfectBalance(effects, player)
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))
    mnkShift(effects, player, STATUS_RAPTOR_FORM)

    return effects
end
