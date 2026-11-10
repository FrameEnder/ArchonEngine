-- Blizzard II (Lv. 12), the player one: the target and enemies near it.
-- Astral Fire and Umbral Ice: the job gauge's element (Global.lua's blm* functions).
-- (Action 146 is the older Blizzard II that NPCs use.)
-- Potencies: 7.x tooltip (max level); check the in-game tooltip at your level.
local POTENCY = 80
local EFFECT_THUNDERHEAD = 3870
local THUNDERHEAD_DURATION = 0.0 -- no time limit on retail (Bob's Chronofoil captures; the Status sheet's permanent)

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:damage(DAMAGE_TYPE_MAGIC, magicalDamage(player, blmIcePotency(player, classPotency(player, POTENCY, TRAITS_THM_MAIM_AND_MEND))))
    if blmElement(player) < 0 then
        effects:set_mp_cost(0)
    end
    blmIceStep(effects, player)

    return effects
end
