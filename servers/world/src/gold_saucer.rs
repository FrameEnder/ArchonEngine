//! The Gold Saucer's games of chance and skill, and what the character keeps of them: the Mini Cactpot (a scratch
//! card a day, three tickets), the Jumbo Cactpot (a weekly lottery, three tickets) and the arcade machines. See
//! `events/gold_saucer.rs` for their events and docs/src/fork/gold-saucer.md.
//!
//! The numbers are retail's, from Bob's Chronofoil capture deaf7e28 (2026-10-29: three Mini Cactpot tickets, three
//! Jumbo Cactpot tickets, 50 arcade games) unless a comment says they're from the public wikis.

use diesel::backend::Backend;
use diesel::deserialize::{self, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::serialize;
use diesel::sql_types::Text;
use diesel::sqlite::Sqlite;
use serde::{Deserialize, Serialize};

/// The Gold Saucer's day begins at 15:00 UTC; retail counts them from 2013-08-25 15:00 UTC (the Mini Cactpot's day
/// 4813 was 2026-10-29.)
const DAY_EPOCH: u64 = 1_377_442_800;
/// The Jumbo Cactpot's weeks begin at this time and every 7 days after it (retail's ActorControlSelf 912 at every zone
/// change, `SetWeeklyLotOffsetTime`: 2013-09-01 02:00 UTC, so the drawing is Sunday 02:00 UTC, Saturday evening in
/// North America); week 686 was the one of 2026-10-29.
pub const WEEKLY_LOT_OFFSET_TIME: u64 = 1_378_000_800;
const WEEK: u64 = 7 * 24 * 60 * 60;

/// The Gold Saucer's day now (see `DAY_EPOCH`.)
pub fn day(now: u64) -> u32 {
    (now.saturating_sub(DAY_EPOCH) / (24 * 60 * 60)) as u32
}

/// The Jumbo Cactpot's week now (see `WEEKLY_LOT_OFFSET_TIME`.)
pub fn week(now: u64) -> u32 {
    (now.saturating_sub(WEEKLY_LOT_OFFSET_TIME) / WEEK) as u32
}

/// What the character keeps of the Gold Saucer (`unlock.gold_saucer`, JSON.)
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = Text)]
#[serde(default)]
pub struct GoldSaucerState {
    /// The day the Mini Cactpot tickets below were bought on.
    pub mini_cactpot_day: u32,
    /// How many Mini Cactpot tickets were bought that day.
    pub mini_cactpot_bought: u8,
    /// Whether the Mini Cactpot broker was ever talked to (the first time, retail's explained the game.)
    pub mini_cactpot_played: bool,
    /// The week of the Jumbo Cactpot tickets below.
    pub jumbo_week: u32,
    /// That week's Jumbo Cactpot tickets' numbers (0-9999.)
    pub jumbo_tickets: Vec<u16>,
}

impl GoldSaucerState {
    /// Forgets the Mini Cactpot tickets of an earlier day.
    pub fn refresh_mini_cactpot(&mut self, now: u64) {
        if self.mini_cactpot_day != day(now) {
            self.mini_cactpot_day = day(now);
            self.mini_cactpot_bought = 0;
        }
    }

    /// The Mini Cactpot tickets left today.
    pub fn mini_cactpot_left(&self) -> u8 {
        MINI_CACTPOT_TICKETS.saturating_sub(self.mini_cactpot_bought)
    }

    /// The tickets of a week that's been drawn (the week and the numbers), taken out to be checked; this week's stay.
    pub fn take_drawn_jumbo_tickets(&mut self, now: u64) -> Option<(u32, Vec<u16>)> {
        if self.jumbo_tickets.is_empty() || self.jumbo_week >= week(now) {
            return None;
        }
        Some((self.jumbo_week, std::mem::take(&mut self.jumbo_tickets)))
    }

    /// This week's Jumbo Cactpot tickets (none if those kept are an earlier week's, still to be checked.)
    pub fn jumbo_tickets_now(&self, now: u64) -> &[u16] {
        if self.jumbo_week == week(now) {
            &self.jumbo_tickets
        } else {
            &[]
        }
    }

    /// The Gold Saucer flags the client is told at every zone change (ActorControlSelf 914, `SetGoldSaucerFlags`):
    /// retail set 8 once a Mini Cactpot ticket had been played that day and 16 once a Jumbo Cactpot ticket had been
    /// bought that week (0, then 8, then 24 in Bob's Chronofoil capture deaf7e28.) What the others are isn't known.
    pub fn flags(&self, now: u64) -> u32 {
        let mut flags = 0;
        if self.mini_cactpot_day == day(now) && self.mini_cactpot_bought > 0 {
            flags |= 8;
        }
        if !self.jumbo_tickets_now(now).is_empty() {
            flags |= 16;
        }
        flags
    }
}

impl serialize::ToSql<Text, Sqlite> for GoldSaucerState {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(serde_json::to_string(&self).unwrap());
        Ok(serialize::IsNull::No)
    }
}

impl deserialize::FromSql<Text, Sqlite> for GoldSaucerState {
    fn from_sql(mut bytes: <Sqlite as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        Ok(serde_json::from_str(bytes.read_text())
            .ok()
            .unwrap_or_default())
    }
}

// --- The Mini Cactpot ---

/// Tickets a day.
pub const MINI_CACTPOT_TICKETS: u8 = 3;
/// What a ticket costs in MGP.
pub const MINI_CACTPOT_COST: u32 = 10;
/// What a line pays, by its sum (6 to 24), as retail sent it with every ticket.
pub const MINI_CACTPOT_PAYOUTS: [u32; 19] = [
    10000, 36, 720, 360, 80, 252, 108, 72, 54, 180, 72, 180, 119, 36, 306, 1080, 144, 1800, 3600,
];
/// The cells a ticket's line covers (row * 3 + column), by the line the client sends: 0-2 the rows top to bottom, 3-5
/// the columns left to right (each of retail's three tickets agreed: line 1 summed row 1, 4 column 1, 5 column 2), 6
/// and 7 the diagonals from the top left and the top right (not played; the order MiniCactpotSolver uses, which the
/// rows and columns match. The client's own radio buttons are in another order, diagonal, columns, diagonal, rows.)
pub const MINI_CACTPOT_LINES: [[usize; 3]; 8] = [
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8],
    [0, 4, 8],
    [2, 4, 6],
];

/// A Mini Cactpot ticket being scratched: its numbers (1-9 once each, row by row), which are showing, and how many the
/// player has scratched (three, then a line.)
#[derive(Debug, Clone)]
pub struct MiniCactpotTicket {
    pub numbers: [u8; 9],
    pub shown: [bool; 9],
    pub scratched: u8,
}

impl MiniCactpotTicket {
    /// A new ticket with one cell already showing, as retail's were.
    pub fn new() -> Self {
        let mut numbers: [u8; 9] = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        fastrand::shuffle(&mut numbers);
        let mut shown = [false; 9];
        shown[fastrand::usize(..9)] = true;
        Self {
            numbers,
            shown,
            scratched: 0,
        }
    }

    /// The cell showing from the start.
    pub fn first_shown(&self) -> usize {
        self.shown.iter().position(|shown| *shown).unwrap_or(0)
    }

    /// Scratches a cell: its number, if it may be (not showing yet, fewer than three scratched.)
    pub fn scratch(&mut self, cell: usize) -> Option<u8> {
        if cell >= 9 || self.shown[cell] || self.scratched >= 3 {
            return None;
        }
        self.shown[cell] = true;
        self.scratched += 1;
        Some(self.numbers[cell])
    }

    /// A line's sum and what it pays.
    pub fn line(&self, line: usize) -> Option<(u32, u32)> {
        let cells = MINI_CACTPOT_LINES.get(line)?;
        let sum: u32 = cells.iter().map(|cell| self.numbers[*cell] as u32).sum();
        Some((sum, MINI_CACTPOT_PAYOUTS[(sum - 6) as usize]))
    }
}

// --- The Jumbo Cactpot ---

/// Tickets a week.
pub const JUMBO_CACTPOT_TICKETS: usize = 3;
/// What the first, second and third ticket of a week cost in MGP (retail's 100, 150 and 200.)
pub const JUMBO_CACTPOT_COSTS: [u32; JUMBO_CACTPOT_TICKETS] = [100, 150, 200];
/// What retail showed for a ticket not bought.
pub const JUMBO_CACTPOT_NO_TICKET: u32 = 10000;

/// The winning number of a week: the same for everyone on the server, drawn from the week alone (retail draws one for
/// a data centre; Kawari has no other players to share it with.)
pub fn jumbo_cactpot_number(week: u32) -> u16 {
    let mut x = (week as u64) ^ 0x9E37_79B9_7F4A_7C15;
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    (x % 10000) as u16
}

/// The prizes in MGP: all four digits, the last three, the last two, the last one, none (a consolation prize). Retail's
/// broker listed 1,008,000, 200,000, 52,500, 6,550 and 1,160 in Bob's Chronofoil capture deaf7e28; the base prizes since
/// patch 4.4 are 1,008,000, 200,000, 50,000, 5,000 and 1,000 (the patch notes; the rest is retail's participant bonus,
/// which grows with how many bought tickets. Kawari has no other players, so it pays the base prizes.)
pub const JUMBO_CACTPOT_PRIZES: [u32; 5] = [1_008_000, 200_000, 50_000, 5_000, 1_000];

/// A ticket's prize against the winning number: digits are matched from the last one.
pub fn jumbo_cactpot_prize(ticket: u16, winning: u16) -> u32 {
    let digits = [10000u16, 1000, 100, 10];
    let tier = digits
        .iter()
        .position(|modulus| ticket % modulus == winning % modulus)
        .unwrap_or(4);
    JUMBO_CACTPOT_PRIZES[tier]
}

/// The most MGP one can hold (the public wikis: 9,999,999.)
pub const MGP_CAP: u32 = 9_999_999;

// --- The arcade machines ---

/// How a swing or a throw went: 0 poor, 1 good, 2 great, 3 excellent (GoldSaucerArcadeMachine's four columns.)
pub type ArcadeTier = u8;

/// What one play costs in MGP (retail: 1 at every machine.)
pub const ARCADE_COST: u32 = 1;

/// What each machine's tiers pay, by its GoldSaucerArcadeMachine row. Retail paid The Moogle's Paw 10 and 25 and
/// Cuff-a-Cur 10, 15 and 25 (the sheet's own numbers, which the result screen shows, say 20 for the excellent ones); the
/// others are as each machine's payout text has them: Monster Toss 20/30/50, Crystal Tower Striker 10/15/25.
pub fn arcade_payout(machine: u32, tier: ArcadeTier) -> Option<u32> {
    let payouts: [u32; 4] = match machine {
        1 => [0, 20, 30, 50],
        2 => [0, 10, 25, 25],
        3 | 4 => [0, 10, 15, 25],
        _ => return None,
    };
    payouts.get(tier as usize).copied()
}

/// The source ActorControlSelf 910 names for a machine's payouts (retail's: 8 Cuff-a-Cur, 10 The Moogle's Paw, 13
/// The Finer Miner, 14 Out on a Limb; Monster Toss's and Crystal Tower Striker's weren't seen.)
pub fn arcade_source(machine: u32) -> u32 {
    match machine {
        1 => 11,
        2 => 10,
        3 => 9,
        4 => 8,
        5 => 13,
        6 => 14,
        _ => 0,
    }
}

/// How many rounds a staged game can go to (the first and five double downs; the public wikis.)
pub const STAGED_ROUNDS: usize = 6;

/// What winning a staged game's rounds pays in all, by its choice (0 Cactuar, 1 Morbol, 2 Titan: the first swing, at
/// the gauge of the three) and the rounds won. Retail paid Out on a Limb 50 (Cactuar), 60 (Morbol) and 85 (Titan) for a
/// round and offered 100, 125 and 175 for a second, then 150; The Finer Miner 35, then 85, then 125 (Bob's Chronofoil
/// capture deaf7e28.) The rest of the table is the public wikis' (Miqobot's forum and HubPages agree), which retail's
/// numbers match.
pub fn staged_payout(machine: u32, choice: u32, round: usize) -> u32 {
    const OUT_ON_A_LIMB: [[u32; STAGED_ROUNDS]; 3] = [
        [50, 100, 150, 400, 500, 600],
        [60, 125, 185, 500, 625, 750],
        [85, 175, 260, 700, 875, 1050],
    ];
    const FINER_MINER: [[u32; STAGED_ROUNDS]; 3] = [
        [35, 85, 125, 175, 210, 250],
        [50, 150, 360, 735, 1425, 2625],
        [60, 200, 600, 1600, 4000, 9600],
    ];
    let table = if machine == 6 {
        &OUT_ON_A_LIMB
    } else {
        &FINER_MINER
    };
    table[(choice as usize).min(2)][round.min(STAGED_ROUNDS - 1)]
}

/// A staged game's swings and gauge speed (ms) by its choice. Out on a Limb: 10 swings, the gauge at 750, 375 or 250
/// ms; The Finer Miner: 6, 5 or 4 swings, 750 ms (retail: Out on a Limb's three, The Finer Miner's Cactuar; the Finer
/// Miner's Morbol and Titan swings are the public wikis'.)
pub fn staged_setup(machine: u32, choice: u32) -> (u8, u32) {
    match (machine, choice) {
        (6, 1) => (10, 375),
        (6, 2) => (10, 250),
        (6, _) => (10, 750),
        (_, 1) => (5, 750),
        (_, 2) => (4, 750),
        _ => (6, 750),
    }
}

/// A staged game (The Finer Miner, row 5; Out on a Limb, row 6): a hidden spot on a 0-99 gauge to find in up to
/// `swings` tries, each swing's distance from it taking a tier's damage off the 10 the rock or tree has. Winning a
/// round may be doubled down on, up to `STAGED_ROUNDS` rounds.
#[derive(Debug, Clone)]
pub struct StagedGame {
    pub machine: u32,
    pub choice: u32,
    pub round: usize,
    pub target: u8,
    pub hp: u8,
    pub swings: u8,
    pub gauge: u32,
    pub won: bool,
    pub over: bool,
}

impl StagedGame {
    pub fn new(machine: u32, choice: u32) -> Self {
        let (swings, gauge) = staged_setup(machine, choice);
        Self {
            machine,
            choice,
            round: 0,
            target: fastrand::u8(..100),
            hp: 10,
            swings,
            gauge,
            won: false,
            over: false,
        }
    }

    /// What winning this round pays in all.
    pub fn stake(&self) -> u32 {
        staged_payout(self.machine, self.choice, self.round)
    }

    /// What winning the next round would.
    pub fn next(&self) -> u32 {
        staged_payout(self.machine, self.choice, self.round + 1)
    }

    /// Whether a won round may be doubled down on.
    pub fn can_double_down(&self) -> bool {
        self.won && self.round + 1 < STAGED_ROUNDS
    }

    /// The next round after doubling down: a new spot, the swings back, the rock or tree whole again.
    pub fn double_down(&mut self) {
        let (swings, _) = staged_setup(self.machine, self.choice);
        self.round += 1;
        self.target = fastrand::u8(..100);
        self.hp = 10;
        self.swings = swings;
        self.won = false;
        self.over = false;
    }

    /// A swing at `position` (0-99): its tier by the distance from the spot (the machine's thresholds, from its
    /// GoldSaucerArcadeMachine row: `ranges` for good, great, excellent), the damage it does (`damage`, poor to
    /// excellent), and whether that ends the round.
    pub fn swing(&mut self, position: u8, ranges: [(u8, u8); 3], damage: [u8; 4]) -> ArcadeTier {
        let distance = position.abs_diff(self.target);
        let tier = ranges
            .iter()
            .rposition(|(low, high)| (*low..=*high).contains(&distance))
            .map(|index| index as u8 + 1)
            .unwrap_or(0);
        self.hp = self.hp.saturating_sub(damage[tier as usize]);
        self.swings = self.swings.saturating_sub(1);
        if self.hp == 0 {
            self.won = true;
            self.over = true;
        } else if self.swings == 0 {
            self.over = true;
        }
        tier
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn days_and_weeks() {
        // Bob's Chronofoil capture deaf7e28: day 4813 and week 686 at 2026-10-29 17:10 UTC.
        assert_eq!(day(1_793_292_600), 4813);
        assert_eq!(week(1_793_292_600), 686);
    }

    #[test]
    fn mini_cactpot_lines() {
        // Retail's first ticket: 1 9 3 / 7 2 8 / 5 4 6; line 5 summed 17 and paid 180.
        let ticket = MiniCactpotTicket {
            numbers: [1, 9, 3, 7, 2, 8, 5, 4, 6],
            shown: [false; 9],
            scratched: 0,
        };
        assert_eq!(ticket.line(5), Some((17, 180)));
        assert_eq!(ticket.line(1), Some((17, 180)));
        assert_eq!(ticket.line(6), Some((9, 360)));
    }

    #[test]
    fn jumbo_prizes() {
        assert_eq!(jumbo_cactpot_prize(1234, 1234), 1_008_000);
        assert_eq!(jumbo_cactpot_prize(9234, 1234), 200_000);
        assert_eq!(jumbo_cactpot_prize(9934, 1234), 50_000);
        assert_eq!(jumbo_cactpot_prize(9994, 1234), 5_000);
        assert_eq!(jumbo_cactpot_prize(9999, 1234), 1_000);
    }

    #[test]
    fn staged_swings() {
        // Retail's Out on a Limb, target 55: 43 good (-1), 30 poor, 59 great (-4), 63 great (-4), 80 poor, 73 good.
        let mut game = StagedGame::new(6, 0);
        game.target = 55;
        let ranges = [(11, 20), (3, 10), (0, 2)];
        let damage = [0, 1, 4, 10];
        let tiers: Vec<u8> = [43, 30, 59, 63, 80, 73]
            .iter()
            .map(|position| game.swing(*position, ranges, damage))
            .collect();
        assert_eq!(tiers, [1, 0, 2, 2, 0, 1]);
        assert_eq!(game.hp, 0);
        assert!(game.won);
        assert_eq!((game.stake(), game.next()), (50, 100));
        game.double_down();
        assert_eq!((game.stake(), game.next()), (100, 150));
    }
}
