//! Fishing's rules: what bites at a fishing spot on a bait, how hard (the tug), which hookset lands it, and after how
//! long, from the community's catch logs (`resources/data/fishing/fishing.json`, made by `questkit fishing` from
//! AutoHook's fish data: Teamcraft's catches and bite times, AutoHook's and FF14 Fish Tracker's fish; licences in that
//! folder's README). See `docs/src/fork/gathering.md` (Fishing).
//!
//! Nothing in the game's sheets says what bites where or how often; these are the counts of what thousands of players
//! caught, so a bite here is drawn as often as it was caught there. The packets of casting, the bite and the hook
//! aren't known yet (no retail capture of a fisher), so nothing calls this from a connection yet.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

/// A bite's tug (the ActionTimeline AutoHook names it by): light, medium, heavy.
pub const TUG_LIGHT: u8 = 36;
pub const TUG_MEDIUM: u8 = 37;
pub const TUG_HEAVY: u8 = 38;
/// The hooksets (Action rows) that land a fish: Powerful Hookset for most heavy ones, Precision for most light ones.
pub const POWERFUL_HOOKSET: u32 = 4103;
pub const PRECISION_HOOKSET: u32 = 4179;

/// What's known of a fish (`fishing.json`'s `fish`.)
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Fish {
    pub tug: Option<u8>,
    pub hookset: Option<u32>,
    /// The bait it takes first, and the fish it's mooched from.
    pub bait: Option<u32>,
    #[serde(default)]
    pub mooch: Vec<u32>,
    /// The fish to catch first for its intuition: [fish, how many].
    #[serde(default)]
    pub predators: Vec<[u32; 2]>,
    /// The weathers it bites in, and the weathers before them (Weather rows.)
    #[serde(default)]
    pub weathers: Vec<u32>,
    #[serde(default)]
    pub weathers_from: Vec<u32>,
    /// Its window in Eorzea minutes of the day [start, end) (the end may wrap past midnight; some begin on the half
    /// hour.)
    pub minutes: Option<[u32; 2]>,
    /// The least Gathering it bites for.
    pub gathering: Option<u32>,
    #[serde(default)]
    pub snagging: bool,
    #[serde(default)]
    pub fish_eyes: bool,
    /// How long its intuition lasts (seconds.)
    pub intuition: Option<u32>,
    #[serde(default)]
    pub big: bool,
}

/// A catch in a spot's log: the fish (-1: nothing on the line), how many were caught, and the bite seconds
/// (minimum, tenth, median, ninetieth, maximum.)
#[derive(Debug, Clone, Deserialize)]
struct Logged(i64, u32, Option<[f32; 5]>);

#[derive(Debug, Deserialize)]
struct Table {
    spots: HashMap<String, HashMap<String, Vec<Logged>>>,
    fish: HashMap<String, Fish>,
}

struct Data {
    spots: HashMap<(u32, u32), Vec<Logged>>,
    fish: HashMap<u32, Fish>,
}

fn data() -> &'static Data {
    static DATA: OnceLock<Data> = OnceLock::new();
    DATA.get_or_init(|| {
        let table: Table =
            serde_json::from_str(include_str!("../../../resources/data/fishing/fishing.json"))
                .expect("resources/data/fishing/fishing.json doesn't read");
        let mut spots = HashMap::new();
        for (spot, baits) in table.spots {
            for (bait, logged) in baits {
                if let (Ok(spot), Ok(bait)) = (spot.parse(), bait.parse()) {
                    spots.insert((spot, bait), logged);
                }
            }
        }
        let fish = table
            .fish
            .into_iter()
            .filter_map(|(id, fish)| id.parse().ok().map(|id| (id, fish)))
            .collect();
        Data { spots, fish }
    })
}

/// What's known of a fish.
pub fn fish(id: u32) -> Option<&'static Fish> {
    data().fish.get(&id)
}

/// The baits anything was logged on at a spot (FishingSpot row.)
pub fn baits_at(spot: u32) -> Vec<u32> {
    let mut baits: Vec<u32> = data()
        .spots
        .keys()
        .filter(|(s, _)| *s == spot)
        .map(|(_, bait)| *bait)
        .collect();
    baits.sort_unstable();
    baits
}

/// The moment a line is cast: what the angler can bring out.
#[derive(Debug, Clone, Default)]
pub struct Conditions {
    /// Eorzea minute of the day (`gathering::eorzea_minute_of_day`.)
    pub eorzea_minute: u32,
    /// The zone's weather now and before (Weather rows), if known: a fish that wants one bites only in it.
    pub weather: Option<u32>,
    pub previous_weather: Option<u32>,
    /// The angler's Gathering.
    pub gathering: u32,
    /// Whether the angler has its intuition (Fisher's Intuition): fish with predators bite only then.
    pub intuition: bool,
}

/// A bite: the fish (None: the line came back empty), its tug and hookset, and after how many seconds it bit.
#[derive(Debug, Clone, PartialEq)]
pub struct Bite {
    pub fish: Option<u32>,
    pub tug: Option<u8>,
    pub hookset: Option<u32>,
    pub seconds: f32,
}

/// Whether a fish bites now: its hours, weathers, Gathering and intuition (one with nothing known always does.)
pub fn bites_now(fish: &Fish, conditions: &Conditions) -> bool {
    if let Some([start, end]) = fish.minutes {
        if start != end && !crate::gathering::window_holds((start, end), conditions.eorzea_minute) {
            return false;
        }
    }
    if !fish.weathers.is_empty()
        && !conditions
            .weather
            .is_some_and(|weather| fish.weathers.contains(&weather))
    {
        return false;
    }
    if !fish.weathers_from.is_empty()
        && !conditions
            .previous_weather
            .is_some_and(|weather| fish.weathers_from.contains(&weather))
    {
        return false;
    }
    if fish
        .gathering
        .is_some_and(|least| conditions.gathering < least)
    {
        return false;
    }
    if !fish.predators.is_empty() && !conditions.intuition {
        return false;
    }
    true
}

/// What bites at `spot` on `bait` now, drawn as often as each was caught there (the fish that can't bite now left
/// out), with its bite time drawn from where most of its bites fell (between the tenth and ninetieth, around the
/// median.) `roll` gives numbers in [0, 1). None if nothing was ever logged there on that bait.
pub fn bite(
    spot: u32,
    bait: u32,
    conditions: &Conditions,
    mut roll: impl FnMut() -> f32,
) -> Option<Bite> {
    let logged = data().spots.get(&(spot, bait))?;
    let open: Vec<&Logged> = logged
        .iter()
        .filter(|entry| {
            entry.0 < 0 || fish(entry.0 as u32).is_none_or(|known| bites_now(known, conditions))
        })
        .collect();
    let total: u32 = open.iter().map(|entry| entry.1).sum();
    if total == 0 {
        return None;
    }
    let mut pick = (roll() * total as f32) as u32;
    let chosen = open
        .iter()
        .find(|entry| {
            if pick < entry.1 {
                true
            } else {
                pick -= entry.1;
                false
            }
        })
        .copied()
        .unwrap_or(open[open.len() - 1]);
    let seconds = match chosen.2 {
        Some([_, tenth, median, ninetieth, _]) => {
            let r = roll();
            if r < 0.5 {
                tenth + (median - tenth) * r * 2.0
            } else {
                median + (ninetieth - median) * (r - 0.5) * 2.0
            }
        }
        // Nothing on the line: it comes back after a while all the same.
        None => 10.0 + roll() * 20.0,
    };
    let caught = (chosen.0 >= 0).then_some(chosen.0 as u32);
    let known = caught.and_then(fish);
    Some(Bite {
        fish: caught,
        tug: known.and_then(|known| known.tug),
        hookset: known.and_then(|known| known.hookset),
        seconds,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Spot 2 on bait 2586: the community's 4927 (6461 caught), 4925 (5884), nothing (1205), 4952 (219), 7705 (211).
    #[test]
    fn draws_as_often_as_caught() {
        let conditions = Conditions {
            gathering: 1000,
            ..Default::default()
        };
        let mut counts: HashMap<Option<u32>, u32> = HashMap::new();
        let mut state = 1u32;
        for _ in 0..5000 {
            let bite = bite(2, 2586, &conditions, || {
                state = state.wrapping_mul(1_103_515_245).wrapping_add(12345);
                (state >> 8) as f32 / (1 << 24) as f32
            })
            .unwrap();
            assert!(bite.seconds > 0.0);
            *counts.entry(bite.fish).or_default() += 1;
        }
        assert!(counts[&Some(4927)] > counts[&Some(4952)] * 10);
        assert!(counts.contains_key(&None), "nothing on the line sometimes");
        assert!(!counts.contains_key(&Some(8769)));
    }

    /// 8769 (heavy, Powerful Hookset): in Eorzea hours 19-4, in weathers 15 and 16, with 382 Gathering and its
    /// intuition (five 5544 first.)
    #[test]
    fn a_big_fish_waits_for_its_moment() {
        let known = fish(8769).unwrap();
        assert_eq!(
            (known.tug, known.hookset),
            (Some(TUG_HEAVY), Some(POWERFUL_HOOKSET))
        );
        let mut now = Conditions {
            eorzea_minute: 20 * 60,
            weather: Some(16),
            previous_weather: None,
            gathering: 400,
            intuition: true,
        };
        assert!(bites_now(known, &now));
        now.eorzea_minute = 12 * 60;
        assert!(!bites_now(known, &now), "out of its hours");
        now.eorzea_minute = 2 * 60;
        assert!(bites_now(known, &now), "past midnight, still in its window");
        now.weather = Some(1);
        assert!(!bites_now(known, &now), "the wrong weather");
        now.weather = Some(15);
        now.intuition = false;
        assert!(!bites_now(known, &now), "without its intuition");
    }

    #[test]
    fn nothing_logged_nothing_bites() {
        assert!(bite(2, 1, &Conditions::default(), || 0.5).is_none());
        assert!(baits_at(2).contains(&2586));
    }
}
