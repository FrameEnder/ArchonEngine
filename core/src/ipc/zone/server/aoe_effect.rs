use binrw::binrw;

use crate::{
    common::{
        ObjectId, ObjectTypeId, Position, read_packed_position, read_quantized_rotation,
        write_packed_position, write_quantized_rotation,
    },
    ipc::zone::{
        ActionEffectFlag, ActionType, ServerZoneIpcData, ServerZoneIpcSegment, TargetEffect,
    },
};

/// An action's results on up to eight targets (AoeEffect8), as retail sends area attacks: the same header as
/// ActionEffect, then eight effects (64 bytes) for each target slot, the target list, and where it was aimed. Retail
/// sends it for an area attack that hit no one or several, or that was aimed at the ground or at no one; one aimed at
/// someone that hit one target goes as ActionEffect1 (Bob's Copperbell Mines captures of 2026-10-25: 838 ActionEffect1
/// with one target and 172 AoeEffect8, 164 of them with none). The layout: Bob's Chronofoil captures (636 bytes, the
/// effects at 42, the targets at 560, the packed position at 624; 0x7FFF on each axis, the middle, when aimed at
/// someone.)
#[binrw]
#[derive(Debug, Default, Clone)]
#[brw(import{max_targets: usize})]
#[brw(assert(targets.len() <= max_targets, "Too many targets! {} > {}", targets.len(), max_targets))]
#[brw(assert(effects.len() == targets.len(), "Each target needs its effects!"))]
pub struct AoeEffect {
    /// Whom the animation plays on: the one it was aimed at, or the caster.
    pub animation_target_id: ObjectTypeId,
    /// Index into the Action Excel sheet.
    pub action_id: u32,
    pub global_sequence: u32,
    /// Controls how long the next action should be delayed, in seconds.
    pub animation_lock: f32,
    pub ballista_entity_id: ObjectId,
    /// The same as `sequence` from this action's `ActionRequest`.
    pub source_sequence: u16,
    #[br(map = read_quantized_rotation)]
    #[bw(map = write_quantized_rotation)]
    pub rotation: f32,
    /// Usually the same as `action_id`.
    pub spell_id: u16,
    pub animation_variation: u8,
    pub action_type: ActionType,
    pub flags: ActionEffectFlag,
    #[br(temp)]
    #[bw(calc = targets.len() as u8)]
    pub target_count: u8,
    /// Each target's effects, in the order of `targets`.
    #[brw(pad_before = 8)] // not read?
    #[br(count = target_count, pad_after = (max_targets - target_count as usize) * 64)]
    #[bw(pad_size_to = max_targets * 64)]
    pub effects: Vec<[TargetEffect; 8]>,
    #[brw(pad_before = 6)]
    #[br(count = target_count, pad_after = (max_targets - target_count as usize) * 8)]
    #[bw(pad_size_to = max_targets * 8)]
    pub targets: Vec<ObjectTypeId>,
    /// Where it was aimed, for an attack aimed at the ground.
    #[brw(pad_after = 6)] // empty
    #[br(map = read_packed_position)]
    #[bw(map = write_packed_position)]
    pub position: Position,
}

impl AoeEffect {
    pub fn package(&self) -> Option<ServerZoneIpcSegment> {
        match self.targets.len() {
            0..=8 => Some(ServerZoneIpcSegment::new(ServerZoneIpcData::AoeEffect8(
                self.clone(),
            ))),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use binrw::{BinRead, BinWrite};

    use crate::{common::ObjectTypeKind, ipc::zone::TargetEffectKind};

    use super::*;

    /// Retail's Medica (124) on four, from Bob's Copperbell Mines capture with Duty Support (2026-10-25): 636 bytes, each
    /// target's heal at 42 + 64 × its slot, the targets at 560.
    #[test]
    fn aoe_effect8_layout() {
        let target = |id| ObjectTypeId {
            object_id: ObjectId(id),
            object_type: ObjectTypeKind::None,
        };
        let heal = |amount| {
            let mut effects = [TargetEffect::default(); 8];
            effects[0] = TargetEffect(TargetEffectKind::Heal {
                unk1: [0; 5],
                amount,
            });
            effects
        };
        let packet = AoeEffect {
            animation_target_id: target(0x1002B65E),
            action_id: 124,
            global_sequence: 24428,
            animation_lock: 0.1,
            source_sequence: 51,
            spell_id: 124,
            action_type: ActionType::Action,
            effects: vec![heal(127), heal(128)],
            targets: vec![target(0x1002B65E), target(0x40008F3E)],
            ..Default::default()
        };
        let mut buffer = Cursor::new(Vec::new());
        packet
            .write_le_args(&mut buffer, binrw::args! { max_targets: 8 })
            .unwrap();
        let bytes = buffer.into_inner();
        assert_eq!(bytes.len(), 636);
        assert_eq!(bytes[33], 2);
        assert_eq!(&bytes[42..49], &[4, 0, 0, 0, 0, 0, 127]);
        assert_eq!(&bytes[106..113], &[4, 0, 0, 0, 0, 0, 128]);
        assert_eq!(&bytes[560..564], &0x1002B65Eu32.to_le_bytes());
        assert_eq!(&bytes[568..572], &0x40008F3Eu32.to_le_bytes());
        assert_eq!(&bytes[624..630], &[0xFF, 0x7F, 0xFF, 0x7F, 0xFF, 0x7F]);

        let read =
            AoeEffect::read_le_args(&mut Cursor::new(bytes), binrw::args! { max_targets: 8 })
                .unwrap();
        assert_eq!(read.targets.len(), 2);
        assert_eq!(read.action_id, 124);
    }
}
