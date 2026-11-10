-- Pictomancer: its rotation, 1-100.
-- Priorities from Boss Mod's standard Pictomancer rotation (awgil/ffxiv_bossmod, BSD-3), The Balance's Pictomancer guide
-- and the official job guide (7.x): Fire - Aero - Water in Red, Green and Blue (their IIs on three or more) for the
-- Palette and White Paint; Subtractive Palette at 50 Palette for Blizzard - Stone - Thunder in Cyan, Yellow and Magenta;
-- Comet in Black on Monochrome Tones, Holy in White before the Paint overflows; the motifs painted when their muse is
-- nearly back (creature, hammer, starry sky, as the canvas shows), the muses on cooldown (Pom, Winged, Clawed, Fanged),
-- Mog of the Ages and Retribution of the Madeen on their portraits, Striking Muse's three hammers, Starry Muse with
-- Rainbow Drip and Star Prism after it. Retail's Pictomancers at 50-59 (two players, 1044 s) used Fire in Red 85, Aero
-- in Green 81, Water in Blue 78, Hammer Stamp 60, Striking Muse 22, Hammer Motif 20, Pom Muse 18, Winged Muse 17, Mog
-- of the Ages 17, Wing Motif 17, Pom Motif 16; at 80-89 (three, 1012 s) Hammer Stamp 48, Fire 36, Aero 34, Holy in
-- White 31, Water 30, Subtractive Palette 20, Thunder 20, Stone 19, Blizzard 18.

local FIRE, AERO, WATER, FIRE_II, AERO_II, WATER_II = 34650, 34651, 34652, 34656, 34657, 34658
local BLIZZARD, STONE, THUNDER, BLIZZARD_II, STONE_II, THUNDER_II = 34653, 34654, 34655, 34659, 34660, 34661
local HOLY, COMET, RAINBOW_DRIP, STAR_PRISM, SUBTRACTIVE = 34662, 34663, 34688, 34681, 34683
local POM_MOTIF, WING_MOTIF, CLAW_MOTIF, MAW_MOTIF, HAMMER_MOTIF, STARRY_SKY_MOTIF = 34664, 34665, 34666, 34667, 34668, 34669
local POM_MUSE, WINGED_MUSE, CLAWED_MUSE, FANGED_MUSE = 34670, 34671, 34672, 34673
local MOG, MADEEN, STRIKING_MUSE, STARRY_MUSE = 34676, 34677, 34674, 34675
local HAMMER_STAMP, HAMMER_BRUSH, POLISHING_HAMMER = 34678, 34679, 34680

local S_AETHERHUES, S_AETHERHUES_II, S_SUBTRACTIVE, S_MONOCHROME, S_HAMMER_TIME = 3675, 3676, 3674, 3691, 3680
local S_SPECTRUM, S_RAINBOW_BRIGHT, S_STARSTRUCK = 3690, 3679, 3681

-- The creature motif due next (Pom, Wing; Claw and Maw from 96), by what's depicted.
local function next_creature(b)
    local creature = b:gauge_of("creature")
    if b:has(CLAW_MOTIF) then
        if creature & 1 == 0 then return POM_MOTIF end
        if creature & 2 == 0 then return WING_MOTIF end
        if creature & 4 == 0 then return CLAW_MOTIF end
        return MAW_MOTIF
    end
    return (creature & 1 == 0) and POM_MOTIF or WING_MOTIF
end

local MUSE_OF = { [1] = POM_MUSE, [2] = WINGED_MUSE, [4] = CLAWED_MUSE, [8] = FANGED_MUSE }

local function gcd(b)
    local canvas = b:gauge_of("canvas")
    local area = b:aoe(3)
    if b:has_status(S_STARSTRUCK) and b:has(STAR_PRISM) then return STAR_PRISM end
    if b:has_status(S_RAINBOW_BRIGHT) and b:has(RAINBOW_DRIP) then return RAINBOW_DRIP end
    if b:has_status(S_HAMMER_TIME) then
        if b.combo == HAMMER_BRUSH and b:has(POLISHING_HAMMER) then return POLISHING_HAMMER end
        if b.combo == HAMMER_STAMP and b:has(HAMMER_BRUSH) then return HAMMER_BRUSH end
        return HAMMER_STAMP
    end
    if b:has_status(S_MONOCHROME) and b:gauge_of("white_paint") > 0 and b:has(COMET) then return COMET end
    if b:gauge_of("white_paint") >= 5 and b:has(HOLY) then return HOLY end
    -- Motifs painted ahead of their muses.
    if canvas & 15 == 0 and b:has(POM_MOTIF) and b:cd(POM_MUSE) < 10 then return next_creature(b), "self" end
    if canvas & 16 == 0 and b:has(HAMMER_MOTIF) and b:cd(STRIKING_MUSE) < 10 and not b:has_status(S_HAMMER_TIME) then
        return HAMMER_MOTIF, "self"
    end
    if canvas & 32 == 0 and b:has(STARRY_SKY_MOTIF) and b:cd(STARRY_MUSE) < 15 then return STARRY_SKY_MOTIF, "self" end
    if b:has_status(S_SUBTRACTIVE) then
        if b:has_status(S_AETHERHUES_II) then return area and THUNDER_II or THUNDER end
        if b:has_status(S_AETHERHUES) then return area and STONE_II or STONE end
        return area and BLIZZARD_II or BLIZZARD
    end
    if b:has_status(S_AETHERHUES_II) and b:has(WATER) then return (area and b:has(WATER_II)) and WATER_II or WATER end
    if b:has_status(S_AETHERHUES) and b:has(AERO) then return (area and b:has(AERO_II)) and AERO_II or AERO end
    return (area and b:has(FIRE_II)) and FIRE_II or FIRE
end

local function ogcd(b)
    local canvas, creature = b:gauge_of("canvas"), b:gauge_of("creature")
    if canvas & 32 ~= 0 and b:ready(STARRY_MUSE) then return STARRY_MUSE, "self" end
    if creature & 32 ~= 0 and b:ready(MADEEN) then return MADEEN end
    if creature & 16 ~= 0 and b:ready(MOG) then return MOG end
    for bit, muse in pairs(MUSE_OF) do
        if canvas & bit ~= 0 and b:ready(muse) then return muse end
    end
    if canvas & 16 ~= 0 and b:ready(STRIKING_MUSE) and not b:has_status(S_HAMMER_TIME) then return STRIKING_MUSE, "self" end
    if b:ready(SUBTRACTIVE) and not b:has_status(S_SUBTRACTIVE)
        and (b:gauge_of("palette") >= 50 or b:has_status(S_SPECTRUM)) then
        return SUBTRACTIVE, "self"
    end
    return nil
end

return {
    classjobs = { 42 },
    choose = function(b)
        if b:gcd_ready() then
            local action, on = gcd(b)
            if action then return action, on end
        end
        -- Abilities between GCD actions (or with the GCD free when there's nothing to cast on it.)
        if b:can_weave() or b:gcd_ready() then return ogcd(b) end
    end,
}
