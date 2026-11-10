-- Samurai: its rotation, 1-100.
-- Priorities from Boss Mod's standard Samurai rotation (awgil/ffxiv_bossmod, BSD-3), The Balance's Samurai guide and
-- the official job guide (7.x): the three combos from Hakaze / Gyofu for the Sen (Yukikaze for Setsu, Jinpu - Gekko for
-- Getsu and Fugetsu, Shifu - Kasha for Ka; Fuga / Fuko - Mangetsu / Oka on three or more), Meikyo Shisui's three
-- finishers for the Sen missing; Midare Setsugekka at three Sen (Tenka Goken at two on three or more, Higanbana at one
-- when its damage over time has run out), Tsubame-gaeshi's Kaeshi after it, Ogi Namikiri and Kaeshi: Namikiri,
-- the Tendo ones from 100; the Kenki on Senei / Guren, Zanshin and Shinten (Kyuten on three or more), Ikishoten and Shoha
-- on cooldown. Retail's Samurai at 50-59 (three players, 1379 s) used Hakaze 152, Gekko 74, Kasha 61, Yukikaze 60, Jinpu
-- 57, Midare Setsugekka 56, Shifu 41, Meikyo Shisui 24, Higanbana 14; at 80-89 (two, 818 s) Hakaze 85, Shinten 67,
-- Gekko 44, Kasha 42, Yukikaze 39, Midare 34, Kaeshi: Setsugekka 34, Shoha 15, Meikyo 15, Senei 7, Ikishoten 5.

local HAKAZE, GYOFU, JINPU, SHIFU, GEKKO, KASHA, YUKIKAZE, ENPI = 7477, 36963, 7478, 7479, 7481, 7482, 7480, 7486
local FUGA, FUKO, MANGETSU, OKA = 7483, 25780, 7484, 7485
local HIGANBANA, TENKA_GOKEN, MIDARE, TENDO_GOKEN, TENDO_SETSUGEKKA = 7489, 7488, 7487, 36965, 36966
local KAESHI_GOKEN, KAESHI_SETSUGEKKA, TENDO_KAESHI_GOKEN, TENDO_KAESHI_SETSUGEKKA = 16485, 16486, 36967, 36968
local OGI_NAMIKIRI, KAESHI_NAMIKIRI = 25781, 25782
local MEIKYO, IKISHOTEN, SHINTEN, KYUTEN, SENEI, GUREN, ZANSHIN, SHOHA = 7499, 16482, 7490, 7491, 16481, 7496, 36964, 16487

local S_FUGETSU, S_FUKA, S_MEIKYO, S_HIGANBANA, S_TSUBAME, S_OGI_READY, S_ZANSHIN_READY, S_TENDO =
    1298, 1299, 1233, 1228, 4216, 2959, 3855, 3856
local SETSU, GETSU, KA = 1, 2, 4

local function sen_count(sen)
    return (sen & 1) + ((sen >> 1) & 1) + ((sen >> 2) & 1)
end

local function gcd(b)
    if not b:in_melee() then
        if b:has(ENPI) then return ENPI end
        return nil
    end
    local sen = b:gauge_of("sen")
    local kaeshi = b:gauge_of("kaeshi")
    -- Kaeshi: Namikiri, and Tsubame-gaeshi's repeat of the last Iaijutsu.
    if kaeshi == 4 and b:has(KAESHI_NAMIKIRI) then return KAESHI_NAMIKIRI end
    if b:has_status(S_TSUBAME) then
        if kaeshi == 3 then return b.mem.tendo and TENDO_KAESHI_SETSUGEKKA or KAESHI_SETSUGEKKA end
        if kaeshi == 2 then return b.mem.tendo and TENDO_KAESHI_GOKEN or KAESHI_GOKEN end
    end
    if b:has_status(S_OGI_READY) and b:has(OGI_NAMIKIRI) then return OGI_NAMIKIRI end
    -- Iaijutsu.
    local count = sen_count(sen)
    local tendo = b:has_status(S_TENDO)
    if count == 3 and b:has(MIDARE) then
        b.mem.tendo = tendo
        return tendo and TENDO_SETSUGEKKA or MIDARE
    end
    if count == 2 and b:aoe(3) and b:has(TENKA_GOKEN) then
        b.mem.tendo = tendo
        return tendo and TENDO_GOKEN or TENKA_GOKEN
    end
    if count == 1 and b:has(HIGANBANA) and b:dot(S_HIGANBANA) < 5 and b.target_hp > 30 and not b:aoe(3) then
        return HIGANBANA
    end
    -- Meikyo Shisui: the finishers for the Sen missing.
    if b:has_status(S_MEIKYO) then
        if b:aoe(3) and b:has(MANGETSU) then
            return (sen & GETSU == 0 or b:status(S_FUGETSU) < b:status(S_FUKA)) and MANGETSU or OKA
        end
        if sen & GETSU == 0 and b:has(GEKKO) then return GEKKO end
        if sen & KA == 0 and b:has(KASHA) then return KASHA end
        if sen & SETSU == 0 and b:has(YUKIKAZE) then return YUKIKAZE end
        return GEKKO
    end
    if b:aoe(3) and b:has(FUGA) then
        if (b.combo == FUGA or b.combo == FUKO) and b:has(MANGETSU) then
            if b:has(OKA) and (b:status(S_FUKA) < b:status(S_FUGETSU) or sen & KA == 0) then return OKA end
            return MANGETSU
        end
        return b:best(FUKO, FUGA)
    end
    if b.combo == JINPU and b:has(GEKKO) then return GEKKO end
    if b.combo == SHIFU and b:has(KASHA) then return KASHA end
    if b.combo == HAKAZE or b.combo == GYOFU then
        if b:has(YUKIKAZE) and sen & SETSU == 0 and b:status(S_FUGETSU) > 6 and (b:status(S_FUKA) > 6 or not b:has(SHIFU)) then
            return YUKIKAZE
        end
        if b:has(SHIFU) and (b:status(S_FUKA) < b:status(S_FUGETSU) or (sen & KA == 0 and sen & GETSU ~= 0)) then
            return SHIFU
        end
        if b:has(JINPU) then return JINPU end
    end
    return b:best(GYOFU, HAKAZE)
end

local function ogcd(b)
    if not b:in_melee() then return nil end
    local kenki = b:gauge_of("kenki")
    if b:ready(IKISHOTEN) and kenki <= 50 then return IKISHOTEN, "self" end
    if b:ready(MEIKYO) and not b:has_status(S_MEIKYO) and sen_count(b:gauge_of("sen")) == 0
        and (b.combo == 0 or b.combo == GEKKO or b.combo == KASHA or b.combo == YUKIKAZE or b.combo == MIDARE) then
        return MEIKYO, "self"
    end
    if b:has_status(S_ZANSHIN_READY) and kenki >= 50 and b:ready(ZANSHIN) then return ZANSHIN end
    local senei = b:aoe(3) and b:best(GUREN) or b:best(SENEI, GUREN)
    if senei and kenki >= 25 and b:ready(senei) then return senei end
    if b:gauge_of("meditation") >= 3 and b:ready(SHOHA) then return SHOHA end
    if kenki >= 65 or (kenki >= 25 and (not b:has(SENEI) or b:cd(SENEI) > 15)) then
        local spend = (b:aoe(3) and b:has(KYUTEN)) and KYUTEN or SHINTEN
        if b:ready(spend) then return spend, spend == KYUTEN and "self" or nil end
    end
    return nil
end

return {
    classjobs = { 34 },
    choose = function(b)
        if b:gcd_ready() then
            local action, on = gcd(b)
            if action then return action, on end
        end
        -- Abilities between GCD actions (or with the GCD free when there's nothing to cast on it.)
        if b:can_weave() or b:gcd_ready() then return ogcd(b) end
    end,
}
