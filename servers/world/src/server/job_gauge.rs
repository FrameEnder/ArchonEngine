//! Server-side job gauge state: the White Mage Healing Gauge (Lilies), Arcanist/Summoner Aetherflow, the Warrior's
//! Beast Gauge, the Monk's chakra and the Black Mage's Astral Fire / Umbral Ice and Umbral Hearts. Players' and party
//! bots' alike (a bot's is kept on it, `PartyBot::gauge`); the action scripts change and read the counters by name
//! (`effects:gauge_add`, `player:gauge`.)

use kawari::ipc::zone::{ServerZoneIpcData, ServerZoneIpcSegment};

use crate::lua::{GaugeField, GaugeOp};

/// ClassJob ID for White Mage.
pub const CLASSJOB_WHITE_MAGE: u8 = 24;

/// ClassJob IDs for Arcanist and Summoner, whose gauge carries Aetherflow.
pub const CLASSJOB_ARCANIST: u8 = 26;
pub const CLASSJOB_SUMMONER: u8 = 27;

/// Warrior, Monk (and Pugilist) and Black Mage (and Thaumaturge): the gauges kept for them.
pub const CLASSJOB_WARRIOR: u8 = 21;
pub const CLASSJOB_PUGILIST: u8 = 2;
pub const CLASSJOB_MONK: u8 = 20;
pub const CLASSJOB_THAUMATURGE: u8 = 7;
pub const CLASSJOB_BLACK_MAGE: u8 = 25;

pub const CLASSJOB_LANCER: u8 = 4;
pub const CLASSJOB_DRAGOON: u8 = 22;
pub const CLASSJOB_GLADIATOR: u8 = 1;
pub const CLASSJOB_PALADIN: u8 = 19;
pub const CLASSJOB_BARD: u8 = 23;
pub const CLASSJOB_NINJA: u8 = 30;
pub const CLASSJOB_SCHOLAR: u8 = 28;

/// Enhanced Enochian (70): Polyglot every 30 s of Enochian.
const ENOCHIAN_POLYGLOT_LEVEL: u8 = 70;
const POLYGLOT_TENTHS: i16 = 300;

/// Where a client gauge field at `offset` (FFXIVClientStructs' JobGauges) sits in ActorGauge's data, for the jobs
/// whose packets aren't in Bob's captures: as the Paladin's and Summoner's are (retail: the Paladin's Oath, 0x08, was
/// data[1] at 100; the Summoner's AetherFlags, 0x0F, data[8]). The White Mage's (Lilies) were one lower (its timer,
/// 0x0A, at data[2..4], its Lily at data[4]), so this is a guess for the others until their packets are seen.
fn at(offset: usize) -> usize {
    offset - 7
}

/// Aetherflow is the low two bits of this byte of ActorGauge's data, the rest being Summoner's attunement flags
/// (retail, Bob's Chronofoil capture of 2026-10-21: 0x02 after each Energy Drain, 0x01 after the Fester that followed,
/// 0x22/0x21 with a summon's flag set.)
const DATA_OFFSET_AETHERFLOW: usize = 8;

/// Level of the "Secret of the Lily" trait (Trait 196), which starts Lily generation.
pub const SECRET_OF_THE_LILY_LEVEL: u8 = 52;

/// Level of the "Transcendent Afflatus" trait (Trait 309), which adds the Blood Lily.
pub const TRANSCENDENT_AFFLATUS_LEVEL: u8 = 74;

/// A Lily is added for every 20 seconds spent in combat.
const LILY_INTERVAL_MS: u32 = 20_000;

/// Maximum number of Lilies.
const MAX_LILIES: u8 = 3;

/// The Blood Lily blooms after three Afflatus Solace/Rapture uses.
const BLOOD_LILY_BLOOMED: u8 = 3;

// Byte offsets into ActorGauge's `data`. The client's WhiteMageGauge (FFXIVClientStructs) has
// LilyTimer at 0x0A, Lily at 0x0C and BloodLily at 0x0D, and gauge data starts at 0x08.
// NOTE: These assume `data[0]` lines up with offset 0x08. If the gauge displays wrong in-game, adjust these.
const DATA_OFFSET_LILY_TIMER: usize = 2;
const DATA_OFFSET_LILY: usize = 4;
const DATA_OFFSET_BLOOD_LILY: usize = 5;

#[derive(Debug, Clone, Default)]
pub struct JobGauge {
    /// Number of Lilies available (0-3.)
    pub lilies: u8,
    /// Milliseconds spent in combat towards the next Lily.
    pub lily_timer_ms: u32,
    /// Blood Lily growth (0-3, 3 meaning it's in full bloom.)
    pub blood_lily: u8,
    /// Aetherflow stacks (Arcanist/Summoner, 0-2.)
    pub aetherflow: u8,
    /// The counters scripts change by name (`GaugeField`, in its order.)
    pub fields: [i16; GaugeField::ALL.len()],
}

impl JobGauge {
    /// Clears the gauge, for example when changing classes.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Advances the White Mage Lily timer. Returns true if the gauge should be re-sent to the client.
    pub fn tick_white_mage(&mut self, elapsed_ms: u32, in_combat: bool, level: u8) -> bool {
        if level < SECRET_OF_THE_LILY_LEVEL || !in_combat {
            return false;
        }

        if self.lilies >= MAX_LILIES {
            // The timer doesn't progress while the gauge is full.
            let changed = self.lily_timer_ms != 0;
            self.lily_timer_ms = 0;
            return changed;
        }

        self.lily_timer_ms += elapsed_ms;
        if self.lily_timer_ms >= LILY_INTERVAL_MS {
            self.lily_timer_ms -= LILY_INTERVAL_MS;
            self.lilies += 1;
        }

        true
    }

    /// A counter by its field.
    pub fn get(&self, field: GaugeField) -> i16 {
        self.fields[field.index()]
    }

    fn slot(&mut self, field: GaugeField) -> &mut i16 {
        &mut self.fields[field.index()]
    }

    /// The Black Mage's Astral Fire (positive) or Umbral Ice (negative).
    pub fn element(&self) -> i16 {
        self.get(GaugeField::Element)
    }

    /// What passes with time (every server tick, for players and party bots): Life of the Dragon runs down, and the
    /// Black Mage's Enochian (Astral Fire or Umbral Ice held, from 70) gives a Polyglot every 30 s, up to one, two
    /// from 80 (Enhanced Polyglot) and three from 98 (the job guide, 7.x). Returns whether the client's gauge changed.
    pub fn tick(&mut self, elapsed_ms: u32, classjob: u8, level: u8) -> bool {
        let tenths = (elapsed_ms / 100) as i16;
        let mut changed = false;
        if matches!(classjob, CLASSJOB_DRAGOON | CLASSJOB_LANCER)
            && self.get(GaugeField::LifeOfTheDragon) > 0
        {
            let left = self.get(GaugeField::LifeOfTheDragon) - tenths;
            *self.slot(GaugeField::LifeOfTheDragon) = left.max(0);
            changed = true;
        }
        if matches!(classjob, CLASSJOB_BLACK_MAGE | CLASSJOB_THAUMATURGE)
            && level >= ENOCHIAN_POLYGLOT_LEVEL
        {
            if self.element() == 0 {
                *self.slot(GaugeField::EnochianTimer) = 0;
            } else {
                let timer = self.get(GaugeField::EnochianTimer) + tenths;
                if timer >= POLYGLOT_TENTHS {
                    let max = if level >= 98 {
                        3
                    } else if level >= 80 {
                        2
                    } else {
                        1
                    };
                    let polyglot = self.slot(GaugeField::Polyglot);
                    *polyglot = (*polyglot + 1).min(max);
                    *self.slot(GaugeField::EnochianTimer) = timer - POLYGLOT_TENTHS;
                } else {
                    *self.slot(GaugeField::EnochianTimer) = timer;
                }
                changed = true;
            }
        }
        changed
    }

    /// Every counter by name, with the Lilies and Aetherflow (for scripts and the bots' rotations.)
    pub fn named(&self) -> Vec<(&'static str, i32)> {
        let mut named: Vec<(&'static str, i32)> = GaugeField::ALL
            .into_iter()
            .map(|field| (field.name(), self.get(field) as i32))
            .collect();
        named.push(("lilies", self.lilies as i32));
        named.push(("blood_lily", self.blood_lily as i32));
        named.push(("aetherflow", self.aetherflow as i32));
        named
    }

    /// Applies a change requested by an action script.
    pub fn apply(&mut self, op: GaugeOp, level: u8) {
        match op {
            GaugeOp::Add(field, amount) => {
                let (low, high) = field.range();
                let slot = self.slot(field);
                *slot = slot.saturating_add(amount).clamp(low, high);
            }
            GaugeOp::Set(field, value) => {
                let (low, high) = field.range();
                *self.slot(field) = value.clamp(low, high);
            }
            GaugeOp::SpendLily => {
                self.lilies = self.lilies.saturating_sub(1);
                if level >= TRANSCENDENT_AFFLATUS_LEVEL {
                    self.blood_lily = (self.blood_lily + 1).min(BLOOD_LILY_BLOOMED);
                }
            }
            GaugeOp::SpendBloodLily => {
                self.blood_lily = 0;
            }
            GaugeOp::GainAetherflow(stacks) => {
                self.aetherflow = stacks.min(3);
            }
            GaugeOp::SpendAetherflow => {
                self.aetherflow = self.aetherflow.saturating_sub(1);
            }
        }
    }

    /// Serializes this gauge into the ActorGauge packet data for the given class.
    pub fn to_packet_data(&self, classjob_id: u8) -> [u8; 15] {
        let mut data = [0u8; 15];
        if classjob_id == CLASSJOB_WHITE_MAGE {
            let timer = (self.lily_timer_ms.min(i16::MAX as u32) as i16).to_le_bytes();
            data[DATA_OFFSET_LILY_TIMER..DATA_OFFSET_LILY_TIMER + 2].copy_from_slice(&timer);
            data[DATA_OFFSET_LILY] = self.lilies;
            data[DATA_OFFSET_BLOOD_LILY] = self.blood_lily;
        } else if classjob_id == CLASSJOB_ARCANIST || classjob_id == CLASSJOB_SUMMONER {
            // SummonerGauge's AttunementType (bits 0-1) and count (bits 2-7) at 0x0E, AetherFlags at 0x0F (Aetherflow in
            // its low two bits, Ifrit, Titan and Garuda ready in bits 5-7): AetherFlags is data[8] in retail's packets.
            data[at(0x0E)] = (self.get(GaugeField::AttunementType) as u8 & 3)
                | ((self.get(GaugeField::Attunement) as u8) << 2);
            data[DATA_OFFSET_AETHERFLOW] =
                (self.aetherflow & 0x03) | ((self.get(GaugeField::Arcana) as u8 & 7) << 5);
        } else if classjob_id == CLASSJOB_WARRIOR {
            data[at(0x08)] = self.get(GaugeField::Beast) as u8;
        } else if classjob_id == CLASSJOB_MONK || classjob_id == CLASSJOB_PUGILIST {
            // MonkGauge: Chakra 0x08, BeastChakra1-3 0x09-0x0B, the Furies at 0x0C (two bits each: Opo-opo, Raptor,
            // Coeurl), Nadi 0x0D.
            data[at(0x08)] = self.get(GaugeField::Chakra) as u8;
            let chakra = self.get(GaugeField::BeastChakra) as u8;
            data[at(0x09)] = chakra & 3;
            data[at(0x0A)] = (chakra >> 2) & 3;
            data[at(0x0B)] = (chakra >> 4) & 3;
            data[at(0x0C)] = (self.get(GaugeField::OpoFury) as u8 & 3)
                | ((self.get(GaugeField::RaptorFury) as u8 & 3) << 2)
                | ((self.get(GaugeField::CoeurlFury) as u8 & 3) << 4);
            data[at(0x0D)] = self.get(GaugeField::Nadi) as u8;
        } else if classjob_id == CLASSJOB_BLACK_MAGE || classjob_id == CLASSJOB_THAUMATURGE {
            // BlackMageGauge: EnochianTimer 0x08 (ms), ElementStance 0x0A, UmbralHearts 0x0B, PolyglotStacks 0x0C,
            // 0x0D Enochian (bit 0), Paradox (bit 1), Astral Soul (bits 2-4).
            let timer = (self.get(GaugeField::EnochianTimer) * 100).to_le_bytes();
            data[at(0x08)..at(0x08) + 2].copy_from_slice(&timer);
            data[at(0x0A)] = self.element() as i8 as u8;
            data[at(0x0B)] = self.get(GaugeField::UmbralHearts) as u8;
            data[at(0x0C)] = self.get(GaugeField::Polyglot) as u8;
            data[at(0x0D)] = u8::from(self.element() != 0)
                | ((self.get(GaugeField::Paradox) as u8 & 1) << 1)
                | ((self.get(GaugeField::AstralSoul) as u8 & 7) << 2);
        } else if classjob_id == CLASSJOB_PALADIN || classjob_id == CLASSJOB_GLADIATOR {
            data[at(0x08)] = self.get(GaugeField::Oath) as u8;
        } else if classjob_id == CLASSJOB_DRAGOON || classjob_id == CLASSJOB_LANCER {
            // DragoonGauge: LotdTimer 0x08 (ms), LotdState 0x0A (2 while it lasts), FirstmindsFocusCount 0x0C.
            let left = self.get(GaugeField::LifeOfTheDragon);
            data[at(0x08)..at(0x08) + 2].copy_from_slice(&(left * 100).to_le_bytes());
            data[at(0x0A)] = if left > 0 { 2 } else { 0 };
            data[at(0x0C)] = self.get(GaugeField::FirstmindsFocus) as u8;
        } else if classjob_id == CLASSJOB_BARD {
            // BardGauge: Repertoire 0x0C, SoulVoice 0x0D, the Codas in SongFlags (bits 4-6) at 0x0F.
            data[at(0x0C)] = self.get(GaugeField::Repertoire) as u8;
            data[at(0x0D)] = self.get(GaugeField::SoulVoice) as u8;
            data[at(0x0F)] = (self.get(GaugeField::Coda) as u8 & 7) << 4;
        } else if classjob_id == CLASSJOB_NINJA {
            // NinjaGauge: Ninki 0x08, Kazematoi 0x0A.
            data[at(0x08)] = self.get(GaugeField::Ninki) as u8;
            data[at(0x0A)] = self.get(GaugeField::Kazematoi) as u8;
        } else if classjob_id == CLASSJOB_SCHOLAR {
            // ScholarGauge: Aetherflow 0x08, FairyGauge 0x09.
            data[at(0x08)] = self.aetherflow;
            data[at(0x09)] = self.get(GaugeField::Fairy) as u8;
        }
        data
    }

    /// Builds the ActorGauge packet for this gauge.
    pub fn to_ipc(&self, classjob_id: u8) -> ServerZoneIpcSegment {
        ServerZoneIpcSegment::new(ServerZoneIpcData::ActorGauge {
            classjob_id,
            data: self.to_packet_data(classjob_id),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lilies_only_grow_in_combat_after_level_52() {
        let mut gauge = JobGauge::default();
        assert!(!gauge.tick_white_mage(20_000, true, 51));
        assert!(!gauge.tick_white_mage(20_000, false, 90));
        assert_eq!(gauge.lilies, 0);

        for _ in 0..40 {
            gauge.tick_white_mage(500, true, 90);
        }
        assert_eq!(gauge.lilies, 1);
        assert_eq!(gauge.lily_timer_ms, 0);
    }

    #[test]
    fn lilies_cap_at_three() {
        let mut gauge = JobGauge::default();
        for _ in 0..(40 * 5) {
            gauge.tick_white_mage(500, true, 90);
        }
        assert_eq!(gauge.lilies, 3);
        assert_eq!(gauge.lily_timer_ms, 0);
    }

    #[test]
    fn spending_lilies_blooms_blood_lily() {
        let mut gauge = JobGauge {
            lilies: 3,
            ..Default::default()
        };
        for _ in 0..3 {
            gauge.apply(GaugeOp::SpendLily, 100);
        }
        assert_eq!(gauge.lilies, 0);
        assert_eq!(gauge.blood_lily, 3);

        // Doesn't overgrow
        gauge.lilies = 1;
        gauge.apply(GaugeOp::SpendLily, 100);
        assert_eq!(gauge.blood_lily, 3);

        gauge.apply(GaugeOp::SpendBloodLily, 100);
        assert_eq!(gauge.blood_lily, 0);
    }

    #[test]
    fn no_blood_lily_before_trait() {
        let mut gauge = JobGauge {
            lilies: 3,
            ..Default::default()
        };
        gauge.apply(GaugeOp::SpendLily, 73);
        assert_eq!(gauge.blood_lily, 0);
    }

    #[test]
    fn named_counters_stay_in_range_and_reach_the_packet() {
        let mut gauge = JobGauge::default();
        gauge.apply(GaugeOp::Add(GaugeField::Beast, 60), 50);
        gauge.apply(GaugeOp::Add(GaugeField::Beast, 60), 50);
        assert_eq!(gauge.get(GaugeField::Beast), 100);
        gauge.apply(GaugeOp::Add(GaugeField::Beast, -50), 50);
        assert_eq!(gauge.to_packet_data(CLASSJOB_WARRIOR)[1], 50);
        gauge.apply(GaugeOp::Set(GaugeField::Element, -3), 50);
        assert_eq!(gauge.to_packet_data(CLASSJOB_BLACK_MAGE)[3], 0xFD);
        gauge.apply(GaugeOp::Set(GaugeField::Element, 7), 50);
        assert_eq!(gauge.element(), 3);
        // The Paladin's Oath at 100 is data[1], as retail sent it (Chronofoil capture 341808b0.)
        gauge.apply(GaugeOp::Set(GaugeField::Oath, 100), 50);
        assert_eq!(gauge.to_packet_data(CLASSJOB_PALADIN)[1], 0x64);
        assert!(gauge.named().contains(&("beast", 50)));
    }

    #[test]
    fn aetherflow_fills_and_spends() {
        let mut gauge = JobGauge::default();
        gauge.apply(GaugeOp::GainAetherflow(2), 15);
        assert_eq!(gauge.to_packet_data(CLASSJOB_ARCANIST)[8], 2);
        gauge.apply(GaugeOp::SpendAetherflow, 15);
        assert_eq!(gauge.to_packet_data(CLASSJOB_SUMMONER)[8], 1);
        gauge.apply(GaugeOp::SpendAetherflow, 15);
        gauge.apply(GaugeOp::SpendAetherflow, 15);
        assert_eq!(gauge.aetherflow, 0);
        // White Mage's gauge doesn't carry it.
        gauge.aetherflow = 2;
        assert_eq!(gauge.to_packet_data(CLASSJOB_WHITE_MAGE)[8], 0);
    }

    #[test]
    fn packet_layout() {
        let gauge = JobGauge {
            lilies: 2,
            lily_timer_ms: 0x1234,
            blood_lily: 1,
            ..Default::default()
        };
        let data = gauge.to_packet_data(CLASSJOB_WHITE_MAGE);
        assert_eq!(data[2], 0x34);
        assert_eq!(data[3], 0x12);
        assert_eq!(data[4], 2);
        assert_eq!(data[5], 1);

        // Other classes get an empty gauge
        assert_eq!(gauge.to_packet_data(6), [0u8; 15]);
    }
}
