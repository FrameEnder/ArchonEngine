use std::collections::BTreeMap;

use serde_derive::{Deserialize, Serialize};

/// A JSON file that appends an existing (and usually empty) LGB.
///
/// These can only add gathering points for now.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Timeline {
    /// Which action is used for the auto-attack. Index into the Action Excel sheet.
    pub autoattack_action_id: u32,
    /// Whether the timeline always plays.
    pub timeline_always_plays: bool,
    /// The timeline points.
    pub timepoints: Vec<Timepoint>,
    /// The points' times count from the NPC's pull (its first target), to the half second, and each action is used
    /// when due whether its target is in reach or not, as retail's bosses keep their schedule: the Copperbell Mines'
    /// Kottos cast at 5.2, 15.3, 29.6 and 39.1 s from its pull in all three of Bob's runs, however long it stood
    /// before. Otherwise they count from its spawn in whole seconds, used only when in reach (upstream's.)
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub from_pull: bool,
    /// A series of actions (to play in sequence) on death.
    #[serde(default)]
    pub on_death: Vec<TimepointData>,
    /// How close (in yalms) this NPC gets to its target before attacking, e.g. further away for casters.
    #[serde(default = "default_attack_range")]
    pub attack_range: f32,
    /// How much this NPC's auto-attack hits players for on retail, by its level (an average, before critical hits.)
    /// Its other actions hit for this times their potency / 100. Levels in between or beyond are scaled by how
    /// ordinary enemies' health grows. Without any, NPC damage is worked out from its stats instead.
    #[serde(default)]
    pub damage_to_players: BTreeMap<u8, f32>,
    /// What this NPC's actions did on retail, by action ID: the average damage (or healing, for a heal) to a player
    /// and to another NPC, as Kawari Recorder's recording of a duty shows it. When there's one for the action and the
    /// kind of target, the hit is scaled to it (then varied, missed, parried or critical like any NPC hit), whatever
    /// the action's script and the NPC's stats would give. This is how allies and enemies in quest battles hurt (and
    /// heal) each other the way they do on retail.
    #[serde(default)]
    pub recorded_amounts: BTreeMap<u32, RecordedAmount>,
    /// This NPC's hitbox radius in yalms (as Kawari Recorder reads it on retail.) Its area attacks reach this much
    /// further, since they're measured from the edge of its hitbox.
    #[serde(default)]
    pub hitbox_radius: f32,
    /// The animation lock retail's ActionEffect carried for each of this NPC's actions, by action ID (the same every
    /// use: Bob's Copperbell Mines captures, e.g. 2.1 for its bosses' attacks, 2.3 or 2.4 for some trash abilities, 1.1
    /// for the helpers' markers). Its auto-attack is always 0.1; an action not here gets a player's lock.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub animation_locks: BTreeMap<u32, f32>,
    /// How this NPC fights with its class's actions, instead of (or as well as) fixed timepoints: see `Rotation`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<Rotation>,
    /// What this NPC shouts with its actions, by action ID: see `Shouts`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub shouts: BTreeMap<u32, Shouts>,
    /// Where (x, y, z) this NPC fights from once pulled: it walks there and stays, turning to whom it fights and acting
    /// on them from there, its timeline's actions too, never chasing anyone. Retail's Magitek Vanguard F-1 (Castrum
    /// Meridianum's second boss) walked to its arena's middle in the 3 s after its pull and never moved again, every
    /// cast and auto-attack from there (Bob's Chronofoil capture 281f92fb); Kawari's chased the bots out of its arena
    /// (Bob's marks of 2026-11-22, "he needs to be in center of arena".) `questkit chronofoil --spots` finds the ones
    /// that did in the captures (walked to their arena's middle and held it) and fails where a timeline lacks it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fights_from: Option<[f32; 3]>,
    /// This NPC never moves or turns: it fights from where it was put, facing where it was put. Retail's Proto Ultima
    /// arm units in the Praetorium (BNpcBase 14566) sent no ActorMove from their spawn to their fall, their 29185 and
    /// 29186 and auto-attacks all from there (Bob's Chronofoil capture 35e30205, all four); Kawari's walked about after
    /// the party (Bob's mark of 2026-11-26, "should stay in place").
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub fixed: bool,
}

/// The lines an NPC shouts as it uses one of its actions, as retail sent them: ActorControl 73 `[line]` from the NPC,
/// in the same instant as the action's ActionEffect (Bob's Chronofoil captures: 26 actions of 22 BNpcBases, all the
/// quest battles' and Sastasha's). Retail's went through a list in order, one line per use, then said nothing: Into a
/// Copper Hell's pugilist said 1351 with its first Snap Punch and 1352 with its second in each of seven runs; Biggs
/// (657) said 1354, 2300 and 1355 with his first three uses of 21. The line is an index the client reads its text
/// from, so no game text is here.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone, Default)]
pub struct Shouts {
    /// The line for each use in turn (the first use's first); 0 says nothing that time (Garibald, BNpcBase 891, said
    /// his with his fifth Heavy Swing, in the one capture of it.)
    pub lines: Vec<u32>,
    /// The last line is said with every use after it too (Sastasha's captain said 1350 with every use of 1000.)
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub repeat_last: bool,
}

impl Shouts {
    /// The line said with the `nth` use (1 for the first), if any.
    pub fn line(&self, nth: u32) -> Option<u32> {
        let index = nth.checked_sub(1)? as usize;
        let line = match self.lines.get(index) {
            Some(line) => *line,
            None if self.repeat_last => *self.lines.last()?,
            None => 0,
        };
        (line != 0).then_some(line)
    }
}

/// A class rotation: which actions an NPC uses, in priority order, by the class's own rules. The mechanics come from
/// the Action sheet at runtime (whether an action is on the GCD, its recast, the action it follows in a combo), so an
/// entry is usually just an action ID; the rest is what retail's recordings show (`questkit battle` writes these.)
///
/// Each time the NPC can act, the first entry that's ready is used:
/// * a weaponskill or spell (CooldownGroup 58) when `pace` seconds have passed since its last one, only as the next
///   step of its combo (right after its ActionCombo, within 15 s) if it has one;
/// * an ability (off the GCD) once its recast (or `every`) has passed, at most one between two GCD actions;
/// * `refresh`: not again on the same target for that long (a damage-over-time or debuff, re-applied as it wears off);
/// * `then`: the GCD action that has to follow this one (Life Surge, then Vorpal Thrust);
/// * `target_hp_below` / `self_hp_below`: only then (in %);
/// * `not_before`: only once its fight is that old;
/// * `on_ally` / `ally_hp_below`: on one of its own side, while there is one.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Rotation {
    /// Seconds between this NPC's weaponskills and spells. NPCs act far less often than players (retail's quest battle
    /// allies use one every 3-12 s); questkit reads it from a recording.
    #[serde(default = "default_pace")]
    pub pace: f32,
    pub actions: Vec<RotationAction>,
}

fn default_pace() -> f32 {
    2.5
}

/// One entry of a `Rotation`.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone, Default)]
pub struct RotationAction {
    pub action: u32,
    /// Use it at most this often, in seconds (instead of its recast.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub every: Option<f32>,
    /// Not again on the same target for this many seconds (its status's duration, less a little.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh: Option<f32>,
    /// The GCD action that must come next.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub then: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_hp_below: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub self_hp_below: Option<f32>,
    /// Use it on a living NPC of this BNpcBase on its own side (the lowest layout ID of them) instead of its target, and
    /// only while there is one: the Akoman's drain (1127) on the yodomi that feed it in quest battle 17 (Bob's
    /// Chronofoil capture df6f5af9.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_ally: Option<u32>,
    /// Use it on a living NPC of its own side (not itself) that's below this % of its health instead of its target
    /// (the lowest layout ID of them), and only while there is one. Castrum Meridianum's BNpcBase 14574 and its heal
    /// (29178), three uses in five retail runs: on allies at 9, 21 and 31 % of their health, the last with another
    /// at 23 % (so not the most hurt; each time the lowest layout ID of those below half), and none of the others,
    /// at 59 % and more (Bob's Chronofoil captures 9ba7d981, 6320ae56 and deea324c, 2026-11-17 to 19.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ally_hp_below: Option<f32>,
    /// Not until its fight is this many seconds old (from its first action of it). The same heal began 43.3, 43.4
    /// and 45.3 s after the NPC's first cast, in the first of its spell's turns from 43 s on, though allies had been
    /// at 6 to 8 % from 25 s (the two that lived 42 and 45 s never used it.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub not_before: Option<f32>,
    /// On one of those it's fighting, at random (its enmity list), instead of its target: a ground area aimed where
    /// they stand as it begins. Castrum Meridianum's BNpcBase 14572 and its 29176 (a 2.7 s cast on a spot), nine uses
    /// in five retail runs, each on the spot a member of the party stood on (0.0 to 0.1 yalms off), the tank's or
    /// another's, 0.8 to 8.9 yalms from the caster (Bob's Chronofoil captures af05916c, 9ba7d981, 50e72631, 6320ae56
    /// and deea324c; Bob's mark of 2026-11-20: they should be "cannon bombardment", not at a set place.)
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub at_random_foe: bool,
    /// Retail's first uses of it after the pull, in seconds (one per NPC that used it): each fight the NPC takes one of
    /// them at random as its `not_before`, so those of a kind don't all begin at the same second. Castrum Meridianum's
    /// 14570 first cast 29171 at 14.0, 16.2, 26.0, 33.4, 36.6, 37.6, 40.1 and 42.8 s in five retail runs (`questkit
    /// rates`); with one `not_before` (35, the median) every 14570 began it at 35 s (Bob, 2026-11-21: "can you have
    /// every caster of a kind vary like retail does").
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub first_uses: Vec<f32>,
    /// Entries with the same `slot` are one turn between them: it comes every `every` (counted from the last use of
    /// any of them), and one of them is used then, at random by `weight`. Castrum Meridianum's BNpcBase 14578, 14663
    /// and 14664 cast 29181 or 29182 (29183 or 29184) every 17.2 s, which of the two at random (all five retail runs,
    /// `questkit rates`): with an `every` of its own each, both came, twice as often as retail's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slot: Option<String>,
    /// How likely this entry of its `slot` is to be the one used (1 when not given.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<f32>,
    /// How likely it is to be used each time it comes due (0 to 1): when not, it waits `every` again as if it had been.
    /// Castrum Meridianum's packs of three 14577 cast 29175 once by one of the three in most runs (7 of 15 lives in
    /// five retail runs, `questkit rates`), not once by each.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chance: Option<f32>,
    /// Where the numbers came from (ignored by the server.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

/// See `Timeline::recorded_amounts`.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone, Default)]
pub struct RecordedAmount {
    /// The average amount against players.
    #[serde(default)]
    pub players: Option<f32>,
    /// The average amount against other NPCs.
    #[serde(default)]
    pub npcs: Option<f32>,
    /// The average against players by the NPC's level, for a kind met at several levels: the tainted earth sprite
    /// (BNpcBase 294) hit for 7 at level 5 in Trial by Earth and 20 at level 20 in Sylphie's Trials (Bob's Chronofoil
    /// capture b23b91ef). Its level's comes first, then `players`, then the nearest level's.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub players_by_level: BTreeMap<u8, f32>,
    /// For an attack that's weaker the further from the caster it lands: what it did to players at distances from
    /// the caster's middle (yalms, amount), in order; between two the amount goes straight from one to the other,
    /// beyond the first or last it's theirs. Every hit (on a player or an NPC) is its recorded amount times this
    /// amount over `players`. The Navel's Titan's Geocrush (651), from the arena's middle: 991 at 4.7 yalms, 367 at 22
    /// (17 hits on players in Bob's three runs fitted to a line, 1161 − 36.1 per yalm.)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub by_distance: Vec<(f32, f32)>,
}

impl RecordedAmount {
    /// How much of the recorded amount lands `distance` yalms from the caster (`by_distance`): 1 without.
    pub fn distance_factor(&self, distance: f32) -> f32 {
        let (Some(players), Some(first), Some(last)) = (
            self.players,
            self.by_distance.first(),
            self.by_distance.last(),
        ) else {
            return 1.0;
        };
        if players <= 0.0 {
            return 1.0;
        }
        let amount = if distance <= first.0 {
            first.1
        } else if distance >= last.0 {
            last.1
        } else {
            self.by_distance
                .windows(2)
                .find(|pair| distance <= pair[1].0)
                .map(|pair| {
                    let ((near, near_amount), (far, far_amount)) = (pair[0], pair[1]);
                    if far - near <= f32::EPSILON {
                        far_amount
                    } else {
                        near_amount + (far_amount - near_amount) * (distance - near) / (far - near)
                    }
                })
                .unwrap_or(last.1)
        };
        amount / players
    }
}

impl Timeline {
    /// What retail's recording says this NPC's `action_id` does to a player (`to_player`) or another NPC, if known.
    pub fn recorded_amount(&self, action_id: u32, to_player: bool) -> Option<f32> {
        let recorded = self.recorded_amounts.get(&action_id)?;
        if to_player {
            recorded.players
        } else {
            recorded.npcs
        }
    }

    /// `recorded_amount` for an NPC of `level`: against players, its level's `players_by_level` when there is one.
    pub fn recorded_amount_at(&self, action_id: u32, to_player: bool, level: u8) -> Option<f32> {
        let recorded = self.recorded_amounts.get(&action_id)?;
        // An NPC target with no amount of its own recorded is hit as a player was: across the timelines with both,
        // what an action did to NPCs was a median 0.98 of what it did to players. Without it the hit went unscaled,
        // 2.4-2.8 times as hard: In the Eyes of Gods and Men's wyvern's auto-attack on the gladiator did 229 where it
        // did 96 to players (Bob's bundle 20261110-095652.)
        if !to_player && recorded.npcs.is_some() {
            return recorded.npcs;
        }
        if let Some(amount) = recorded.players_by_level.get(&level) {
            return Some(*amount);
        }
        recorded.players.or_else(|| {
            recorded
                .players_by_level
                .iter()
                .min_by_key(|(recorded, _)| (**recorded as i32 - level as i32).abs())
                .map(|(_, amount)| *amount)
        })
    }

    /// How much of `action_id`'s recorded amount lands `distance` yalms from this NPC (`RecordedAmount::by_distance`.)
    pub fn distance_factor(&self, action_id: u32, distance: f32) -> f32 {
        self.recorded_amounts
            .get(&action_id)
            .map(|recorded| recorded.distance_factor(distance))
            .unwrap_or(1.0)
    }

    /// The average auto-attack damage this NPC does to players at `level`, if it's known.
    ///
    /// `health_at` is ordinary enemies' health by level, used to scale from the nearest recorded level.
    pub fn auto_attack_damage(
        &self,
        level: u8,
        health_at: impl Fn(u8) -> Option<u32>,
    ) -> Option<f32> {
        if let Some(damage) = self.damage_to_players.get(&level) {
            return Some(*damage);
        }

        let (nearest, damage) = self
            .damage_to_players
            .iter()
            .min_by_key(|(recorded, _)| (**recorded as i32 - level as i32).abs())?;
        let ratio = health_at(level)? as f32 / health_at(*nearest)? as f32;
        Some(damage * ratio)
    }
}

fn default_attack_range() -> f32 {
    crate::common::MINIMUM_PATHFINDING_DISTANCE
}

impl Timeline {
    /// Duration of the entire timeline in seconds.
    pub fn duration(&self) -> i32 {
        let mut duration = 0;
        for point in &self.timepoints {
            duration = duration.max(point.time as i32);
        }

        duration
    }

    /// Returns all points happening at the specified time, if any.
    pub fn points_at(&self, point: i32) -> Vec<&Timepoint> {
        self.timepoints
            .iter()
            .filter(|x| x.time == point as f32)
            .collect()
    }

    /// A `from_pull` timeline's points due in the tick from `from` up to (not including) `to` seconds after the pull:
    /// each at its `time`, then every `every` seconds after it if it repeats.
    pub fn points_between(&self, from: f32, to: f32) -> Vec<&Timepoint> {
        self.points_between_after(from, to, None)
    }

    /// `points_between` for the clock started by `anchor` (an action with `restarts`; `None` is the pull.)
    pub fn points_between_after(&self, from: f32, to: f32, anchor: Option<u32>) -> Vec<&Timepoint> {
        self.timepoints
            .iter()
            .filter(|point| point.hp_below.is_none() && point.after == anchor)
            .filter(|point| match point.every {
                Some(every) if every > 0.0 => {
                    let since = (from - point.time).max(0.0);
                    let next = point.time + (since / every).ceil() * every;
                    next >= from && next < to
                }
                _ => point.time >= from && point.time < to,
            })
            .collect()
    }
}

/// Represents a point on the timeline.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Timepoint {
    /// Seconds: from the NPC's spawn (whole), or from its pull with `Timeline::from_pull` (to the half second.)
    pub time: f32,
    /// With `Timeline::from_pull`: it comes again every this many seconds after `time` (Kottos' 28545 every 81 s,
    /// its 28543, 28546 and 28547 every 37), each point keeping its own period.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub every: Option<f32>,
    /// With `Timeline::from_pull`: its time counts from the last use of this action (one with `restarts`) instead of
    /// the pull, and it only comes while that's the last such: the Copperbell Mines' second boss's 28462 came 22.3 s
    /// after each of its 28463s, and its opening 28461 never once a 28463 had come first (Bob's three runs.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<u32>,
    /// With `Timeline::from_pull`: instead of a time, it comes once, as soon as the NPC's health is at or below this
    /// percentage (not mid-cast): the second boss's 28463 at 80, 50 and 25 % (75.5-76.9, 42.6-49.8 and 14.8-24.6 % as
    /// it began, in the three runs, its timing different in each.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hp_below: Option<f32>,
    /// Its use starts the clock again for the points `after` it (see `after`.)
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub restarts: bool,
    pub data: TimepointData,
}

/// Whom a timeline's action is used on.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone, Copy, Default)]
#[serde(rename_all = "snake_case")]
pub enum TimepointTarget {
    /// Its target (the one it's fighting.)
    #[default]
    Target,
    /// Someone else on its enmity list, at random (Kottos' 28547: a healer, a ninja, a lancer in the three runs,
    /// never the one it was fighting.) Its target when there's no one else.
    Other,
    /// Itself (an area around it, cast on itself: Kottos' 28546.)
    Itself,
    /// No one (0xE0000000), cast where it stands: Kottos' 28543.
    Nobody,
}

/// Which way an action cast on itself (`TimepointTarget::Itself`, an area in front of or around it) goes.
#[derive(Deserialize, Serialize, PartialEq, Debug, Clone, Copy, Default)]
#[serde(rename_all = "snake_case")]
pub enum TimepointFacing {
    /// Towards the one it's fighting.
    #[default]
    Target,
    /// Towards the nearest player, whoever it's fighting: The Lominsan Way's 1769 aimed every Rive (1299, a line) at
    /// the player, within a degree, while it fought the one it was there to kill (Bob's Chronofoil captures 44df02fc and
    /// c9145fdc: 19 casts in seven runs.)
    Player,
    /// Towards one of those on its enmity list, at random (whom it's fighting among them): the Praetorium's first boss
    /// aimed each Grand Sword (28473, a cone) within a hundredth of a radian at one of the party, a Thaumaturge, a
    /// White Mage, a Lancer, an Arcanist as often as the tank (nine casts in five retail runs, Bob's and Connor's
    /// captures.)
    AnyFoe,
    /// The way it stands, unturned: a director turned it (the Praetorium's second boss aimed each Augmented Uprising,
    /// 28478, straight at the middle from the edge it had leapt to, ten casts in five runs, whoever it fought.)
    AsItStands,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
#[serde(tag = "type")]
pub enum TimepointData {
    /// Represents doing an action.
    #[serde(rename = "action")]
    Action {
        /// Index into the Action Excel sheet.
        action_id: u32,
        /// A lock-on marker (Lockon sheet row) put over the target as the cast begins, as retail marks a tank-buster's
        /// target (218 on the Copperbell Mines' three bosses' busters: ActorControl 34 from the target.)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        lockon: Option<u32>,
        /// Whom it's used on (its target unless said.)
        #[serde(default, skip_serializing_if = "is_default_target")]
        target: TimepointTarget,
        /// For one cast on itself, which way it faces (its target unless said.)
        #[serde(default, skip_serializing_if = "is_default_facing")]
        facing: TimepointFacing,
        /// How near (yalms, centre to centre) it has to be to whom it's used on: further away, it walks there first and
        /// uses it once in reach (the rest of its timeline waiting meanwhile), as retail's Notorious Biggs boss left
        /// the fighter it fought to walk to whoever its 1068 was for and cast it from 9.1-10.5 yalms (Bob's Chronofoil
        /// captures 689b15da, da424662, 898dd0b9: four runs.) None: used wherever they are.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reach: Option<f32>,
    },
    /// Animates timelines for the gimmick this actor is bound to, such as Giant Clams.
    #[serde(rename = "timeline_state")]
    TimelineState { states: Vec<u32> },
    /// Changes the invulnerability state of this NPC.
    #[serde(rename = "invulnerability")]
    Invulnerability { invulnerable: bool },
}

fn is_default_target(target: &TimepointTarget) -> bool {
    *target == TimepointTarget::Target
}

fn is_default_facing(facing: &TimepointFacing) -> bool {
    *facing == TimepointFacing::Target
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shouts_go_through_their_lines_in_order() {
        let shouts = Shouts {
            lines: vec![0, 1331],
            repeat_last: false,
        };
        let said: Vec<_> = (1..=3).map(|nth| shouts.line(nth)).collect();
        assert_eq!(said, [None, Some(1331), None]);
        let every_time = Shouts {
            lines: vec![1350],
            repeat_last: true,
        };
        assert_eq!(every_time.line(5), Some(1350));
    }

    #[test]
    fn recorded_amount_by_level() {
        // The tainted earth sprite: 7 at level 5 (Trial by Earth), 21 at level 20 (Sylphie's Trials.)
        let timeline: Timeline = serde_json::from_str(
            r#"{"autoattack_action_id": 0, "timeline_always_plays": false, "timepoints": [],
                "recorded_amounts": {"970": {"players": 7.0, "npcs": 3.0, "players_by_level": {"5": 7.0, "20": 21.0}},
                                     "971": {"players_by_level": {"15": 16.0, "20": 20.0}}}}"#,
        )
        .unwrap();
        assert_eq!(timeline.recorded_amount_at(970, true, 20), Some(21.0));
        assert_eq!(timeline.recorded_amount_at(970, true, 12), Some(7.0)); // not recorded: `players`
        assert_eq!(timeline.recorded_amount_at(970, false, 20), Some(3.0));
        assert_eq!(timeline.recorded_amount_at(971, true, 18), Some(20.0)); // no `players`: the nearest level's
        assert_eq!(timeline.recorded_amount_at(1, true, 20), None);
    }

    #[test]
    fn recorded_amount_by_distance() {
        // The Navel's Geocrush: 991 at 4.7 yalms, 367 at 22, 443 the players' median.
        let timeline: Timeline = serde_json::from_str(
            r#"{"autoattack_action_id": 0, "timeline_always_plays": false, "timepoints": [],
                "recorded_amounts": {"651": {"players": 443.0, "npcs": 391.0, "by_distance": [[4.7, 991.0], [22.0, 367.0]]},
                                     "642": {"players": 175.0}}}"#,
        )
        .unwrap();
        let at = |distance: f32| timeline.distance_factor(651, distance) * 443.0;
        assert!((at(0.0) - 991.0).abs() < 0.1); // nearer than the first: the first's
        assert!((at(4.7) - 991.0).abs() < 0.1);
        assert!((at(13.35) - 679.0).abs() < 0.1); // halfway
        assert!((at(30.0) - 367.0).abs() < 0.1); // beyond the last: the last's
        assert_eq!(timeline.distance_factor(642, 10.0), 1.0); // none
        assert_eq!(timeline.distance_factor(1, 10.0), 1.0);
    }

    #[test]
    fn test_simple_example() {
        let json =
            std::fs::read_to_string("../resources/data/tests/example_timeline.json").unwrap();
        let timeline: Timeline = serde_json::from_str(&json).unwrap();

        assert_eq!(
            timeline,
            Timeline {
                autoattack_action_id: 872,
                timeline_always_plays: false,
                on_death: Vec::default(),
                attack_range: crate::common::MINIMUM_PATHFINDING_DISTANCE,
                damage_to_players: BTreeMap::default(),
                recorded_amounts: BTreeMap::default(),
                hitbox_radius: 0.0,
                animation_locks: BTreeMap::default(),
                rotation: None,
                shouts: BTreeMap::default(),
                fights_from: None,
                fixed: false,
                from_pull: false,
                timepoints: vec![Timepoint {
                    time: 20.0,
                    every: None,
                    after: None,
                    hp_below: None,
                    restarts: false,
                    data: TimepointData::Action {
                        action_id: 872,
                        lockon: None,
                        target: TimepointTarget::Target,
                        facing: TimepointFacing::Target,
                        reach: None,
                    }
                }]
            }
        );
    }

    /// Every timeline in resources/timelines reads, and the Copperbell Mines' last boss marks its tank-buster's target.
    #[test]
    fn every_timeline_reads() {
        for entry in std::fs::read_dir("../resources/timelines")
            .unwrap()
            .flatten()
        {
            let json = std::fs::read_to_string(entry.path()).unwrap();
            let timeline: Result<Timeline, _> = serde_json::from_str(&json);
            assert!(timeline.is_ok(), "{:?}: {:?}", entry.path(), timeline.err());
        }
        let json =
            std::fs::read_to_string("../resources/timelines/CopperbellBoss3_14537.json").unwrap();
        let timeline: Timeline = serde_json::from_str(&json).unwrap();
        assert!(timeline.timepoints.iter().any(|point| matches!(
            point.data,
            TimepointData::Action {
                action_id: 28764,
                lockon: Some(218),
                ..
            }
        )));
    }

    /// The server gives an NPC the first file it finds whose name ends `_<its BNpcBase>.json` (`Instance::insert_npc`),
    /// so two files for one base leave one of them unused. The helpers' base, 9020, had two from 2026-11-16 to
    /// 2026-11-18 (CopperbellHelper_9020.json and Helper_9020.json), and Cape Westwind's helpers' 28869 took 7 from Bob
    /// where retail's takes about 1060.
    #[test]
    fn one_timeline_file_for_each_base() {
        let mut seen: std::collections::BTreeMap<String, String> =
            std::collections::BTreeMap::new();
        for entry in std::fs::read_dir("../resources/timelines")
            .unwrap()
            .flatten()
        {
            let name = entry.file_name().to_string_lossy().to_string();
            let Some(base) = name
                .strip_suffix(".json")
                .and_then(|stem| stem.rsplit_once('_'))
                .map(|(_, base)| base.to_string())
                .filter(|base| base.chars().all(|c| c.is_ascii_digit()))
            else {
                continue;
            };
            if let Some(other) = seen.insert(base.clone(), name.clone()) {
                panic!(
                    "{other} and {name} are both for BNpcBase {base}: only one of them is ever read"
                );
            }
        }
        // And the one for the helpers has both duties' amounts.
        let json = std::fs::read_to_string("../resources/timelines/Helper_9020.json").unwrap();
        let timeline: Timeline = serde_json::from_str(&json).unwrap();
        for action in [28544, 28761, 28865, 28867, 28869] {
            assert!(timeline.recorded_amounts.contains_key(&action), "{action}");
        }
    }

    /// Kottos' timeline gives retail's schedule: the ticks its casts begin in, from its pull (Bob's three Copperbell
    /// Mines runs: 5.2, 15.3, 29.6, 39.1, 52.3, 66.6, 76.0, 86.2, 99.4.)
    #[test]
    fn kottos_schedule() {
        let json =
            std::fs::read_to_string("../resources/timelines/CopperbellBoss1_14460.json").unwrap();
        let timeline: Timeline = serde_json::from_str(&json).unwrap();
        assert!(timeline.from_pull);
        let mut casts = Vec::new();
        for tick in 0..200 {
            let from = tick as f32 * 0.5;
            for point in timeline.points_between(from, from + 0.5) {
                if let TimepointData::Action { action_id, .. } = point.data {
                    casts.push((from, action_id));
                }
            }
        }
        assert_eq!(
            casts,
            vec![
                (5.0, 28545),
                (15.0, 28543),
                (29.5, 28546),
                (39.0, 28547),
                (52.0, 28543),
                (66.5, 28546),
                (76.0, 28547),
                (86.0, 28545),
                (99.0, 28543),
            ]
        );
    }

    /// The second boss's points after the pull only while no 28463 has come, and its 28462 22 s after each.
    #[test]
    fn second_boss_restarts() {
        let json =
            std::fs::read_to_string("../resources/timelines/CopperbellBoss2_14448.json").unwrap();
        let timeline: Timeline = serde_json::from_str(&json).unwrap();
        let actions = |points: Vec<&Timepoint>| -> Vec<u32> {
            points
                .iter()
                .filter_map(|point| match point.data {
                    TimepointData::Action { action_id, .. } => Some(action_id),
                    _ => None,
                })
                .collect()
        };
        assert_eq!(
            actions(timeline.points_between_after(6.0, 6.5, None)),
            vec![28462]
        );
        assert_eq!(
            actions(timeline.points_between_after(17.5, 18.0, None)),
            vec![28461]
        );
        assert!(
            timeline
                .points_between_after(17.5, 18.0, Some(28463))
                .is_empty()
        );
        assert_eq!(
            actions(timeline.points_between_after(22.0, 22.5, Some(28463))),
            vec![28462]
        );
        assert_eq!(
            timeline
                .timepoints
                .iter()
                .filter(|point| point.hp_below.is_some() && point.restarts)
                .count(),
            3
        );
    }

    /// A pull-relative timeline's points come at their time, then every `every` seconds after it.
    #[test]
    fn points_every_so_often() {
        let timeline: Timeline = serde_json::from_str(
            r#"{"autoattack_action_id": 870, "timeline_always_plays": false, "from_pull": true, "timepoints": [
                {"time": 5.2, "every": 81, "data": {"type": "action", "action_id": 28545}},
                {"time": 15.3, "every": 37, "data": {"type": "action", "action_id": 28543, "target": "nobody"}}
            ]}"#,
        )
        .unwrap();
        let due = |from: f32| -> Vec<u32> {
            timeline
                .points_between(from, from + 0.5)
                .iter()
                .filter_map(|point| match point.data {
                    TimepointData::Action { action_id, .. } => Some(action_id),
                    _ => None,
                })
                .collect()
        };
        assert_eq!(due(5.0), vec![28545]);
        assert_eq!(due(5.5), Vec::<u32>::new());
        assert_eq!(due(15.0), vec![28543]);
        assert_eq!(due(52.0), vec![28543]);
        assert_eq!(due(86.0), vec![28545]);
        assert_eq!(due(0.0), Vec::<u32>::new());
    }

    /// Cape Westwind's boss, still above 15 % after the first pass: what Bob's Paladin saw him do from the pull
    /// (seconds after talk 33400, his Chronofoil capture 178b0de0 of 2026-11-18, visit 0.6.) The timeline went round
    /// the first pass again every 103.6 s until then, a guess.
    #[test]
    fn cape_westwinds_first_part_goes_round_as_retails_did() {
        let json =
            std::fs::read_to_string("../resources/timelines/CapeWestwindBoss_14581.json").unwrap();
        let timeline: Timeline = serde_json::from_str(&json).unwrap();
        let retail: [(f32, u32); 35] = [
            (6.1, 28871),
            (11.3, 28870),
            (21.6, 28864),
            (37.7, 28866),
            (52.9, 28872),
            (55.0, 28868),
            (68.2, 28872),
            (70.3, 28868),
            (82.4, 28871),
            (93.6, 28870),
            (103.8, 28870),
            (114.9, 28871),
            (126.1, 28864),
            (133.3, 28870),
            (139.5, 28870),
            (145.7, 28872),
            (147.7, 28871),
            (157.9, 28872),
            (160.0, 28868),
            (165.3, 28870),
            (171.4, 28872),
            (173.5, 28868),
            (178.7, 28870),
            (184.8, 28872),
            (186.9, 28871),
            (198.1, 28870),
            (210.2, 28866),
            (231.4, 28870),
            (243.6, 28864),
            (250.8, 28870),
            (257.1, 28870),
            (263.3, 28872),
            (265.4, 28871),
            (275.6, 28872),
            (277.7, 28868),
        ];
        let mut mine: Vec<(f32, u32)> = Vec::new();
        let mut at = 0.0;
        while at < 280.0 {
            for point in timeline.points_between(at, at + 0.1) {
                if let TimepointData::Action { action_id, .. } = point.data {
                    mine.push((at, action_id));
                }
            }
            at += 0.1;
        }
        assert_eq!(
            mine.iter().map(|(_, action)| *action).collect::<Vec<_>>(),
            retail.iter().map(|(_, action)| *action).collect::<Vec<_>>()
        );
        for ((at, action), (retail_at, _)) in mine.iter().zip(retail.iter()) {
            assert!(
                (at - retail_at).abs() <= 0.6,
                "{action} at {at:.1} s, retail's at {retail_at:.1} s"
            );
        }
    }

    #[test]
    fn recorded_amounts_by_target() {
        let timeline: Timeline = serde_json::from_str(
            r#"{"autoattack_action_id": 873, "timeline_always_plays": false, "timepoints": [],
                "recorded_amounts": {"873": {"npcs": 15.4}, "120": {"players": 95.0, "npcs": 87.0}}}"#,
        )
        .unwrap();
        assert_eq!(timeline.recorded_amount(873, false), Some(15.4));
        assert_eq!(timeline.recorded_amount(873, true), None);
        assert_eq!(timeline.recorded_amount(120, true), Some(95.0));
        assert_eq!(timeline.recorded_amount(7, false), None);
        // An NPC target with nothing of its own recorded is hit as a player was (no unscaled hits.)
        assert_eq!(timeline.recorded_amount_at(120, false, 40), Some(87.0));
        let wyvern: Timeline = serde_json::from_str(
            r#"{"autoattack_action_id": 870, "timeline_always_plays": false, "timepoints": [],
                "recorded_amounts": {"870": {"players": 96.0}}}"#,
        )
        .unwrap();
        assert_eq!(wyvern.recorded_amount_at(870, false, 41), Some(96.0));
    }

    #[test]
    fn auto_attack_damage_scales_from_the_nearest_level() {
        let timeline: Timeline = serde_json::from_str(
            r#"{"autoattack_action_id": 870, "timeline_always_plays": false, "timepoints": [],
                "damage_to_players": {"10": 8.5, "12": 12.4}}"#,
        )
        .unwrap();
        let health = |level: u8| Some(level as u32 * 10);

        assert_eq!(timeline.auto_attack_damage(10, health), Some(8.5));
        assert_eq!(
            timeline.auto_attack_damage(13, health),
            Some(12.4 * 130.0 / 120.0)
        );
        assert_eq!(
            timeline.auto_attack_damage(5, health),
            Some(8.5 * 50.0 / 100.0)
        );

        let none: Timeline = serde_json::from_str(
            r#"{"autoattack_action_id": 870, "timeline_always_plays": false, "timepoints": []}"#,
        )
        .unwrap();
        assert_eq!(none.auto_attack_damage(10, health), None);
    }
}
