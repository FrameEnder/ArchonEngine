//! Keybound Brawler's casters, big enemies and summoned enemies: what `resources/data/keybound_mechanics.json` says
//! of them (its comment has the format and where the numbers came from), and the parts of them that need no game
//! running: which path a big enemy's rotation takes, where its summons stand, the barrier's look by its HP. The game
//! itself (`keybound.rs`) plays them.

use std::{collections::HashMap, sync::OnceLock};

use serde_json::Value;

/// What a caster does once its cast has completed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Then {
    /// It goes on as before (a marcher sets off.)
    Stay,
    /// It falls, as one felled.
    Fall,
    /// It's gone, as an enemy at the wave's end.
    Leave,
    /// Another kind takes its place, so long after.
    Become(u32, f32),
}

/// An enemy's own cast, begun `after` seconds after it gets its words.
#[derive(Debug, Clone, PartialEq)]
pub struct Caster {
    /// The casts its kind shares out (one each among those on the field), with their times.
    pub actions: Vec<(u32, f32)>,
    pub after: f32,
    pub cast: Spell,
    pub then: Then,
}

/// What a cast is, whoever's.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Spell {
    pub action: u32,
    pub time: f32,
    /// To the barrier as it completes.
    pub damage: u32,
    /// The stacks of the counter status it begins with (0: it can't be broken.)
    pub counter: u32,
    /// The status every player gets as it completes, and the lock's parameter for one whose client asks for it.
    pub warn: u32,
    pub lock: u32,
    /// The kind of its own effect as it completes without damage (27 unless the data says.)
    pub effect: u8,
    /// The helper's blows after it completes: the action, the damage, and when.
    pub helper: Option<(u32, u32, Vec<f32>)>,
}

/// A summoned enemy's (or any enemy's) extras.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Add {
    /// `(action, seconds after its SpawnNpc, effect kind)`.
    pub appear: Option<(u32, f32, u8)>,
    /// `(action, damage, seconds after its SpawnNpc)`.
    pub hit: Option<(u32, u32, f32)>,
    /// When it gets its words, in seconds after its SpawnNpc.
    pub words: Option<f32>,
    /// `(action, damage)` as it reaches the barrier.
    pub blast: Option<(u32, u32)>,
    /// Its own action as it falls.
    pub death: u32,
    /// The status it has from when it gets its words, and its action as it reaches the barrier (and leaves.)
    pub status: Option<(u16, u16)>,
    pub arrive: u32,
    /// An ActorControl as it gets its words: its category and first two numbers.
    pub control: Option<(u32, [u32; 2])>,
    pub spots: Vec<(f32, f32)>,
    pub charge: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Rule {
    Columns,
    Front,
    All,
    Free,
}

#[derive(Debug, Clone, PartialEq)]
pub enum What {
    Cast(Spell),
    /// An action at once, with an effect of this kind on itself (0: on no one.)
    Act(u32, u8),
    Spawn {
        base: u32,
        rule: Rule,
        count: usize,
    },
    /// `[notice, its argument, switch]` on for so many seconds, with the argument a status every player has
    /// meanwhile (its parameter the fourth number.)
    State {
        notice: u32,
        arg: u32,
        switch: u32,
        param: u32,
        seconds: f32,
    },
    Words(u32),
    ShieldOff,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    pub at: f32,
    pub what: What,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Block {
    pub length: f32,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Script {
    pub start: f32,
    pub paths: Vec<Vec<String>>,
    pub loop_from: usize,
    pub blocks: HashMap<String, Block>,
    /// A spawn repeated: `(base, rule, from, interval)`.
    pub every: Option<(u32, Rule, f32, f32)>,
}

#[derive(Debug, Default)]
pub struct Mechanics {
    pub helper: (u32, u32, u32),
    pub counter_status: u16,
    pub lock_status: u16,
    pub lock_seconds: f32,
    pub warn_seconds: f32,
    pub warn_after: f32,
    pub lock_after: f32,
    pub lock_notice: [u32; 2],
    /// The director trigger a client asks its lock with: its ID and last number.
    pub lock_trigger: (u32, u32),
    /// The ActorControl a broken cast is told with, and its second number.
    pub interrupted: (u32, u32),
    pub shield_notice: [u32; 2],
    /// How long after an action's effect its result came, by action (retail's; none: at once.)
    pub effect_after: HashMap<u32, f32>,
    pub casters: HashMap<u32, Caster>,
    pub adds: HashMap<u32, Add>,
    pub scripts: HashMap<u32, Script>,
}

fn number(value: &Value) -> f32 {
    value.as_f64().unwrap_or(0.0) as f32
}

fn whole(value: &Value) -> u32 {
    value.as_u64().unwrap_or(0) as u32
}

fn rule(value: &Value) -> Rule {
    match value.as_str() {
        Some("columns") => Rule::Columns,
        Some("front") => Rule::Front,
        Some("all") => Rule::All,
        _ => Rule::Free,
    }
}

fn spell(value: &Value, action: u32, time: f32) -> Spell {
    Spell {
        action,
        time,
        damage: whole(&value["damage"]),
        counter: whole(&value["counter"]),
        warn: whole(&value["warn"]),
        lock: whole(&value["lock"]),
        effect: value["effect"].as_u64().unwrap_or(27) as u8,
        helper: value["helper"].as_array().map(|helper| {
            (
                whole(&helper[0]),
                whole(&helper[1]),
                helper[2]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(number)
                    .collect(),
            )
        }),
    }
}

fn step(value: &Value) -> Option<Step> {
    let what = if let Some(action) = value["cast"].as_u64() {
        What::Cast(spell(value, action as u32, number(&value["time"])))
    } else if let Some(action) = value["act"].as_u64() {
        What::Act(action as u32, whole(&value["effect"]) as u8)
    } else if let Some(base) = value["spawn"].as_u64() {
        What::Spawn {
            base: base as u32,
            rule: rule(&value["rule"]),
            count: value["count"].as_u64().unwrap_or(1) as usize,
        }
    } else if let Some(state) = value["state"].as_array() {
        What::State {
            notice: whole(&state[0]),
            arg: whole(&state[1]),
            switch: whole(&state[2]),
            param: whole(&state[3]),
            seconds: number(&value["for"]),
        }
    } else if let Some(base) = value["words"].as_u64() {
        What::Words(base as u32)
    } else if value["shield_off"].as_bool() == Some(true) {
        What::ShieldOff
    } else {
        return None;
    };
    Some(Step {
        at: number(&value["at"]),
        what,
    })
}

/// The mechanics (`resources/data/keybound_mechanics.json`.)
pub fn mechanics() -> &'static Mechanics {
    static MECHANICS: OnceLock<Mechanics> = OnceLock::new();
    MECHANICS.get_or_init(|| {
        let value: Value = serde_json::from_str(include_str!(
            "../../../../resources/data/keybound_mechanics.json"
        ))
        .expect("resources/data/keybound_mechanics.json doesn't read");
        let keyed = |value: &Value| -> Vec<(u32, Value)> {
            value
                .as_object()
                .into_iter()
                .flatten()
                .filter_map(|(base, value)| Some((base.parse().ok()?, value.clone())))
                .collect()
        };
        let pair = |value: &Value| [whole(&value[0]), whole(&value[1])];
        let status = &value["status"];
        Mechanics {
            helper: (
                whole(&value["helper"]["base"]),
                whole(&value["helper"]["layout"]),
                whole(&value["helper"]["hp"]),
            ),
            counter_status: whole(&status["counter"]) as u16,
            lock_status: whole(&status["lock"]) as u16,
            lock_seconds: number(&status["lock_seconds"]),
            warn_seconds: number(&status["warn_seconds"]),
            warn_after: number(&status["warn_after"]),
            lock_after: number(&status["lock_after"]),
            lock_notice: pair(&status["lock_notice"]),
            lock_trigger: (
                whole(&status["lock_trigger"][0]),
                whole(&status["lock_trigger"][1]),
            ),
            interrupted: (
                whole(&status["interrupted"][0]),
                whole(&status["interrupted"][1]),
            ),
            shield_notice: pair(&value["shield"]["notice"]),
            effect_after: keyed(&value["effect_after"])
                .into_iter()
                .map(|(action, after)| (action, number(&after)))
                .collect(),
            casters: keyed(&value["casters"])
                .into_iter()
                .map(|(base, caster)| {
                    let actions: Vec<(u32, f32)> = caster["actions"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .zip(caster["time"].as_array().into_iter().flatten())
                        .map(|(action, time)| (whole(action), number(time)))
                        .collect();
                    let then = match caster["then"].as_str() {
                        Some("fall") => Then::Fall,
                        Some("leave") => Then::Leave,
                        Some("become") => {
                            Then::Become(whole(&caster["become"]), number(&caster["become_after"]))
                        }
                        _ => Then::Stay,
                    };
                    (
                        base,
                        Caster {
                            cast: spell(&caster, 0, 0.0),
                            actions,
                            after: number(&caster["after"]),
                            then,
                        },
                    )
                })
                .collect(),
            adds: keyed(&value["adds"])
                .into_iter()
                .map(|(base, add)| {
                    (
                        base,
                        Add {
                            appear: add["appear"].as_array().map(|appear| {
                                (
                                    whole(&appear[0]),
                                    number(&appear[1]),
                                    whole(&appear[2]) as u8,
                                )
                            }),
                            hit: add["hit"]
                                .as_array()
                                .map(|hit| (whole(&hit[0]), whole(&hit[1]), number(&hit[2]))),
                            words: add["words"].as_f64().map(|at| at as f32),
                            blast: add["blast"]
                                .as_array()
                                .map(|blast| (whole(&blast[0]), whole(&blast[1]))),
                            death: whole(&add["death"]),
                            status: add["status"]
                                .as_array()
                                .map(|status| (whole(&status[0]) as u16, whole(&status[1]) as u16)),
                            arrive: whole(&add["arrive"]),
                            control: add["control"].as_array().map(|control| {
                                (whole(&control[0]), [whole(&control[1]), whole(&control[2])])
                            }),
                            spots: add["spots"]
                                .as_array()
                                .into_iter()
                                .flatten()
                                .map(|spot| (number(&spot[0]), number(&spot[1])))
                                .collect(),
                            charge: add["charge"].as_bool().unwrap_or(false),
                        },
                    )
                })
                .collect(),
            scripts: keyed(&value["scripts"])
                .into_iter()
                .map(|(base, script)| {
                    (
                        base,
                        Script {
                            start: number(&script["start"]),
                            paths: script["paths"]
                                .as_array()
                                .into_iter()
                                .flatten()
                                .map(|path| {
                                    path.as_array()
                                        .into_iter()
                                        .flatten()
                                        .filter_map(|name| name.as_str().map(str::to_string))
                                        .collect()
                                })
                                .collect(),
                            loop_from: whole(&script["loop_from"]) as usize,
                            blocks: script["blocks"]
                                .as_object()
                                .into_iter()
                                .flatten()
                                .map(|(name, block)| {
                                    (
                                        name.clone(),
                                        Block {
                                            length: number(&block["length"]),
                                            steps: block["steps"]
                                                .as_array()
                                                .into_iter()
                                                .flatten()
                                                .filter_map(step)
                                                .collect(),
                                        },
                                    )
                                })
                                .collect(),
                            every: script["every"].as_object().map(|every| {
                                (
                                    whole(&every["spawn"]),
                                    rule(&every["rule"]),
                                    number(&every["from"]),
                                    number(&every["interval"]),
                                )
                            }),
                        },
                    )
                })
                .collect(),
        }
    })
}

/// Where a big enemy's rotation stands: which of its paths, which block of it, and how far into that block's steps.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Place {
    pub path: usize,
    pub block: usize,
    pub step: usize,
    /// Seconds into the block.
    pub clock: f32,
    /// How many of the repeated spawns have come.
    pub repeated: u32,
    /// Seconds since the rotation began.
    pub total: f32,
}

impl Script {
    /// The block a place is in.
    pub fn block(&self, place: &Place) -> Option<&Block> {
        self.blocks
            .get(self.paths.get(place.path)?.get(place.block)?)
    }

    /// The rotation `seconds` on: the steps that came due, in order (a block's steps up to its length, then the
    /// next block's; after the path's last, from `loop_from` again.)
    pub fn advance(&self, place: &mut Place, seconds: f32) -> Vec<What> {
        let mut due = Vec::new();
        place.clock += seconds;
        place.total += seconds;
        if let Some((base, rule, from, interval)) = self.every
            && interval > 0.0
        {
            while place.total >= from + interval * place.repeated as f32 {
                place.repeated += 1;
                due.push(What::Spawn {
                    base,
                    rule,
                    count: 1,
                });
            }
        }
        // A path of nothing but empty blocks would never end.
        for _ in 0..64 {
            let Some(block) = self.block(place) else {
                break;
            };
            while let Some(step) = block.steps.get(place.step) {
                if step.at > place.clock.min(block.length) {
                    break;
                }
                due.push(step.what.clone());
                place.step += 1;
            }
            if place.clock < block.length || block.length <= 0.0 {
                break;
            }
            place.clock -= block.length;
            place.step = 0;
            place.block += 1;
            if place.block >= self.paths[place.path].len() {
                place.block = self.loop_from.min(self.paths[place.path].len() - 1);
            }
        }
        due
    }
}

/// Where `count` summons of a rule stand: `spots` their kind's, `taken` those someone stands on, `pick` a number
/// below its argument.
pub fn stand(
    rule: Rule,
    count: usize,
    spots: &[(f32, f32)],
    taken: &[(f32, f32)],
    mut pick: impl FnMut(usize) -> usize,
) -> Vec<(f32, f32)> {
    let same = |a: &(f32, f32), b: &(f32, f32)| (a.0 - b.0).abs() < 0.1 && (a.1 - b.1).abs() < 0.1;
    let mut columns: Vec<f32> = Vec::new();
    let mut rows: Vec<f32> = Vec::new();
    for (x, z) in spots {
        if !columns.iter().any(|c| (c - x).abs() < 0.1) {
            columns.push(*x);
        }
        if !rows.iter().any(|r| (r - z).abs() < 0.1) {
            rows.push(*z);
        }
    }
    rows.sort_by(|a, b| a.total_cmp(b));
    match rule {
        Rule::All => spots.to_vec(),
        // One in each column, each in a row of its own (retail's four bombs: 480 of 480.)
        Rule::Columns => {
            let mut left = rows.clone();
            columns
                .iter()
                .filter_map(|x| {
                    if left.is_empty() {
                        return None;
                    }
                    let z = left.remove(pick(left.len()));
                    Some((*x, z))
                })
                .collect()
        }
        // The row farthest from the barrier.
        Rule::Front => {
            let Some(z) = rows.first() else {
                return Vec::new();
            };
            (0..count.min(columns.len()))
                .map(|_| (columns[pick(columns.len())], *z))
                .collect()
        }
        Rule::Free => {
            let mut free: Vec<(f32, f32)> = spots
                .iter()
                .filter(|spot| !taken.iter().any(|other| same(spot, other)))
                .copied()
                .collect();
            let mut stood = Vec::new();
            while stood.len() < count && !free.is_empty() {
                stood.push(free.remove(pick(free.len())));
            }
            stood
        }
    }
}

/// The barrier's look by its HP: 0 whole, 1 from 20,300 down, 2 from 10,280 down (retail's blows on it played
/// timeline 2 down to 20,400 left, 4 from 20,300 to 10,400, 8 from 10,280: 6,324 blows.)
pub fn barrier_look(hp: u32) -> usize {
    if hp > 20_300 {
        0
    } else if hp > 10_280 {
        1
    } else {
        2
    }
}

/// The barrier's map effects, as `(state, timeline, index)`: a blow on it by its look, its look changing, its fall,
/// whole again, a word on its words, its shield going up and taking a blow.
pub const MAP_BLOW: [(u16, u16, u8); 3] = [(1, 2, 1), (1, 4, 1), (1, 8, 1)];
pub const MAP_LOOK: [(u16, u16, u8); 3] = [(1, 2, 0), (0x10, 0x20, 0), (0x40, 0x80, 0)];
pub const MAP_FALLEN: (u16, u16, u8) = (4, 0x400, 0);
pub const MAP_CHARGE: (u16, u16, u8) = (1, 0x10, 1);
pub const MAP_SHIELD: (u16, u16, u8) = (0x100, 0x200, 0);
pub const MAP_ABSORB: (u16, u16, u8) = (1, 0x20, 1);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_mechanics_read() {
        let mechanics = mechanics();
        assert_eq!(mechanics.helper, (9020, 12_572_305, 44));
        assert_eq!(mechanics.counter_status, 5354);
        assert_eq!(mechanics.lock_trigger, (0x1000_0002, 365));
        // The three 19590s' casts, and what a 19590 does after its own.
        let caster = &mechanics.casters[&19590];
        assert_eq!(caster.actions, [(49526, 5.7), (49527, 8.2), (49528, 10.7)]);
        assert_eq!(caster.cast.damage, 1500);
        assert_eq!(caster.then, Then::Fall);
        assert_eq!(mechanics.casters[&19589].cast.counter, 4);
        assert_eq!(mechanics.casters[&19827].then, Then::Become(19828, 0.7));
        assert_eq!(mechanics.adds[&19825].spots.len(), 16);
        assert!(mechanics.adds[&20079].charge);
        // Every block a path names is there, and every summon has its spots and its kind.
        for (base, script) in &mechanics.scripts {
            assert!(!script.paths.is_empty(), "{base}");
            for path in &script.paths {
                assert!(script.loop_from < path.len(), "{base}");
                for name in path {
                    let block = script
                        .blocks
                        .get(name)
                        .unwrap_or_else(|| panic!("{base}: {name}"));
                    assert!(block.length > 0.0);
                    for step in &block.steps {
                        assert!(step.at <= block.length, "{base}: {name}");
                        if let What::Spawn { base, .. } | What::Words(base) = &step.what {
                            assert!(!mechanics.adds[base].spots.is_empty(), "{base}");
                            assert!(
                                crate::server::keybound::waves().enemies.contains_key(base),
                                "{base}"
                            );
                        }
                    }
                }
            }
        }
        for base in mechanics.casters.keys().chain(mechanics.adds.keys()) {
            assert!(
                crate::server::keybound::waves().enemies.contains_key(base),
                "{base}"
            );
        }
    }

    /// The last boss's rotation, as retail's ran: its long cast with the barrier's words 0.2 s later, the shield gone
    /// 35.0 s in, and what follows 42.1 s in.
    #[test]
    fn a_rotation_s_steps_come_in_order() {
        let script = &mechanics().scripts[&19596];
        let mut place = Place::default();
        let first = script.advance(&mut place, 0.0);
        assert!(
            matches!(&first[..], [What::Cast(spell)] if spell.action == 49699 && spell.time == 29.7)
        );
        assert_eq!(script.advance(&mut place, 0.25), [What::Words(20079)]);
        assert!(script.advance(&mut place, 30.0).is_empty());
        assert_eq!(script.advance(&mut place, 5.0), [What::ShieldOff]);
        // Into the second block: the first path's is the part that begins with 49710.
        let next = script.advance(&mut place, 7.0);
        assert_eq!(place.block, 1);
        assert_eq!(next.len(), 1);
        assert!(matches!(&next[0], What::Cast(spell) if spell.action == 49710));
        // A long turn skips nothing: every step of the blocks gone through comes.
        let mut place = Place::default();
        let all = script.advance(&mut place, 42.1 + 56.6 + 8.2 + 42.1 + 62.8 + 10.2 + 0.1);
        assert_eq!(place.block, 6);
        assert_eq!(
            all.iter()
                .filter(|what| matches!(what, What::Cast(spell) if spell.action == 49699))
                .count(),
            2
        );
        assert!(matches!(all.last(), Some(What::Cast(spell)) if spell.action == 49711));
        // The last block goes round.
        assert!(
            matches!(&script.advance(&mut place, 12.15)[..], [What::Cast(spell)] if spell.action == 49711)
        );
        assert_eq!(place.block, 6);
    }

    /// 19595's summons come every 5.0 s whatever its blocks do.
    #[test]
    fn a_repeated_spawn() {
        let script = &mechanics().scripts[&19595];
        let mut place = Place::default();
        let due = script.advance(&mut place, 14.3);
        let spawns = due
            .iter()
            .filter(|what| {
                matches!(
                    what,
                    What::Spawn {
                        base: 19829,
                        rule: Rule::Free,
                        count: 1
                    }
                )
            })
            .count();
        // 4.2, 9.2 and 14.2 s in.
        assert_eq!(spawns, 3);
        assert!(
            due.iter()
                .any(|what| matches!(what, What::Cast(spell) if spell.action == 49554))
        );
    }

    #[test]
    fn where_summons_stand() {
        let mechanics = mechanics();
        let bombs = &mechanics.adds[&19825].spots;
        let mut next = 0;
        let stood = stand(Rule::Columns, 4, bombs, &[], |below| {
            next += 1;
            next % below
        });
        assert_eq!(stood.len(), 4);
        for (index, (x, z)) in stood.iter().enumerate() {
            for (other_x, other_z) in &stood[index + 1..] {
                assert_ne!(x, other_x);
                assert_ne!(z, other_z);
            }
        }
        assert_eq!(stand(Rule::Front, 1, bombs, &[], |_| 2), [(2.5, -18.0)]);
        assert_eq!(
            stand(Rule::All, 4, &mechanics.adds[&19827].spots, &[], |_| 0).len(),
            4
        );
        // A free spot is one nobody stands on.
        let spots = &mechanics.adds[&19829].spots;
        let taken: Vec<(f32, f32)> = spots[1..].to_vec();
        assert_eq!(stand(Rule::Free, 3, spots, &taken, |_| 0), [spots[0]]);
    }

    #[test]
    fn the_barrier_s_look() {
        assert_eq!(barrier_look(40_000), 0);
        assert_eq!(barrier_look(20_400), 0);
        assert_eq!(barrier_look(20_300), 1);
        assert_eq!(barrier_look(10_400), 1);
        assert_eq!(barrier_look(10_280), 2);
        assert_eq!(barrier_look(0), 2);
    }
}
