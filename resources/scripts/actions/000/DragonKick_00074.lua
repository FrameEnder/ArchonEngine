-- Dragon Kick (Lv. 50): changes to raptor form.
-- In Opo-opo form gives Opo-opo's Fury (the gauge's "opo_fury").
-- Potencies: retail's for levels 1-50, measured from level-synced party members in Bob's Chronofoil captures
-- (docs/src/fork/combat.md); higher levels' traits raise them.
local POTENCY = 250
local FORM_DURATION = 30.0

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    if isPlayerCaster(player) then
        mnkBeastChakra(effects, player, 1)
    end
    mnkUsePerfectBalance(effects, player)
    -- In Opo-opo form it gives Opo-opo's Fury.
    if isPlayerCaster(player) and mnkInOpoOpo(player) then
        effects:gauge_set("opo_fury", 1)
    end
    effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, POTENCY))
    mnkShift(effects, player, STATUS_RAPTOR_FORM)

    return effects
end
