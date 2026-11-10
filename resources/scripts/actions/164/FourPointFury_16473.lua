-- Four-point Fury (Lv. 45): all nearby enemies, changes to coeurl form.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 140
local FORM_DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if isPlayerCaster(player) then
        mnkBeastChakra(effects, player, 2)
    end
    mnkUsePerfectBalance(effects, player)
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))
    mnkShift(effects, player, STATUS_COEURL_FORM)

    return effects
end
