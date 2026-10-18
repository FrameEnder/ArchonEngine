use std::time::Instant;

use kawari::{common::ObjectId, ipc::zone::StatusEffect};

#[derive(Debug, Default, Clone)]
pub struct StatusEffects {
    status_effects: Vec<StatusEffect>,
    /// When each status (same index) runs out; None for those that never do. StatusEffectList carries the time
    /// remaining (retail counts it down), so it's worked out from this whenever the list is sent.
    expires: Vec<Option<Instant>>,
    dirty: bool,
}

impl StatusEffects {
    /// Adds a status, or refreshes it (new param, duration and source.) A duration of 0 never runs out.
    pub fn add(&mut self, effect_id: u16, effect_param: u16, duration: f32, source: ObjectId) {
        let index = self.find_or_create_status_effect(effect_id);
        let status_effect = &mut self.status_effects[index];
        status_effect.param = effect_param;
        status_effect.duration = duration;
        status_effect.source_actor_id = source;
        self.expires[index] = expiry(duration);
        self.dirty = true
    }

    fn find_or_create_status_effect(&mut self, effect_id: u16) -> usize {
        if let Some(i) = self
            .status_effects
            .iter()
            .position(|effect| effect.effect_id == effect_id)
        {
            i
        } else {
            self.status_effects.push(StatusEffect {
                effect_id,
                ..Default::default()
            });
            self.expires.push(None);
            self.status_effects.len() - 1
        }
    }

    pub fn get(&self, effect_id: u16) -> Option<StatusEffect> {
        self.status_effects
            .iter()
            .position(|effect| effect.effect_id == effect_id)
            .map(|i| self.status_effects[i])
    }

    /// Changes the param (e.g. stack count) of an existing effect. Returns false if the effect isn't active.
    pub fn set_param(&mut self, effect_id: u16, param: u16) -> bool {
        if let Some(effect) = self
            .status_effects
            .iter_mut()
            .find(|effect| effect.effect_id == effect_id)
        {
            effect.param = param;
            self.dirty = true;
            true
        } else {
            false
        }
    }

    /// Changes the remaining duration of an existing effect.
    pub fn set_duration(&mut self, effect_id: u16, duration: f32) {
        if let Some(i) = self
            .status_effects
            .iter()
            .position(|effect| effect.effect_id == effect_id)
        {
            self.status_effects[i].duration = duration;
            self.expires[i] = expiry(duration);
            self.dirty = true;
        }
    }

    pub fn remove(&mut self, effect_id: u16) {
        if let Some(i) = self
            .status_effects
            .iter()
            .position(|effect| effect.effect_id == effect_id)
        {
            self.status_effects.remove(i);
            self.expires.remove(i);
            self.dirty = true;
        }
    }

    /// The statuses as StatusEffectList sends them: with the time each has left (0 for those that never run out.)
    pub fn data(&self) -> Vec<StatusEffect> {
        let now = Instant::now();
        self.status_effects
            .iter()
            .zip(&self.expires)
            .map(|(effect, expires)| StatusEffect {
                duration: expires
                    .map(|at| at.saturating_duration_since(now).as_secs_f32())
                    .unwrap_or(effect.duration),
                ..*effect
            })
            .collect()
    }

    /// The IDs of the active statuses.
    pub fn ids(&self) -> impl Iterator<Item = u16> + '_ {
        self.status_effects.iter().map(|effect| effect.effect_id)
    }

    /// If the list is dirty and must be propagated to the client
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn reset_dirty(&mut self) {
        self.dirty = false;
    }

    /// Number of status effects.
    pub fn len(&self) -> usize {
        self.status_effects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.status_effects.is_empty()
    }
}

fn expiry(duration: f32) -> Option<Instant> {
    (duration > 0.0).then(|| Instant::now() + std::time::Duration::from_secs_f32(duration))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_effects() {
        // Ensure sensible initial state
        let mut status_effects = StatusEffects::default();
        assert_eq!(status_effects.get(0), None);
        assert_eq!(status_effects.is_dirty(), false);

        // Add a status effect, check that it can be grabbed again, and that the dirty flag is set:
        status_effects.add(0, 0, 0.0, ObjectId::default());
        assert_eq!(
            status_effects.get(0),
            Some(StatusEffect {
                effect_id: 0,
                param: 0,
                duration: 0.0,
                source_actor_id: Default::default()
            })
        );
        assert_eq!(status_effects.is_dirty(), true);

        // Try resetting the dirty flag:
        status_effects.reset_dirty();
        assert_eq!(status_effects.is_dirty(), false);

        // Removing a status should mark it as dirty, and it should really be gone:
        status_effects.remove(0);
        assert_eq!(status_effects.get(0), None);
        assert_eq!(status_effects.is_dirty(), true);
    }

    /// The list sends the time left, so an older status doesn't jump back to its full duration whenever the list is
    /// sent again; refreshing takes the new param and source.
    #[test]
    fn statuses_count_down_and_refresh() {
        let mut status_effects = StatusEffects::default();
        status_effects.add(50, 30, 20.0, ObjectId(1));
        std::thread::sleep(std::time::Duration::from_millis(50));
        let sent = status_effects.data()[0];
        assert!(sent.duration < 20.0 && sent.duration > 19.0);
        assert_eq!(sent.source_actor_id, ObjectId(1));

        status_effects.add(50, 20, 10.0, ObjectId(2));
        let sent = status_effects.data()[0];
        assert_eq!(sent.param, 20);
        assert_eq!(sent.source_actor_id, ObjectId(2));
        assert!(sent.duration <= 10.0);

        // Permanent ones stay at 0.
        status_effects.add(4209, 20, 0.0, ObjectId(1));
        assert_eq!(status_effects.data()[1].duration, 0.0);
        assert_eq!(status_effects.ids().collect::<Vec<_>>(), [50, 4209]);
    }
}
