//! What Bob's bundles showed, kept as tests (docs/src/fork/testing.md): each puts the bundle's situation into a
//! `TestWorld` and checks what the fix made the server do, so the fix stays. Named by what he wrote, with the
//! bundle. Only what needs none of the game's data fits here.

use std::time::Duration;

use kawari::{
    common::{CharacterMode, HandlerType, ObjectId, WarpType},
    ipc::zone::{
        ActorControlCategory, Condition, ServerZoneIpcData, TargetEffect, TargetEffectKind,
    },
};

use crate::{
    FromServer,
    server::{
        action::{
            STATUS_RAISED, cannot_be_hurt, ends_a_cast, ends_protection, keep_for_the_dead,
            toss_target, tossed_by,
        },
        actor::{NetworkedActor, NpcState, kill_actor},
        director::{duty_flags_due, engage_npc, ready_to_begin, waits_for_its_opening, won},
        modifiers,
        npc_behavior::{CastAim, cast_aim, out_of_the_fight, retail_cast_aim, unnoticed},
        party::tell_cutscene_state,
        party_bot,
        test_world::{TestWorld, control_from, packet_from},
    },
};

const PLAYER: u32 = 0x1000_0001;
const BOT: u32 = 0x4000_0011;
const ENEMY: u32 = 0x4000_0021;
const CURE_II: u32 = 135;

/// Bundle 20261110-141944, "it all sees me as casting": the player fell 1.5 s into a Cure II, the cast's task went
/// with the rest of theirs, and nothing ever took Casting off, so after the wipe's reset the client refused every
/// action. Retail's Condition at a death mid-cast has no Casting, and no CancelCast comes.
#[test]
fn a_player_who_dies_mid_cast_stops_casting() {
    let mut world = TestWorld::new();
    let player = world.player(PLAYER, [0.0, 4.0, 290.0]);
    world.casting(player, CURE_II, Duration::from_millis(500), player);
    assert_eq!(world.casts_queued(player), 1);
    world.sent(player);

    world
        .instance
        .find_actor_mut(player)
        .unwrap()
        .get_common_spawn_mut()
        .health_points = 0;
    kill_actor(world.network.clone(), &mut world.instance, player);

    let sent = world.sent(player);
    assert!(
        sent.iter()
            .any(|m| matches!(m, FromServer::ToggleCondition(Condition::Casting, false))),
        "Casting is taken off as they die: {sent:?}"
    );
    assert!(
        !sent.iter().any(|m| matches!(
            m,
            FromServer::ActorControlSelf(ActorControlCategory::CancelCast { .. })
        )),
        "retail sends no CancelCast for a cast a death cut short"
    );
    assert_eq!(world.casts_queued(player), 0, "the cast never lands");
    assert_eq!(
        world
            .instance
            .find_actor(player)
            .unwrap()
            .get_common_spawn()
            .mode,
        CharacterMode::Dead
    );
}

/// Bundle 20261110-141944, "on reset the party list broke": at a wipe the Duty Finder's bots were taken away
/// (DeleteActor) and spawned again, which retail does to Duty Support's members and never to a party's players
/// (three wipes in parties of players: each member moved with an ActorSetPos of the player's own warp type, their
/// mode back to normal, alive and whole.)
#[test]
fn a_player_npc_goes_back_at_a_wipe_without_leaving() {
    let mut world = TestWorld::new();
    let player = world.player(PLAYER, [0.0, 4.0, 306.0]);
    let bot = world.player_npc(BOT, player, [3.0, 4.0, 250.0]);
    let enemy = world.enemy(ENEMY, [3.0, 4.0, 248.0], 9000);
    // The bot fell fighting; the enemy still hates it.
    if let Some(NetworkedActor::Npc {
        spawn,
        state,
        enmity,
        ..
    }) = world.instance.find_actor_mut(bot)
    {
        spawn.common.health_points = 0;
        spawn.common.mode = CharacterMode::Dead;
        *state = NpcState::Dead;
        enmity.insert(ObjectId(ENEMY), 50);
    }
    world.instance.party_bots.get_mut(&bot).unwrap().dead_since = Some(std::time::Instant::now());
    world.sent(player);

    party_bot::back_to_the_start(&world.network.clone(), &mut world.instance, bot, player);

    let sent = world.sent(player);
    assert!(
        !sent
            .iter()
            .any(|m| matches!(m, FromServer::DeleteActor(..) | FromServer::ActorSpawn(..))),
        "it's neither taken away nor spawned again: {sent:?}"
    );
    assert!(
        sent.iter().any(|m| packet_from(m, bot, |data| matches!(
            data,
            ServerZoneIpcData::ActorSetPos(set) if matches!(set.warp_type, WarpType::Event) && set.arg == 2
        ))),
        "it's moved as retail moved a party member: {sent:?}"
    );
    assert!(
        sent.iter()
            .any(|m| control_from(m, bot, |category| matches!(
                category,
                ActorControlCategory::ZoneIn {
                    warp_finish_anim: 1,
                    ..
                }
            ))),
        "it shows the arrival: {sent:?}"
    );
    let Some(NetworkedActor::Npc {
        spawn,
        state,
        enmity,
        ..
    }) = world.instance.find_actor(bot)
    else {
        panic!("the bot is still in the instance");
    };
    assert_eq!(
        (spawn.common.health_points, spawn.common.mode),
        (spawn.common.max_health_points, CharacterMode::Normal)
    );
    assert_eq!(*state, NpcState::Follow);
    assert!(enmity.is_empty());
    // Beside the player, at the start.
    assert!((spawn.common.position.0.z - 306.0).abs() < 5.0);
    assert!(world.instance.party_bots[&bot].dead_since.is_none());
    // The enemy is untouched by this (the wipe's own reset sends it home.)
    assert!(world.instance.find_actor(enemy).is_some());
}

/// Bundle 20261110-182408, "isgebind's aoes are showing in the middle rather than their intended targeted areas":
/// the helpers' Sheets of Ice (1024, placed on the ground) were cast at each of the party, and their ActorCasts
/// carried the helper's own position and the target's ID. Retail's carry the spot and no target; a cast on someone
/// carries their position and their ID.
#[test]
fn a_cast_is_aimed_where_its_target_stands() {
    let member = ObjectId(BOT);
    let spot = glam::Vec3A::new(17.75, 0.05, -253.44);
    let (target, position) = cast_aim(CastAim::Ground, member, spot);
    assert_eq!(
        target,
        ObjectId::default(),
        "placed on the ground: no target"
    );
    assert_eq!(position.0, spot);
    let (target, position) = cast_aim(CastAim::Target, member, spot);
    assert_eq!((target, position.0), (member, spot));
}

/// Bundle 20261110-182408, "tornado doesn't throw character in air when ran into": retail's whirlwind hit tossed
/// whoever it caught where they stood (ActorControl 220, GimmickJump row 64, the toss's number in the duty.)
#[test]
fn a_whirlwind_tosses_whoever_it_hits() {
    let mut world = TestWorld::new();
    let player = world.player(PLAYER, [44.79, 4.0, -72.57]);
    let watcher = world.player(PLAYER + 1, [40.0, 4.0, -77.0]);
    let toss = TargetEffect(TargetEffectKind::SummonPet {
        unk: [64, 0, 0, 0, 0, 173, 0],
    });
    // A pet's summoning (the same kind of effect, no row) tosses nobody.
    let pet = TargetEffect(TargetEffectKind::SummonPet {
        unk: [0, 0, 0, 0, 128, 157, 0],
    });
    assert_eq!(tossed_by(std::slice::from_ref(&pet)), None);
    assert_eq!(tossed_by(&[pet, toss]), Some(64));
    world.sent(player);
    world.sent(watcher);

    toss_target(
        &mut world.network.lock(),
        &mut world.instance,
        player,
        &[pet],
    );
    assert!(world.sent(player).is_empty());
    for number in 1..=2 {
        toss_target(
            &mut world.network.lock(),
            &mut world.instance,
            player,
            &[toss],
        );
        let landing = |category: &ActorControlCategory| {
            matches!(category, ActorControlCategory::ExecuteGimmickJump {
                landing_position_x, landing_position_z, gimmick_jump_type: 64, unk1, ..
            } if (*landing_position_x - 44.79).abs() < 0.01
                && (*landing_position_z + 72.57).abs() < 0.01
                && *unk1 == number)
        };
        let own = world.sent(player);
        assert!(
            own.iter()
                .any(|m| matches!(m, FromServer::ActorControlSelf(category) if landing(category))),
            "the one hit is tossed where they stand: {own:?}"
        );
        let seen = world.sent(watcher);
        assert!(
            seen.iter().any(|m| control_from(m, player, landing)),
            "and everyone sees it: {seen:?}"
        );
    }
}

/// Bob, 2026-11-10: "characters do not take damage for the next indeterminate time after a revive if they stand still,
/// and do not use actions for that period of time". Status 418 (Transcendent) was given at a raise and did nothing:
/// a raised player or bot took every hit. Now no damage reaches whoever has it, their first action takes it off, and
/// a raised bot uses no action while it lasts (it still moves: moving doesn't end it.)
#[test]
fn someone_just_raised_takes_no_damage_until_they_act() {
    let mut world = TestWorld::new();
    let player = world.player(PLAYER, [0.0, 4.0, 290.0]);
    let bot = world.player_npc(BOT, player, [2.0, 4.0, 290.0]);
    for id in [player, bot] {
        let actor = world.instance.find_actor_mut(id).unwrap();
        assert!(
            !cannot_be_hurt(actor),
            "nothing protects them before the raise"
        );
        actor
            .status_effects_mut()
            .unwrap()
            .add(STATUS_RAISED, 0, 5.0, ObjectId(0xE000_0000));
        assert!(cannot_be_hurt(world.instance.find_actor(id).unwrap()));
    }
    let bot_statuses = world
        .instance
        .find_actor(bot)
        .unwrap()
        .status_effects()
        .unwrap();
    assert!(
        modifiers::holds_actions(bot_statuses),
        "a raised bot uses no action"
    );
    assert!(!modifiers::is_rooted(bot_statuses), "but it moves");

    world.sent(player);
    ends_protection(&world.network, &mut world.instance, player);
    assert!(!cannot_be_hurt(world.instance.find_actor(player).unwrap()));
    let sent = world.sent(player);
    assert!(
        sent.iter().any(|message| matches!(
            message,
            FromServer::ActorControlSelf(ActorControlCategory::LoseEffect { effect_id, .. })
                if *effect_id == STATUS_RAISED as u32
        )),
        "they see it end: {sent:?}"
    );
}

/// Bundle 20261111-072219, "some of the enemies didn't have their AOEs correctly set" (quest battle 86): Kawari aimed
/// every cast at the enemy's target, where retail's NPCs cast 866 on themselves (437 of 437 casts) and 1070 and 1304 at
/// no one. Each action's aim is retail's (`resources/data/cast_aims.json`); one not seen goes by its CastType.
#[test]
fn an_npc_cast_is_aimed_as_retails_was() {
    assert_eq!(retail_cast_aim(866, Some(2)), CastAim::Caster);
    assert_eq!(retail_cast_aim(1070, Some(1)), CastAim::Ground);
    assert_eq!(retail_cast_aim(1304, None), CastAim::Ground);
    assert_eq!(retail_cast_aim(999_999, Some(7)), CastAim::Ground);
    assert_eq!(retail_cast_aim(999_999, Some(2)), CastAim::Target);
    assert_eq!(retail_cast_aim(999_999, None), CastAim::Target);
}

/// Bundle 20261111-072219, "not all of the sirens effects are being replicated": a heal landed on someone already
/// dead. Nothing lands on the dead but a raise's offer.
#[test]
fn nothing_but_a_raise_lands_on_the_dead() {
    let mut effects = vec![
        TargetEffect(TargetEffectKind::Heal {
            unk1: [0; 5],
            amount: 500,
        }),
        TargetEffect(TargetEffectKind::GainEffect {
            unk1: 0,
            unk2: 0,
            param: 0,
            unk3: 0,
            effect_id: 148,
            duration: 60.0,
        }),
    ];
    keep_for_the_dead(&mut effects);
    assert_eq!(effects.len(), 1);
    assert!(matches!(
        effects[0].0,
        TargetEffectKind::GainEffect { effect_id: 148, .. }
    ));
}

/// Bundle 20261111-072219, "not all of the sirens effects are being replicated": retail's talker of quest battle 86
/// drew its weapon, went red and targeted its foe as the fight began, and put its weapon away after (Bob's Chronofoil
/// captures); Kawari's never did. `engage_npc` sends what retail's did.
#[test]
fn an_npc_engages_and_disengages_as_retails_did() {
    let mut world = TestWorld::new();
    let player = world.player(PLAYER, [0.0, 0.0, 0.0]);
    let enemy = world.enemy(ENEMY, [5.0, 0.0, 0.0], 1000);
    let helper = world.enemy(BOT, [8.0, 0.0, 0.0], 1000);
    world.sent(player);
    engage_npc(
        &mut world.network.lock(),
        &mut world.instance,
        enemy,
        Some(helper),
    );
    let sent = world.sent(player);
    assert!(
        sent.iter()
            .any(|message| control_from(message, enemy, |category| matches!(
                category,
                ActorControlCategory::ToggleWeapon { shown: true, .. }
            )))
    );
    assert!(
        sent.iter()
            .any(|message| control_from(message, enemy, |category| matches!(
                category,
                ActorControlCategory::SetBattle { battle: true }
            )))
    );
    assert!(sent.iter().any(|message| matches!(message,
        FromServer::ActorControlTarget(from, target, ActorControlCategory::SetTarget)
            if *from == enemy && target.object_id == helper)));
    assert_eq!(
        world
            .instance
            .find_actor(enemy)
            .unwrap()
            .get_common_spawn()
            .target_id
            .object_id,
        helper
    );
    engage_npc(&mut world.network.lock(), &mut world.instance, enemy, None);
    let sent = world.sent(player);
    assert!(
        sent.iter()
            .any(|message| control_from(message, enemy, |category| matches!(
                category,
                ActorControlCategory::ToggleWeapon { shown: false, .. }
            )))
    );
    assert!(
        sent.iter()
            .any(|message| control_from(message, enemy, |category| matches!(
                category,
                ActorControlCategory::SetBattle { battle: false }
            )))
    );
}

/// Bundle 20261111-112150, "the battle was playing out while I was in a cutscene … so long as even a single party
/// member is viewing a cutscene there should be a ring that keeps other party members waiting": the duty began for
/// whoever finished its opening first. It waits for every player (`Instance::awaiting_commence`, which holds its
/// enemies and the party's bots), then begins for all; someone coming in later gets it alone.
#[test]
fn a_duty_begins_when_everyone_has_seen_its_opening() {
    let mut world = TestWorld::new();
    let first = world.player(PLAYER, [0.0, 0.0, 0.0]);
    let second = world.player(PLAYER + 1, [2.0, 0.0, 0.0]);
    world.instance.awaiting_commence = true;
    assert_eq!(ready_to_begin(&mut world.instance, first), None);
    assert!(world.instance.awaiting_commence);
    let (to, first_time) = ready_to_begin(&mut world.instance, second).unwrap();
    assert!(first_time && to.contains(&first) && to.contains(&second));
    assert!(!world.instance.awaiting_commence);
    let late = world.player(PLAYER + 2, [4.0, 0.0, 0.0]);
    assert_eq!(
        ready_to_begin(&mut world.instance, late),
        Some((vec![late], false))
    );
}

/// Bundle 20261111-194208, "notorious bigs battle starts during cut-scene this should never happen": a quest battle's
/// NPCs, spawned with whom they hate (Biggs' archers on the player, his side on them), fought through the opening and
/// put the player in combat before it began. A quest battle waits for its opening as a duty does
/// (`waits_for_its_opening`), its NPCs out of the fight until then (`out_of_the_fight`: no target, stance or hater
/// list, so no InCombat), and it begins when the player has seen it (retail: scene 60000 7.3 s in, the first blow 2 s
/// later, Bob's Chronofoil capture 689b15da.)
#[test]
fn a_quest_battle_waits_for_its_opening() {
    assert!(waits_for_its_opening(HandlerType::QuestBattle));
    assert!(waits_for_its_opening(HandlerType::InstanceContent));

    let mut world = TestWorld::new();
    // Made as a quest battle's instance is (the wait was set only for duties' until 2026-11-12, Bob's bundle
    // 20261112-185310: this test set it by hand, so it passed while the server's quest battles never waited.) Its
    // NPCs come out after, from the director's onSetup.
    crate::server::prepare_quest_battle_instance(&mut world.instance);
    assert!(world.instance.awaiting_commence);
    let player = world.player(PLAYER, [0.0, 0.0, 0.0]);
    let archer = world.enemy(ENEMY, [5.0, 0.0, 5.0], 2000);
    if let Some(NetworkedActor::Npc { enmity, state, .. }) = world.instance.find_actor_mut(archer) {
        enmity.insert(player, 100);
        *state = NpcState::Hate;
    }
    assert!(out_of_the_fight(&world.instance, archer, &NpcState::Hate));

    // The opening is over: the fight is on.
    assert_eq!(
        ready_to_begin(&mut world.instance, player),
        Some((vec![player], true))
    );
    assert!(!out_of_the_fight(&world.instance, archer, &NpcState::Hate));
}

/// Bob, on the known issues: "if something doesn't exist implement it". Retail tells a player's party that they're
/// watching a cutscene, and when they've finished (ActorControl 519 `[them, 3]`, then `[them, 1]`, from them; the Howling
/// Eye's party, Bob's Chronofoil capture 09ecf8e7), or the player alone with ActorControlSelf (71045ef2, 41 cutscenes.)
#[test]
fn a_player_s_cutscene_is_told() {
    let mut world = TestWorld::new();
    let player = world.player(PLAYER, [0.0, 0.0, 0.0]);
    tell_cutscene_state(&mut world.network.lock(), player, true);
    tell_cutscene_state(&mut world.network.lock(), player, false);
    let states: Vec<u32> = world
        .sent(player)
        .iter()
        .filter_map(|message| match message {
            FromServer::ActorControlSelf(ActorControlCategory::SetPartyMemberCutsceneFlags {
                actor_id,
                unk2,
            }) if *actor_id == player => Some(*unk2),
            _ => None,
        })
        .collect();
    assert_eq!(states, vec![3, 1]);
}

/// Bob, on the known issues: a key item thrown on the ground (Rock of Rancor's firesand) had its cast bar without
/// retail's Casting condition (Condition 4 with the ActorCast, off with the effect, as for any event item's cast: Bob's
/// Chronofoil captures da3b6285 and 10108225). Its landing ends the cast; the event it then starts on what it landed on
/// isn't one.
#[test]
fn a_thrown_key_item_s_bar_is_a_cast() {
    use crate::server::instance::QueuedTaskData;
    use kawari::{common::ObjectTypeId, ipc::zone::ActionRequest};
    let throw = |landed_on| QueuedTaskData::EventItemOnGround {
        request: ActionRequest::default(),
        position: Default::default(),
        landed_on,
    };
    assert!(ends_a_cast(&throw(None)));
    assert!(!ends_a_cast(&throw(Some(ObjectTypeId::default()))));
}

/// Bob, on the known issues: retail gives Jog (4209, +20 % speed) when Sprint runs out, out of combat, from the
/// player with no end, and takes it away (LoseEffect `[4209, 20, the player]`) when they act or are fought (Bob's
/// Chronofoil captures: 126 Jogs, 20.2 s after a Sprint.)
#[test]
fn sprint_leaves_jog_until_the_player_acts() {
    use crate::server::effect::{STATUS_JOG, jog_ends, sprint_ran_out};
    let mut world = TestWorld::new();
    let player = world.player(PLAYER, [0.0, 0.0, 0.0]);
    let jog = |world: &TestWorld| {
        world
            .instance
            .find_actor(player)
            .and_then(|actor| actor.status_effects())
            .and_then(|effects| effects.get(STATUS_JOG))
    };
    sprint_ran_out(world.network.clone(), &mut world.instance, player);
    let given = jog(&world).expect("Jog after Sprint");
    assert_eq!((given.param, given.duration), (20, 0.0));
    assert_eq!(given.source_actor_id, player);

    jog_ends(world.network.clone(), &mut world.instance, player);
    assert!(jog(&world).is_none());
    assert!(world.sent(player).iter().any(|message| matches!(
        message,
        FromServer::ActorControlSelf(ActorControlCategory::LoseEffect {
            effect_id: 4209,
            unk2: 20,
            source_actor_id,
        }) if *source_actor_id == player
    )));

    // Being fought: no Jog.
    let enemy = world.enemy(ENEMY, [5.0, 0.0, 5.0], 1000);
    if let Some(NetworkedActor::Npc { enmity, .. }) = world.instance.find_actor_mut(enemy) {
        enmity.insert(player, 1);
    }
    sprint_ran_out(world.network.clone(), &mut world.instance, player);
    assert!(jog(&world).is_none());
}

/// Bob, on the known issues: retail ticks all of a target's HoTs as one (ActorControl 1540, status 0, their total) and
/// its MP restored over time as ActorControl 23 `[0, 11, amount, the target]` (Bob's Chronofoil captures: 785, Lucid
/// Dreaming's 550); Kawari sent one tick per effect, each on its own timer, and nothing for MP.
#[test]
fn a_target_s_over_time_effects_tick_together() {
    use crate::lua::{PeriodicKind, PeriodicSpec};
    use crate::server::periodic::{add_periodic_effect, periodic_tick};
    let mut world = TestWorld::new();
    let player = world.player(PLAYER, [0.0, 0.0, 0.0]);
    let healer = world.player(PLAYER + 1, [2.0, 0.0, 0.0]);
    if let Some(actor) = world.instance.find_actor_mut(player) {
        actor.get_common_spawn_mut().health_points = 100;
        let effects = actor.status_effects_mut().unwrap();
        effects.add(158, 0, 15.0, player);
        effects.add(150, 0, 18.0, healer);
        effects.add(1204, 0, 21.0, player);
    }
    let spec = |effect_id, kind, amount| PeriodicSpec {
        effect_id,
        kind,
        amount,
        on_self: false,
    };
    add_periodic_effect(
        &mut world.instance,
        player,
        player,
        &spec(158, PeriodicKind::Heal, 100),
    );
    add_periodic_effect(
        &mut world.instance,
        player,
        healer,
        &spec(150, PeriodicKind::Heal, 200),
    );
    add_periodic_effect(
        &mut world.instance,
        player,
        player,
        &spec(1204, PeriodicKind::RestoreMp, 550),
    );
    // All three tick together, now.
    let ticks: Vec<_> = world
        .instance
        .periodic_effects
        .iter()
        .map(|effect| effect.next_tick)
        .collect();
    assert!(ticks.iter().all(|tick| *tick == ticks[0]));
    for effect in &mut world.instance.periodic_effects {
        effect.next_tick = std::time::Instant::now() - Duration::from_millis(1);
    }
    periodic_tick(world.network.clone(), &mut world.instance);

    let sent = world.sent(player);
    let heals: Vec<u32> = sent
        .iter()
        .filter_map(|message| match message {
            FromServer::ActorControlSelf(ActorControlCategory::HotDot {
                status_id: 0,
                amount,
                source_actor_id,
                ..
            }) if *source_actor_id == player => Some(*amount),
            _ => None,
        })
        .collect();
    assert_eq!(heals.len(), 1, "one tick for both HoTs");
    assert!(
        (285..=315).contains(&heals[0]),
        "100 and 200, ±5 % each: {}",
        heals[0]
    );
    assert!(sent.iter().any(|message| matches!(
        message,
        FromServer::ActorControlSelf(ActorControlCategory::FloatingNumberVFX {
            unk1: 0,
            kind: 11,
            amount: 550,
            object_id,
        }) if *object_id == player
    )));
}

/// Bob, on the known issues: barriers (Galvanize...) took no damage, and retail's EffectResult tells what's left of a
/// target's barriers as a share of their maximum HP (7,270 results in Bob's Chronofoil captures.) A barrier takes the
/// damage first, one used up loses its status, and the share is what's left.
#[test]
fn a_barrier_takes_the_damage_first() {
    use crate::server::barrier::{absorb_and_break, grant, share};
    let mut world = TestWorld::new();
    let player = world.player(PLAYER, [0.0, 0.0, 0.0]);
    if let Some(actor) = world.instance.find_actor_mut(player) {
        actor
            .status_effects_mut()
            .unwrap()
            .add(297, 0, 30.0, player);
    }
    // 250 of their 1000 HP.
    grant(&mut world.instance, player, 297, 250);
    assert_eq!(share(&mut world.instance, player), 25);
    assert_eq!(
        absorb_and_break(world.network.clone(), &mut world.instance, player, 100),
        0
    );
    assert_eq!(share(&mut world.instance, player), 15);
    // The rest of it, and 50 through; Galvanize is gone.
    assert_eq!(
        absorb_and_break(world.network.clone(), &mut world.instance, player, 200),
        50
    );
    assert_eq!(share(&mut world.instance, player), 0);
    let gone = world
        .instance
        .find_actor(player)
        .and_then(|actor| actor.status_effects())
        .is_some_and(|effects| effects.get(297).is_none());
    assert!(gone);
    // A barrier whose status has gone holds nothing.
    grant(&mut world.instance, player, 297, 500);
    assert_eq!(
        absorb_and_break(world.network.clone(), &mut world.instance, player, 100),
        100
    );
}

/// Bob, on the known issues: a won quest battle's last DirectorVars, as its closing scene ends, is sequence 0xFE with
/// the last data byte's 0x80 set (every one of 32 won runs of 27 battles in Bob's Chronofoil captures; a lost one's 0xFE
/// has it clear.) Kawari left the sequence the script had set (Into a Copper Hell's 5.)
#[test]
fn a_won_quest_battle_ends_as_retail_s() {
    let (mut sequence, mut data) = (5, [0u8; 10]);
    won(&mut sequence, &mut data);
    assert_eq!(sequence, 0xFE);
    assert_eq!(data, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0x80]);
}

/// Known issues, Into a Copper Hell: retail's pugilist (BNpcBase 960) said line 1351 with its first Snap Punch (56)
/// and 1352 with its second, an ActorControl 73 from it in the same instant as the ActionEffect, and nothing with the
/// rest (seven runs in Bob's Chronofoil captures); Kawari never sent one. Its shipped timeline says so.
#[test]
fn an_npc_shouts_with_its_actions_as_retail_s_did() {
    use crate::server::action::npc_shout;
    let timeline: kawari::common::Timeline = serde_json::from_str(
        &std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../resources/timelines/IntoACopperHellPugilist_960.json"
        ))
        .unwrap(),
    )
    .unwrap();
    let mut world = TestWorld::new();
    world.player(PLAYER, [0.0, 0.0, 0.0]);
    let pugilist = world.enemy(ENEMY, [5.0, 0.0, 5.0], 1844);
    if let Some(NetworkedActor::Npc { timeline: own, .. }) = world.instance.find_actor_mut(pugilist)
    {
        *own = timeline;
    }
    let said: Vec<_> = (0..4)
        .map(|_| npc_shout(&mut world.instance, pugilist, 56))
        .collect();
    assert_eq!(said, [Some(1351), Some(1352), None, None]);
    // Its other actions say nothing.
    assert_eq!(npc_shout(&mut world.instance, pugilist, 53), None);
}

/// Known issues, Brayflox's Longstop and others: retail's duty director set its event 0x40000007 `[0, 1]` 300 s and
/// `[1, 1]` 900 s after DutyCommence in every run that lasted that long (65 and 20 in Bob's Chronofoil captures); Kawari
/// never sent either. Each goes once.
#[test]
fn a_duty_s_time_flags_come_when_retail_s_did() {
    use std::time::Duration;
    assert!(duty_flags_due(Duration::from_secs(299), 0).is_empty());
    assert_eq!(duty_flags_due(Duration::from_secs(300), 0), [0]);
    assert!(duty_flags_due(Duration::from_secs(600), 0b01).is_empty());
    assert_eq!(duty_flags_due(Duration::from_secs(900), 0b01), [1]);
    // An unrestricted party's second was set as it began.
    assert_eq!(duty_flags_due(Duration::from_secs(300), 0b10), [0]);
    assert!(duty_flags_due(Duration::from_secs(1000), 0b11).is_empty());
}

/// Known issues, coffers: retail sent an opened coffer's SpawnTreasure again to someone coming back into range while
/// its items were rolled for, with the time left and its items (41 in Bob's Chronofoil captures), and deleted it 1 s
/// after it faded; Kawari's came back closed, even after fading.
#[test]
fn an_opened_coffer_comes_back_opened() {
    use crate::server::{ClientState, loot::show_opened, network::NetworkState};
    use kawari::ipc::zone::SpawnTreasure;
    use std::time::Instant;
    let mut world = TestWorld::new();
    let coffer = ObjectId(0x4000_e2a7);
    world
        .instance
        .insert_treasure(coffer, SpawnTreasure::default());
    let opened = Instant::now() - Duration::from_secs(75);
    show_opened(&mut world.instance, coffer, &[3034], opened);
    let actor = world.instance.find_actor(coffer).unwrap();
    let Some(FromServer::TreasureSpawn(spawn)) =
        NetworkState::spawn_existing_actor_message(&mut ClientState::default(), coffer, actor)
    else {
        panic!("no SpawnTreasure");
    };
    // Retail's, 74.8 s after it opened: 225.2 left of 300.
    assert!((spawn.countdown_time - 225.0).abs() < 1.0);
    assert_eq!(
        (spawn.countdown_start_time, spawn.claim_time),
        (300.0, 300.0)
    );
    assert_eq!(spawn.item_count, 1);
    assert_eq!(spawn.lootable_item_ids[..2], [3034, 0]);
}

/// Bundle 20261111-125206, "Garudo attacks the pillar that's not right": an NPC that can't be targeted (the Howling
/// Eye's monoliths, display flags 0x40208) is never noticed by an enemy; retail's Garuda chose a player or a Duty
/// Support member every time. One a director made untargetable isn't either.
#[test]
fn an_untargetable_npc_is_never_noticed() {
    let mut world = TestWorld::new();
    world.player(PLAYER, [0.0, 0.0, 0.0]);
    let monolith = world.enemy(ENEMY, [5.0, 0.0, 5.0], 7800);
    let other = world.enemy(ENEMY + 1, [6.0, 0.0, 6.0], 7800);
    let hidden = world.enemy(ENEMY + 2, [7.0, 0.0, 7.0], 7800);
    if let Some(NetworkedActor::Npc { spawn, .. }) = world.instance.find_actor_mut(monolith) {
        spawn.common.display_flags = kawari::ipc::zone::DisplayFlag::from_bits_retain(0x40208);
    }
    world.instance.untargetable_npcs.insert(hidden);
    let unseen = unnoticed(&world.instance);
    assert!(unseen.contains(&monolith) && unseen.contains(&hidden));
    assert!(!unseen.contains(&other));
}

/// Bob's bundle 20261112-195353 (Notorious Biggs): retail's boss left the fighter it fought, walked 3.4-4.1 s to
/// whoever its 1068 was for and cast it from 9.1-10.5 yalms (four runs); Kawari's cast it on the fighter wherever
/// everyone was. An action out of reach waits for the walk, comes once within reach, and is given up when its target
/// is gone or after `PENDING_ACTION_PATIENCE`.
#[test]
fn an_npc_walks_into_reach_before_its_action() {
    use crate::server::npc_behavior::{PendingAction, pending_action_ready};
    use kawari::common::{TimepointData, TimepointFacing, TimepointTarget};
    use std::collections::HashMap;
    use std::time::{Duration, Instant};
    let timeline: kawari::common::Timeline = serde_json::from_str(
        &std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../resources/timelines/QB62Boss_1652.json"
        ))
        .unwrap(),
    )
    .unwrap();
    // The shipped boss walks for its 1068, on anyone but the one it fights.
    let data = timeline
        .timepoints
        .iter()
        .map(|point| point.data.clone())
        .find(|data| {
            matches!(
                data,
                TimepointData::Action {
                    action_id: 1068,
                    ..
                }
            )
        })
        .unwrap();
    assert!(matches!(
        data,
        TimepointData::Action {
            target: TimepointTarget::Other,
            facing: TimepointFacing::Target,
            reach: Some(reach),
            ..
        } if (9.0..=10.5).contains(&reach)
    ));
    let escort = ObjectId(0x4000_0001);
    let pending = |since| {
        Some(PendingAction {
            data: data.clone(),
            on: escort,
            reach: 9.5,
            since,
        })
    };
    let mut positions = HashMap::new();
    // Retail's: the boss by the fighter, the escort 33 yalms off down the slope.
    positions.insert(escort, glam::Vec3A::new(-506.0, 278.4, 192.0));
    let mut waiting = pending(Instant::now());
    let here = glam::Vec3A::new(-501.0, 289.6, 160.4);
    assert!(pending_action_ready(&mut waiting, here, &positions).is_none());
    assert!(waiting.is_some(), "still on its way");
    // Within its reach (retail's stopped 9.3-9.6 yalms off.)
    let there = glam::Vec3A::new(-504.2, 282.5, 184.0);
    let (used, on) = pending_action_ready(&mut waiting, there, &positions).unwrap();
    assert_eq!((used, on), (data.clone(), escort));
    assert!(waiting.is_none());
    // Given up: too long, or the one it's for gone.
    let mut stale = pending(Instant::now() - Duration::from_secs(11));
    assert!(pending_action_ready(&mut stale, here, &positions).is_none());
    assert!(stale.is_none());
    let mut gone = pending(Instant::now());
    assert!(pending_action_ready(&mut gone, here, &HashMap::new()).is_none());
    assert!(gone.is_none());
}

/// Bob's bundle 20261112-195353 (Notorious Biggs): retail's escort stopped 3.0-3.7 yalms from the small enemies and
/// 5.5-5.7 from the big ones (1655, 1657), the fighter and conjurer likewise (four runs); Kawari's `attack_range` is
/// measured to a player-sized target, and its NPCs stood as near a big one. A bigger target is reached from its edge.
#[test]
fn a_big_target_is_reached_from_further_off() {
    use crate::server::npc_behavior::reach_for_size;
    // A player, an NPC of unknown size, or one no bigger than a player: its range as it is.
    assert_eq!(reach_for_size(3.5, None), 3.5);
    assert_eq!(reach_for_size(3.5, Some(0.4)), 3.5);
    // Two yalms more of hitbox than a player's: two yalms further off, where retail's escort stood.
    let big = reach_for_size(3.5, Some(2.5));
    assert!((5.4..=5.7).contains(&big), "{big}");
}
