-- Dancer: its rotation, 1-100.
-- Priorities from Boss Mod's standard Dancer rotation (awgil/ffxiv_bossmod, BSD-3), The Balance's Dancer guide and the
-- official job guide (7.x): Standard Step (two steps, then Double Standard Finish) and Technical Step (four, then
-- Quadruple Technical Finish) on cooldown, dancing the steps the gauge shows; Tillana, Dance of the Dawn, Starfall
-- Dance, Finishing Move and Last Dance when they're ready; Saber Dance with the Esprit; the procs (Fountainfall /
-- Reverse Cascade, Bloodshower / Rising Windmill on three or more) before Cascade - Fountain (Windmill - Bladeshower);
-- Devilment and Flourish with Technical Finish, the Fan Dances with the Feathers. Retail's Dancers at 80-89 (three
-- players, 1187 s) used Cascade 83, Fountain 78, Fountainfall 57, Saber Dance 53, Reverse Cascade 52, Fan Dance 50, Fan
-- Dance III 41, Standard Step 26 and Double Standard Finish 26 (each after two steps), Flourish 19, Technical Step 11
-- and Quadruple Technical Finish 11 (and its 33218 straight after), Devilment 11.

local CASCADE, FOUNTAIN, REVERSE_CASCADE, FOUNTAINFALL = 15989, 15990, 15991, 15992
local WINDMILL, BLADESHOWER, RISING_WINDMILL, BLOODSHOWER = 15993, 15994, 15995, 15996
local STANDARD_STEP, TECHNICAL_STEP = 15997, 15998
local STEPS = { 15999, 16000, 16001, 16002 }
local STANDARD_FINISHES = { [0] = 16003, 16191, 16192 }
local TECHNICAL_FINISHES = { [0] = 16004, 16193, 16194, 16195, 16196 }
local SABER_DANCE, TILLANA, STARFALL_DANCE, LAST_DANCE, FINISHING_MOVE, DANCE_OF_THE_DAWN = 16005, 25790, 25792, 36983, 36984, 36985
local FAN_DANCE, FAN_DANCE_II, FAN_DANCE_III, FAN_DANCE_IV, DEVILMENT, FLOURISH = 16007, 16008, 16009, 25791, 16011, 16013

local S_SILKEN_SYMMETRY, S_SILKEN_FLOW, S_FLOURISHING_SYMMETRY, S_FLOURISHING_FLOW = 2693, 2694, 3017, 3018
local S_STANDARD_STEP, S_TECHNICAL_STEP, S_TECHNICAL_FINISH = 1818, 1819, 1822
local S_FLOURISHING_FINISH, S_FLOURISHING_STARFALL, S_DEVILMENT = 2698, 2700, 1825
local S_THREEFOLD, S_FOURFOLD, S_LAST_DANCE_READY, S_FINISHING_MOVE_READY, S_DAWN_READY = 1820, 2699, 3867, 3868, 3869

local function dance(b, status, count, finishes)
    local done = b:gauge_of("step_index")
    if done < count then
        local due = (b:gauge_of("dance_steps") >> (done * 3)) & 7
        if STEPS[due] then return STEPS[due], "self" end
    end
    return finishes[math.min(done, count)], "self"
end

local function gcd(b)
    if b:has_status(S_TECHNICAL_STEP) then return dance(b, S_TECHNICAL_STEP, 4, TECHNICAL_FINISHES) end
    if b:has_status(S_STANDARD_STEP) then return dance(b, S_STANDARD_STEP, 2, STANDARD_FINISHES) end
    local esprit = b:gauge_of("esprit")
    local technical = b:has_status(S_TECHNICAL_FINISH)
    if b:ready(TECHNICAL_STEP) then return TECHNICAL_STEP, "self" end
    if b:has_status(S_FLOURISHING_FINISH) and b:has(TILLANA) then return TILLANA, "self" end
    if b:has_status(S_DAWN_READY) and esprit >= 50 and b:has(DANCE_OF_THE_DAWN) then return DANCE_OF_THE_DAWN end
    if b:has_status(S_FLOURISHING_STARFALL) and b:has(STARFALL_DANCE) then return STARFALL_DANCE end
    if b:has_status(S_FINISHING_MOVE_READY) and b:ready(FINISHING_MOVE) then return FINISHING_MOVE, "self" end
    if b:ready(STANDARD_STEP) and not b:has_status(S_FINISHING_MOVE_READY) then return STANDARD_STEP, "self" end
    if b:has_status(S_LAST_DANCE_READY) and b:has(LAST_DANCE) then return LAST_DANCE end
    if b:has(SABER_DANCE) and (esprit >= 80 or (technical and esprit >= 50)) then return SABER_DANCE end
    local area = b.near_self >= 3
    local flow = b:has_status(S_SILKEN_FLOW) or b:has_status(S_FLOURISHING_FLOW)
    local symmetry = b:has_status(S_SILKEN_SYMMETRY) or b:has_status(S_FLOURISHING_SYMMETRY)
    if flow then
        local proc = (area and b:has(BLOODSHOWER)) and BLOODSHOWER or FOUNTAINFALL
        if b:has(proc) then return proc, area and "self" or nil end
    end
    if symmetry then
        local proc = (area and b:has(RISING_WINDMILL)) and RISING_WINDMILL or REVERSE_CASCADE
        if b:has(proc) then return proc, area and "self" or nil end
    end
    if area and b:has(WINDMILL) then
        if b.combo == WINDMILL and b:has(BLADESHOWER) then return BLADESHOWER, "self" end
        return WINDMILL, "self"
    end
    if b.combo == CASCADE and b:has(FOUNTAIN) then return FOUNTAIN end
    return CASCADE
end

local function ogcd(b)
    if b:has_status(S_STANDARD_STEP) or b:has_status(S_TECHNICAL_STEP) then return nil end
    local technical = b:has_status(S_TECHNICAL_FINISH) or not b:has(TECHNICAL_STEP)
    if b:ready(DEVILMENT) and technical then return DEVILMENT, "self" end
    if b:ready(FLOURISH) and not b:has_status(S_SILKEN_FLOW) and not b:has_status(S_SILKEN_SYMMETRY)
        and not b:has_status(S_THREEFOLD) then
        return FLOURISH, "self"
    end
    if b:has_status(S_FOURFOLD) and b:ready(FAN_DANCE_IV) then return FAN_DANCE_IV end
    if b:has_status(S_THREEFOLD) and b:ready(FAN_DANCE_III) then return FAN_DANCE_III end
    local feathers = b:gauge_of("feathers")
    if feathers >= 4 or (feathers >= 1 and (technical or b:has_status(S_DEVILMENT))) then
        local fan = (b.near_self >= 3 and b:has(FAN_DANCE_II)) and FAN_DANCE_II or FAN_DANCE
        if b:ready(fan) then return fan, fan == FAN_DANCE_II and "self" or nil end
    end
    return nil
end

return {
    classjobs = { 38 },
    choose = function(b)
        if b:gcd_ready() then
            local action, on = gcd(b)
            if action then return action, on end
        end
        -- Abilities between GCD actions (or with the GCD free when there's nothing to cast on it.)
        if b:can_weave() or b:gcd_ready() then return ogcd(b) end
    end,
}
