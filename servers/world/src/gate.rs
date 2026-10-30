//! The GATEs (Gold Saucer Active Time Events): when they are, which one each is, and what the client is told of them.
//! Everything here follows from the time alone, so every connection works out the same GATE without asking anyone.
//! The events are in `events/gold_saucer.rs` (the entrances, ContentEntry), the client's side of the director in
//! `zone_connection/gate.rs`; docs/src/fork/gold-saucer.md tells the whole story.
//!
//! Retail, from Bob's Chronofoil captures deaf7e28 and 2c98f9e8 (2026-10-29, eight GATEs) and his archive (dd09ac61,
//! ea704111 and 61a9a4f3, five more):
//! * A GATE every 20 minutes, a second past the hour, :20 and :40. Its director is GoldSaucer (0x800A) with an ID
//!   retail counts up by one a GATE (680 to 685 over 17:00-18:40); Kawari's comes from the slot.
//! * As it begins, everyone in the Manderville Gold Saucer (territory 144) gets NpcYell 2450 `[location, type]`,
//!   ActorControlSelf 100 (InitDirector) with the director and its content ID (`GateKind::content_id`) and 109 `[11, 3]`
//!   (the phase: 3 registration, 4 starting, 5 under way, 1 over.) Coming into the zone while it lasts gives the
//!   InitDirector alone.
//! * The client then asks (client trigger 808 `[1]`, `[0]`), answered with 109 `[9, the GFATE row, its place among its
//!   type's rows]`, `[6, its entrance (the NPC's layout ID), when registration ends]`, for a player who has joined `[8, 1, 0]`
//!   and `[5, 1, the start, the registration window]`, and `[11, phase]`.
//! * An instanced GATE (Air Force One, Leap of Faith) is over once registration ends: 109 `[11, 1]` and
//!   ActorControlSelf 101 (TerminateDirector.)

use kawari::common::{HandlerId, HandlerType};

use crate::{GameData, gamedata::GateRow};

/// A GATE every 20 minutes.
pub const GATE_INTERVAL: u64 = 20 * 60;
/// Retail's begin a second past the twenty minutes (NpcYell 2450 at 17:40:01, 18:00:01, 18:20:01, 18:40:01.)
pub const GATE_START_OFFSET: u64 = 1;
/// The territory the GATEs are announced in, the Manderville Gold Saucer.
pub const GATE_TERRITORY: u16 = 144;

/// GFateType rows.
pub const KIND_CLIFFHANGER: u8 = 1;
pub const KIND_ANY_WAY_THE_WIND_BLOWS: u8 = 5;
pub const KIND_LEAP_OF_FAITH: u8 = 6;
pub const KIND_AIR_FORCE_ONE: u8 = 7;
pub const KIND_THE_SLICE_IS_RIGHT: u8 = 8;

/// The phases retail's DirectorEvent `[11, phase]` gives.
pub const PHASE_OVER: u32 = 1;
pub const PHASE_REGISTRATION: u32 = 3;

/// The NpcYell announcing a GATE (`[location, type]`.)
pub const NPC_YELL_GATE_BEGINS: u32 = 2450;

/// What retail told of each type of GATE (Bob's Chronofoil captures; the archive's agree.)
#[derive(Debug, Clone, Copy)]
pub struct GateKind {
    pub kind: u8,
    /// How long registration lasts, in seconds: `[5, 1, start, this]` and `[6, entrance, start + this]`.
    pub registration: u64,
    /// InitDirector's content ID.
    pub content_id: u16,
    /// The first parameter of the entrance's scene 0 (`[this, 0, MGP]`), and the one after the player has played this
    /// GATE (Air Force One's 501, 3 minutes after the first: deaf7e28.) What they are isn't known; they don't change
    /// with the time left.
    pub entrance: u32,
    pub entrance_played: u32,
    /// Played in its own duty, entered through a ContentEntry event and the Duty Finder.
    pub instanced: bool,
    /// Its set piece in the square (an EObj, by layout ID) and the timeline states (ActorControl 413, see
    /// `GATE_SCREENS`) it's given as the GATE begins and as it's over.
    pub stage: Option<(u32, [u32; 2], [u32; 2])>,
}

/// The Gold Saucer's GATE screens (EObj 2005037, by layout ID): retail gave all of them ActorControl 413 `[4, 8]` as
/// a GATE began (they show it) and `[1, 2]` as its registration ended, from each screen (Bob's Chronofoil captures
/// deaf7e28 and 2c98f9e8, and the archive's dd09ac61, ea704111 and 61a9a4f3: whichever were near the player.) An EObj
/// spawned afterwards had the first number as its SpawnObject `args1`.
pub const GATE_SCREENS: [u32; 6] = [5485250, 5485251, 5485252, 5485253, 5668781, 5668782];
pub const SCREEN_ON: [u32; 2] = [4, 8];
pub const SCREEN_OFF: [u32; 2] = [1, 2];

/// The Round Square's set piece (EObj 2007457): Air Force One's (`[16, 32]` as it begins, `[64, 128]` as it's over) or
/// Leap of Faith's (`[1, 2]`, `[4, 8]`.)
const ROUND_SQUARE_STAGE: u32 = 7833432;

pub const GATE_KINDS: &[GateKind] = &[
    // 18:20 on 2026-10-29 (2c98f9e8), 17:20 on dd09ac61 and 20:20 on ea704111.
    GateKind {
        kind: KIND_ANY_WAY_THE_WIND_BLOWS,
        registration: 120,
        content_id: 6,
        entrance: 89,
        entrance_played: 89,
        instanced: false,
        // The Event Square's (EObj 2005129) went on through the game (`[256, 512]` and `[4, 8]` by turns.)
        stage: None,
    },
    // 18:00 (2c98f9e8) and 17:40 on dd09ac61.
    GateKind {
        kind: KIND_LEAP_OF_FAITH,
        registration: 480,
        content_id: 9,
        entrance: 493,
        entrance_played: 493,
        instanced: true,
        stage: Some((ROUND_SQUARE_STAGE, [1, 2], [4, 8])),
    },
    // 17:40 (deaf7e28), 18:40 (2c98f9e8), 20:00 on ea704111 and 17:40 on 61a9a4f3.
    GateKind {
        kind: KIND_AIR_FORCE_ONE,
        registration: 600,
        content_id: 10,
        entrance: 498,
        entrance_played: 501,
        instanced: true,
        stage: Some((ROUND_SQUARE_STAGE, [16, 32], [64, 128])),
    },
    // 17:20 (deaf7e28.)
    GateKind {
        kind: KIND_THE_SLICE_IS_RIGHT,
        registration: 120,
        content_id: 11,
        entrance: 503,
        entrance_played: 503,
        instanced: false,
        stage: None,
    },
];

/// The GATEs a slot may hold since patch 7.4, by the slot's minute (the public wikis' schedule): at :00 Air Force One
/// (the Cieldalaes), Cliffhanger or Leap of Faith; at :20 Any Way the Wind Blows, The Slice Is Right or Air Force One
/// (the Gold Saucer); at :40 The Slice Is Right, Leap of Faith or Air Force One (the Cieldalaes.) Only the ones Kawari
/// can play are here, as their GFATE rows: Air Force One's Cieldalaes courses are rows 96-98 (GFateRideShooting 4-6,
/// ContentFinderCondition 1081-1083), the Gold Saucer's 74-76 (GFateRideShooting 1-3, 640-642.) Retail played the
/// Cieldalaes ones at :00 and :40 (rows 96 and 98); the :20 one wasn't seen.
const ROTATION: [&[&[u32]]; 3] = [&[&[96, 97, 98]], &[&[74, 75, 76]], &[&[96, 97, 98]]];

/// Which twenty minutes `now` is in.
pub fn slot_at(now: u64) -> u64 {
    now.saturating_sub(GATE_START_OFFSET) / GATE_INTERVAL
}

/// When a slot's GATE begins.
pub fn slot_start(slot: u64) -> u64 {
    slot * GATE_INTERVAL + GATE_START_OFFSET
}

pub fn gate_kind(kind: u8) -> Option<&'static GateKind> {
    GATE_KINDS.iter().find(|x| x.kind == kind)
}

/// A slot's GFATE row (see `ROTATION`), the same for everyone: which GATE among its minute's, then which course.
pub fn slot_row(slot: u64) -> Option<u32> {
    let choices = ROTATION[(slot % 3) as usize];
    if choices.is_empty() {
        return None;
    }
    // Mixed, so consecutive hours don't walk through the courses in order.
    let mixed = slot.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32;
    let rows = choices[(mixed % choices.len() as u64) as usize];
    Some(rows[((mixed / choices.len() as u64) % rows.len() as u64) as usize])
}

/// The kind of GATE a GFATE row is, as ROTATION knows them without the sheet.
fn row_kind(row: u32) -> u8 {
    match row {
        74..=76 | 96..=98 => KIND_AIR_FORCE_ONE,
        _ => 0,
    }
}

/// When a slot's GATE begins, when its registration ends and when it's over, without the game data (for timers.)
pub fn slot_times(slot: u64) -> Option<(u64, u64, u64)> {
    let kind = gate_kind(row_kind(slot_row(slot)?))?;
    let start = slot_start(slot);
    let registration_end = start + kind.registration;
    // An instanced one is over in the Gold Saucer when registration ends. The in-zone ones would go on with their game
    // after it, but none of them are played yet.
    let end = registration_end;
    Some((start, registration_end, end))
}

/// What the Gold Saucer shows of its GATEs, as the server keeps it for the zone (`server/gate.rs`): which GATE (its
/// slot) is on, and its entrance.
#[derive(Debug, Default, Clone, Copy)]
pub struct GateShown {
    pub slot: u64,
    pub entrance: Option<kawari::common::ObjectId>,
}

/// A GATE that is (or will be) on.
#[derive(Debug, Clone, Copy)]
pub struct Gate {
    pub slot: u64,
    pub row: GateRow,
    pub kind: GateKind,
    /// The row's place among its type's rows, the row of its type's sheet (`GameData::gate_rows`.)
    pub place: u32,
    pub start: u64,
    pub registration_end: u64,
    pub end: u64,
}

impl Gate {
    /// The slot's GATE, if Kawari can play one then.
    pub fn in_slot(game_data: &mut GameData, slot: u64) -> Option<Gate> {
        let row = game_data.get_gate_row(slot_row(slot)?)?;
        let kind = *gate_kind(row.kind)?;
        let place = game_data
            .gate_rows(row.kind)
            .iter()
            .position(|x| *x == row.row)? as u32
            + 1;
        let (start, registration_end, end) = slot_times(slot)?;
        Some(Gate {
            slot,
            row,
            kind,
            place,
            start,
            registration_end,
            end,
        })
    }

    /// The GATE on now, from its start to its end.
    pub fn current(game_data: &mut GameData, now: u64) -> Option<Gate> {
        Self::in_slot(game_data, slot_at(now)).filter(|gate| gate.start <= now && now < gate.end)
    }

    /// The next GATE to begin after `now`.
    pub fn next(game_data: &mut GameData, now: u64) -> Option<Gate> {
        let first = slot_at(now) + 1;
        (first..first + 3).find_map(|slot| Self::in_slot(game_data, slot))
    }

    /// Its director: GoldSaucer, with an ID of its own (retail counts them up by one a GATE.)
    pub fn director(&self) -> HandlerId {
        HandlerId::new(HandlerType::GoldSaucer, (self.slot % 0xFFFE + 1) as u16)
    }

    pub fn registering(&self, now: u64) -> bool {
        self.start <= now && now < self.registration_end
    }

    pub fn phase(&self, now: u64) -> u32 {
        if self.registering(now) {
            PHASE_REGISTRATION
        } else {
            PHASE_OVER
        }
    }
}

/// The next moment something happens to a GATE after `now` (one begins, or registration ends), for the connections'
/// timers.
pub fn next_change(now: u64) -> u64 {
    let slot = slot_at(now);
    for slot in slot..slot + 4 {
        if let Some((start, registration_end, end)) = slot_times(slot) {
            for at in [start, registration_end, end] {
                if at > now {
                    return at;
                }
            }
        }
    }
    slot_start(slot + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_begin_past_the_twenty_minutes() {
        // 2026-10-29 18:40:01 UTC, when retail's Air Force One began (2c98f9e8.)
        let start = 1_793_299_201;
        assert_eq!(slot_start(slot_at(start)), start);
        assert_eq!(slot_at(start - 1), slot_at(start) - 1);
        assert_eq!(slot_at(start + GATE_INTERVAL - 1), slot_at(start));
    }

    #[test]
    fn every_slot_has_a_playable_gate() {
        for slot in 1_494_000..1_494_100 {
            let row = slot_row(slot).unwrap();
            match slot % 3 {
                1 => assert!((74..=76).contains(&row)),
                _ => assert!((96..=98).contains(&row)),
            }
            let (start, registration_end, end) = slot_times(slot).unwrap();
            assert_eq!(registration_end - start, 600);
            assert_eq!(end, registration_end);
        }
    }

    #[test]
    fn next_change_walks_the_timeline() {
        let start = 1_793_299_201;
        assert_eq!(next_change(start - 5), start);
        assert_eq!(next_change(start), start + 600);
        assert_eq!(next_change(start + 600), start + GATE_INTERVAL);
    }
}
