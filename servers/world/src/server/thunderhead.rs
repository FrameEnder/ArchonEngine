//! An NPC thaumaturge's Thunderhead (FrameEnder fork, 2026-11-14).
//!
//! Retail's thaumaturge NPCs (ClassJob 7: Papalymo and the imperial signifers of Escape from Castrum Centri's battle,
//! and every other in Bob's and Connor's Chronofoil captures) gain status 3870 as a spell puts them into Astral Fire
//! or Umbral Ice, and lose it to a Thunder spell, as a player's Black Mage does; Kawari's had neither (the action
//! scripts' NPC branch only deals the damage). An NPC has no job gauge, so its element is kept here.
//!
//! What moves the element, from 8,637 of their spells in the captures (this gets 8,589 of them right; most of the
//! rest look like an element that had run out, which isn't kept):
//!
//! * Fire (141) and Fire II (147) start Astral Fire from nothing and keep it; under Umbral Ice they only end it.
//!   Blizzard (142) and Blizzard II (146) the same the other way round.
//! * Fire III (152) and Blizzard III (154) go to their element from anything; Transpose (149) swaps the two.
//! * Entering an element (from nothing or from the other) gives Thunderhead: GainEffect 3870, no parameter, no time
//!   limit, sent again when it's already there.
//! * Thunder (144), Thunder III (153) and Thunder II (7447) take it away when it's there.

use std::sync::Arc;

use parking_lot::Mutex;

use kawari::common::ObjectId;

use crate::{
    ClientId,
    server::{
        actor::NetworkedActor,
        effect::{gain_effect_instance, lose_effect_instance, process_effects_list},
        instance::Instance,
        network::NetworkState,
    },
};

pub const THUNDERHEAD: u16 = 3870;
const THAUMATURGE: u8 = 7;
const FIRE: [u32; 2] = [141, 147];
const BLIZZARD: [u32; 2] = [142, 146];
const FIRE_III: u32 = 152;
const BLIZZARD_III: u32 = 154;
const TRANSPOSE: u32 = 149;
const THUNDER: [u32; 3] = [144, 153, 7447];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Element {
    AstralFire,
    UmbralIce,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    Gain,
    Lose,
}

/// The element after `action`, and what it does to Thunderhead (`has`: whether the caster has it.)
pub fn step(element: Option<Element>, has: bool, action: u32) -> (Option<Element>, Option<Change>) {
    use Element::*;
    let next = if action == FIRE_III {
        Some(AstralFire)
    } else if action == BLIZZARD_III {
        Some(UmbralIce)
    } else if FIRE.contains(&action) {
        (element != Some(UmbralIce)).then_some(AstralFire)
    } else if BLIZZARD.contains(&action) {
        (element != Some(AstralFire)).then_some(UmbralIce)
    } else if action == TRANSPOSE {
        match element {
            Some(AstralFire) => Some(UmbralIce),
            Some(UmbralIce) => Some(AstralFire),
            None => None,
        }
    } else {
        element
    };
    let change = if next != element && next.is_some() {
        Some(Change::Gain)
    } else if THUNDER.contains(&action) && has {
        Some(Change::Lose)
    } else {
        None
    };
    (next, change)
}

/// Called as an actor's action goes off: a thaumaturge NPC's element moves and its Thunderhead comes or goes.
pub fn after_action(
    network: &Arc<Mutex<NetworkState>>,
    instance: &mut Instance,
    actor_id: ObjectId,
    action_id: u32,
) {
    let Some(NetworkedActor::Npc { spawn, .. }) = instance.find_actor(actor_id) else {
        return;
    };
    // Party bots play their jobs as players do, gauge and all.
    if spawn.common.class_job != THAUMATURGE || instance.party_bots.contains_key(&actor_id) {
        return;
    }
    let has = instance
        .find_actor(actor_id)
        .and_then(|actor| actor.status_effects())
        .is_some_and(|effects| effects.data().iter().any(|e| e.effect_id == THUNDERHEAD));
    let (next, change) = step(
        instance.npc_elements.get(&actor_id).copied(),
        has,
        action_id,
    );
    match next {
        Some(element) => instance.npc_elements.insert(actor_id, element),
        None => instance.npc_elements.remove(&actor_id),
    };
    match change {
        Some(Change::Gain) => {
            gain_effect_instance(
                network.clone(),
                ClientId::default(),
                instance,
                actor_id,
                THUNDERHEAD,
                0,
                0.0,
                actor_id,
                true,
            );
        }
        Some(Change::Lose) => {
            lose_effect_instance(network.clone(), instance, actor_id, THUNDERHEAD)
        }
        None => return,
    }
    process_effects_list(network.clone(), instance, actor_id);
}

#[cfg(test)]
mod tests {
    use super::{Change::*, Element::*, *};

    /// A signifer of Escape from Castrum Centri's battle (4278577, Bob's Chronofoil capture d1c46dff, the won run):
    /// Fire III and Thunderhead with it; Fire II, a Blizzard that only ends Astral Fire, Thunder III taking
    /// Thunderhead away, and Fire III again giving it back.
    #[test]
    fn a_signifer_s_spells() {
        let mut element = None;
        let mut has = false;
        let mut seen = Vec::new();
        for action in [152, 147, 1304, 142, 153, 152] {
            let (next, change) = step(element, has, action);
            element = next;
            match change {
                Some(Gain) => has = true,
                Some(Lose) => has = false,
                None => {}
            }
            seen.push((action, change));
        }
        assert_eq!(
            seen,
            [
                (152, Some(Gain)),
                (147, None),
                (1304, None),
                (142, None),
                (153, Some(Lose)),
                (152, Some(Gain))
            ]
        );
    }

    /// An open-world thaumaturge's Fire, Fire, Blizzard, Blizzard, Blizzard, Fire, Fire (a capture's, 8 s apart):
    /// Thunderhead with the first Fire, the second Blizzard and the second Fire, though it never lost it.
    #[test]
    fn the_first_spell_of_the_other_element_only_ends_this_one() {
        let mut element = None;
        let gains: Vec<bool> = [141, 141, 142, 142, 142, 141, 141]
            .into_iter()
            .map(|action| {
                let (next, change) = step(element, true, action);
                element = next;
                change == Some(Gain)
            })
            .collect();
        assert_eq!(gains, [true, false, false, true, false, false, true]);
        // Transpose swaps, and Blizzard III goes straight to Umbral Ice.
        assert_eq!(
            step(Some(AstralFire), true, 149),
            (Some(UmbralIce), Some(Gain))
        );
        assert_eq!(step(None, false, 149), (None, None));
        assert_eq!(
            step(Some(AstralFire), false, 154),
            (Some(UmbralIce), Some(Gain))
        );
        assert_eq!(step(Some(UmbralIce), false, 154), (Some(UmbralIce), None));
        // A Thunder with nothing to take.
        assert_eq!(step(None, false, 144), (None, None));
    }
}
