-- Sage: its damage, 1-100 (its healing is the party bots' own, `party_bot.rs`, which uses its GCD first).
-- Priorities from Boss Mod's standard Sage rotation (awgil/ffxiv_bossmod, BSD-3), The Balance's Sage guide and the
-- official job guide (7.x): Eukrasia then Eukrasian Dosis (II, III; Eukrasian Dyskrasia on three or more) kept on the
-- target, Dosis (to Dosis III) otherwise, Phlegma's charges in reach, Dyskrasia on three or more round it; Psyche on
-- cooldown; the Addersgall spent on Druochole or Ixochole before it overflows, Rhizomata when it's low. Retail's Sages at
-- 80-89 (three players, 1175 s) cast Dosis II 222 times, Eukrasia 77, Eukrasian Dosis II 38, Druochole 29, Eukrasian
-- Prognosis 26, Phlegma II 22, Ixochole 17, Toxikon 13.

local DOSIS, DOSIS_II, DOSIS_III = 24283, 24306, 24312
local EUKRASIA, E_DOSIS, E_DOSIS_II, E_DOSIS_III, E_DYSKRASIA = 24290, 24293, 24308, 24314, 37032
local DYSKRASIA, DYSKRASIA_II = 24297, 24315
local PHLEGMA, PHLEGMA_II, PHLEGMA_III, TOXIKON, TOXIKON_II, PSYCHE = 24289, 24307, 24313, 24304, 24316, 37033
local DRUOCHOLE, IXOCHOLE, RHIZOMATA = 24296, 24299, 24309

local S_E_DOSIS, S_E_DOSIS_II, S_E_DOSIS_III, S_E_DYSKRASIA = 2614, 2615, 2616, 3897

local function gcd(b)
    local eukrasian = b:best(E_DOSIS_III, E_DOSIS_II, E_DOSIS)
    local left = math.max(b:dot(S_E_DOSIS), b:dot(S_E_DOSIS_II), b:dot(S_E_DOSIS_III), b:dot(S_E_DYSKRASIA))
    if b:gauge_of("eukrasia") > 0 and eukrasian then
        if b.near_self >= 3 and b:has(E_DYSKRASIA) then return E_DYSKRASIA, "self" end
        return eukrasian
    end
    if eukrasian and left < 3 and b.target_hp > 10 and b:has(EUKRASIA) then return EUKRASIA, "self" end
    local phlegma = b:best(PHLEGMA_III, PHLEGMA_II, PHLEGMA)
    if phlegma and b:ready(phlegma) and b.distance <= 6 then return phlegma end
    if b:gauge_of("addersting") > 0 then
        local toxikon = b:best(TOXIKON_II, TOXIKON)
        if toxikon then return toxikon end
    end
    if b.near_self >= 3 then
        local dyskrasia = b:best(DYSKRASIA_II, DYSKRASIA)
        if dyskrasia and b:affords(dyskrasia) then return dyskrasia, "self" end
    end
    local dosis = b:best(DOSIS_III, DOSIS_II, DOSIS)
    if dosis and b:affords(dosis) then return dosis end
    return nil
end

local function ogcd(b)
    if b:ready(PSYCHE) then return PSYCHE end
    local gall = b:gauge_of("addersgall")
    if gall >= 3 then
        if b:ready(IXOCHOLE) then return IXOCHOLE, "self" end
        if b:ready(DRUOCHOLE) then return DRUOCHOLE, "self" end
    end
    if gall <= 1 and b:ready(RHIZOMATA) then return RHIZOMATA, "self" end
    return nil
end

return {
    classjobs = { 40 },
    choose = function(b)
        if b:gcd_ready() then
            local action, on = gcd(b)
            if action then return action, on end
        end
        -- Abilities between GCD actions (or with the GCD free when there's nothing to cast on it.)
        if b:can_weave() or b:gcd_ready() then return ogcd(b) end
    end,
}
