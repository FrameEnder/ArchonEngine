//! Server-side job gauge state. Currently only the White Mage Healing Gauge (Lilies) is simulated.

use kawari::ipc::zone::{ServerZoneIpcData, ServerZoneIpcSegment};

use crate::lua::GaugeOp;

/// ClassJob ID for White Mage.
pub const CLASSJOB_WHITE_MAGE: u8 = 24;

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

    /// Applies a change requested by an action script.
    pub fn apply(&mut self, op: GaugeOp, level: u8) {
        match op {
            GaugeOp::SpendLily => {
                self.lilies = self.lilies.saturating_sub(1);
                if level >= TRANSCENDENT_AFFLATUS_LEVEL {
                    self.blood_lily = (self.blood_lily + 1).min(BLOOD_LILY_BLOOMED);
                }
            }
            GaugeOp::SpendBloodLily => {
                self.blood_lily = 0;
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
    fn packet_layout() {
        let gauge = JobGauge {
            lilies: 2,
            lily_timer_ms: 0x1234,
            blood_lily: 1,
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
