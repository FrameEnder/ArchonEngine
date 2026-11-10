-- Ninjutsu (Lv. 30): casts what the mudras made before it make (Global.lua's ninNinjutsuOf and ninCastNinjutsu: Fuma Shuriken, Katon, Raiton, Hyoton, Huton, Doton, Suiton, Goka Mekkyaku, Hyosho Ranryu, or Rabbit Medium for a mudra repeated).
-- Potencies: the official job guide (7.x), at level 100.

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    local which = ninNinjutsuOf(player)
    if which == nil then
        return effects
    end
    ninCastNinjutsu(effects, player, which)

    return effects
end
