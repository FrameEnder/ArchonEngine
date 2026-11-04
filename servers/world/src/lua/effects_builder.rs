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

/// Changes to a job gauge requested by an action script.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GaugeOp {
    /// Spend one Lily (White Mage), which also nourishes the Blood Lily if the trait is unlocked.
    SpendLily,
    /// Spend the fully bloomed Blood Lily (White Mage.)
    SpendBloodLily,
    /// Fill Aetherflow (Arcanist/Summoner: Energy Drain.)
    GainAetherflow,
    /// Spend one Aetherflow stack (Arcanist/Summoner: Fester, Painflare.)
    SpendAetherflow,
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
        methods.add_method_mut("gain_aetherflow", |_, this, _: ()| {
            this.gauge_ops.push(GaugeOp::GainAetherflow);
            Ok(())
        });
        methods.add_method_mut("spend_aetherflow", |_, this, _: ()| {
            this.gauge_ops.push(GaugeOp::SpendAetherflow);
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
