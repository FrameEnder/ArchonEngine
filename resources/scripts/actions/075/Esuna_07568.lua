-- Esuna (Lv. 10) - role action: removes one detrimental effect from the target (retail: the effect, or "no effect").

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:cleanse()

    return effects
end
