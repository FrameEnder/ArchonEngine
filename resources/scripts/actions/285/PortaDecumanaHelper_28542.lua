-- The Porta Decumana's helpers' 28542: no cast, on the whole party, in the second fight's last part.
-- Retail's is an AoeEffect8 aimed at the helper itself naming every member (the player and the three Duty Support
-- members), each with the action's ExecuteCombo (27, 128) and nothing else (Connor's Chronofoil capture af05916c of
-- 2026-11-09, Bob's 9ba7d981 and 6320ae56: all of them.)

function doAction(player, in_combo)
    local effects = EffectsBuilder()
    effects:execute_combo(0)
    effects:on_party()
    return effects
end
