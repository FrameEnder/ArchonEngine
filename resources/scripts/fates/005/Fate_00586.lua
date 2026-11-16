-- FATE 586 (Outer La Noscea): one enemy to defeat.
--
-- From Bob's Chronofoil capture 651981e7 (2026-11-16), the whole FATE with Bob alone on it: its state 1 then 4, and
-- 0.3 s later the SpawnNpc of its one enemy (BNpcBase 2153, level 44, 20240 HP, layout 4306103; drop-in layer
-- FATE_00586, timeline Fate586Boss_2153). A FateUpdate every 1.33 s from then on with the share of its health taken,
-- in whole percent; when it fell (306 s after it came, 246 s after Bob's first hit) the state went to 5, 7, 10 and 0.
-- `fate:spawn_boss` does all of that (server/fate.rs.)
BNPC_BOSS = 4306103

function onSetup(fate)
    fate:spawn_boss(BNPC_BOSS)
end
