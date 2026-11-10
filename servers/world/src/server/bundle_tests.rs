//! What Bob's bundles showed, kept as tests (docs/src/fork/testing.md): each puts the bundle's situation into a
//! `TestWorld` and checks what the fix made the server do, so the fix stays. Named by what he wrote, with the
//! bundle. Only what needs none of the game's data fits here.

use std::time::Duration;

use kawari::{
    common::{CharacterMode, ObjectId, WarpType},
    ipc::zone::{ActorControlCategory, Condition, ServerZoneIpcData},
};

use crate::{
    FromServer,
    server::{
        actor::{NetworkedActor, NpcState, kill_actor},
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
