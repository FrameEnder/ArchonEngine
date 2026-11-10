use mlua::{FromLua, Lua, UserData, UserDataMethods, Value};

use kawari::ipc::zone::{DamageElement, DamageKind, DamageType, TargetEffect, TargetEffectKind};

/// What a periodic (over-time) effect does every tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PeriodicKind {
    /// Damage over time, applied to whoever has the status effect.
    Damage,
    /// Healing over time, applied to whoever has the status effect.
    Heal,
    /// MP restored over time, applied to whoever has the status effect.
    RestoreMp,
}

/// A periodic effect requested by an action script. The amount is calculated by the script when the action is used.
#[derive(Clone, Copy, Debug)]
pub struct PeriodicSpec {
    /// The status effect that drives this periodic effect. Ticks stop once it's gone.
    pub effect_id: u16,
    pub kind: PeriodicKind,
    /// The amount applied every tick.
    pub amount: u32,
    /// If true, this is applied to the caster instead of the action's targets.
    pub on_self: bool,
}

/// Aetherflow stacks Energy Drain fills (Arcanist/Summoner.)
pub const MAX_AETHERFLOW: u8 = 2;

/// Changes to a job gauge requested by an action script.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GaugeOp {
    /// Spend one Lily (White Mage), which also nourishes the Blood Lily if the trait is unlocked.
    SpendLily,
    /// Spend the fully bloomed Blood Lily (White Mage.)
    SpendBloodLily,
    /// Fill Aetherflow to this many stacks (Arcanist/Summoner: Energy Drain, two; Scholar: Aetherflow, three.)
    GainAetherflow(u8),
    /// Spend one Aetherflow stack (Arcanist/Summoner: Fester, Painflare.)
    SpendAetherflow,
    /// Add to (or take from) one of the job gauges' counters, within its range (`GaugeField::range`.)
    Add(GaugeField, i16),
    /// Set one of them.
    Set(GaugeField, i16),
}

/// The job gauges' counters scripts change by name (`effects:gauge_add("beast", 10)`, `player:gauge("beast")`); the
/// client's gauges are FFXIVClientStructs' (`server/job_gauge.rs` packs them.)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GaugeField {
    /// Warrior's Beast Gauge.
    Beast,
    /// Monk's chakra (up to ten under Brotherhood).
    Chakra,
    /// Black Mage's Astral Fire (1 to 3) or Umbral Ice (-1 to -3); 0 for neither.
    Element,
    /// Black Mage's Umbral Hearts.
    UmbralHearts,
    /// Black Mage's Polyglot.
    Polyglot,
    /// Black Mage's Paradox (ready).
    Paradox,
    /// Black Mage's Astral Soul.
    AstralSoul,
    /// Black Mage: tenths of a second towards the next Polyglot (`JobGauge::tick`).
    EnochianTimer,
    /// Paladin's Oath Gauge.
    Oath,
    /// Monk's Opo-opo's Fury.
    OpoFury,
    /// Monk's Raptor's Fury.
    RaptorFury,
    /// Monk's Coeurl's Fury.
    CoeurlFury,
    /// Monk's three Beast Chakra, one in each two bits (1 Opo-opo, 2 Raptor, 3 Coeurl), first in the lowest.
    BeastChakra,
    /// Monk's Nadi (1 Lunar, 2 Solar).
    Nadi,
    /// Dragoon: tenths of a second of Life of the Dragon left.
    LifeOfTheDragon,
    /// Dragoon's Firstminds' Focus.
    FirstmindsFocus,
    /// Bard's Repertoire.
    Repertoire,
    /// Bard's Soul Voice.
    SoulVoice,
    /// Bard's Codas (1 Mage's, 2 Army's, 4 Wanderer's).
    Coda,
    /// Ninja's Ninki.
    Ninki,
    /// Ninja's Kazematoi.
    Kazematoi,
    /// Ninja's mudras so far, one in each two bits (1 Ten, 2 Chi, 3 Jin), first in the lowest.
    Mudras,
    /// Summoner's attunement stacks.
    Attunement,
    /// Summoner's attunement (1 Ruby / fire, 2 Topaz / earth, 3 Emerald / wind).
    AttunementType,
    /// Summoner's arcana left (1 Ruby, 2 Topaz, 4 Emerald).
    Arcana,
    /// Scholar's Faerie Gauge.
    Fairy,
}

impl GaugeField {
    pub const ALL: [GaugeField; 26] = [
        GaugeField::Beast,
        GaugeField::Chakra,
        GaugeField::Element,
        GaugeField::UmbralHearts,
        GaugeField::Polyglot,
        GaugeField::Paradox,
        GaugeField::AstralSoul,
        GaugeField::EnochianTimer,
        GaugeField::Oath,
        GaugeField::OpoFury,
        GaugeField::RaptorFury,
        GaugeField::CoeurlFury,
        GaugeField::BeastChakra,
        GaugeField::Nadi,
        GaugeField::LifeOfTheDragon,
        GaugeField::FirstmindsFocus,
        GaugeField::Repertoire,
        GaugeField::SoulVoice,
        GaugeField::Coda,
        GaugeField::Ninki,
        GaugeField::Kazematoi,
        GaugeField::Mudras,
        GaugeField::Attunement,
        GaugeField::AttunementType,
        GaugeField::Arcana,
        GaugeField::Fairy,
    ];

    pub fn name(self) -> &'static str {
        match self {
            GaugeField::Beast => "beast",
            GaugeField::Chakra => "chakra",
            GaugeField::Element => "element",
            GaugeField::UmbralHearts => "umbral_hearts",
            GaugeField::Polyglot => "polyglot",
            GaugeField::Paradox => "paradox",
            GaugeField::AstralSoul => "astral_soul",
            GaugeField::EnochianTimer => "enochian_timer",
            GaugeField::Oath => "oath",
            GaugeField::OpoFury => "opo_fury",
            GaugeField::RaptorFury => "raptor_fury",
            GaugeField::CoeurlFury => "coeurl_fury",
            GaugeField::BeastChakra => "beast_chakra",
            GaugeField::Nadi => "nadi",
            GaugeField::LifeOfTheDragon => "lotd",
            GaugeField::FirstmindsFocus => "firstminds_focus",
            GaugeField::Repertoire => "repertoire",
            GaugeField::SoulVoice => "soul_voice",
            GaugeField::Coda => "coda",
            GaugeField::Ninki => "ninki",
            GaugeField::Kazematoi => "kazematoi",
            GaugeField::Mudras => "mudras",
            GaugeField::Attunement => "attunement",
            GaugeField::AttunementType => "attunement_type",
            GaugeField::Arcana => "arcana",
            GaugeField::Fairy => "fairy",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|field| field.name() == name)
    }

    /// Its place among `ALL`.
    pub fn index(self) -> usize {
        self as usize
    }

    /// The values it can take.
    pub fn range(self) -> (i16, i16) {
        match self {
            GaugeField::Beast => (0, 100),
            GaugeField::Chakra => (0, 10),
            GaugeField::Element => (-3, 3),
            GaugeField::UmbralHearts => (0, 3),
            GaugeField::Polyglot => (0, 3),
            GaugeField::Paradox => (0, 1),
            GaugeField::AstralSoul => (0, 6),
            GaugeField::EnochianTimer => (0, 300),
            GaugeField::Oath => (0, 100),
            GaugeField::OpoFury => (0, 1),
            GaugeField::RaptorFury => (0, 1),
            GaugeField::CoeurlFury => (0, 2),
            GaugeField::BeastChakra => (0, 63),
            GaugeField::Nadi => (0, 3),
            GaugeField::LifeOfTheDragon => (0, 300),
            GaugeField::FirstmindsFocus => (0, 2),
            GaugeField::Repertoire => (0, 4),
            GaugeField::SoulVoice => (0, 100),
            GaugeField::Coda => (0, 7),
            GaugeField::Ninki => (0, 100),
            GaugeField::Kazematoi => (0, 5),
            GaugeField::Mudras => (0, 63),
            GaugeField::Attunement => (0, 4),
            GaugeField::AttunementType => (0, 3),
            GaugeField::Arcana => (0, 7),
            GaugeField::Fairy => (0, 100),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct EffectsBuilder {
    pub effects: Vec<TargetEffect>,
    /// Over-time effects to start after the action resolves.
    pub periodic: Vec<PeriodicSpec>,
    /// If Some, replaces the MP cost from the Action sheet.
    pub mp_cost_override: Option<u16>,
    /// MP restored to the caster after the action resolves.
    pub restore_mp: u32,
    /// Status effects removed from the caster after the action resolves.
    pub consumed_effects: Vec<u16>,
    /// Status effects on the caster that lose a single stack after the action resolves.
    pub consumed_stacks: Vec<u16>,
    /// Job gauge changes to apply to the caster.
    pub gauge_ops: Vec<GaugeOp>,
    /// For area-of-effect damage, how much less damage (in percent) every target after the first takes.
    pub aoe_falloff_percent: u8,
    /// Its damage is surely critical, or a direct hit (Life Surge, Inner Release's Fell Cleave, Bootshine in Opo-opo
    /// form...): `guarantee_critical`, `guarantee_direct_hit`.
    pub critical_guaranteed: bool,
    pub direct_hit_guaranteed: bool,
    /// If true, heals restore the target to full HP regardless of their amount (e.g. Benediction.)
    pub heal_to_full: bool,
    /// If true, the caster goes to the top of the target's enmity (Provoke.)
    pub provoke: bool,
    /// If true, the target's first status that can be removed is (Esuna.)
    pub cleanse: bool,
    /// If true, a gaze: it lands only on those facing the caster (within 90 degrees of where they look); anyone looking
    /// away is missed. Retail's Akoman's 513 in quest battle 17 (Bob's Chronofoil capture df6f5af9: every target facing
    /// it within 58 degrees paralysed, every one turned 94 degrees or more away missed.)
    pub gaze: bool,
    /// If set, each player it lands on is walked towards the caster, whatever they do (`ForcedMoveTowards`), and drawn
    /// in a pose (ActorControl 82 with its value, retail's unknown), for a while: what a Seduced status does. Lady
    /// Amandine's Beguiling Mist (28643) in Haukke Manor (Bob's Chronofoil captures: speed 4, pose 1132, 3.0 s.)
    pub walk_to_caster: Option<WalkToCaster>,
}

/// Players walked towards the one using the action (`EffectsBuilder::walk_to_caster`.)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WalkToCaster {
    /// How long, in seconds.
    pub seconds: f32,
    /// ForcedMoveTowards' speed (retail's 4.)
    pub speed: u8,
    /// ActorControl 82's value on the player meanwhile (retail's 1132), 0 for none.
    pub pose: u32,
}

/// Status: a revive offered (Raise...), which the dead player can accept.
pub const STATUS_REVIVE_OFFER: u16 = 148;

/// The heal effect's flag (in its last parameter) saying it lands on the one using the action, e.g. a drain.
/// TODO: confirm against a retail drain's effect, which Kawari Bestiary 0.2 records.
pub const EFFECT_ON_SOURCE: u8 = 0x80;

impl UserData for EffectsBuilder {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut(
            "damage",
            |_, this, (damage_type, amount): (DamageType, u16)| {
                this.effects.push(TargetEffect(TargetEffectKind::Damage {
                    damage_kind: DamageKind::default(),
                    damage_type,
                    damage_element: DamageElement::Unaspected, // Will be filled in later
                    bonus_percent: 0,
                    unk3: 0,
                    unk4: 0,
                    amount,
                }));
                Ok(())
            },
        );
        methods.add_method_mut(
            "gain_effect",
            |_, this, (effect_id, param, duration): (u16, u16, f32)| {
                this.effects
                    .push(TargetEffect(TargetEffectKind::GainEffect {
                        unk1: 0,
                        unk2: 0,
                        unk3: 0,
                        effect_id,
                        duration,
                        param,
                    }));
                Ok(())
            },
        );
        // Raise and the like: the dead target is offered a revive (status 148, param 8, 60 s) as retail's effect
        // carries it (Bob's Chronofoil captures: 20 in the second byte.)
        methods.add_method_mut("offer_revive", |_, this, _: ()| {
            this.effects
                .push(TargetEffect(TargetEffectKind::GainEffect {
                    unk1: 0,
                    unk2: 20,
                    unk3: 0,
                    effect_id: STATUS_REVIVE_OFFER,
                    duration: 60.0,
                    param: 8,
                }));
            Ok(())
        });
        methods.add_method_mut(
            "gain_effect_self",
            |_, this, (effect_id, param, duration): (u16, u16, f32)| {
                this.effects
                    .push(TargetEffect(TargetEffectKind::GainEffectSelf {
                        unk1: 0,
                        unk2: 0,
                        // Retail's always has 0x80 here (5,517 of 5,517 in Bob's Chronofoil captures.)
                        unk3: 0x80,
                        effect_id,
                        duration,
                        param,
                    }));
                Ok(())
            },
        );
        // TODO: if we ever decide to redo the effectbuilder to not directly push actioneffects, this should also be redone
        // so we can remove the param arg. Assuming there isn't effects with the same ID but different params?
        methods.add_method_mut(
            "lose_effect",
            |_, this, (effect_id, effect_param): (u16, u16)| {
                this.effects
                    .push(TargetEffect(TargetEffectKind::LoseEffect {
                        param: effect_param,
                        unk: [0; 3],
                        effect_id,
                    }));
                Ok(())
            },
        );
        methods.add_method_mut("heal", |_, this, amount: u16| {
            this.effects.push(TargetEffect(TargetEffectKind::Heal {
                unk1: [0; 5],
                amount,
            }));
            Ok(())
        });
        // Restores HP to whoever used the action, e.g. a drain. Against players an NPC's drain gives back what the
        // action's damage took (see action.rs), so `amount` only matters otherwise.
        methods.add_method_mut("heal_self", |_, this, amount: u16| {
            this.effects.push(TargetEffect(TargetEffectKind::Heal {
                unk1: [0, 0, 0, 0, EFFECT_ON_SOURCE],
                amount,
            }));
            Ok(())
        });
        // Restores all of the target's HP.
        methods.add_method_mut("heal_to_full", |_, this, _: ()| {
            this.heal_to_full = true;
            this.effects.push(TargetEffect(TargetEffectKind::Heal {
                unk1: [0; 5],
                amount: u16::MAX, // Replaced with the real amount later
            }));
            Ok(())
        });
        // Players it lands on walk towards the caster for `seconds` (see `EffectsBuilder::walk_to_caster`.)
        methods.add_method_mut(
            "walk_to_caster",
            |_, this, (seconds, speed, pose): (f32, u8, Option<u32>)| {
                this.walk_to_caster = Some(WalkToCaster {
                    seconds,
                    speed,
                    pose: pose.unwrap_or_default(),
                });
                Ok(())
            },
        );
        // A gaze: only those facing the caster are affected (see `EffectsBuilder::gaze`.)
        methods.add_method_mut("gaze", |_, this, _: ()| {
            this.gaze = true;
            Ok(())
        });
        // The caster goes to the top of the target's enmity list.
        methods.add_method_mut("provoke", |_, this, _: ()| {
            this.provoke = true;
            Ok(())
        });
        // Removes the target's first status that can be (Status.CanDispel), e.g. Esuna.
        methods.add_method_mut("cleanse", |_, this, _: ()| {
            this.cleanse = true;
            Ok(())
        });
        methods.add_method_mut("interrupt", |_, this, _: ()| {
            this.effects
                .push(TargetEffect(TargetEffectKind::InterruptAction {}));
            Ok(())
        });
        // Knocks the target back by the Knockback sheet's row (the client moves them; see `TargetEffectKind::Knockback`.)
        // Nothing on a miss.
        methods.add_method_mut("knockback", |_, this, knockback_id: u16| {
            this.effects.push(TargetEffect(TargetEffectKind::Knockback {
                unk: [0; 5],
                knockback_id,
            }));
            Ok(())
        });
        methods.add_method_mut("play_vfx", |_, this, effect_id: u16| {
            this.effects.push(TargetEffect(TargetEffectKind::PlayVFX {
                unk: [0; 5],
                effect_id,
            }));
            Ok(())
        });
        methods.add_method_mut("summon_pet", |_, this, _: ()| {
            this.effects.push(TargetEffect(TargetEffectKind::SummonPet {
                unk: [0, 0, 0, 0, 128, 157, 0],
            }));
            Ok(())
        });
        methods.add_method_mut("execute_combo", |_, this, sequence: u8| {
            this.effects
                .push(TargetEffect(TargetEffectKind::ExecuteCombo {
                    sequence,
                    unk2: 0,
                    unk3: 0,
                    unk4: 0,
                    unk5: 128,
                    action_id: 0, // Filled in later
                }));
            Ok(())
        });
        methods.add_method_mut("summon_companion", |_, this, _: ()| {
            this.effects
                .push(TargetEffect(TargetEffectKind::SummonCompanion {
                    param0: 0,
                    param1: 0,
                    param2: 0,
                    param3: 0,
                    param4: 0,
                    value: 1,
                }));
            Ok(())
        });

        // Over-time effects

        // Gives the target(s) a status effect that deals `tick_amount` damage every 3 seconds while it lasts.
        methods.add_method_mut(
            "damage_over_time",
            |_, this, (effect_id, duration, tick_amount): (u16, f32, u32)| {
                this.effects
                    .push(TargetEffect(TargetEffectKind::GainEffect {
                        unk1: 0,
                        unk2: 0,
                        unk3: 0,
                        effect_id,
                        duration,
                        param: 0,
                    }));
                this.periodic.push(PeriodicSpec {
                    effect_id,
                    kind: PeriodicKind::Damage,
                    amount: tick_amount,
                    on_self: false,
                });
                Ok(())
            },
        );
        // Gives the target(s) a status effect that heals `tick_amount` every 3 seconds while it lasts.
        methods.add_method_mut(
            "heal_over_time",
            |_, this, (effect_id, duration, tick_amount): (u16, f32, u32)| {
                this.effects
                    .push(TargetEffect(TargetEffectKind::GainEffect {
                        unk1: 0,
                        unk2: 0,
                        unk3: 0,
                        effect_id,
                        duration,
                        param: 0,
                    }));
                this.periodic.push(PeriodicSpec {
                    effect_id,
                    kind: PeriodicKind::Heal,
                    amount: tick_amount,
                    on_self: false,
                });
                Ok(())
            },
        );
        // Gives the caster a status effect that heals it `tick_amount` every 3 seconds while it lasts: the Akoman's drain
        // (1127) gave it Regen (158) for 21 s, about 90 a tick (Bob's Chronofoil capture df6f5af9.)
        methods.add_method_mut(
            "heal_over_time_self",
            |_, this, (effect_id, duration, tick_amount): (u16, f32, u32)| {
                this.effects
                    .push(TargetEffect(TargetEffectKind::GainEffectSelf {
                        unk1: 0,
                        unk2: 0,
                        unk3: 0,
                        effect_id,
                        duration,
                        param: 0,
                    }));
                this.periodic.push(PeriodicSpec {
                    effect_id,
                    kind: PeriodicKind::Heal,
                    amount: tick_amount,
                    on_self: true,
                });
                Ok(())
            },
        );
        // Gives the caster a status effect that restores `tick_amount` MP every 3 seconds while it lasts.
        methods.add_method_mut(
            "restore_mp_over_time_self",
            |_, this, (effect_id, duration, tick_amount): (u16, f32, u32)| {
                this.effects
                    .push(TargetEffect(TargetEffectKind::GainEffectSelf {
                        unk1: 0,
                        unk2: 0,
                        unk3: 0,
                        effect_id,
                        duration,
                        param: 0,
                    }));
                this.periodic.push(PeriodicSpec {
                    effect_id,
                    kind: PeriodicKind::RestoreMp,
                    amount: tick_amount,
                    on_self: true,
                });
                Ok(())
            },
        );

        // Resources

        // Restores MP to the caster.
        methods.add_method_mut("restore_mp", |_, this, amount: u32| {
            this.restore_mp = this.restore_mp.saturating_add(amount);
            Ok(())
        });
        // Overrides the MP cost from the Action sheet (for example, 0 for a free cast.)
        methods.add_method_mut("set_mp_cost", |_, this, cost: u16| {
            this.mp_cost_override = Some(cost);
            Ok(())
        });
        // Removes a status effect from the caster once the action resolves.
        methods.add_method_mut("consume_effect", |_, this, effect_id: u16| {
            this.consumed_effects.push(effect_id);
            Ok(())
        });
        // Removes one stack of a status effect from the caster once the action resolves. The effect is removed at zero stacks.
        methods.add_method_mut("consume_effect_stack", |_, this, effect_id: u16| {
            this.consumed_stacks.push(effect_id);
            Ok(())
        });

        // Job gauges

        methods.add_method_mut("spend_lily", |_, this, _: ()| {
            this.gauge_ops.push(GaugeOp::SpendLily);
            Ok(())
        });
        methods.add_method_mut("spend_blood_lily", |_, this, _: ()| {
            this.gauge_ops.push(GaugeOp::SpendBloodLily);
            Ok(())
        });
        // Two stacks, or as many as given (`gain_aetherflow(3)` for the Scholar.)
        methods.add_method_mut("gain_aetherflow", |_, this, stacks: Option<u8>| {
            this.gauge_ops
                .push(GaugeOp::GainAetherflow(stacks.unwrap_or(MAX_AETHERFLOW)));
            Ok(())
        });
        methods.add_method_mut("spend_aetherflow", |_, this, _: ()| {
            this.gauge_ops.push(GaugeOp::SpendAetherflow);
            Ok(())
        });
        // Its damage is surely critical / a direct hit.
        methods.add_method_mut("guarantee_critical", |_, this, _: ()| {
            this.critical_guaranteed = true;
            Ok(())
        });
        methods.add_method_mut("guarantee_direct_hit", |_, this, _: ()| {
            this.direct_hit_guaranteed = true;
            Ok(())
        });
        // Changes a job gauge counter by name (`GaugeField`): `gauge_add("beast", -50)`, `gauge_set("element", 3)`.
        methods.add_method_mut("gauge_add", |_, this, (name, amount): (String, i16)| {
            let field = GaugeField::from_name(&name)
                .ok_or_else(|| mlua::Error::runtime(format!("no job gauge counter {name}")))?;
            this.gauge_ops.push(GaugeOp::Add(field, amount));
            Ok(())
        });
        methods.add_method_mut("gauge_set", |_, this, (name, value): (String, i16)| {
            let field = GaugeField::from_name(&name)
                .ok_or_else(|| mlua::Error::runtime(format!("no job gauge counter {name}")))?;
            this.gauge_ops.push(GaugeOp::Set(field, value));
            Ok(())
        });

        // Area-of-effect

        // Every target after the first takes `percent` less damage.
        methods.add_method_mut("aoe_falloff", |_, this, percent: u8| {
            this.aoe_falloff_percent = percent.min(100);
            Ok(())
        });
    }
}

impl FromLua for EffectsBuilder {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        match value {
            Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
            _ => unreachable!(),
        }
    }
}
