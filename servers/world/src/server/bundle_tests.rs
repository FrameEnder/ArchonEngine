//! What Bob's bundles showed, kept as tests (docs/src/fork/testing.md): each puts the bundle's situation into a
//! `TestWorld` and checks what the fix made the server do, so the fix stays. Named by what he wrote, with the
//! bundle. Only what needs none of the game's data fits here.

use std::time::Duration;

use glam::Vec3A;

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
        party::{own_cutscene_state, own_status_icon, tell_cutscene_state},
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

/// The Porta Decumana's long casts never land: the fight stops its boss's 29435 19.0 s into its 32.7 s (ActorControl
/// 15 `[537, 1, 29435, 0]` in both of Connor's and Bob's runs, Chronofoil captures af05916c and 9ba7d981), and the
/// four of base 14590's 28989 with `[540, 1, 28989, 1]` as they fall. `director:stop_cast` (`cut_cast_short`.)
#[test]
fn a_cast_the_fight_stops_never_lands() {
    use crate::server::action::{CAST_STOPPED, cut_cast_short};
    const LONG_CAST: u32 = 29435;
    let mut world = TestWorld::new();
    let player = world.player(PLAYER, [0.0, 4.0, 290.0]);
    let enemy = world.enemy(ENEMY, [3.0, 4.0, 288.0], 9000);
    world.casting(enemy, LONG_CAST, Duration::from_secs(30), enemy);
    assert_eq!(world.casts_queued(enemy), 1);
    world.sent(player);

    let stopped = cut_cast_short(
        world.network.clone(),
        &mut world.instance,
        enemy,
        CAST_STOPPED,
    );

    assert_eq!(stopped, vec![LONG_CAST]);
    assert_eq!(world.casts_queued(enemy), 0, "the cast never lands");
    let sent = world.sent(player);
    assert!(
        sent.iter().any(|m| matches!(
            m,
            FromServer::ActorControl(
                _,
                ActorControlCategory::CancelCast {
                    unk1: 537,
                    unk2: 1,
                    action_id: LONG_CAST,
                    ..
                }
            )
        )),
        "everyone near sees the cast stopped, as retail's [537, 1, action]: {sent:?}"
    );
    // Nothing being cast: nothing sent.
    assert!(
        cut_cast_short(
            world.network.clone(),
            &mut world.instance,
            enemy,
            CAST_STOPPED
        )
        .is_empty()
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

/// Bob's mark of 2026-11-20 in Castrum Meridianum, "the player NPCS are standing in the air not to mention they do not
/// respect the interior lock of the starting ring barrier": at the wipe every bot went back to one spot behind the
/// player, at the circle's edge, and walked out through it. Each goes to its corner of the diamond round the player
/// (retail's Duty Support members came back in theirs, inside the circle), facing their way.
#[test]
fn player_npcs_go_back_to_their_places_in_the_diamond() {
    let mut world = TestWorld::new();
    let player = world.player(PLAYER, [0.0, 4.0, 306.0]);
    let tank = world.player_npc(BOT, player, [3.0, 4.0, 250.0]);
    let other = world.player_npc(BOT + 1, player, [9.0, 4.0, 250.0]);
    // Their corners as `update_party` worked them out: the tank 2.0 ahead, the other 1.0 ahead and 1.72 aside.
    world.instance.party_bots.get_mut(&tank).unwrap().diamond = Some((2.0, 0.0));
    world.instance.party_bots.get_mut(&other).unwrap().diamond = Some((1.0, 1.72));
    let facing = world.instance.find_actor(player).unwrap().rotation();

    party_bot::back_to_the_start(&world.network.clone(), &mut world.instance, tank, player);
    party_bot::back_to_the_start(&world.network.clone(), &mut world.instance, other, player);

    let at = |id| world.instance.find_actor(id).unwrap().position().0;
    let (sin, cos) = facing.sin_cos();
    let tank_at = at(tank);
    assert!(
        (tank_at.x - 2.0 * sin).abs() < 0.01 && (tank_at.z - (306.0 + 2.0 * cos)).abs() < 0.01,
        "{tank_at}"
    );
    let other_at = at(other);
    assert!(
        (other_at.x - (sin + 1.72 * cos)).abs() < 0.01
            && (other_at.z - (306.0 + cos - 1.72 * sin)).abs() < 0.01,
        "{other_at}"
    );
    // Within 2.3 yalms of the player, not on one spot.
    for spot in [tank_at, other_at] {
        assert!(Vec3A::new(spot.x, 0.0, spot.z - 306.0).length() < 2.3);
    }
    assert!(tank_at.distance(other_at) > 1.0);
    assert_eq!(world.instance.find_actor(tank).unwrap().rotation(), facing);
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

/// Bob's mark in Castrum Meridianum, 2026-11-21: "enemies are not suppose to leave the wall barrier and allies are
/// still struggling to enter the black efts boss arena". The first boss's four noticed the party at the room's door,
/// 11 s before their boss, and walked out after it; retail's five were pulled together as the player came in, and none
/// went more than 10.7 yalms from the middle. A room's NPC notices only whoever is inside, and a chase stops short of
/// its edge (`Room`); the way in is a map effect's barrier, gone when map effect 1 takes it down.
#[test]
fn a_boss_rooms_enemies_keep_to_it() {
    use crate::server::instance::{ROOM_MARGIN, Room};
    let room = Room {
        middle: Vec3A::new(11.8, 70.2, -38.6),
        radius: 24.0,
        members: [9060841, 9169252].into_iter().collect(),
    };
    // The party at the door (where Kawari's four went after them) isn't in it; 7 yalms inside the line is.
    let door = Vec3A::new(34.7, 71.7, -46.4);
    assert!(!room.holds(door));
    assert!(room.holds(Vec3A::new(27.0, 71.0, -44.0)));
    // After someone at the door, a member goes no further than a yalm inside the edge, on the way there.
    let goal = room.keep_inside(door);
    let out = Vec3A::new(goal.x - 11.8, 0.0, goal.z + 38.6).length();
    assert!((out - (24.0 - ROOM_MARGIN)).abs() < 0.01, "{goal:?}");
    assert_eq!(goal.y, door.y);
    // Someone inside: there.
    let inside = Vec3A::new(20.0, 70.5, -40.0);
    assert_eq!(room.keep_inside(inside), inside);
    // The way in comes down with map effect 1 (timeline 2, state 1), as the five before the room fall.
    assert!(crate::map_effect_barriers::takes_down(1043, 1, 2, 1));
}

/// Bob's run of Castrum Meridianum, 2026-11-22 (harness capture zone-1795359118): "allies and boss left the damn arena",
/// "Magitek Vanguard F-1 … needs to be in center of arena for his large area of effect attack", "walls are still
/// blocking NPCS". Dodging out past the Vanguard's ring (28782, 12.5 to 30 from the middle), three bots walked through
/// its arena's east side (held at x = 6) after their 2 s, and it chased them out, casting from 30 yalms off the middle;
/// retail's walked to the middle in 3 s and never moved again (capture 281f92fb).
#[test]
fn castrum_meridianums_vanguard_holds_its_middle_and_nobody_walks_through_its_walls() {
    use crate::server::npc_behavior::{holds_spot, joins_directly};
    // Its timeline fights from the middle (`fights_from`).
    let timeline: kawari::common::Timeline = serde_json::from_str(
        &std::fs::read_to_string("../../resources/timelines/CastrumMeridianumBoss2_14541.json")
            .unwrap(),
    )
    .unwrap();
    assert_eq!(timeline.fights_from, Some([-13.0, 70.1, 31.0]));
    // While fighting someone it goes to its spot, not to them; walking home after the fight, home.
    let middle = Vec3A::new(-13.0, 70.1, 31.0);
    assert_eq!(
        holds_spot(
            &timeline,
            &crate::server::actor::NpcTarget::Actor(ObjectId(7))
        ),
        Some(middle)
    );
    let home = crate::server::actor::NpcTarget::Position(Vec3A::new(-14.1, 70.1, 25.1));
    assert_eq!(holds_spot(&timeline, &home), None);

    // A wall the navmesh goes round stops a walker: from the east side's inside (x = 6) to the courtyard past it, the
    // navmesh's way is round by the door, many times the distance.
    let inside = Vec3A::new(6.0, 70.1, 14.0);
    let outside = Vec3A::new(8.0, 70.1, 14.0);
    let round_by_the_door = [
        Vec3A::new(-13.0, 70.1, 50.0),
        Vec3A::new(-13.0, 70.1, 60.0),
        Vec3A::new(8.0, 70.1, 40.0),
        outside,
    ];
    assert!(!joins_directly(inside, outside, &round_by_the_door));
    // Nor where the navmesh has no way there at all.
    assert!(!joins_directly(inside, outside, &[]));
    // A bump the navmesh doesn't have (its way goes straight on) lets it on.
    assert!(joins_directly(inside, outside, &[outside]));

    // Its floor as a shape (Boss Mod's arena): the bots stay on it, 0.5 in from its edge.
    let floor = crate::server::director::ArenaFloor {
        x: -13.0,
        z: 31.0,
        half_x: 20.0,
        half_z: 20.0,
        round: false,
    };
    let arena = crate::server::dodge::Arena::from_floor(&floor);
    assert!(arena.holds(Vec3A::new(6.0, 70.1, 14.0)));
    assert!(!arena.holds(Vec3A::new(8.0, 70.1, 14.0)));
    assert!(
        !arena.holds(Vec3A::new(2.5, 70.1, 8.2)),
        "where a bot went in Bob's run"
    );
    // Out past the ring (30 from the middle) is off the floor everywhere, so the bots dodge into the hole instead.
    assert!(!arena.holds(Vec3A::new(-13.0 + 30.5, 70.1, 31.0)));
}

/// Bob's mark of 2026-11-23 in Castrum Meridianum: "reinforcements from the black eft fight don't auto aggro when
/// spawned like in retail", and his correction: an add that appears has pulled no one, so it goes by the party's
/// enmity, whoever holds the most (the tank, its stance), not by whom the boss happens to face. Retail's three waves
/// each went for the tank the moment they appeared (14 of 14, capture 281f92fb); now each takes on the party's enmity
/// on the boss (`director:join_fight`, `fight_enmity_of`): the most is the top, the rest keep their standing.
#[test]
fn a_bosss_wave_takes_on_the_partys_enmity() {
    use crate::server::director::fight_enmity_of;
    let mut world = TestWorld::new();
    let healer = world.player(PLAYER, [0.0, 0.0, 0.0]);
    let tank = world.player(PLAYER + 1, [3.0, 0.0, 3.0]);
    let boss = world.enemy(ENEMY, [10.0, 0.0, -40.0], 61542);
    // Holding none yet: the player.
    assert_eq!(
        fight_enmity_of(&world.instance, boss.0),
        vec![(healer, 300)]
    );
    // The tank holds the most, though the boss faces the healer (a mechanic, a taunt wearing off): the wave goes for
    // the tank, the healer second.
    if let Some(NetworkedActor::Npc { spawn, enmity, .. }) = world.instance.find_actor_mut(boss) {
        spawn.common.target_id.object_id = healer;
        enmity.insert(tank, 9000);
        enmity.insert(healer, 2500);
    }
    let mut taken = fight_enmity_of(&world.instance, boss.0);
    taken.sort_by_key(|(_, amount)| std::cmp::Reverse(*amount));
    assert_eq!(taken, vec![(tank, 9000), (healer, 2500)]);
}

/// Bob's marks of 2026-11-23 in Castrum Meridianum: "enemies with tethers between eachother should all be alerted and
/// aggro'd when a enemy with a tether connected to them is aggro'd", "I had to manually aggro each". The lines are the
/// client's own for a link family (SpawnNpc's `link_family`, `link_range`, `max_links`); retail's two families there
/// went for the party each at one moment (capture 281f92fb: family 1, five, limit 4, range 50, at 33.8 s; family 2,
/// six, limit 5, at 41.0 s). Kawari sent the families but never pulled them together.
#[test]
fn a_link_family_is_pulled_together() {
    use crate::server::npc_behavior::{LinkMember, link_target};
    let player = ObjectId(0x1000_0001);
    // Family 2's first pulled, at (141, -61); another of it at (133, -66), 9.4 yalms off.
    let first = LinkMember {
        family: 2,
        position: Vec3A::new(141.0, 91.4, -61.0),
        target: player,
    };
    let other = Vec3A::new(133.0, 91.4, -66.0);
    assert_eq!(link_target(&[first], 2, 50, 5, other), Some(player));
    // Not another family's, nor one with none.
    let family_1 = LinkMember {
        family: 1,
        position: Vec3A::new(184.0, 94.2, -89.0),
        target: player,
    };
    assert_eq!(link_target(&[family_1], 2, 50, 5, other), None);
    assert_eq!(link_target(&[first], 0, 50, 5, other), None);
    // Not beyond the family's range.
    assert_eq!(link_target(&[first], 2, 5, 5, other), None);
    // Not once the first and `max_links` others are fighting.
    let six: Vec<LinkMember> = (0..6)
        .map(|_| LinkMember {
            family: 2,
            position: Vec3A::new(141.0, 91.4, -61.0),
            target: player,
        })
        .collect();
    assert_eq!(link_target(&six, 2, 50, 5, other), None);
    assert_eq!(link_target(&six[..5], 2, 50, 5, other), Some(player));
}

/// Bob, 2026-11-22: "I legit saw him cast targetted support inside a thermobaric strike". The Magitek Vanguard F-1's
/// Targeted Support marks were still down 0.5 and 1.0 s into its Hypercharge: its timeline and its script's helpers
/// each kept their own half-second clock. Each attack now holds its timeline until its helpers' have landed
/// (`director:hold_timeline`, `content/w1d4_re.lua`); a hold already longer stays.
#[test]
fn a_bosss_next_attack_waits_for_its_helpers_to_land() {
    use crate::server::director::hold_timeline_until;
    let mut held = std::collections::HashMap::new();
    let now = std::time::Instant::now();
    let vanguard = ObjectId(370689582);
    hold_timeline_until(&mut held, vanguard, now + Duration::from_secs_f32(8.7));
    // A shorter hold (Hypercharge's own 4.7 s) doesn't cut Targeted Support's short.
    hold_timeline_until(&mut held, vanguard, now + Duration::from_secs_f32(4.7));
    assert_eq!(held[&vanguard], now + Duration::from_secs_f32(8.7));
    // A longer one lengthens it.
    hold_timeline_until(&mut held, vanguard, now + Duration::from_secs_f32(12.2));
    assert_eq!(held[&vanguard], now + Duration::from_secs_f32(12.2));
}

/// Bob's run of Castrum Meridianum, 2026-11-21: "the area of effect data … not even attached to the enemy". Retail
/// placed 29174 (BNpcBase 14573's 2.7 s cast, named to no one) on the caster's own spot in all five of its retail runs;
/// Kawari aimed it at the caster's target, 3 to 4 yalms off. An area retail placed on the ground at the caster is cast
/// there (`ground_at_caster`); one placed where its target stood (Cape Westwind's helpers' 28865) still goes there.
#[test]
fn an_area_retail_put_on_its_caster_is_cast_there() {
    use crate::server::npc_behavior::{cast_spot, retail_ground_at_caster};
    let caster = Vec3A::new(55.8, 75.2, -49.8);
    let target = Vec3A::new(52.2, 74.8, -51.5);
    assert_eq!(retail_cast_aim(29174, Some(7)), CastAim::Ground);
    assert!(retail_ground_at_caster(29174));
    assert_eq!(cast_spot(29174, caster, target), caster);
    // Retail's other ground areas of the same duty went where someone stood (29182, 29184): as before.
    assert!(!retail_ground_at_caster(29182));
    assert_eq!(cast_spot(29182, caster, target), target);
    assert_eq!(cast_spot(28865, caster, target), target);
    // And the same run: "the enemies aren't waiting to cast their area of effect action before moving". A cast
    // chosen on a tick the NPC took a step waits for the next (it stands then); an instant action doesn't wait.
    assert!(!crate::server::npc_behavior::begins_now(true, 2700));
    assert!(crate::server::npc_behavior::begins_now(false, 2700));
    assert!(crate::server::npc_behavior::begins_now(true, 0));
    // Its ActorCast names no one and gives the caster's spot (`cast_aim`.)
    assert_eq!(
        cast_aim(
            CastAim::Ground,
            ObjectId(7),
            cast_spot(29174, caster, target)
        ),
        (ObjectId::default(), kawari::common::Position(caster))
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
    // Since 2026-11-18 the table has the captures of 2026-11-16 and 2026-11-17 as well: Cape Westwind's helpers' rows
    // (28865) name no one, as retail's (Kawari's named the helper), its circles (28869) the helper itself.
    assert_eq!(retail_cast_aim(28865, Some(2)), CastAim::Ground);
    assert_eq!(retail_cast_aim(28869, Some(2)), CastAim::Caster);
    // And retail's omen delays: 28869's marker comes 5.5 s into its 7.7 s cast; nearly every action's at once.
    assert_eq!(crate::server::npc_behavior::retail_omen_delay(28869), 55);
    assert_eq!(crate::server::npc_behavior::retail_omen_delay(28787), 20);
    assert_eq!(crate::server::npc_behavior::retail_omen_delay(28865), 0);
    // And the landing names no one either (retail's AoeEffect8 for 28865: 0xE0000000), where the director had the
    // helper use it on itself; a player's own, or an NPC's aimed at someone, still names its target.
    let helper = ObjectId(0x4000_EAB3);
    assert!(!crate::server::action::aimed_at_someone(
        helper,
        true,
        28865,
        Some(2)
    ));
    assert!(crate::server::action::aimed_at_someone(
        helper,
        true,
        28869,
        Some(2)
    ));
    assert!(crate::server::action::aimed_at_someone(
        helper,
        false,
        28865,
        Some(2)
    ));
    assert!(!crate::server::action::aimed_at_someone(
        ObjectId(0xE000_0000),
        false,
        133,
        Some(2)
    ));
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
/// Eye's party, Bob's Chronofoil capture 09ecf8e7.) The player's own comes from their connection
/// (`own_cutscene_state`), so the server sends a player alone nothing.
#[test]
fn a_player_s_cutscene_is_told() {
    let mut world = TestWorld::new();
    let player = world.player(PLAYER, [0.0, 0.0, 0.0]);
    tell_cutscene_state(&mut world.network.lock(), player, true);
    tell_cutscene_state(&mut world.network.lock(), player, false);
    assert!(!world.sent(player).iter().any(|message| matches!(
        message,
        FromServer::ActorControlSelf(ActorControlCategory::SetPartyMemberCutsceneFlags { .. })
            | FromServer::ActorControl(_, ActorControlCategory::SetPartyMemberCutsceneFlags { .. })
    )));
}

/// Bob, on Gaius standing in the Praetorium after his cutscene: Kawari Witness showed retail's client hiding the
/// cutscene's ENpcs (render flags 0x2000800) as each cutscene began, and Kawari's never did. Retail's player heard their
/// 519 and 504 as ActorControls from themselves before the EventScene (Bob's Chronofoil capture 35e30205, in a party);
/// Kawari sent them as ActorControlSelf after it. Alone, retail's 519 was ActorControlSelf (71045ef2); its 504 never was
/// (none in 123 captures.)
#[test]
fn a_player_s_own_cutscene_controls_are_retail_s() {
    use kawari::ipc::zone::{OnlineStatus, ServerZoneIpcData};
    let player = ObjectId(0x1234);
    let flags = |data: &ServerZoneIpcData| match data {
        ServerZoneIpcData::ActorControl(control) => (false, control.category.clone()),
        ServerZoneIpcData::ActorControlSelf(control) => (true, control.category.clone()),
        other => panic!("{other:?}"),
    };
    let start = ActorControlCategory::SetPartyMemberCutsceneFlags {
        actor_id: player,
        unk2: 3,
    };
    let end = ActorControlCategory::SetPartyMemberCutsceneFlags {
        actor_id: player,
        unk2: 1,
    };
    assert_eq!(
        flags(&own_cutscene_state(player, true, true)),
        (false, start.clone())
    );
    assert_eq!(
        flags(&own_cutscene_state(player, true, false)),
        (false, end)
    );
    assert_eq!(
        flags(&own_cutscene_state(player, false, true)),
        (true, start)
    );
    assert_eq!(
        flags(&own_status_icon(OnlineStatus::ViewingCutscene)),
        (
            false,
            ActorControlCategory::SetStatusIcon {
                icon: OnlineStatus::ViewingCutscene
            }
        )
    );
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

/// Cape Westwind (InstanceContent 5060; Bob's Chronofoil capture 667a68c5): its boss's health stopped at 1 twice,
/// 380 to 1 and 525 to 1 under the player's hits, and he went on to the next part of the fight; only in the last
/// part did he fall. `director:set_health_floor` holds an NPC there (`Instance::health_floors`), damage over time
/// included.
#[test]
fn a_director_s_floor_holds_an_npc_s_health() {
    use crate::server::periodic::{add_status_tick, periodic_tick};

    let mut world = TestWorld::new();
    world.player(PLAYER, [0.0, 0.0, 0.0]);
    let boss = world.enemy(0x4000_0002, [2.0, 0.0, 0.0], 380);
    world.instance.health_floors.insert(boss, 1);
    if let Some(actor) = world.instance.find_actor_mut(boss) {
        actor
            .status_effects_mut()
            .unwrap()
            .add(18, 0, 0.0, ObjectId::default());
    }
    add_status_tick(&mut world.instance, boss, 18, 5000);
    let tick = |world: &mut TestWorld| {
        for effect in &mut world.instance.periodic_effects {
            effect.next_tick = std::time::Instant::now() - Duration::from_millis(1);
        }
        periodic_tick(world.network.clone(), &mut world.instance);
        world
            .instance
            .find_actor(boss)
            .map(|actor| actor.get_common_spawn().health_points)
    };
    assert_eq!(tick(&mut world), Some(1));
    assert_eq!(tick(&mut world), Some(1));

    // The last part: no floor, and he falls.
    world.instance.health_floors.remove(&boss);
    assert_eq!(tick(&mut world).unwrap_or(0), 0);
}

/// Heart of the Forest (quest battle 76; Bob's Chronofoil capture 667a68c5, three runs): the one who prays loses
/// health every 3 s under status 324, ActorControl 1541 `[0, 1763-1940, 0xE0000000, 0xFFFFFFFF]`, from the director's
/// own status (`director:gain_effect`'s tick), and no longer once the status is off.
#[test]
fn a_director_s_status_ticks_from_no_one() {
    use crate::server::periodic::{add_status_tick, periodic_tick};

    let mut world = TestWorld::new();
    let player = world.player(PLAYER, [0.0, 0.0, 0.0]);
    let prayer = world.enemy(0x4000_0001, [2.0, 0.0, 0.0], 50688);
    if let Some(actor) = world.instance.find_actor_mut(prayer) {
        actor
            .status_effects_mut()
            .unwrap()
            .add(324, 0, 0.0, ObjectId::default());
    }
    add_status_tick(&mut world.instance, prayer, 324, 1850);
    let due = |world: &mut TestWorld| {
        for effect in &mut world.instance.periodic_effects {
            effect.next_tick = std::time::Instant::now() - Duration::from_millis(1);
        }
    };
    due(&mut world);
    periodic_tick(world.network.clone(), &mut world.instance);
    let left = world
        .instance
        .find_actor(prayer)
        .unwrap()
        .get_common_spawn()
        .health_points;
    assert!((50688 - 1943..=50688 - 1757).contains(&left), "{left}");
    assert!(world.sent(player).iter().any(|message| matches!(
        message,
        FromServer::ActorControl(
            from,
            ActorControlCategory::DamageFlyingText {
                unk1: 0,
                amount: 1757..=1943,
                source_actor_id,
                unk4: u32::MAX,
            },
        ) if *from == prayer && *source_actor_id == ObjectId::default()
    )));

    // The status off (a new wave's larger tick is given with it again): nothing more of the old one.
    if let Some(actor) = world.instance.find_actor_mut(prayer) {
        actor.status_effects_mut().unwrap().remove(324);
    }
    due(&mut world);
    periodic_tick(world.network.clone(), &mut world.instance);
    assert!(world.instance.periodic_effects.is_empty());
    assert_eq!(
        world
            .instance
            .find_actor(prayer)
            .unwrap()
            .get_common_spawn()
            .health_points,
        left
    );
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

/// Bob, 2026-11-21: "enemies and allies don't always follow the policy of facing the target of their attack unless
/// their doing AOE's". A single-target action goes out turned to its target, whichever way its user faced, and an NPC
/// or party bot stays turned to it.
#[test]
fn an_attack_faces_its_target() {
    use crate::server::action::facing_towards;
    use std::f32::consts::{FRAC_PI_2, PI};
    let at = glam::Vec3A::new(10.0, 0.0, 10.0);
    assert_eq!(
        facing_towards(at, glam::Vec3A::new(10.0, 0.0, 15.0)),
        Some(0.0)
    );
    assert_eq!(
        facing_towards(at, glam::Vec3A::new(15.0, 2.0, 10.0)),
        Some(FRAC_PI_2)
    );
    assert_eq!(
        facing_towards(at, glam::Vec3A::new(5.0, 0.0, 10.0)),
        Some(-FRAC_PI_2)
    );
    assert!(
        (facing_towards(at, glam::Vec3A::new(10.0, 0.0, 5.0))
            .unwrap()
            .abs()
            - PI)
            .abs()
            < 1e-6
    );
    // On the same spot (height aside): no way to face.
    assert_eq!(facing_towards(at, glam::Vec3A::new(10.0, 3.0, 10.0)), None);
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

/// Bob's test of Cape Westwind, 2026-11-18: his Holy killed five of the boss's twelve helpers (44 HP, held by the
/// director where they cast his rows), which retail's area actions never touch. A held NPC spawned without the HOSTILE
/// flag is a director's helper, and one made untargetable is out of reach; a held enemy (the four before they're let
/// go) and any other are hit as ever.
#[test]
fn a_players_area_action_leaves_a_directors_helper_alone() {
    use crate::server::action::beyond_area_actions;
    use kawari::ipc::zone::CharacterDataFlag;
    let mut world = TestWorld::new();
    world.player(PLAYER, [0.0, 0.0, 0.0]);
    let helper = world.enemy(ENEMY, [2.0, 0.0, 2.0], 44);
    let held = world.enemy(ENEMY + 1, [3.0, 0.0, 3.0], 2501);
    let hidden = world.enemy(ENEMY + 2, [4.0, 0.0, 4.0], 24609);
    let passive = world.enemy(ENEMY + 3, [5.0, 0.0, 5.0], 300);
    for (id, flags) in [(helper, 1), (held, 3), (hidden, 3), (passive, 1)] {
        if let Some(NetworkedActor::Npc { spawn, .. }) = world.instance.find_actor_mut(id) {
            spawn.character_data_flags = CharacterDataFlag::from_bits_retain(flags);
        }
    }
    world.instance.held_npcs.insert(helper);
    world.instance.held_npcs.insert(held);
    world.instance.untargetable_npcs.insert(hidden);
    let beyond =
        |id| beyond_area_actions(&world.instance, id, world.instance.find_actor(id).unwrap());
    assert!(beyond(helper));
    assert!(beyond(hidden));
    assert!(!beyond(held));
    assert!(!beyond(passive));
}

/// Bob's run of 2026-11-24 in the Praetorium: his tank bot attacked one of the launchers (BNpcBase 6358, battalion 4 as
/// the enemies, no HOSTILE flag) and everyone's areas caught it till it died; retail's ten took no hit in five runs.
/// A zone's launcher is beyond the players' area actions and no bot's foe; an enemy beside it is hit as ever.
#[test]
fn nobody_fights_a_launcher() {
    use crate::server::action::beyond_area_actions;
    use crate::server::launcher::is_launcher;
    let mut world = TestWorld::new();
    world.instance.zone.id = 1044;
    world.player(PLAYER, [0.0, 0.0, 0.0]);
    let launcher = world.enemy(ENEMY, [2.0, 0.0, 2.0], 2589);
    let enemy = world.enemy(ENEMY + 1, [3.0, 0.0, 3.0], 6472);
    for (id, layout) in [(launcher, 9082875), (enemy, 9081969)] {
        if let Some(NetworkedActor::Npc { spawn, .. }) = world.instance.find_actor_mut(id) {
            spawn.common.layout_id = layout;
        }
    }
    {
        let instance = &world.instance;
        let actor = |id| instance.find_actor(id).unwrap();
        assert!(is_launcher(instance, actor(launcher)));
        assert!(!is_launcher(instance, actor(enemy)));
        assert!(beyond_area_actions(instance, launcher, actor(launcher)));
        assert!(!beyond_area_actions(instance, enemy, actor(enemy)));
    }
    // Another zone's NPC of that layout is no launcher.
    world.instance.zone.id = 1043;
    let instance = &world.instance;
    assert!(!is_launcher(
        instance,
        instance.find_actor(launcher).unwrap()
    ));
}

/// Cape Westwind's boss with his shield up (statuses 680 and 2195 after his 28866; `director:guard_front`): retail's
/// blocked the hit from straight in front for nothing and let the ones from 46° off and more through (Bob's and
/// Connor's Chronofoil captures, four runs). Kawari's took every hit, and the counter that answers one never came.
#[test]
fn a_guard_takes_a_hit_from_in_front() {
    use crate::server::action::{BLOCKED_FOR_NOTHING, guards_against, shown_to_a_guard};
    use kawari::ipc::zone::{TargetEffect, TargetEffectKind};
    let mut world = TestWorld::new();
    // He faces +Z (rotation 0); one player stands in front of him, one 50 degrees off, one behind.
    let front = world.player(PLAYER, [0.5, 0.0, 5.0]);
    let side = world.player(
        PLAYER + 1,
        [
            50f32.to_radians().sin() * 5.0,
            0.0,
            50f32.to_radians().cos() * 5.0,
        ],
    );
    let behind = world.player(PLAYER + 2, [0.0, 0.0, -5.0]);
    let boss = world.enemy(ENEMY, [0.0, 0.0, 0.0], 24609);
    if let Some(NetworkedActor::Npc { spawn, .. }) = world.instance.find_actor_mut(boss) {
        spawn.common.rotation = 0.0;
    }
    // No guard: nothing is blocked.
    assert!(!guards_against(&world.instance, boss, front));
    world.instance.front_guards.insert(boss, 45f32.to_radians());
    assert!(guards_against(&world.instance, boss, front));
    assert!(!guards_against(&world.instance, boss, side));
    assert!(!guards_against(&world.instance, boss, behind));
    // What the blocked hit shows: retail's eight bytes in the damage's place, the combo kept, no status.
    let combo = TargetEffect(TargetEffectKind::ExecuteCombo {
        sequence: 0,
        unk2: 0,
        unk3: 0,
        unk4: 0,
        unk5: 128,
        action_id: 25811,
    });
    let stun = TargetEffect(TargetEffectKind::GainEffect {
        unk1: 0,
        unk2: 0,
        param: 0,
        unk3: 0,
        effect_id: 2,
        duration: 4.0,
    });
    let hit = TargetEffect(TargetEffectKind::Damage {
        damage_kind: Default::default(),
        damage_type: kawari::ipc::zone::DamageType::Magic,
        damage_element: kawari::ipc::zone::DamageElement::Unaspected,
        bonus_percent: 0,
        unk3: 0,
        unk4: 0,
        amount: 442,
    });
    let shown = shown_to_a_guard(&[hit, stun, combo]);
    assert_eq!(shown.len(), 2);
    assert!(matches!(
        shown[0].0,
        TargetEffectKind::Block { amount: 0, .. }
    ));
    let mut bytes = std::io::Cursor::new(Vec::new());
    binrw::BinWrite::write_le(&TargetEffect(BLOCKED_FOR_NOTHING), &mut bytes).unwrap();
    assert_eq!(bytes.into_inner(), [0x06, 0, 0, 0x9C, 0, 0, 0, 0]);
}

/// Bob's test of Cape Westwind, 2026-11-18 ("the timing of the AOE damage isn't exactly right"): retail's boss stood
/// through the animation lock of each action he cast on himself (never moving sooner in four runs), Kawari's walked
/// off as it landed. The lock keeps it where it stands; a short one doesn't, and a director's longer hold stays.
/// (Only a cast on itself asks: `action.rs` where the effect goes out.)
#[test]
fn an_npc_stands_through_its_actions_lock() {
    use crate::server::action::stand_through_lock;
    let mut world = TestWorld::new();
    world.player(PLAYER, [0.0, 0.0, 0.0]);
    let boss = world.enemy(ENEMY, [2.0, 0.0, 2.0], 24609);
    let now = std::time::Instant::now();
    stand_through_lock(&mut world.instance, boss, 0.1, now);
    assert!(!world.instance.rooted_npcs.contains_key(&boss));
    stand_through_lock(&mut world.instance, boss, 3.1, now);
    assert_eq!(
        world.instance.rooted_npcs[&boss],
        now + std::time::Duration::from_secs_f32(3.1)
    );
    // A shorter one after it doesn't let him go sooner, nor does any lock end a director's hold.
    stand_through_lock(&mut world.instance, boss, 2.1, now);
    assert_eq!(
        world.instance.rooted_npcs[&boss],
        now + std::time::Duration::from_secs_f32(3.1)
    );
    let held = now + std::time::Duration::from_secs(3600);
    world.instance.rooted_npcs.insert(boss, held);
    stand_through_lock(&mut world.instance, boss, 5.1, now);
    assert_eq!(world.instance.rooted_npcs[&boss], held);
}

/// Bob's test of Cape Westwind, 2026-11-18 ("Imperial adds ... didn't attack for awhile"): his Holy gave the four its
/// Stun in full every 2.5 s, 273 s to 307 s into the server's capture, where retail's four showed "no effect" every
/// time. What the action's packet shows is what the enemy took: the status, the status for less, or effect 55 with
/// the status's ID; a party member's is never touched.
#[test]
fn an_enemy_takes_a_stun_as_retails_did() {
    use crate::server::action::status_as_taken;
    use kawari::ipc::zone::{TargetEffect, TargetEffectKind};
    let mut world = TestWorld::new();
    world.player(PLAYER, [0.0, 0.0, 0.0]);
    let add = world.enemy(ENEMY, [2.0, 0.0, 2.0], 2501);
    let other = world.enemy(ENEMY + 1, [3.0, 0.0, 3.0], 2501);
    for (id, base) in [(add, 14582), (other, 1)] {
        if let Some(NetworkedActor::Npc { spawn, .. }) = world.instance.find_actor_mut(id) {
            spawn.common.base_id = base;
        }
    }
    let stun = TargetEffect(TargetEffectKind::GainEffect {
        unk1: 0,
        unk2: 0,
        param: 0,
        unk3: 0,
        effect_id: 2,
        duration: 4.0,
    });
    assert!(matches!(
        status_as_taken(&mut world.instance, add, stun).0,
        TargetEffectKind::StatusNoEffect { effect_id: 2, .. }
    ));
    assert!(matches!(
        status_as_taken(&mut world.instance, other, stun).0,
        TargetEffectKind::GainEffect { effect_id: 2, duration, .. } if duration == 4.0
    ));
    // The player's own (an enemy's Stun on them) is as it was.
    assert!(matches!(
        status_as_taken(&mut world.instance, ObjectId(PLAYER), stun).0,
        TargetEffectKind::GainEffect { duration, .. } if duration == 4.0
    ));
    // And the "no effect" goes out as retail's eight bytes did.
    let refused = TargetEffect(TargetEffectKind::StatusNoEffect {
        unk: [0; 5],
        effect_id: 2,
    });
    let mut bytes = std::io::Cursor::new(Vec::new());
    binrw::BinWrite::write_le(&refused, &mut bytes).unwrap();
    assert_eq!(bytes.into_inner(), [0x37, 0, 0, 0, 0, 0, 0x02, 0]);
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

/// Bob's bundle 20261112-223259 (blank entries in the journal): `mark_quest_completed(464)` (26 quest scripts mark
/// another done by its number) marked quest 0 instead, so the journal showed a quest with no name and 464, 708, 717,
/// 718 and the rest stayed unfinished. A number is kept as it is, and a character hit by it is put right at login.
#[test]
fn a_quest_marked_done_by_its_number_is_that_quest() {
    use crate::common::adjust_quest_id;
    use crate::zone_connection::quest::repair_quests_marked_as_zero;
    use kawari::constants::COMPLETED_QUEST_BITMASK_SIZE;
    assert_eq!(adjust_quest_id(464), 464);
    assert_eq!(adjust_quest_id(65536 + 464), 464);
    assert_eq!(adjust_quest_id(65536), 0);

    // Bob's character: Close to Home (3856) and its successors done, 0 marked by the bug, 708 never.
    let mut completed = crate::bitmask::QuestBitmask::<COMPLETED_QUEST_BITMASK_SIZE>::default();
    for quest in [0, 3856, 3857, 3858, 709] {
        completed.set(quest);
    }
    assert!(repair_quests_marked_as_zero(&mut completed));
    assert!(!completed.contains(0));
    for quest in [708, 717, 718] {
        assert!(completed.contains(quest), "{quest}");
    }
    // Not the ones whose quest isn't done.
    assert!(!completed.contains(733));
    // Nothing to do the second time.
    assert!(!repair_quests_marked_as_zero(&mut completed));
}

/// Bob's report and bundle of 2026-11-13: with Drowning Out the Voices (1001) and Acting the Part (1002) both underway,
/// 1001 stayed on the Main Scenario Guide once done. Fool Me Twice (1004) after it also waits on Dressed for Conquest
/// (3871, after 1002), so nothing was available after 1001 and no guide was sent. Retail (Bob's Chronofoil capture
/// da424662) sent 1001 as completed last and 1002 as next.
#[test]
fn a_split_main_scenario_moves_on_to_its_other_branch() {
    use crate::scenario_guide::{GuideQuest, scenario_guide_step};
    use std::collections::HashSet;
    let quest = |id: u32, previous: &[u32], sort: u16| GuideQuest {
        id,
        previous: previous.to_vec(),
        order: (0, sort),
        ..Default::default()
    };
    let quests = [
        quest(986, &[984], 1),
        quest(1001, &[986], 2),
        quest(1002, &[986], 3),
        quest(3871, &[1002], 4),
        quest(1004, &[1001, 3871], 5),
    ];
    // 1002 accepted, not done: still available as far as the guide goes (accepting changes nothing on retail).
    let done = |id: u32| [984, 986, 1001].contains(&id);
    let available: HashSet<u32> = [1002].into();
    assert_eq!(
        scenario_guide_step(&quests, done, &available),
        Some((1001, vec![1002]))
    );
}

/// Bob's bundle 20261114-024122: he left Cid's workshop before accepting Every Little Thing She Does Is Magitek (1005),
/// and Slafborn's "go back in?" did nothing: the transcribed script answered him only in sequence 3, where Connor's
/// capture saw it. The client asks the same question (scenes 2, 4, 51) before the quest is accepted, in sequence 1 and
/// in 255; `quest_travel.json` gives each its scene and retail's ride, which Travel.lua answers.
#[test]
fn slafborn_takes_you_back_in_before_the_quest_is_accepted() {
    let entries = crate::quest_travel::entries(1005).expect("1005 has its rides");
    let slafborn = entries
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["actor"] == 1006530)
        .expect("Slafborn's");
    assert_eq!(slafborn["scenes"]["0"], 2, "before it's accepted");
    assert_eq!(slafborn["move"][1], 156, "into Cid's workshop (Mor Dhona)");
}

/// Going over Escape from Castrum Centri's battle against retail's packets (2026-11-14): ActorControl 503 went to
/// the player as each enemy put them on its list, numbered from 0 (the signifer as the battle commenced, the eques
/// 1.15 s after the player's first hit on it, the optio...), before the HaterList that had it; Kawari sent none. The
/// count goes round after 25, and an enemy still on the list isn't told again.
#[test]
fn each_enemy_that_joins_the_list_is_numbered() {
    use kawari::ipc::zone::{Hater, HaterList};
    let list = |actors: &[u32]| HaterList {
        count: actors.len() as u32,
        list: actors
            .iter()
            .map(|actor| Hater {
                actor_id: ObjectId(*actor),
                enmity: 100,
            })
            .collect(),
    };
    let mut next = 0;
    let first = list(&[ENEMY]);
    assert_eq!(
        super::enemy_list_joins(None, &first, &mut next),
        [(0, ObjectId(ENEMY))]
    );
    let second = list(&[ENEMY, ENEMY + 1, ENEMY + 2]);
    assert_eq!(
        super::enemy_list_joins(Some(&first), &second, &mut next),
        [(1, ObjectId(ENEMY + 1)), (2, ObjectId(ENEMY + 2))]
    );
    assert!(super::enemy_list_joins(Some(&second), &second, &mut next).is_empty());
    // One that left and comes back joins again; 25 is followed by 0.
    next = 25;
    assert_eq!(
        super::enemy_list_joins(Some(&list(&[])), &first, &mut next),
        [(25, ObjectId(ENEMY))]
    );
    assert_eq!(next, 0);
    // As it goes out: [503, index, enemy].
    let mut bytes = std::io::Cursor::new(Vec::new());
    binrw::BinWrite::write_le(
        &ActorControlCategory::EnemyListJoin {
            index: 3,
            enemy: ObjectId(0x4001_BF39),
        },
        &mut bytes,
    )
    .unwrap();
    assert_eq!(
        bytes.into_inner()[..12],
        [0xF7, 0x01, 0, 0, 3, 0, 0, 0, 0x39, 0xBF, 0x01, 0x40]
    );
}

/// The same going-over: a player's pet came into view 0.8 s after it was summoned in retail (spawned with display
/// flags 0x40028, then Appear with timeline 142 and SetTargetable; 338 summonings), and Kawari's was simply there.
/// One with no layout ID is brought into view by its actor.
#[test]
fn a_summoned_pet_comes_into_view() {
    use crate::server::director::{PET_APPEAR_TIMELINE, PendingAppearance, update_appearances};
    use kawari::ipc::zone::DisplayFlag;
    use std::time::Instant;
    let mut world = TestWorld::new();
    let player = world.player(PLAYER, [0.0, 0.0, 0.0]);
    let pet = world.enemy(ENEMY, [1.0, 0.0, 1.0], 500);
    if let Some(NetworkedActor::Npc { spawn, .. }) = world.instance.find_actor_mut(pet) {
        spawn.common.display_flags = DisplayFlag::INVISIBLE | DisplayFlag::UNK2 | DisplayFlag::UNK1;
    }
    world.instance.untargetable_npcs.insert(pet);
    let summoned = Instant::now();
    world.instance.pending_appearances.push(PendingAppearance {
        at: summoned + Duration::from_millis(800),
        layout_id: 0,
        action_timeline: PET_APPEAR_TIMELINE,
        actor: Some(pet),
    });
    world.sent(player);
    // Not yet.
    update_appearances(
        &mut world.network.lock(),
        &mut world.instance,
        summoned + Duration::from_millis(500),
    );
    assert!(world.sent(player).is_empty());
    update_appearances(
        &mut world.network.lock(),
        &mut world.instance,
        summoned + Duration::from_millis(800),
    );
    let sent = world.sent(player);
    assert!(sent.iter().any(|message| control_from(message, pet, |c| {
        matches!(
            c,
            ActorControlCategory::Appear {
                fade_in: true,
                action_timeline: 142
            }
        )
    })));
    assert!(sent.iter().any(|message| control_from(message, pet, |c| {
        matches!(c, ActorControlCategory::SetTargetable { targetable: true })
    })));
    assert!(!world.instance.untargetable_npcs.contains(&pet));
    let Some(NetworkedActor::Npc { spawn, .. }) = world.instance.find_actor(pet) else {
        panic!("the pet is there");
    };
    assert!(spawn.common.display_flags.is_empty());
}

/// Bob's mark of 2026-11-15, in Escape from Castrum Centri's battle: "the magitek vanguard h-1 should be invulnerable
/// untill the requisite magitek field generator is destroyed". The hits on it did show "Invincible", but Thunder
/// (144) still left its damage over time (status 161), which took 226 and 218 a tick off it behind its generator. On
/// retail an attack on a target that can't be hurt showed effect 7 (and the combo's 27) and no status, 1,270 times.
#[test]
fn an_attack_leaves_no_status_on_one_that_can_t_be_hurt() {
    use crate::server::action::shown_to_the_invulnerable;
    use kawari::ipc::zone::{
        DamageElement, DamageKind, DamageType, TargetEffect, TargetEffectKind,
    };
    let damage = TargetEffect(TargetEffectKind::Damage {
        damage_kind: DamageKind::default(),
        damage_type: DamageType::Magic,
        damage_element: DamageElement::Unaspected,
        bonus_percent: 0,
        unk3: 0,
        unk4: 0,
        amount: 466,
    });
    let status = TargetEffect(TargetEffectKind::GainEffect {
        unk1: 0,
        unk2: 0,
        param: 0,
        unk3: 0,
        effect_id: 161,
        duration: 18.0,
    });
    let combo = TargetEffect(TargetEffectKind::ExecuteCombo {
        sequence: 0,
        unk2: 0,
        unk3: 0,
        unk4: 0,
        unk5: 128,
        action_id: 144,
    });
    let kinds = |effects: &[TargetEffect]| -> Vec<u8> {
        effects
            .iter()
            .map(|effect| match effect.0 {
                TargetEffectKind::Invincible => 7,
                TargetEffectKind::GainEffect { .. } => 14,
                TargetEffectKind::ExecuteCombo { .. } => 27,
                TargetEffectKind::Damage { .. } => 3,
                _ => 0,
            })
            .collect()
    };
    // Thunder on a shielded vanguard: retail's (7, 27).
    let (shown, statuses_gone) = shown_to_the_invulnerable(&[damage, status, combo], true);
    assert_eq!(kinds(&shown), [7, 27]);
    assert!(statuses_gone);
    // A status alone from the other side doesn't land either.
    let (shown, statuses_gone) = shown_to_the_invulnerable(&[status], true);
    assert!(shown.is_empty());
    assert!(statuses_gone);
    // A friend's on someone just raised (who can't be hurt either) does.
    let (shown, statuses_gone) = shown_to_the_invulnerable(&[status], false);
    assert_eq!(kinds(&shown), [14]);
    assert!(!statuses_gone);
}

/// Bob's mark of 2026-11-15 ("I'm taking far too much damage, and/or not healing enough back", quest battle 65
/// through the harness): a signifer cast Fire at the player for 500 to 720 every 4 to 8 s and nothing else, 8,285
/// in the run where retail's four did 4,051 (his Chronofoil capture d1c46dff, the lost run). Its rotation had Fire
/// first with no `every`, so Fire was always the first one ready. Now every action waits its own time, at retail's
/// pace, with retail's amounts.
#[test]
fn the_signifers_cast_more_than_fire() {
    let timeline: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../resources/timelines/QB65Signifer_2171.json"
    ))
    .unwrap();
    let rotation = &timeline["rotation"];
    assert_eq!(rotation["pace"].as_f64(), Some(5.5));
    let actions = rotation["actions"].as_array().unwrap();
    assert!(actions.len() >= 12);
    let mut a_minute = 0.0;
    for action in actions {
        let every = action["every"].as_f64().unwrap_or_else(|| {
            panic!(
                "{}: no `every`, so it's cast whenever nothing else is ready",
                action["action"]
            )
        });
        assert!(every >= 30.0, "{}", action["action"]);
        a_minute += 60.0 / every;
    }
    // About eleven actions a minute, as retail's (113 in 603 s.)
    assert!((10.5..12.0).contains(&a_minute), "{a_minute}");
    let fire = timeline["recorded_amounts"]["141"]["players"]
        .as_f64()
        .unwrap();
    assert!((330.0..370.0).contains(&fire));
}

/// The quest battle prompt's difficulty (the same mark: retail's win of quest battle 65 was a retry at Very Easy
/// with the Echo, which Kawari never gave): only Very Easy, and only after a loss.
#[test]
fn an_easier_quest_battle_comes_after_a_loss() {
    use crate::{QUEST_BATTLE_VERY_EASY, quest_battle_difficulty};
    assert_eq!(quest_battle_difficulty(0, QUEST_BATTLE_VERY_EASY), 0);
    assert_eq!(
        quest_battle_difficulty(1, QUEST_BATTLE_VERY_EASY),
        QUEST_BATTLE_VERY_EASY
    );
    assert_eq!(quest_battle_difficulty(3, 0), 0);
    // Easy is in no capture: Normal.
    assert_eq!(quest_battle_difficulty(1, 1), 0);
}

/// Don't Let the Cat out of the Bag (863), Bob's Chronofoil capture 651981e7 (2026-11-16): the quest's bag, used on a
/// hecatoncheir, takes it: 0.63 s after the quest's event the NPC's mode goes to dead with ActorControl 14 and an
/// EffectResult at 0 HP, and it's deleted 0.49 s later, with no Defeated and nothing counted as a kill.
#[test]
fn a_battle_npc_a_key_item_took_goes_without_being_defeated() {
    let mut world = TestWorld::new();
    let player = world.player(PLAYER, [0.0, 4.0, 290.0]);
    let enemy = world.enemy(ENEMY, [3.0, 4.0, 288.0], 3000);
    world.sent(player);

    crate::server::actor::capture_battle_npc(&world.network, &mut world.instance, enemy, false);

    let Some(NetworkedActor::Npc { spawn, state, .. }) = world.instance.find_actor(enemy) else {
        panic!("it's still there until its despawn");
    };
    assert_eq!(*state, NpcState::Dead);
    assert_eq!(spawn.common.health_points, 0);
    assert_eq!(spawn.common.mode, CharacterMode::Dead);
    let sent = world.sent(player);
    assert!(sent.iter().any(|m| matches!(
        m,
        FromServer::ActorControl(from, ActorControlCategory::Kill { animation_id: 0 }) if *from == enemy
    )));
    assert!(sent.iter().any(|m| matches!(
        m,
        FromServer::PacketSegment(segment, from)
            if *from == enemy && matches!(&segment.data, ServerZoneIpcData::EffectResult(result) if result.health_points == 0)
    )));
    assert!(
        !sent.iter().any(|m| matches!(
            m,
            FromServer::ActorControl(_, ActorControlCategory::Defeated { .. })
                | FromServer::BattleNpcKilled { .. }
        )),
        "it wasn't defeated: {sent:?}"
    );
    // Its despawn is queued, and nothing that says who defeated it.
    assert!(world.instance.queued_task.iter().any(|task| matches!(
        task.data,
        crate::server::QueuedTaskData::DeadDespawn { actor_id, .. } if actor_id == enemy
    )));
    assert!(!world.instance.queued_task.iter().any(|task| matches!(
        task.data,
        crate::server::QueuedTaskData::SendDefeated { .. }
    )));
    // A second call does nothing.
    crate::server::actor::capture_battle_npc(&world.network, &mut world.instance, enemy, false);
    assert!(world.sent(player).is_empty());
}

/// Shocking Discoveries (805), Bob's Chronofoil capture b58bf0d5 (2026-11-23): the quest's ITEM0 used on a zu egg takes
/// it quietly, ActorControl 607 with the quest's event and its DeleteActor 1.11 s later, nothing between: no Kill, no
/// EffectResult (`event_item_effects.json`'s `quiet`).
#[test]
fn a_battle_npc_a_key_item_took_quietly_is_only_gone() {
    let mut world = TestWorld::new();
    let player = world.player(PLAYER, [0.0, 4.0, 290.0]);
    let egg = world.enemy(ENEMY, [3.0, 4.0, 288.0], 750);
    world.sent(player);
    crate::server::actor::capture_battle_npc(&world.network, &mut world.instance, egg, true);
    let Some(NetworkedActor::Npc { state, .. }) = world.instance.find_actor(egg) else {
        panic!("it's still there until its despawn");
    };
    assert_eq!(*state, NpcState::Dead);
    assert!(world.sent(player).is_empty());
    assert!(world.instance.queued_task.iter().any(|task| matches!(
        task.data,
        crate::server::QueuedTaskData::DeadDespawn { actor_id, .. } if actor_id == egg
    )));
}

/// FATE 586, Bob's Chronofoil capture 651981e7 (2026-11-16; "586 was the one implement that"): one enemy, and the
/// FATE's progress is the health it has lost, told every 1.33 s; when it falls the FATE's state goes to 5 with its
/// Defeated, 7 and 10 a second later, 0 6.5 s after that.
#[test]
fn a_boss_fate_follows_its_boss_s_health() {
    use crate::server::fate::{
        BOSS_PROGRESS_EVERY, FateInstance, boss_progress, boss_tick, send_fate_updates,
    };
    use kawari::common::FateState;
    const FATE: u32 = 586;
    const LAYOUT: u32 = 4306103;

    // Retail's numbers: 20240 HP; whole percent, rounded down; 100 only at none.
    assert_eq!(boss_progress(20240, 20240), 0);
    assert_eq!(boss_progress(20038, 20240), 0);
    assert_eq!(boss_progress(20037, 20240), 1);
    assert_eq!(boss_progress(15180, 20240), 25);
    assert_eq!(boss_progress(1, 20240), 99);
    assert_eq!(boss_progress(0, 20240), 100);

    let mut world = TestWorld::new();
    let player = world.player(PLAYER, [80.0, 55.0, -465.0]);
    let boss = world.enemy(ENEMY, [82.6, 55.5, -467.7], 20240);
    if let Some(NetworkedActor::Npc { spawn, .. }) = world.instance.find_actor_mut(boss) {
        spawn.common.layout_id = LAYOUT;
        spawn.common.fate_id = FATE as u16;
    }
    world.instance.fates.push(FateInstance {
        fate_id: FATE,
        fate_state: FateState::Running,
        boss: Some(LAYOUT),
        ..Default::default()
    });
    world.sent(player);
    let progress = |world: &mut TestWorld| -> Vec<u32> {
        world
            .sent(player)
            .iter()
            .filter_map(|m| match m {
                FromServer::ActorControlSelf(ActorControlCategory::FateUpdate {
                    fate_id: FATE,
                    progress,
                    ..
                }) => Some(*progress),
                _ => None,
            })
            .collect()
    };
    let mut tick = |world: &mut TestWorld, now: std::time::Instant| {
        let updates = boss_tick(&mut world.instance, now);
        send_fate_updates(&mut world.network.lock(), &mut world.instance, updates);
    };
    let start = std::time::Instant::now();

    // Told at once, then not again until 1.33 s on, changed or not.
    tick(&mut world, start);
    assert_eq!(progress(&mut world), [0]);
    world
        .instance
        .find_actor_mut(boss)
        .unwrap()
        .get_common_spawn_mut()
        .health_points = 15180;
    tick(&mut world, start + Duration::from_millis(700));
    assert!(progress(&mut world).is_empty());
    tick(&mut world, start + BOSS_PROGRESS_EVERY);
    assert_eq!(progress(&mut world), [25]);
    tick(&mut world, start + BOSS_PROGRESS_EVERY * 2);
    assert_eq!(progress(&mut world), [25]);

    // It falls: 100 without waiting for the next turn, and the FATE's end queued as retail's came.
    world
        .instance
        .find_actor_mut(boss)
        .unwrap()
        .get_common_spawn_mut()
        .health_points = 0;
    tick(
        &mut world,
        start + BOSS_PROGRESS_EVERY * 2 + Duration::from_millis(100),
    );
    assert_eq!(progress(&mut world), [100]);
    let queued: Vec<String> = world
        .instance
        .queued_task
        .iter()
        .filter_map(|task| match &task.data {
            crate::server::QueuedTaskData::FateState {
                fate_id: FATE,
                state,
            } => Some(format!("{state:?}")),
            crate::server::QueuedTaskData::EndFate { fate_id: FATE } => Some("end".to_string()),
            _ => None,
        })
        .collect();
    assert_eq!(queued.len(), 3, "{queued:?}");
    assert!(queued.contains(&"Ending".to_string()) && queued.contains(&"end".to_string()));
    assert!(queued.contains(&"Unk0".to_string()));
    // And nothing more once it's at 100.
    tick(&mut world, start + BOSS_PROGRESS_EVERY * 5);
    assert!(progress(&mut world).is_empty());
}

/// Bob's second test of Cape Westwind, 2026-11-18 ("still teleporting to ring end and ring center"): 5.5 s after
/// beginning 28868 in the arena's middle the boss was on the player's own spot, 13.9 yalms off, in one ActorMove (the
/// server's capture, 85.8 s in.) An NPC that doesn't move this tick (held by its director or an action's lock, or
/// casting) isn't taken further along its path either: only the pass that moves it asked, so its whole path was
/// behind it when it was let go.
#[test]
fn an_npc_that_stands_still_gets_no_further_along_its_path() {
    use crate::server::npc_behavior::stands_still;
    let now = std::time::Instant::now();
    let second = Duration::from_secs(1);
    // Held (`busy_bnpc`, `root_bnpc`, an action's lock, a status.)
    assert!(stands_still(true, None, now));
    assert!(!stands_still(false, None, now));
    // In a cast (28868; its auto-attack is 6499), and for a moment after its bar.
    assert!(stands_still(false, Some((28868, now + second, 6499)), now));
    assert!(stands_still(
        false,
        Some((28868, now - Duration::from_millis(300), 6499)),
        now
    ));
    assert!(!stands_still(false, Some((28868, now - second, 6499)), now));
    // An auto-attack's swing holds no one once it's swung.
    assert!(!stands_still(false, Some((6499, now, 6499)), now));
}

/// Bob's second and third tests of Cape Westwind ("he is showing his boss aoe in the right spot then shooting it in
/// the wrong spot", "attack still doesn't perfectly align with aoe"): kept in the middle while the four were out, the
/// boss never turned, so 28874's marker (the cast's own rotation) lay towards the player and he swung the way he'd
/// been left facing. Retail sends an ActorMove with the cast's facing with every ActorCast; so does `face_for_cast`,
/// whatever facing the NPC is taken to have (it had already been given the cast's, which is why a first version
/// that compared the two sent nothing.)
#[test]
fn an_npc_turns_to_its_cast() {
    use crate::server::npc_behavior::face_for_cast;
    let mut world = TestWorld::new();
    let player = world.player(PLAYER, [0.0, 0.0, 0.0]);
    let boss = world.enemy(ENEMY, [2.0, 0.0, 2.0], 24609);
    if let Some(NetworkedActor::Npc { spawn, .. }) = world.instance.find_actor_mut(boss) {
        spawn.common.rotation = -2.8;
    }
    world.sent(player);
    face_for_cast(&mut world.network.lock(), &mut world.instance, boss, -2.8);
    let moves: Vec<f32> = world
        .sent(player)
        .into_iter()
        .filter_map(|message| match message {
            FromServer::ActorMove(id, _, rotation, ..) if id == boss => Some(rotation),
            _ => None,
        })
        .collect();
    assert_eq!(moves, vec![-2.8]);
}

/// Bob's third test of Cape Westwind ("boss is using melee when he shouldn't"): kneeling in the middle with the four
/// out, the boss auto-attacked whoever stood by him. Retail's didn't once there, nor in his last part, in six runs.
/// A director stops an NPC's auto-attacks (`director:hold_attacks`); the rest of what it does goes on.
#[test]
fn a_director_stops_an_npcs_auto_attacks() {
    use crate::server::npc_behavior::auto_attacks;
    let mut world = TestWorld::new();
    let boss = world.enemy(ENEMY, [2.0, 0.0, 2.0], 24609);
    assert!(auto_attacks(world.instance.attacks_held.contains(&boss)));
    world.instance.attacks_held.insert(boss);
    assert!(!auto_attacks(world.instance.attacks_held.contains(&boss)));
}

/// Bob's second test of Cape Westwind ("another circle end to circle middle teleport"): the boss's leap (28872)
/// landed as an ActionEffect1 naming himself, and all Bob saw of it was the move to the middle a second later.
/// Retail's is an AoeEffect8 with no targets, its animation at no one and its position the middle
/// (`director:aim_action`.) What makes an action one of those:
#[test]
fn a_leap_lands_on_no_one() {
    use crate::server::action::{aimed_at_someone, lands_on_no_one};
    const BOSS: ObjectId = ObjectId(ENEMY);
    // 28872 is in `cast_aims.json`'s "none" since this: an NPC's use of it on itself isn't aimed at anyone.
    let aimed = aimed_at_someone(BOSS, true, 28872, None);
    assert!(!aimed);
    assert!(lands_on_no_one(true, aimed, false, true, true));
    // And Castrum Meridianum's last boss's 28788 since 2026-11-23 (Bob's mark: "the same leap issue that cape westwind's
    // boss had"): retail's an AoeEffect8 at no one, six of six in capture 281f92fb.
    assert!(!aimed_at_someone(BOSS, true, 28788, None));
    // Not a player's action, an area (its own packet already), one on someone, or one that does something.
    assert!(!lands_on_no_one(false, aimed, false, true, true));
    assert!(!lands_on_no_one(true, aimed, true, true, true));
    assert!(!lands_on_no_one(true, aimed, false, false, true));
    assert!(!lands_on_no_one(true, aimed, false, true, false));
    // Nor his 28875, which he casts on the player.
    assert!(!lands_on_no_one(
        true,
        aimed_at_someone(ObjectId(PLAYER), true, 28875, None),
        false,
        false,
        false
    ));
}

/// Bob's second test of Cape Westwind ("third stage chain doesn't work in anyway like it should it should pull you to
/// him, and freeze you in place only to attack"). The chain's status (3134) is one that holds a player where they
/// stand (retail's condition 43 with it, Bob's Chronofoil capture 178b0de0); others aren't.
#[test]
fn the_chains_status_holds_a_player_in_place() {
    use crate::server::modifiers::is_held_in_place;
    let mut statuses = crate::StatusEffects::default();
    assert!(!is_held_in_place(&statuses));
    statuses.add(2194, 0, 0.0, ObjectId(ENEMY));
    assert!(!is_held_in_place(&statuses));
    statuses.add(3134, 0, 0.0, ObjectId(ENEMY));
    assert!(is_held_in_place(&statuses));
    statuses.remove(3134);
    assert!(!is_held_in_place(&statuses));
}

/// The chain's pull is the Attract sheet's row and nothing else (Cape Westwind's helper's 28880): an action that does
/// no damage can't miss, whatever the levels (the helper is level 1), so the pull always comes. A hit that misses
/// still takes its pull or push with it.
#[test]
fn only_a_hit_can_miss() {
    use crate::server::action::roll_npc_hit;
    let pull = TargetEffect(TargetEffectKind::Attract {
        unk: [0; 5],
        attract_id: 17,
    });
    for _ in 0..50 {
        let mut effects = vec![pull];
        roll_npc_hit(&mut effects, 1.0, 100);
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].0, pull.0);
    }
    let mut effects = vec![
        TargetEffect(TargetEffectKind::Damage {
            damage_kind: Default::default(),
            damage_type: Default::default(),
            damage_element: Default::default(),
            bonus_percent: 0,
            unk3: 0,
            unk4: 0,
            amount: 100,
        }),
        pull,
    ];
    roll_npc_hit(&mut effects, 1.0, 100);
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].0, TargetEffectKind::Miss);
}

/// Bob's mark of 2026-11-23 in Castrum Meridianum: "thermobaric strike is doing no damage when it should". Its 28779 is
/// a helper's, level 1 (retail's as Kawari's), so a level 50 player was 49 above it and 40 % of its hits missed (7 of
/// 12 that run). Retail's helpers hit as the duty's enemies: Thermobaric Strike 8 of 8 (capture 281f92fb), helpers'
/// attacks on players 79 levels above 888 of 924. In a synced duty each side counts at the duty's level.
#[test]
fn a_helper_hits_as_the_dutys_level() {
    use crate::server::action::{levels_for_a_miss, npc_miss_percent};
    let mut world = TestWorld::new();
    world.instance.synced_level = Some(50);
    let (player, helper) = levels_for_a_miss(&world.instance, 50, 1);
    assert_eq!((player, helper), (50, 50));
    assert_eq!(npc_miss_percent(player, helper), 2);
    // A higher player synced down; an enemy above the sync keeps its level.
    assert_eq!(levels_for_a_miss(&world.instance, 90, 1), (50, 50));
    assert_eq!(levels_for_a_miss(&world.instance, 50, 52), (50, 52));
    // Unsynced (the open world, a duty above the player): as they are.
    world.instance.synced_level = None;
    assert_eq!(levels_for_a_miss(&world.instance, 50, 1), (50, 1));
}

/// Bob's mark of 2026-11-23 in Castrum Meridianum: "cid is staionary he should move to a specific location wait for us
/// then begin moving to target the field generator". At the generator retail's Cid hit it every 2.05 s, each a Miss
/// (266 in four runs), its health untouched: `director:show_attacks` (`only_shown`), for him on it and no one else.
#[test]
fn a_shown_attack_is_only_shown() {
    use crate::server::action::only_shown;
    let mut world = TestWorld::new();
    let player = world.player(PLAYER, [0.0, 0.0, 0.0]);
    let cid = world.enemy(ENEMY, [0.0, 0.0, 3.0], 258900);
    let generator = world.enemy(ENEMY + 1, [0.0, 0.0, 6.0], 2589);
    world.instance.shown_attacks.insert((cid, generator));
    assert!(only_shown(&world.instance, cid, generator));
    assert!(!only_shown(&world.instance, cid, player));
    assert!(!only_shown(&world.instance, generator, cid));
}

/// Bob's marks of 2026-11-25 at the Praetorium's hangar: "the player bots did not follow me here", and its armour
/// refused him. The hangar object moved him up (`director:warp_players`), and the server kept him on the first boss's
/// floor: the client sends no position until it moves, and he stood still, so his bots were put back beside where he
/// had been and the armour (which works from the hangar's floor) said no. The server has him where it sent him now.
#[test]
fn a_player_a_director_moves_is_where_it_moved_them() {
    use crate::server::director::warp_player_within;
    let mut world = TestWorld::new();
    let leader = world.player(PLAYER, [175.67, 76.0, 2.68]);
    let to = Vec3A::new(146.86, 102.0, -37.53);
    warp_player_within(
        &mut world.instance,
        &mut world.network.lock(),
        leader,
        kawari::common::Position(to),
        2.654,
    );
    let at = world.instance.find_actor(leader).unwrap().position().0;
    assert_eq!(at, to);
}

/// Bob's mark of 2026-11-23 in Castrum Meridianum: "player NPCs didn't go with me on teleport normally on duties like
/// this if a character is teleported then every character is teleported". The cutscene before the last room sent him
/// in (`director:warp_players`) and left his bots outside its walls. Now each goes with him, to its corner of the
/// diamond round where he's sent (`party_bot::spots_beside`, `warp_beside`).
#[test]
fn a_players_bots_go_with_them_when_a_director_sends_them_on() {
    use crate::server::party_bot::{spots_beside, warp_beside};
    let mut world = TestWorld::new();
    let leader = world.player(PLAYER, [0.0, 0.0, 0.0]);
    let first = world.player_npc(PLAYER + 10, leader, [-74.0, 0.0, -39.7]);
    let second = world.player_npc(PLAYER + 11, leader, [-74.1, 0.0, -37.6]);
    let to = Vec3A::new(-83.9, 0.0, -37.1);
    let spots = spots_beside(&world.instance, leader, to, 3.14);
    assert_eq!(spots.len(), 2);
    for (_, spot) in &spots {
        assert!(spot.distance(to) <= 4.0, "{spot}");
    }
    warp_beside(
        &mut world.network.lock(),
        &mut world.instance,
        leader,
        to,
        3.14,
    );
    for bot in [first, second] {
        let at = world.instance.find_actor(bot).unwrap().position().0;
        assert!(at.distance(to) <= 4.0, "{at}");
    }
}

/// Bob's Praetorium run of 2026-11-25 on `23467df9a`: Kawari sent the director's variables with the ZoneInit, before
/// the client had loaded (the script's `set_sequence(1)` at setup); retail's first came after the client's
/// EnterTerritoryEvent, when it asked for them (ReadyForDirectorVars), in all three runs read. A director's variables go
/// only to players who have asked (`Instance::vars_ready`.)
#[test]
fn a_directors_variables_wait_for_the_client_to_ask() {
    let mut world = TestWorld::new();
    let player = world.player(PLAYER, [149.5, 187.0, 0.4]);
    assert!(world.instance.vars_ready().is_empty());
    world.instance.director_vars_ready.insert(player);
    assert_eq!(world.instance.vars_ready(), vec![player]);
}

/// Bob, on Gaius still standing after cutscene 280 (2026-11-25): retail numbers what the server spawns from
/// 0x40000000 (Gaius 0x400024A1, capture 35e30205); Kawari's random IDs put two of the Praetorium's event objects in
/// the client's own range (0xE0000000 and up).
#[test]
fn actor_ids_are_retail_s() {
    use crate::server::instance::Instance;
    let ids: Vec<u32> = (0..1000).map(|_| Instance::generate_actor_id().0).collect();
    assert!(
        ids.iter()
            .all(|id| id & 0xF000_0000 == 0x4000_0000 && *id != 0x4000_0000)
    );
    let mut unique = ids.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), ids.len());
}

/// Bob, on Gaius still in the Praetorium after cutscene 280 ("the server must send something at some point ... that
/// WOULD show in the packets from Chronofoil"): it did. 2.4 s into cutscene 280 retail's SpawnNpc for one of the first
/// boss's helpers took Gaius's spawn index (5) with no DeleteActor before it; the client put the helper in his place
/// and Gaius was gone as the cutscene ended (capture 35e30205; the helpers went the same way at 543.8 s.) An NPC let go
/// unseen tells no one, and the next spawn takes its index.
#[test]
fn an_npc_let_go_unseen_is_replaced_by_the_next_spawn() {
    use crate::server::network::NetworkState;
    let mut world = TestWorld::new();
    let player = world.player(PLAYER, [0.0, 0.0, 0.0]);
    world.enemy(ENEMY, [1.0, 0.0, 0.0], 100);
    let gaius = world.enemy(ENEMY + 1, [2.0, 0.0, 0.0], 100);
    world.enemy(ENEMY + 2, [3.0, 0.0, 0.0], 100);
    world.sent(player);
    let mut network = world.network.lock();
    network.let_go_unseen(&mut world.instance, gaius);
    drop(network);
    assert!(world.instance.find_actor(gaius).is_none());
    assert!(world.sent(player).is_empty());

    let helper = ObjectId(ENEMY + 3);
    let actor = world.instance.find_actor(ObjectId(ENEMY)).unwrap().clone();
    let mut network = world.network.lock();
    let state = &mut network.clients.values_mut().next().unwrap().1;
    let index = |state: &mut crate::server::ClientState, id| {
        match NetworkState::spawn_existing_actor_message(state, id, &actor) {
            Some(FromServer::ActorSpawn(_, crate::common::SpawnKind::Npc(spawn))) => {
                spawn.common.spawn_index
            }
            other => panic!("{other:?}"),
        }
    };
    assert_eq!(index(state, helper), 2);
    assert_eq!(index(state, ObjectId(ENEMY + 4)), 4);
}
