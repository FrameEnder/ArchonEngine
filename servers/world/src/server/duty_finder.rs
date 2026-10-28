//! The Duty Finder's settings (the leader's, `DutyFinderSetting`, sent with QueueDuties) and what each does in the
//! duty, as retail plays them out. See `docs/src/fork/duty-finder.md`.

use std::{sync::Arc, time::Duration};

use kawari::{common::ObjectId, ipc::zone::DutyFinderSetting};
use parking_lot::Mutex;

use crate::{
    ClientId, FromServer,
    server::{
        actor::NetworkedActor,
        effect::gain_effect_instance,
        instance::Instance,
        network::{DestinationNetwork, NetworkState},
    },
};

/// The Echo (status 42) and Epic Echo (2734): maximum HP, damage dealt and the potency of healing raised (their
/// Status rows.) Retail gave both with param 0, no duration, from 0xE0000000, only in the status lists (Bob's
/// Chronofoil captures 1aac5be2 and 61a9a4f3, a level 87 White Mage unsynced): the percentage is the server's.
pub const STATUS_THE_ECHO: u16 = 42;
pub const STATUS_EPIC_ECHO: u16 = 2734;

/// Whether the Duty Finder fills the party's empty places (with bots, `party_bot::fill_duty_party`): not when the party
/// goes in as it is. Unrestricted Party enters "with fewer players than the minimum", ignoring role requirements, and
/// with matching disabled; Explorer Mode disables matching too (FFXIV Console Games Wiki, *Duty Finder*.)
pub fn fills_the_party(settings: DutyFinderSetting) -> bool {
    !settings.intersects(DutyFinderSetting::UNRESTRICTED_PARTY | DutyFinderSetting::EXPLORER_MODE)
}

/// An unrestricted party without Level Sync, in its duty: retail gives no EXP, spiritbond or item drops from enemies
/// there (coffers are unaffected; quest EXP and first-clear rewards still come.) With Level Sync the party is synced
/// to the duty's level, "allowing EXP and spiritbond gain" (FFXIV Console Games Wiki, *Duty Finder*.)
pub fn unsynced_unrestricted(settings: DutyFinderSetting) -> bool {
    settings.contains(DutyFinderSetting::UNRESTRICTED_PARTY)
        && !settings.contains(DutyFinderSetting::LEVEL_SYNC)
}

/// A party's loot rule (FFXIV Console Games Wiki, *Duty Finder*: "only fully premade parties can set these; random
/// matches are always Normal"; they can't be changed after entering.)
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum LootRule {
    /// Need, Greed and Pass on everything.
    #[default]
    Normal,
    /// Everyone may only Greed.
    GreedOnly,
    /// The leader distributes each drop, or has the party choose Greed only. Kawari has no lootmaster window (its
    /// packets aren't known), so it plays out as the second: Greed only.
    Lootmaster,
}

impl LootRule {
    /// Whether a Need is taken as a Greed.
    pub fn greed_only(self) -> bool {
        self != LootRule::Normal
    }
}

/// The loot rule a party goes in with: the leader's setting, if the duty allows one
/// (`GameData::content_allows_loot_rules`) and the Duty Finder had no place to fill (the party was full, or went in as
/// it was: Unrestricted Party), otherwise Normal.
pub fn loot_rule(settings: DutyFinderSetting, allowed: bool, places_filled: usize) -> LootRule {
    if !allowed || places_filled > 0 {
        LootRule::Normal
    } else if settings.contains(DutyFinderSetting::LOOTMASTER) {
        LootRule::Lootmaster
    } else if settings.contains(DutyFinderSetting::GREED_ONLY) {
        LootRule::GreedOnly
    } else {
        LootRule::Normal
    }
}

/// Epic Echo, for an unrestricted party without Level Sync (FFXIV Console Games Wiki, *The Echo*): by the duty's level
/// and the highest-level member's, everyone the same: 30 % in a duty of their own level band, 60 % one band above, 200 %
/// two, 300 % three (bands 1-50, 51-60, 61-70, 71-80); none in duties above level 80.
pub fn epic_echo_percent(duty_level: u8, highest_level: u8) -> u16 {
    fn band(level: u8) -> u8 {
        match level {
            0..=50 => 0,
            51..=60 => 1,
            61..=70 => 2,
            _ => 3,
        }
    }
    if duty_level == 0 || duty_level > 80 {
        return 0;
    }
    match band(highest_level).saturating_sub(band(duty_level)) {
        0 => 30,
        1 => 60,
        2 => 200,
        _ => 300,
    }
}

/// How long a fight must have lasted for a wipe to add to The Echo (the wiki: "each wipe after at least 3 minutes in
/// combat".)
pub const ECHO_AFTER: Duration = Duration::from_secs(180);

/// The Echo a wipe adds in a trial of an earlier expansion than the latest (the wiki: older trials, +10 % a wipe up to
/// 50 %, extreme ones +5 % up to 25 %): (the step, the most), or None where it doesn't grow. `kind` is
/// `GameData::content_echo_kind`'s.
pub fn echo_on_wipe(kind: (bool, bool, u8, u8)) -> Option<(u16, u16)> {
    let (trial, high_end, ex_version, latest) = kind;
    if !trial || ex_version >= latest {
        return None;
    }
    Some(if high_end { (5, 25) } else { (10, 50) })
}

/// The Echo some duties give from the start, not growing with wipes (the wiki: the Coils of Bahamut and older savage
/// raids, 10-30 % by duty; not Turn 3): recorded on retail, by ContentFinderCondition (Bob's Chronofoil captures
/// 1aac5be2 and 61a9a4f3: maximum HP ×1.3 in 97, ×1.2 in 101 and 107-110, none in 95.) Others aren't known yet.
pub fn echo_from_the_start(content_finder_condition: u16) -> u16 {
    match content_finder_condition {
        97 => 30,              // the Binding Coil of Bahamut - Turn 5
        101 | 107..=110 => 20, // the Second Coil's Turn 4, the Final Coil's four turns
        _ => 0,
    }
}

/// Echo multiplier on HP, damage and healing: The Echo and Epic Echo multiply (the wiki: Turn 5's 30 % and 300 %
/// make 420 %.)
pub fn echo_multiplier(echo: u16, epic_echo: u16) -> f32 {
    (1.0 + echo as f32 / 100.0) * (1.0 + epic_echo as f32 / 100.0)
}

/// Gives everyone in the party here the instance's Echo (`Instance::echo`, `Instance::epic_echo`): the statuses with
/// their percentage, a player's maximum HP raised (`FromServer::EchoMultiplier`, their stats sent again), a bot's
/// likewise. Their damage and healing are raised in `server/action.rs`.
pub(super) fn refresh_echo(network: &Arc<Mutex<NetworkState>>, instance: &mut Instance) {
    let multiplier = echo_multiplier(instance.echo, instance.epic_echo);
    let players: Vec<ObjectId> = instance
        .actors
        .iter()
        .filter(|(_, actor)| matches!(actor, NetworkedActor::Player { .. }))
        .map(|(id, _)| *id)
        .collect();
    let bots: Vec<ObjectId> = instance.party_bots.keys().copied().collect();
    for id in players.iter().chain(bots.iter()) {
        for (status, percent) in [
            (STATUS_THE_ECHO, instance.echo),
            (STATUS_EPIC_ECHO, instance.epic_echo),
        ] {
            if percent > 0 {
                gain_effect_instance(
                    network.clone(),
                    ClientId::default(),
                    instance,
                    *id,
                    status,
                    0,
                    0.0,
                    ObjectId(0xE000_0000),
                    true,
                );
            }
        }
    }
    for id in &players {
        network.lock().send_to_by_actor_id(
            *id,
            FromServer::EchoMultiplier(multiplier),
            DestinationNetwork::ZoneClients,
        );
    }
    // Bots: their maximum HP from what it was without Echo.
    let before = instance.echo_applied.max(1.0);
    for id in bots {
        if let Some(NetworkedActor::Npc { spawn, .. }) = instance.find_actor_mut(id) {
            let base = spawn.common.max_health_points as f32 / before;
            let max = (base * multiplier).round() as u32;
            let hp = (spawn.common.health_points as f32 / before * multiplier).round() as u32;
            spawn.common.max_health_points = max.max(1);
            spawn.common.health_points = hp.min(max);
        }
    }
    instance.echo_applied = multiplier;
    tracing::info!(
        "Echo here: The Echo {} %, Epic Echo {} % (×{multiplier:.2})",
        instance.echo,
        instance.epic_echo
    );
}

/// The gil of each roulette's first clear of the day for a level 90 character (FFXIV Console Games Wiki, *Duty
/// Roulette*: "based on a level 90 character. Lower-level players receive smaller bonuses."), by ContentRoulette row.
pub fn roulette_gil_at_90(roulette: u8) -> u32 {
    match roulette {
        1 => 7770,   // Leveling
        2 => 7050,   // High-level Dungeons
        3 => 9000,   // Main Scenario
        4 => 1120,   // Guildhests
        5 => 7200,   // Expert
        6 => 12000,  // Trials
        8 => 7200,   // Level Cap Dungeons
        9 => 7050,   // Mentor
        15 => 12000, // Alliance Raids
        17 => 12000, // Normal Raids
        _ => 0,
    }
}

/// The EXP "factor" of Duty Roulette: Leveling's daily bonus, used as a quest's ExpFactor is (`GameData::
/// get_quest_rewards`: factor × ParamGrow.ScaledQuestXP × QuestExpModifier / 100.) Retail's amount isn't in the game
/// data or any capture yet (the wiki: "not determined until a duty is selected"); this is ContentRouletteRoleBonus's
/// BaseExperience (row 1, Leveling's In Need bonus: 500) doubled, a stand-in: about a level at 16, a fifth of one at
/// 90.
pub const LEVELING_BONUS_EXP_FACTOR: u32 = 1000;

/// A roulette's first clear of the day for a character of `level` (its ParamGrow row's ScaledQuestXP and
/// QuestExpModifier, and level 90's QuestExpModifier): (EXP, gil.) The gil is the wiki's level 90 amount scaled by
/// QuestExpModifier; EXP only for Duty Roulette: Leveling (`LEVELING_BONUS_EXP_FACTOR`.)
pub fn roulette_bonus(
    roulette: u8,
    scaled_quest_xp: u32,
    quest_exp_modifier: u32,
    quest_exp_modifier_at_90: u32,
) -> (u32, u32) {
    let gil = roulette_gil_at_90(roulette) as u64
        * quest_exp_modifier.min(quest_exp_modifier_at_90) as u64
        / quest_exp_modifier_at_90.max(1) as u64;
    let exp = if roulette == 1 {
        LEVELING_BONUS_EXP_FACTOR * scaled_quest_xp * quest_exp_modifier / 100
    } else {
        0
    };
    (exp, gil as u32)
}

/// The settings a roulette may use (`QueueRoulette`'s, whose layout is only assumed): Limited Leveling Roulette and
/// the loot rules. Unrestricted Party and the rest can't be used with a roulette on retail.
pub fn roulette_settings(raw: DutyFinderSetting) -> DutyFinderSetting {
    raw & (DutyFinderSetting::LIMITED_LEVELING_ROULETTE
        | DutyFinderSetting::GREED_ONLY
        | DutyFinderSetting::LOOTMASTER)
}

/// Limited Leveling Roulette (the wiki: with "a full premade party of four", the duty is "within eight levels of the
/// lowest-leveled party member"; no effect otherwise): whether a duty needing `required` level may be rolled for a
/// party of `party_levels` (everyone, the player too.)
pub fn limited_leveling_allows(required: u8, party_levels: &[u8]) -> bool {
    if party_levels.len() != 4 {
        return true;
    }
    let lowest = party_levels.iter().copied().min().unwrap_or(0);
    required <= lowest && required as u16 + 8 >= lowest as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unrestricted_and_explorer_parties_go_in_as_they_are() {
        assert!(fills_the_party(DutyFinderSetting::empty()));
        assert!(fills_the_party(DutyFinderSetting::SILENCE_ECHO));
        assert!(!fills_the_party(DutyFinderSetting::UNRESTRICTED_PARTY));
        assert!(!fills_the_party(
            DutyFinderSetting::UNRESTRICTED_PARTY | DutyFinderSetting::LEVEL_SYNC
        ));
        assert!(!fills_the_party(DutyFinderSetting::EXPLORER_MODE));
    }

    #[test]
    fn only_unsynced_unrestricted_parties_lose_rewards() {
        assert!(unsynced_unrestricted(DutyFinderSetting::UNRESTRICTED_PARTY));
        assert!(!unsynced_unrestricted(
            DutyFinderSetting::UNRESTRICTED_PARTY | DutyFinderSetting::LEVEL_SYNC
        ));
        assert!(!unsynced_unrestricted(DutyFinderSetting::empty()));
    }

    #[test]
    fn loot_rules_are_a_full_partys_only() {
        assert_eq!(
            loot_rule(DutyFinderSetting::GREED_ONLY, true, 0),
            LootRule::GreedOnly
        );
        assert_eq!(
            loot_rule(DutyFinderSetting::LOOTMASTER, true, 0),
            LootRule::Lootmaster
        );
        assert_eq!(
            loot_rule(DutyFinderSetting::GREED_ONLY, true, 2),
            LootRule::Normal
        );
        assert_eq!(
            loot_rule(DutyFinderSetting::GREED_ONLY, false, 0),
            LootRule::Normal
        );
        assert_eq!(
            loot_rule(DutyFinderSetting::empty(), true, 0),
            LootRule::Normal
        );
        assert!(LootRule::Lootmaster.greed_only());
        assert!(!LootRule::Normal.greed_only());
    }

    #[test]
    fn epic_echo_follows_the_wikis_table() {
        assert_eq!(epic_echo_percent(15, 50), 30);
        assert_eq!(epic_echo_percent(15, 60), 60);
        assert_eq!(epic_echo_percent(15, 70), 200);
        assert_eq!(epic_echo_percent(15, 100), 300);
        assert_eq!(epic_echo_percent(60, 70), 60);
        assert_eq!(epic_echo_percent(60, 71), 200);
        assert_eq!(epic_echo_percent(80, 100), 30);
        assert_eq!(epic_echo_percent(90, 100), 0);
        // Turn 5: 30 % Echo and 300 % Epic Echo, 420 % in all.
        assert!((echo_multiplier(30, 300) - 5.2).abs() < 1e-5);
    }

    #[test]
    fn roulette_bonuses_scale_down_from_90() {
        // ParamGrow 90: ScaledQuestXP 1480, QuestExpModifier 168; 16: 125 and 12.
        assert_eq!(roulette_bonus(1, 1480, 168, 168), (2486400, 7770));
        assert_eq!(roulette_bonus(1, 125, 12, 168), (15000, 555));
        assert_eq!(roulette_bonus(6, 125, 12, 168).0, 0);
        assert_eq!(roulette_bonus(7, 125, 12, 168), (0, 0));
    }

    #[test]
    fn limited_leveling_needs_a_party_of_four() {
        assert!(limited_leveling_allows(16, &[50]));
        assert!(limited_leveling_allows(28, &[30, 100, 100, 100]));
        assert!(!limited_leveling_allows(16, &[30, 100, 100, 100]));
        assert!(!limited_leveling_allows(32, &[30, 100, 100, 100]));
        assert_eq!(
            roulette_settings(
                DutyFinderSetting::UNRESTRICTED_PARTY | DutyFinderSetting::GREED_ONLY
            ),
            DutyFinderSetting::GREED_ONLY
        );
    }

    #[test]
    fn echo_grows_in_older_trials_only() {
        assert_eq!(echo_on_wipe((true, false, 0, 5)), Some((10, 50)));
        assert_eq!(echo_on_wipe((true, true, 2, 5)), Some((5, 25)));
        assert_eq!(echo_on_wipe((true, false, 5, 5)), None);
        assert_eq!(echo_on_wipe((false, false, 0, 5)), None);
        assert_eq!(echo_from_the_start(97), 30);
        assert_eq!(echo_from_the_start(95), 0);
        // Retail's maximum HP: ×4.8 with 20 % and 300 % (1aac5be2), ×5.2 with 30 % and 300 % (61a9a4f3.)
        assert!((echo_multiplier(20, 300) - 4.8).abs() < 1e-5);
    }
}
