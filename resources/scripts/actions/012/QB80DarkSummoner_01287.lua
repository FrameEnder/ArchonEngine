-- Light Succor, the dark summoner's in Austerities of Earth (quest battle 80, BNpcBase 2206): a 3 s cast, 10 yalms round
-- itself, that healed it and its Ifrit-Egi for 359-371 (Bob's Kawari Recorder run of 2026-11-12, twice; its timeline's
-- recorded amount.) Here it heals the caster only: an NPC's area heal on its own side isn't played.
local AMOUNT = 365

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:heal_self(AMOUNT)

    return effects
end
