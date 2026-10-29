-- A Grand Company's supply and provisioning missions (CustomTalk 720971, ComDefGCSupplyDuty_00075), opened nested from
-- the personnel officer's menu (ComDefGrandCompanyOfficer_00073 plays its scene 0, flags 0x2000.)
--
-- Bob's Chronofoil captures 9c4e397d and 0845b9a2 (2026-10-29): the client builds the mission list itself (nothing
-- came from the server while it was open) and closing it returns -1, after which retail finished this event and
-- played the officer's menu again. Handing items in (a mission turned in, its seals and EXP) isn't recorded or done.

function onReturn(scene, results, player)
    player:finish_event()
end
