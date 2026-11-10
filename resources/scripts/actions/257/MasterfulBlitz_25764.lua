-- Masterful Blitz (Lv. 60): the Beast Chakra technique the open chakra make: three of one Elixir Field (Burst from 92), three different Flint Strike (Rising Phoenix from 86), two different Celestial Revolution, and with both Nadi Tornado Kick (Phantom Rush from 90).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local count, kinds = mnkChakraKinds(player)
    if count < 3 then
        return effects
    end
    local nadi = player:gauge("nadi")
    if nadi == 3 then
        effects:aoe_falloff(35)
        effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, player.level >= 90 and 1500 or 1200))
        mnkBlitz(effects, player, 3)
    elseif kinds == 1 then
        effects:aoe_falloff(35)
        effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, player.level >= 92 and 900 or 800))
        mnkBlitz(effects, player, 1)
    elseif kinds == 3 then
        effects:aoe_falloff(35)
        effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, player.level >= 86 and 900 or 800))
        mnkBlitz(effects, player, 2)
    else
        effects:damage(DAMAGE_TYPE_BLUNT, physicalDamage(player, 600))
        if nadi & 1 ~= 0 then
            mnkBlitz(effects, player, 2)
        else
            mnkBlitz(effects, player, 1)
        end
    end

    return effects
end
