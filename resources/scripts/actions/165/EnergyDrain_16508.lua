-- Energy Drain (Lv. 10), Arcanist/Summoner.
-- Fills Aetherflow (two stacks, for Fester and Painflare); Further Ruin (60 s, for Ruin IV) from 62. The arcana and
-- attunement come from Aethercharge and the summons (Global.lua's smnSummon); pets' attacks aren't implemented.
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 200

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, classPotency(player, POTENCY, TRAITS_ACN_MAIM_AND_MEND)))

    effects:gain_aetherflow()
    if player.level >= 62 then
        effects:gain_effect_self(SMN_EFFECT_FURTHER_RUIN, 0, 60.0)
    end

    return effects
end
