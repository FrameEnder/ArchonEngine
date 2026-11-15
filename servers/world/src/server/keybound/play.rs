//! Keybound Brawler's casters, big enemies and summoned enemies at play (`keybound_mechanics.rs` has what they do,
//! from `resources/data/keybound_mechanics.json`): the casts and the counter that breaks one, a big enemy's rotation,
//! its summons, the barrier's words and shield, the special states, and the lock a client asks for.

use super::*;

impl Game {
    /// A summoned enemy's first moments: its appearing (ActorControl 36, its action, ActorControl 54 `[1]`) or its
    /// first blow at the barrier.
    pub(super) fn first_moments(
        &mut self,
        network: &Arc<Mutex<NetworkState>>,
        instance: &Instance,
        id: ObjectId,
        now: Instant,
    ) {
        let Some((base, since, appeared, hit)) = self.enemies.get(&id).map(|enemy| {
            (
                enemy.base,
                elapsed(enemy.spawned_at, now),
                enemy.appeared,
                enemy.hit,
            )
        }) else {
            return;
        };
        let Some(add) = mechanics().adds.get(&base) else {
            return;
        };
        if let Some((action, at, kind)) = add.appear
            && !appeared
            && since >= at
        {
            let mut network = network.lock();
            control(&mut network, instance, id, CONTROL_APPEAR, [0; 4]);
            act(
                &mut network,
                instance,
                id,
                action,
                Some(id),
                own_effect(kind, action),
                0,
            );
            control(&mut network, instance, id, CONTROL_WORDS, [1, 0, 0, 0]);
            self.enemies.get_mut(&id).unwrap().appeared = true;
        }
        if let Some((action, damage, at)) = add.hit
            && !hit
            && since >= at
        {
            self.enemies.get_mut(&id).unwrap().hit = true;
            let Some(piece) = self.nearest_piece(instance, id) else {
                return;
            };
            let mut network = network.lock();
            control(&mut network, instance, id, CONTROL_APPEAR, [0; 4]);
            blow(&mut network, instance, id, action, piece, damage, 0);
            self.strike(&mut network, instance, action, damage, now);
            control(&mut network, instance, id, CONTROL_WORDS, [1, 0, 0, 0]);
        }
    }

    /// An enemy has its words: they're sent, and what it does from here begins (its status, its cast, its rotation.)
    pub(super) fn set_off(
        &mut self,
        network: &Arc<Mutex<NetworkState>>,
        instance: &mut Instance,
        game_data: &Arc<Mutex<GameData>>,
        id: ObjectId,
        now: Instant,
    ) {
        let (base, full) = {
            let enemy = &self.enemies[&id];
            (enemy.base, enemy.full)
        };
        let Some(kind) = waves().enemies.get(&base) else {
            return;
        };
        let add = mechanics().adds.get(&base);
        // The barrier's words: each player their own.
        if full != 0 {
            for player in players_in(instance) {
                let first = self.free_word(kind.group, kind.words, |most| fastrand::u32(..most));
                let second = self.free_word(kind.group, kind.words, |most| fastrand::u32(..most));
                self.enemies
                    .get_mut(&id)
                    .unwrap()
                    .own_words
                    .insert(player, (first, second));
            }
        }
        {
            let mut network = network.lock();
            self.send_words(&mut network, instance, id);
            // One that appeared with an action was told then.
            if !add.is_some_and(|add| add.appear.is_some() || add.hit.is_some()) && full == 0 {
                control(&mut network, instance, id, CONTROL_WORDS, [1, 0, 0, 0]);
            }
            if let Some((category, [first, second])) = add.and_then(|add| add.control) {
                control(&mut network, instance, id, category, [first, second, 0, 0]);
            }
        }
        if let Some((status, param)) = add.and_then(|add| add.status) {
            self.statuses
                .push(StatusChange::Gain(id, status, param, 0.0));
        }
        if let Some(caster) = mechanics().casters.get(&base)
            && !caster.actions.is_empty()
        {
            // One of its kind's casts that no other of them on the field has (retail's three 19590s and four 19827s
            // each had their own, in any order.)
            let used: Vec<u32> = self
                .enemies
                .iter()
                .filter(|(other, enemy)| {
                    **other != id && enemy.base == base && enemy.fallen.is_none()
                })
                .filter_map(|(_, enemy)| {
                    enemy
                        .casting
                        .as_ref()
                        .map(|casting| casting.spell.action)
                        .or(enemy.to_cast.as_ref().map(|(spell, _, _)| spell.action))
                })
                .collect();
            let free: Vec<(u32, f32)> = caster
                .actions
                .iter()
                .filter(|(action, _)| !used.contains(action))
                .copied()
                .collect();
            let (action, time) = if free.is_empty() {
                caster.actions[fastrand::usize(..caster.actions.len())]
            } else {
                free[fastrand::usize(..free.len())]
            };
            let spell = Spell {
                action,
                time,
                ..caster.cast.clone()
            };
            self.enemies.get_mut(&id).unwrap().to_cast =
                Some((spell, caster.then, now + seconds(caster.after)));
        }
        if let Some(script) = mechanics().scripts.get(&base) {
            self.enemies.get_mut(&id).unwrap().script_at = Some(now + seconds(script.start));
            // The helper its blows come from stands in the middle, unseen.
            let helped = script.blocks.values().any(|block| {
                block
                    .steps
                    .iter()
                    .any(|step| matches!(&step.what, What::Cast(spell) if spell.helper.is_some()))
            });
            if helped && self.helper.is_none() {
                let (helper, layout, hp) = mechanics().helper;
                self.helper = spawn_piece(
                    instance,
                    &mut game_data.lock(),
                    self.director,
                    helper,
                    ENEMY_NAME,
                    layout,
                    hp,
                    hp,
                    Position(glam::Vec3A::ZERO),
                    false,
                );
            }
        }
    }

    /// An enemy's cast: the one it was to begin, and the one under way landing.
    pub(super) fn casts(
        &mut self,
        network: &Arc<Mutex<NetworkState>>,
        instance: &mut Instance,
        id: ObjectId,
        now: Instant,
    ) {
        if let Some((spell, then, at)) = self
            .enemies
            .get(&id)
            .and_then(|enemy| enemy.to_cast.clone())
            && now >= at
        {
            self.enemies.get_mut(&id).unwrap().to_cast = None;
            self.begin_cast(network, instance, id, spell, then, now);
        }
        let landed = self
            .enemies
            .get(&id)
            .and_then(|enemy| enemy.casting.as_ref())
            .is_some_and(|casting| {
                elapsed(casting.begun, now) >= casting.spell.time + CAST_LANDS_AFTER
            });
        if landed {
            self.complete_cast(network, instance, id, now);
        }
    }

    /// A cast begins: the counter's stacks on the caster (ActorControl 20 `[5354, 4]`, 0.09 s before retail's bar),
    /// and the bar (ActorCast at the barrier's nearest piece.)
    fn begin_cast(
        &mut self,
        network: &Arc<Mutex<NetworkState>>,
        instance: &Instance,
        id: ObjectId,
        spell: Spell,
        then: Then,
        now: Instant,
    ) {
        let Some(actor) = instance.find_actor(id) else {
            return;
        };
        if spell.counter > 0 {
            self.statuses.push(StatusChange::Gain(
                id,
                mechanics().counter_status,
                spell.counter as u16,
                0.0,
            ));
        }
        let ipc = ServerZoneIpcSegment::new(ServerZoneIpcData::ActorCast {
            spell_id: spell.action as u16,
            action_type: ActionType::Action,
            omen_delay: 0,
            action_id: spell.action,
            cast_time: spell.time,
            target: self.nearest_piece(instance, id).unwrap_or_default(),
            rotation: actor.rotation(),
            interruptible: false,
            ballista_entity_id: ObjectId::default(),
            position: actor.position(),
        });
        network.lock().send_in_range_instance(
            id,
            instance,
            FromServer::PacketSegment(ipc, id),
            DestinationNetwork::ZoneClients,
        );
        if spell.warn != 0 {
            // A client asks for its lock about as the cast lands (4.4 to 5.8 s into a 4.7 s cast, 8 of 8.)
            self.warned = Some((
                now + seconds(spell.time + CAST_LANDS_AFTER + mechanics().warn_seconds),
                spell.lock as u16,
            ));
        }
        self.enemies.get_mut(&id).unwrap().casting = Some(Casting {
            stacks: spell.counter,
            spell,
            then,
            begun: now,
        });
    }

    /// A cast lands: its blow at the barrier (or its own effect), the helper's blows to come, the players' warning,
    /// and what the caster does then.
    fn complete_cast(
        &mut self,
        network: &Arc<Mutex<NetworkState>>,
        instance: &mut Instance,
        id: ObjectId,
        now: Instant,
    ) {
        let Some(Casting { spell, then, .. }) = self
            .enemies
            .get_mut(&id)
            .and_then(|enemy| enemy.casting.take())
        else {
            return;
        };
        {
            let mut network = network.lock();
            match self.nearest_piece(instance, id) {
                Some(piece) if spell.damage > 0 => {
                    blow(
                        &mut network,
                        instance,
                        id,
                        spell.action,
                        piece,
                        spell.damage,
                        0,
                    );
                    self.strike(&mut network, instance, spell.action, spell.damage, now);
                }
                _ => act(
                    &mut network,
                    instance,
                    id,
                    spell.action,
                    Some(id),
                    own_effect(spell.effect, spell.action),
                    0,
                ),
            }
        }
        if spell.counter > 0 {
            self.statuses
                .push(StatusChange::Lose(id, mechanics().counter_status));
        }
        if let Some((action, damage, times)) = &spell.helper {
            for after in times {
                self.pending.push((
                    now + seconds(*after),
                    Pending::Blow {
                        from: self.helper.unwrap_or(id),
                        action: *action,
                        damage: *damage,
                    },
                ));
            }
        }
        if spell.warn != 0 {
            self.pending.push((
                now + seconds(mechanics().warn_after),
                Pending::Warn {
                    status: spell.warn as u16,
                },
            ));
        }
        // The barrier's words nobody filled go as the big enemy's cast lands (retail's went 0.1 s after.)
        if self.enemies[&id].script.is_some()
            && let Some(words) = self.barrier_words.take()
            && self
                .enemies
                .get(&words)
                .is_some_and(|enemy| enemy.fallen.is_none())
        {
            self.leave(network, instance, words, now);
        }

        match then {
            Then::Stay => {
                let interval = waves()
                    .enemies
                    .get(&self.enemies[&id].base)
                    .map(|kind| kind.interval)
                    .unwrap_or(ATTACK_INTERVAL);
                self.enemies.get_mut(&id).unwrap().next_attack = Some(now + seconds(interval));
            }
            // It falls by its own cast: ActorControl 2 `[2, 2]` and its words gone with the blow, dead 1.2 s later
            // (retail's 19590s; ActorControl 607 4.3 s after the blow.)
            Then::Fall => {
                {
                    let mut network = network.lock();
                    control(&mut network, instance, id, 2, [2, 2, 0, 0]);
                    command_all(&mut network, instance, self.director, WORDS_GONE, &[id.0]);
                    control(&mut network, instance, id, CONTROL_WORDS, [0; 4]);
                }
                let enemy = self.enemies.get_mut(&id).unwrap();
                enemy.words = None;
                enemy.dying = Some(now + seconds(DIES_AFTER));
                enemy.fallen = Some(now + seconds(DIES_AFTER));
                enemy.after_fall = 1;
            }
            Then::Leave => self.leave(network, instance, id, now),
            Then::Become(base, after) => {
                if let Some(at) = instance.find_actor(id).map(|actor| actor.position().0) {
                    self.pending.push((
                        now + seconds(after),
                        Pending::Become {
                            base,
                            x: at.x,
                            z: at.z,
                        },
                    ));
                }
                self.leave(network, instance, id, now);
            }
        }
    }

    /// A word on a caster with the counter: a stack off (ActorControl 20 `[5354, stacks left]`), and at none the
    /// cast is broken (ActorControl 21 for the status, then 1522 `[the action, 6, the player]`; the second number
    /// was 2 to 10 in retail's 665, 6 the commonest.)
    pub(super) fn counter(
        &mut self,
        network: &Arc<Mutex<NetworkState>>,
        instance: &Instance,
        from: ObjectId,
        target: ObjectId,
        now: Instant,
    ) {
        let Some(enemy) = self.enemies.get_mut(&target) else {
            return;
        };
        let Some(casting) = enemy.casting.as_mut() else {
            return;
        };
        if casting.stacks == 0 {
            return;
        }
        casting.stacks -= 1;
        let status = mechanics().counter_status;
        if casting.stacks > 0 {
            self.statuses.push(StatusChange::Gain(
                target,
                status,
                casting.stacks as u16,
                0.0,
            ));
            return;
        }
        let action = casting.spell.action;
        enemy.casting = None;
        let interval = waves()
            .enemies
            .get(&enemy.base)
            .map(|kind| kind.interval)
            .unwrap_or(ATTACK_INTERVAL);
        enemy.next_attack = Some(now + seconds(interval));
        self.statuses.push(StatusChange::Lose(target, status));
        let (category, second) = mechanics().interrupted;
        control(
            &mut network.lock(),
            instance,
            target,
            category,
            [action, second, from.0, 0],
        );
    }

    /// A big enemy's rotation: the steps that came due.
    pub(super) fn rotation(
        &mut self,
        network: &Arc<Mutex<NetworkState>>,
        instance: &mut Instance,
        game_data: &Arc<Mutex<GameData>>,
        id: ObjectId,
        now: Instant,
        step: f32,
    ) {
        let Some(enemy) = self.enemies.get_mut(&id) else {
            return;
        };
        let Some(script) = mechanics().scripts.get(&enemy.base) else {
            return;
        };
        let due = match (&mut enemy.script, enemy.script_at) {
            (Some(place), _) => script.advance(place, step),
            (None, Some(at)) if now >= at && !script.paths.is_empty() => {
                let mut place = Place {
                    path: fastrand::usize(..script.paths.len()),
                    ..Default::default()
                };
                let due = script.advance(&mut place, elapsed(at, now));
                enemy.script = Some(place);
                due
            }
            _ => return,
        };
        for what in due {
            match what {
                What::Cast(spell) => self.begin_cast(network, instance, id, spell, Then::Stay, now),
                What::Act(action, effect) => act(
                    &mut network.lock(),
                    instance,
                    id,
                    action,
                    (effect != 0).then_some(id),
                    own_effect(effect, action),
                    0,
                ),
                What::Spawn { base, rule, count } => {
                    let Some(add) = mechanics().adds.get(&base) else {
                        continue;
                    };
                    let taken = self.standing(instance, base);
                    for (x, z) in stand(rule, count, &add.spots, &taken, |below| {
                        fastrand::usize(..below)
                    }) {
                        self.summon(instance, game_data, base, x, z, now);
                    }
                }
                What::State {
                    notice,
                    arg,
                    switch,
                    param,
                    seconds: lasting,
                } => {
                    // The state is a status on every player while it lasts (retail's EffectResult with the notice:
                    // 5348 `[1171]` for 28 s, 5472 `[1172]` for 30 s). Kawari sent the notice and the switch alone,
                    // and nothing hid the words (Bob, 2026-11-15: "the effect for obscuring parts of phrases aren't
                    // working at all".)
                    for player in players_in(instance) {
                        self.statuses.push(StatusChange::Gain(
                            player,
                            arg as u16,
                            param as u16,
                            lasting,
                        ));
                    }
                    let mut network = network.lock();
                    command_all(
                        &mut network,
                        instance,
                        self.director,
                        STATE_NOTICE,
                        &[notice, arg],
                    );
                    command_all(
                        &mut network,
                        instance,
                        self.director,
                        STATE_SWITCH,
                        &[switch, 1],
                    );
                    self.pending.push((
                        now + seconds(lasting),
                        Pending::StateOff {
                            notice,
                            arg,
                            switch,
                        },
                    ));
                }
                What::Words(base) => {
                    let Some((x, z)) = mechanics()
                        .adds
                        .get(&base)
                        .and_then(|add| add.spots.first().copied())
                    else {
                        continue;
                    };
                    self.barrier_words = self.summon(instance, game_data, base, x, z, now);
                }
                What::ShieldOff => self.shield_off(&mut network.lock(), instance),
            }
        }
    }

    /// Where the live enemies of a kind stand.
    fn standing(&self, instance: &Instance, base: u32) -> Vec<(f32, f32)> {
        self.enemies
            .iter()
            .filter(|(_, enemy)| enemy.base == base && enemy.fallen.is_none())
            .filter_map(|(id, _)| {
                let at = instance.find_actor(*id)?.position().0;
                Some((at.x, at.z))
            })
            .collect()
    }

    /// A summoned enemy (or the barrier's words) comes: its SpawnNpc now, its words when its kind gets them.
    pub(super) fn summon(
        &mut self,
        instance: &mut Instance,
        game_data: &Arc<Mutex<GameData>>,
        base: u32,
        x: f32,
        z: f32,
        now: Instant,
    ) -> Option<ObjectId> {
        let kind = waves().enemies.get(&base)?;
        let add: Option<&Add> = mechanics().adds.get(&base);
        let hp = kind.hp_for(self.party);
        // The barrier's words come with 1 HP of theirs, to be filled.
        let charge = add.is_some_and(|add| add.charge);
        let hp_now = if charge { 1 } else { hp };
        let id = spawn_piece(
            instance,
            &mut game_data.lock(),
            self.director,
            base,
            kind.name,
            kind.layout,
            hp,
            hp_now,
            Position(glam::Vec3A::new(x, 0.0, z)),
            false,
        )?;
        self.enemies.insert(
            id,
            Enemy {
                active_at: Some(now + seconds(add.and_then(|add| add.words).unwrap_or(0.0))),
                full: if charge { hp } else { 0 },
                ..Enemy::new(base, hp_now, now)
            },
        );
        Some(id)
    }

    /// A marcher at the barrier that doesn't stay to hit it: a bomb's blast, or a 19592's own action; true when it
    /// was one, and is gone.
    pub(super) fn arrived(
        &mut self,
        network: &Arc<Mutex<NetworkState>>,
        instance: &mut Instance,
        id: ObjectId,
        now: Instant,
    ) -> bool {
        let Some(add) = self
            .enemies
            .get(&id)
            .and_then(|enemy| mechanics().adds.get(&enemy.base))
        else {
            return false;
        };
        if let Some((action, damage)) = add.blast {
            if let Some(piece) = self.nearest_piece(instance, id) {
                let mut network = network.lock();
                blow(&mut network, instance, id, action, piece, damage, 0);
                self.strike(&mut network, instance, action, damage, now);
            }
        } else if add.arrive != 0 {
            act(
                &mut network.lock(),
                instance,
                id,
                add.arrive,
                Some(id),
                own_effect(EFFECT_OWN, add.arrive),
                0,
            );
            if let Some((status, _)) = add.status {
                self.statuses.push(StatusChange::Lose(id, status));
            }
        } else {
            return false;
        }
        self.leave(network, instance, id, now);
        true
    }

    /// An enemy goes without falling: its words gone at once (and ActorControl 54 `[0]`), ActorControl 607 a second
    /// later, then its DeleteActor (retail's bombs after their blast.)
    fn leave(
        &mut self,
        network: &Arc<Mutex<NetworkState>>,
        instance: &Instance,
        id: ObjectId,
        now: Instant,
    ) {
        let Some(enemy) = self.enemies.get_mut(&id) else {
            return;
        };
        enemy.words = None;
        enemy.casting = None;
        enemy.to_cast = None;
        enemy.script = None;
        enemy.script_at = None;
        enemy.fallen = Some(now.checked_sub(seconds(LEAVES_SOONER)).unwrap_or(now));
        enemy.after_fall = 1;
        let mut network = network.lock();
        command_all(&mut network, instance, self.director, WORDS_GONE, &[id.0]);
        control(&mut network, instance, id, CONTROL_WORDS, [0; 4]);
    }

    /// The barrier takes a blow: its map effect by how it looks after (and its look changing, or its fall), then its
    /// HP; with its shield up, the shield's effect and nothing else.
    pub(super) fn hurt_barrier(
        &mut self,
        network: &mut NetworkState,
        instance: &Instance,
        damage: u32,
    ) {
        if self.shield {
            map_effect(network, instance, self.director, MAP_ABSORB);
            return;
        }
        let before = barrier_look(self.barrier_hp);
        self.barrier_hp = self.barrier_hp.saturating_sub(damage);
        if self.barrier_hp == 0 {
            map_effect(network, instance, self.director, MAP_FALLEN);
        } else {
            let look = barrier_look(self.barrier_hp);
            if look != before {
                map_effect(network, instance, self.director, MAP_LOOK[look]);
            }
            map_effect(network, instance, self.director, MAP_BLOW[look]);
        }
        command_all(
            network,
            instance,
            self.director,
            BARRIER_NOW,
            &[self.barrier_hp],
        );
    }

    /// The barrier's shield gone: its notice, and the barrier looking as its HP has it.
    fn shield_off(&mut self, network: &mut NetworkState, instance: &Instance) {
        if !self.shield {
            return;
        }
        self.shield = false;
        command_all(
            network,
            instance,
            self.director,
            STATE_NOTICE,
            &[mechanics().shield_notice[1], 0],
        );
        map_effect(
            network,
            instance,
            self.director,
            MAP_LOOK[barrier_look(self.barrier_hp)],
        );
    }

    /// A player's word on the barrier's words: 100 of their HP back (the action's effect of kind 4), the player's
    /// score and gauge as for any word, the map effect, and the player's next words; full, the shield goes up
    /// (notice 17841 and its map effect) and the words go as an enemy felled does.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn charge(
        &mut self,
        network: &Arc<Mutex<NetworkState>>,
        instance: &mut Instance,
        from: ObjectId,
        target: ObjectId,
        request: &ActionRequest,
        group: u32,
        now: Instant,
    ) {
        let (hp, full) = {
            let enemy = self.enemies.get_mut(&target).unwrap();
            enemy.hp = (enemy.hp + HIT_DAMAGE).min(enemy.full);
            (enemy.hp, enemy.full)
        };
        if let Some(actor) = instance.find_actor_mut(target) {
            actor.get_common_spawn_mut().health_points = hp;
        }
        let (score, gauge) = self.scored(from, group, now);
        {
            let mut network = network.lock();
            act(
                &mut network,
                instance,
                from,
                request.action_id,
                Some(target),
                TargetEffectKind::Heal {
                    unk1: [0; 5],
                    amount: HIT_DAMAGE as u16,
                },
                request.sequence,
            );
            command(&mut network, self.director, from, SCORE, &[score]);
            send_gauge(&mut network, from, gauge, true);
            map_effect(&mut network, instance, self.director, MAP_CHARGE);
        }
        if hp >= full {
            let mut network = network.lock();
            command_all(
                &mut network,
                instance,
                self.director,
                STATE_NOTICE,
                &[mechanics().shield_notice[0], 0],
            );
            map_effect(&mut network, instance, self.director, MAP_SHIELD);
            self.shield = true;
            let enemy = self.enemies.get_mut(&target).unwrap();
            enemy.fallen = Some(now);
            enemy.words = None;
            return;
        }
        // The player's next word moves up, and a new one comes behind it.
        let most = waves()
            .enemies
            .get(&self.enemies[&target].base)
            .map(|kind| kind.words)
            .unwrap_or(WORD_COUNT[0]);
        let new = self.free_word(group, most, |most| fastrand::u32(..most));
        let enemy = self.enemies.get_mut(&target).unwrap();
        let next = enemy
            .own_words
            .get(&from)
            .map(|(_, next)| *next)
            .unwrap_or(new);
        enemy.own_words.insert(from, (next, new));
        command(
            &mut network.lock(),
            self.director,
            from,
            WORDS,
            &[target.0, group, WORD_LIST + group, next, new, WORDS_LAST],
        );
    }

    /// A word's score and gauge for a player (as `word` counts them): the score and gauge after.
    fn scored(&mut self, from: ObjectId, group: u32, now: Instant) -> (u32, f32) {
        let fever = self.fever_until.is_some();
        let player = self.players.entry(from).or_insert(Player {
            bonus: Bonus::default(),
            last_hit: None,
            drain_at: None,
            refill_at: None,
        });
        if fever {
            player.bonus.fever_hit(group);
        } else {
            let idle = player
                .last_hit
                .map(|at| elapsed(at, now))
                .unwrap_or(f32::MAX);
            let (_, round) = player.bonus.hit(group, idle);
            player.last_hit = Some(now);
            player.drain_at = Some(now + seconds(GAUGE_HOLD));
            if round {
                player.refill_at = Some(now + seconds(GAUGE_REFILL_AFTER));
            }
        }
        (player.bonus.score, player.bonus.gauge)
    }

    /// What was to come at a set time: the helper's blows, a state's end, the players' warning, a lock and its
    /// end, one kind taking another's place.
    pub(super) fn timed(
        &mut self,
        network: &Arc<Mutex<NetworkState>>,
        instance: &mut Instance,
        now: Instant,
    ) {
        let mut due = Vec::new();
        self.pending.retain(|(at, what)| {
            if now >= *at {
                due.push(what.clone());
                false
            } else {
                true
            }
        });
        for what in due {
            match what {
                Pending::Blow {
                    from,
                    action,
                    damage,
                } => {
                    // At the barrier's middle piece.
                    let Some(piece) = self.barrier.last().copied() else {
                        continue;
                    };
                    let mut network = network.lock();
                    blow(&mut network, instance, from, action, piece, damage, 0);
                    self.strike(&mut network, instance, action, damage, now);
                }
                Pending::Land { damage } => {
                    self.hurt_barrier(&mut network.lock(), instance, damage);
                }
                Pending::Engage { enemy } => {
                    let Some(piece) = self.barrier.first().copied() else {
                        continue;
                    };
                    let mut network = network.lock();
                    control(&mut network, instance, enemy, 0, [1, 1, 0, 0]);
                    control(&mut network, instance, enemy, 4, [1, 0, 0, 0]);
                    network.send_in_range_instance(
                        enemy,
                        instance,
                        FromServer::ActorControlTarget(
                            enemy,
                            ObjectTypeId {
                                object_id: piece,
                                object_type: ObjectTypeKind::None,
                            },
                            ActorControlCategory::SetTarget,
                        ),
                        DestinationNetwork::ZoneClients,
                    );
                }
                Pending::StateOff {
                    notice,
                    arg,
                    switch,
                } => self.state_off(&mut network.lock(), instance, notice, arg, switch),
                Pending::Warn { status } => {
                    for player in players_in(instance) {
                        self.statuses.push(StatusChange::Gain(
                            player,
                            status,
                            0,
                            mechanics().warn_seconds,
                        ));
                    }
                }
                Pending::Lock { player, param } => {
                    let mechanics = mechanics();
                    self.statuses.push(StatusChange::Gain(
                        player,
                        mechanics.lock_status,
                        param,
                        mechanics.lock_seconds,
                    ));
                    let mut network = network.lock();
                    command(
                        &mut network,
                        self.director,
                        player,
                        STATE_NOTICE,
                        &[mechanics.lock_notice[0], mechanics.lock_status as u32],
                    );
                    command(&mut network, self.director, player, WAVE_OVER, &[]);
                    self.locked.push(player);
                    self.pending.push((
                        now + seconds(mechanics.lock_seconds),
                        Pending::Unlock { player },
                    ));
                }
                Pending::Unlock { player } => {
                    let on = matches!(self.stage, Stage::Wave { on: true });
                    self.unlock(&mut network.lock(), player, on);
                }
                Pending::Become { base, x, z } => self.becoming.push((base, x, z)),
                Pending::StatusOff { player } => self
                    .statuses
                    .push(StatusChange::Lose(player, PLAYER_STATUS.0)),
            }
        }
    }

    fn state_off(
        &self,
        network: &mut NetworkState,
        instance: &Instance,
        notice: u32,
        arg: u32,
        switch: u32,
    ) {
        // A state's notices come in pairs: the one after is its end.
        command_all(
            network,
            instance,
            self.director,
            STATE_NOTICE,
            &[notice + 1, arg],
        );
        command_all(network, instance, self.director, STATE_SWITCH, &[switch, 0]);
    }

    /// A player's lock ends: its notice, and (the wave still on) their typing back.
    fn unlock(&mut self, network: &mut NetworkState, player: ObjectId, typing: bool) {
        self.locked.retain(|locked| *locked != player);
        let mechanics = mechanics();
        command(
            network,
            self.director,
            player,
            STATE_NOTICE,
            &[mechanics.lock_notice[1], mechanics.lock_status as u32],
        );
        if typing {
            command(network, self.director, player, WAVE_ON, &[]);
        }
    }

    /// The wave or the game is over: the states still on end, the shield goes, the locked are let go.
    pub(super) fn states_off(&mut self, network: &mut NetworkState, instance: &Instance) {
        for (_, what) in std::mem::take(&mut self.pending) {
            match what {
                Pending::StateOff {
                    notice,
                    arg,
                    switch,
                } => {
                    self.state_off(network, instance, notice, arg, switch);
                    // Cut short by the wave's end: the players' status goes with it.
                    for player in players_in(instance) {
                        self.statuses.push(StatusChange::Lose(player, arg as u16));
                    }
                }
                Pending::Unlock { player } => {
                    self.unlock(network, player, false);
                    self.statuses
                        .push(StatusChange::Lose(player, mechanics().lock_status));
                }
                _ => {}
            }
        }
        self.shield_off(network, instance);
    }

    /// The statuses given and taken this turn, now that the network isn't held.
    pub(super) fn give_statuses(
        &mut self,
        network: &Arc<Mutex<NetworkState>>,
        instance: &mut Instance,
    ) {
        for change in std::mem::take(&mut self.statuses) {
            match change {
                StatusChange::Gain(actor, status, param, duration) => {
                    gain_effect_instance(
                        network.clone(),
                        ClientId::default(),
                        instance,
                        actor,
                        status,
                        param,
                        duration,
                        ObjectId::default(),
                        true,
                    );
                }
                StatusChange::Lose(actor, status) => {
                    lose_effect_instance(network.clone(), instance, actor, status);
                }
            }
        }
    }

    /// A client's director trigger: the one asking for its player's lock (`[0x10000002, 0, 0, 365]`, sent about as
    /// a warning cast lands; retail answered 1.4 s later with status 5347 for 7 s, its notice and the player's typing
    /// off.)
    pub(super) fn trigger(&mut self, player: ObjectId, id: u32, value: u32, now: Instant) {
        if (id, value) != mechanics().lock_trigger || self.locked.contains(&player) {
            return;
        }
        let Some((until, param)) = self.warned else {
            return;
        };
        let asked = self.pending.iter().any(
            |(_, what)| matches!(what, Pending::Lock { player: other, .. } if *other == player),
        );
        if now > until || asked {
            return;
        }
        self.pending.push((
            now + seconds(mechanics().lock_after),
            Pending::Lock { player, param },
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::test_world::{TestWorld, control_from, packet_from};

    const DIRECTOR: HandlerId = HandlerId(0x8003_DEA9);

    fn request(action_id: u32, target: ObjectId) -> ActionRequest {
        ActionRequest {
            action_id,
            action_type: ActionType::Action,
            sequence: 7,
            target: ObjectTypeId {
                object_id: target,
                object_type: ObjectTypeKind::None,
            },
            ..Default::default()
        }
    }

    fn commands(messages: &[FromServer]) -> Vec<(u32, Vec<u32>)> {
        messages
            .iter()
            .filter_map(|message| match message {
                FromServer::ActorControlSelf(ActorControlCategory::DirectorEvent {
                    event: DirectorEvent::Raw { id, args },
                    ..
                }) => Some((
                    *id,
                    args.iter().copied().take_while(|arg| *arg != 0).collect(),
                )),
                _ => None,
            })
            .collect()
    }

    fn map_effects(messages: &[FromServer]) -> Vec<(u16, u16, u8)> {
        messages
            .iter()
            .filter_map(|message| match message {
                FromServer::PacketSegment(segment, _) => match &segment.data {
                    ServerZoneIpcData::DirectorMapEffect {
                        state,
                        timeline_id,
                        index,
                        ..
                    } => Some((*state, *timeline_id, *index)),
                    _ => None,
                },
                _ => None,
            })
            .collect()
    }

    /// A world with one player, a piece of the barrier and the game in its wave.
    fn world() -> (TestWorld, ObjectId, ObjectId, Instant) {
        let mut world = TestWorld::new();
        let player = world.player(1, [4.5, 0.0, 26.0]);
        let piece = world.enemy(9, [0.0, 0.0, 17.5], BARRIER_PIECE_HP);
        world.instance.content_finder_condition_id = DUTY;
        let now = Instant::now();
        let mut game = Game::new(DIRECTOR, now, 4);
        game.stage = Stage::Wave { on: true };
        game.barrier.push(piece);
        world.instance.keybound = Some(game);
        world.sent(player);
        (world, player, piece, now)
    }

    /// Retail's 19589 (383 of them): status 5354 with four stacks and a 6.7 s cast of 49524 as it gets its words; a
    /// word takes a stack off (ActorControl 20 `[5354, 3]`…), and the fourth breaks the cast (ActorControl 21, then
    /// 1522 `[49524, …, the player]`); it never landed after that.
    #[test]
    fn four_words_break_a_caster_s_cast() {
        let (mut world, player, _, now) = world();
        let caster = world.enemy(2, [0.0, 0.0, -24.0], 500);
        let mut game = world.instance.keybound.take().unwrap();
        game.enemies.insert(
            caster,
            Enemy {
                words: Some((1, 2)),
                ..Enemy::new(19589, 500, now)
            },
        );
        let kind = &mechanics().casters[&19589];
        let spell = Spell {
            action: kind.actions[0].0,
            time: kind.actions[0].1,
            ..kind.cast.clone()
        };
        game.begin_cast(
            &world.network,
            &world.instance,
            caster,
            spell,
            kind.then,
            now,
        );
        game.give_statuses(&world.network, &mut world.instance);
        world.instance.keybound = Some(game);
        let sent = world.sent(player);
        assert!(sent.iter().any(|message| packet_from(message, caster, |data| matches!(
            data,
            ServerZoneIpcData::ActorCast { action_id: 49524, cast_time, .. } if *cast_time == 6.7
        ))));
        assert!(
            sent.iter()
                .any(|message| control_from(message, caster, |category| matches!(
                    category,
                    ActorControlCategory::GainEffect {
                        effect_id: 5354,
                        param: 4,
                        ..
                    }
                )))
        );

        for left in [3u32, 2, 1] {
            player_action(
                &world.network,
                &mut world.instance,
                player,
                &request(ACTION_HITS[0], caster),
            );
            let sent = world.sent(player);
            assert!(sent.iter().any(|message| control_from(message, caster, |category| matches!(
                category,
                ActorControlCategory::GainEffect { effect_id: 5354, param, .. } if *param == left
            ))));
        }
        player_action(
            &world.network,
            &mut world.instance,
            player,
            &request(ACTION_HITS[1], caster),
        );
        let sent = world.sent(player);
        assert!(
            sent.iter()
                .any(|message| control_from(message, caster, |category| matches!(
                    category,
                    ActorControlCategory::LoseEffect {
                        effect_id: 5354,
                        ..
                    }
                )))
        );
        assert!(sent.iter().any(|message| control_from(message, caster, |category| matches!(
            category,
            ActorControlCategory::Unknown { category: 1522, param1: 49524, param2: 6, param3, .. }
                if *param3 == player.0
        ))));
        // The cast is over: nothing lands when its time comes.
        let mut game = world.instance.keybound.take().unwrap();
        assert!(game.enemies[&caster].casting.is_none());
        game.casts(
            &world.network,
            &mut world.instance,
            caster,
            now + seconds(8.0),
        );
        assert_eq!(game.barrier_hp, BARRIER_HP);
        assert!(commands(&world.sent(player)).is_empty());
    }

    /// Retail's 19590 (69 casts that landed): its blow 0.3 s after the bar and 1,500 off the barrier 0.58 s after that, ActorControl 2 `[2, 2]` and
    /// its words gone with it, and dead a second later.
    #[test]
    fn a_cast_that_lands_hurts_the_barrier() {
        let (mut world, player, piece, now) = world();
        let caster = world.enemy(2, [2.25, 0.0, 4.0], 300);
        let mut game = world.instance.keybound.take().unwrap();
        game.enemies.insert(
            caster,
            Enemy {
                words: Some((1, 2)),
                ..Enemy::new(19590, 300, now)
            },
        );
        let kind = &mechanics().casters[&19590];
        let spell = Spell {
            action: 49526,
            time: 5.7,
            ..kind.cast.clone()
        };
        game.begin_cast(
            &world.network,
            &world.instance,
            caster,
            spell,
            kind.then,
            now,
        );
        world.sent(player);
        // Not yet as the bar ends, but 0.3 s after.
        game.casts(
            &world.network,
            &mut world.instance,
            caster,
            now + seconds(5.8),
        );
        assert_eq!(game.barrier_hp, BARRIER_HP);
        game.casts(
            &world.network,
            &mut world.instance,
            caster,
            now + seconds(6.05),
        );
        // The barrier's HP comes with the blow's result, 0.58 s after its effect (49526's result: 0.62 s, 49 of 49.)
        assert_eq!(game.barrier_hp, BARRIER_HP);
        game.timed(
            &world.network,
            &mut world.instance,
            now + seconds(6.05 + 0.5),
        );
        assert_eq!(game.barrier_hp, BARRIER_HP);
        game.timed(
            &world.network,
            &mut world.instance,
            now + seconds(6.05 + 0.58),
        );
        assert_eq!(game.barrier_hp, BARRIER_HP - 1500);
        let sent = world.sent(player);
        assert!(sent.iter().any(|message| packet_from(message, caster, |data| matches!(
            data,
            ServerZoneIpcData::ActionEffect1(effect) if effect.action_id == 49526
                && effect.targets[0].object_id == piece
                && matches!(effect.effects[0].0, TargetEffectKind::Damage { amount: 1500, .. })
        ))));
        assert_eq!(map_effects(&sent), [MAP_BLOW[0]]);
        assert_eq!(
            commands(&sent),
            [
                (WORDS_GONE, vec![caster.0]),
                (BARRIER_NOW, vec![BARRIER_HP - 1500])
            ]
        );
        assert!(
            sent.iter()
                .any(|message| control_from(message, caster, |category| matches!(
                    category,
                    ActorControlCategory::Unknown {
                        category: 2,
                        param1: 2,
                        param2: 2,
                        ..
                    }
                )))
        );
        let enemy = &game.enemies[&caster];
        assert!(enemy.words.is_none() && enemy.dying.is_some() && enemy.casting.is_none());
    }

    /// The barrier's look as it's hurt (retail's map effects): a blow, its look changing at 20,300 and 10,280, its
    /// fall.
    #[test]
    fn the_barrier_s_map_effects() {
        let (mut world, player, _, _) = world();
        let mut game = world.instance.keybound.take().unwrap();
        let hurt = |game: &mut Game, world: &mut TestWorld, damage| {
            game.hurt_barrier(&mut world.network.lock(), &world.instance, damage);
            let sent = world.sent(player);
            (map_effects(&sent), commands(&sent))
        };
        assert_eq!(
            hurt(&mut game, &mut world, 400),
            (vec![MAP_BLOW[0]], vec![(BARRIER_NOW, vec![39_600])])
        );
        assert_eq!(
            hurt(&mut game, &mut world, 19_300).0,
            [MAP_LOOK[1], MAP_BLOW[1]]
        );
        assert_eq!(hurt(&mut game, &mut world, 800).0, [MAP_BLOW[1]]);
        assert_eq!(
            hurt(&mut game, &mut world, 10_000).0,
            [MAP_LOOK[2], MAP_BLOW[2]]
        );
        assert_eq!(
            hurt(&mut game, &mut world, 20_000),
            (vec![MAP_FALLEN], vec![(BARRIER_NOW, vec![])])
        );
        assert_eq!(game.barrier_hp, 0);
    }

    /// The barrier's words (retail's 20079, 700 HP of which it comes with 1): a word with action 49518 or 49519 puts
    /// 100 back (the effect of kind 4) for 400 points and the map effect; full, notice 17841 and the shield's map
    /// effect, and the blows that follow are the shield's (998 of retail's 1,001 after a 17841 took nothing off the
    /// barrier); its end is notice 17842 and the barrier's look.
    #[test]
    fn the_barrier_s_words_raise_its_shield() {
        let (mut world, player, _, now) = world();
        let words = world.enemy(3, [0.0, 0.0, 18.5], 700);
        let mut game = world.instance.keybound.take().unwrap();
        game.enemies.insert(
            words,
            Enemy {
                words: Some((0, 0)),
                full: 700,
                own_words: HashMap::from([(player, (5, 6))]),
                ..Enemy::new(20079, 500, now)
            },
        );
        game.barrier_words = Some(words);
        world.instance.keybound = Some(game);

        // An enemy's word on it lands for nothing.
        player_action(
            &world.network,
            &mut world.instance,
            player,
            &request(ACTION_HITS[0], words),
        );
        assert!(commands(&world.sent(player)).is_empty());

        player_action(
            &world.network,
            &mut world.instance,
            player,
            &request(ACTION_CHARGES[0], words),
        );
        let sent = world.sent(player);
        assert!(sent.iter().any(|message| packet_from(message, player, |data| matches!(
            data,
            ServerZoneIpcData::ActionEffect1(effect) if effect.action_id == ACTION_CHARGES[0]
                && matches!(effect.effects[0].0, TargetEffectKind::Heal { amount: 100, .. })
        ))));
        assert_eq!(map_effects(&sent), [MAP_CHARGE]);
        let sent = commands(&sent);
        assert_eq!(sent[0], (SCORE, vec![400]));
        // The player's own next word moves up.
        assert_eq!(sent[1].0, WORDS);
        assert_eq!(sent[1].1[..4], [words.0, 3, WORD_LIST + 3, 6]);

        player_action(
            &world.network,
            &mut world.instance,
            player,
            &request(ACTION_CHARGES[1], words),
        );
        let sent = world.sent(player);
        assert_eq!(map_effects(&sent), [MAP_CHARGE, MAP_SHIELD]);
        assert_eq!(commands(&sent).last(), Some(&(STATE_NOTICE, vec![17841])));
        let mut game = world.instance.keybound.take().unwrap();
        assert!(game.shield && game.enemies[&words].fallen.is_some());

        game.hurt_barrier(&mut world.network.lock(), &world.instance, 6000);
        let sent = world.sent(player);
        assert_eq!(map_effects(&sent), [MAP_ABSORB]);
        assert!(commands(&sent).is_empty());
        assert_eq!(game.barrier_hp, BARRIER_HP);

        game.shield_off(&mut world.network.lock(), &world.instance);
        let sent = world.sent(player);
        assert_eq!(commands(&sent), [(STATE_NOTICE, vec![17842])]);
        assert_eq!(map_effects(&sent), [MAP_LOOK[0]]);
        assert!(!game.shield);
    }

    /// A client asking for its player's lock (retail: 7 of 8 askings, 1.4 s before): status 5347 for 7 s, notice
    /// `[17839, 5347]` and the typing off for that player; then notice `[17840, 5347]` and the typing on. Asked with
    /// no warning cast about, nothing.
    #[test]
    fn a_client_asks_for_its_lock() {
        let (mut world, player, _, now) = world();
        let (id, value) = mechanics().lock_trigger;
        player_trigger(
            &mut world.network.lock(),
            &mut world.instance,
            player,
            id,
            0,
            value,
        );
        let mut game = world.instance.keybound.take().unwrap();
        assert!(game.pending.is_empty());

        game.warned = Some((now + seconds(7.0), 1170));
        game.trigger(player, id, 0, now);
        assert!(game.pending.is_empty());
        game.trigger(player, id, value, now);
        game.trigger(player, id, value, now);
        assert_eq!(game.pending.len(), 1);
        game.timed(&world.network, &mut world.instance, now + seconds(1.0));
        assert!(commands(&world.sent(player)).is_empty());
        game.timed(&world.network, &mut world.instance, now + seconds(1.45));
        game.give_statuses(&world.network, &mut world.instance);
        let sent = world.sent(player);
        assert_eq!(
            commands(&sent),
            [(STATE_NOTICE, vec![17839, 5347]), (WAVE_OVER, vec![])]
        );
        assert!(sent.iter().any(|message| matches!(
            message,
            FromServer::ActorControlSelf(ActorControlCategory::GainEffect {
                effect_id: 5347,
                param: 1170,
                ..
            })
        )));
        game.timed(&world.network, &mut world.instance, now + seconds(8.5));
        assert_eq!(
            commands(&world.sent(player)),
            [(STATE_NOTICE, vec![17840, 5347]), (WAVE_ON, vec![])]
        );
        assert!(game.locked.is_empty());
    }

    /// Bob's infinite loading of 2026-11-15 (through the harness: the client's trigger `0x10000001 [1]` 1.3 s after
    /// ZoneInit, nothing back, its EnterTerritoryEvent 45 s later without the trigger 801 that asks for the director's
    /// variables, and the opening scene never played; the director's `0x10000009` had gone out with ZoneInit). Retail's
    /// 223 runs: the players' content IDs (`0x10000015`) with ZoneInit, and the set-up's packet and `0x10000009` only
    /// in answer to the trigger.
    #[test]
    fn a_loading_client_is_answered() {
        let (mut world, player, _, _) = world();
        if let Some(NetworkedActor::Player { spawn, .. }) = world.instance.find_actor_mut(player) {
            spawn.content_id = 0x0040_0017_4C4D_3B08;
        }
        let mut game = world.instance.keybound.take().unwrap();
        // Nothing for a player whose content ID isn't there yet (still loading.)
        let mut early = Game::new(DIRECTOR, Instant::now(), 4);
        if let Some(NetworkedActor::Player { spawn, .. }) = world.instance.find_actor_mut(player) {
            spawn.content_id = 0;
        }
        assert!(
            early
                .list_players(&mut world.network.lock(), &world.instance)
                .is_empty()
        );
        if let Some(NetworkedActor::Player { spawn, .. }) = world.instance.find_actor_mut(player) {
            spawn.content_id = 0x0040_0017_4C4D_3B08;
        }
        assert_eq!(
            game.list_players(&mut world.network.lock(), &world.instance),
            [player]
        );
        assert!(
            game.list_players(&mut world.network.lock(), &world.instance)
                .is_empty()
        );
        world.instance.keybound = Some(game);
        // Once, and no begin yet.
        assert_eq!(
            commands(&world.sent(player)),
            [(PLAYERS, vec![0x0040_0017, 0x4C4D_3B08])]
        );

        player_trigger(
            &mut world.network.lock(),
            &mut world.instance,
            player,
            TRIGGER_LOADED.0,
            TRIGGER_LOADED.1,
            0x3D4B_6AFC,
        );
        let sent = world.sent(player);
        assert!(matches!(
            &sent[0],
            FromServer::PacketSegment(segment, _) if matches!(
                segment.data,
                ServerZoneIpcData::KeyboundSetup {
                    empty_places: 3,
                    unk1: 6,
                    first: 12_743_016,
                    last: 12_743_021,
                    barrier: 40_000,
                    ..
                }
            )
        ));
        assert_eq!(commands(&sent), [(BEGIN, vec![])]);

        // Leaving mid-wave turns the player's typing off first.
        player_leaves(&mut world.network.lock(), &world.instance, player);
        let sent = world.sent(player);
        assert_eq!(commands(&sent), [(WAVE_OVER, vec![])]);
        assert_eq!(order(&sent), [format!("{WAVE_OVER:x}"), "end".to_string()]);
    }

    /// The game's end, in the order it came: the director's commands by number, the result's packet (with what it
    /// paid) and `RideShootingEnd`.
    fn order(messages: &[FromServer]) -> Vec<String> {
        messages
            .iter()
            .filter_map(|message| match message {
                FromServer::ActorControlSelf(ActorControlCategory::DirectorEvent {
                    event: DirectorEvent::Raw { id, .. },
                    ..
                }) => Some(format!("{id:x}")),
                FromServer::PacketSegment(segment, _) => match &segment.data {
                    ServerZoneIpcData::KeyboundResult { unk1, currency, .. } => {
                        Some(format!("result {unk1} {}", *currency > 0))
                    }
                    ServerZoneIpcData::RideShootingEnd { .. } => Some("end".to_string()),
                    ServerZoneIpcData::KeyboundOver { .. } => Some("over".to_string()),
                    _ => None,
                },
                _ => None,
            })
            .collect()
    }

    /// Bob, 2026-11-15 (a won run on `15f6019`): enemies stood in view before they set off, the last boss wasn't in
    /// its cutscene, a blow's damage came before the blow was seen to land, and nothing hid the words.
    #[test]
    fn enemies_show_and_blows_land_as_retail_s() {
        // The display flags of retail's SpawnNpcs.
        assert_eq!(spawn_flags(19587, false), 0x4_0208);
        assert_eq!(spawn_flags(19587, true), 0x6_0208);
        assert_eq!(spawn_flags(19594, true), 0x6_0208);
        assert_eq!(spawn_flags(19596, false), 0x44_0208);
        assert_eq!(spawn_flags(19825, false), 0x4_0228);
        assert_eq!(spawn_flags(19585, false), 0x4_0208);

        // A common enemy's blow (49715): the barrier's HP 1.02 s after it (the result at 1.07 s.)
        let (mut world, player, _, now) = world();
        let mut game = world.instance.keybound.take().unwrap();
        assert!((lands_after(49715) - 1.025).abs() < 0.001);
        assert_eq!(lands_after(49705), 0.0);
        let hp = game.barrier_hp;
        game.strike(&mut world.network.lock(), &world.instance, 49715, 100, now);
        assert_eq!(game.barrier_hp, hp);
        game.timed(&world.network, &mut world.instance, now + seconds(1.0));
        assert_eq!(game.barrier_hp, hp);
        game.timed(&world.network, &mut world.instance, now + seconds(1.03));
        assert_eq!(game.barrier_hp, hp - 100);
        assert!(commands(&world.sent(player)).contains(&(BARRIER_NOW, vec![hp - 100])));

        // The big enemies' states are statuses on the players: 5348 `[1171]` for 28 s, 5472 `[1172]` for 30 s.
        let states: Vec<(u32, u32, u32)> = [19594u32, 19596]
            .iter()
            .flat_map(|base| mechanics().scripts[base].blocks.values())
            .flat_map(|block| block.steps.iter())
            .filter_map(|step| match step.what {
                What::State {
                    arg,
                    param,
                    seconds,
                    ..
                } => Some((arg, param, seconds as u32)),
                _ => None,
            })
            .collect();
        assert!(!states.is_empty());
        assert!(
            states
                .iter()
                .all(|state| [(5348, 1171, 28), (5472, 1172, 30)].contains(state))
        );
    }

    /// Bob, 2026-11-15 (on `0a85433`): the last boss still wasn't in its cutscene. Retail's EventScene for it has
    /// 0x20 after its flags (220 of 220), the result's scene 0; and the boss turns on the barrier 0.8 s after its
    /// SpawnNpc (ActorControl 0 `[1, 1]`, 4 `[1]`, ActorControlTarget 50 at a piece.)
    #[test]
    fn the_last_boss_s_cutscene_is_retail_s() {
        let (mut world, player, piece, now) = world();
        let mut game = world.instance.keybound.take().unwrap();
        let high = |world: &mut TestWorld| -> Vec<u32> {
            world
                .sent(player)
                .iter()
                .filter_map(|message| match message {
                    FromServer::PlayDirectorSceneWith { flags_high, .. } => Some(*flags_high),
                    _ => None,
                })
                .collect()
        };
        play_scene(
            &mut world.network.lock(),
            DIRECTOR,
            player,
            SCENE,
            CUTSCENE_FLAGS,
            &CUTSCENE_PARAMS,
            SCENE_ARG,
        );
        assert_eq!(high(&mut world), [0x20]);
        play_scene(
            &mut world.network.lock(),
            DIRECTOR,
            player,
            SCENE,
            RESULT_FLAGS,
            &RESULT_PARAMS,
            SCENE_ARG,
        );
        assert_eq!(high(&mut world), [0]);

        let boss = world.enemy(3, [0.0, 0.0, 0.0], 3000);
        game.pending.push((now, Pending::Engage { enemy: boss }));
        game.timed(&world.network, &mut world.instance, now);
        let sent = world.sent(player);
        assert!(sent.iter().any(|message| matches!(
            message,
            FromServer::ActorControlTarget(from, target, ActorControlCategory::SetTarget)
                if *from == boss && target.object_id == piece
        )));
        for (category, first) in [(0, 1), (4, 1)] {
            assert!(
                sent.iter()
                    .any(|message| control_from(message, boss, |sent| matches!(
                        sent,
                        ActorControlCategory::Unknown { category: c, param1, .. }
                            if *c == category && *param1 == first
                    )))
            );
        }
    }

    /// A player with a score that pays.
    fn scorer() -> Player {
        let mut bonus = Bonus::default();
        bonus.score = 100_000;
        Player {
            bonus,
            last_hit: None,
            drain_at: None,
            refill_at: None,
        }
    }

    /// Bob, 2026-11-15, out of a lost game: "my entire hud is now gone", "I can only type in chat". Retail's lost
    /// runs: phase 5 and a result that pays nothing; 6.1 s on `RideShootingEnd`, then `0x40000002`, the flags, the
    /// rewards and a second result with what the loss paid, then the way out, with the game's status off 1.4 s into
    /// it. Kawari sent neither `RideShootingEnd` nor the second result, and left the status on.
    #[test]
    fn a_lost_game_ends_as_retail_s() {
        let (mut world, player, _, now) = world();
        let mut game = world.instance.keybound.take().unwrap();
        game.players.insert(player, scorer());
        game.finish(&world.network, &mut world.instance, false, now);
        let sent = order(&world.sent(player));
        assert_eq!(
            sent[..2],
            [format!("{PHASE:x}"), "result 0 false".to_string()]
        );
        assert!(!sent.contains(&"end".to_string()));
        game.ending(&world.network, &mut world.instance, false, 0, LOST_AT[0]);
        world.sent(player);
        game.ending(&world.network, &mut world.instance, false, 1, LOST_AT[1]);
        let sent = order(&world.sent(player));
        assert_eq!(
            sent,
            [
                "end".to_string(),
                format!("{DUTY_COMPLETED:x}"),
                format!("{DUTY_FLAG:x}"),
                format!("{DUTY_FLAG:x}"),
                "result 0 true".to_string(),
            ]
        );
        assert!(matches!(
            game.pending.last(),
            Some((_, Pending::StatusOff { player: off })) if *off == player
        ));
        game.timed(
            &world.network,
            &mut world.instance,
            Instant::now() + seconds(2.0),
        );
        assert!(matches!(
            game.statuses.last(),
            Some(StatusChange::Lose(off, status)) if *off == player && *status == PLAYER_STATUS.0
        ));
    }

    /// Retail's won runs: `RideShootingEnd` right after the result's `0x10000005` (8.05 s on), and the result's
    /// packet a second time as its scene ends, before the way out.
    #[test]
    fn a_won_game_ends_as_retail_s() {
        let (mut world, player, _, now) = world();
        let mut game = world.instance.keybound.take().unwrap();
        game.players.insert(player, scorer());
        game.finish(&world.network, &mut world.instance, true, now);
        let sent = order(&world.sent(player));
        assert!(sent.contains(&"result 1 true".to_string()));
        assert!(!sent.contains(&"end".to_string()));
        game.ending(&world.network, &mut world.instance, true, 0, RESULT_AT);
        assert_eq!(
            order(&world.sent(player)),
            [format!("{RESULT:x}"), "end".to_string()]
        );
        game.scene_over(&mut world.network.lock(), &world.instance, player, SCENE);
        assert_eq!(order(&world.sent(player)), ["result 1 true"]);
        game.scene_over(
            &mut world.network.lock(),
            &world.instance,
            player,
            LEAVE_SCENE,
        );
        let sent = world.sent(player);
        assert_eq!(order(&sent), ["over"]);
        assert!(matches!(sent.last(), Some(FromServer::LeaveContent)));
        assert!(matches!(
            game.pending.last(),
            Some((_, Pending::StatusOff { .. }))
        ));
        // Nothing of the game's follows as the player leaves (Kawari sent `0x10000002` again until 2026-11-15.)
        world.instance.keybound = Some(game);
        player_leaves(&mut world.network.lock(), &world.instance, player);
        assert!(world.sent(player).is_empty());
    }

    /// A big enemy's state (retail's 19594: notice `[17837, 5348]` and switch `[3071, 1]`, 28.0 s later `[17838, 5348]`
    /// and `[3071, 0]`), ended with the wave if that comes first.
    #[test]
    fn a_state_ends_with_its_time_or_the_wave() {
        let (mut world, player, _, now) = world();
        let mut game = world.instance.keybound.take().unwrap();
        game.pending.push((
            now + seconds(28.0),
            Pending::StateOff {
                notice: 17837,
                arg: 5348,
                switch: 3071,
            },
        ));
        game.timed(&world.network, &mut world.instance, now + seconds(27.0));
        assert!(commands(&world.sent(player)).is_empty());
        let off = [
            (STATE_NOTICE, vec![17838, 5348]),
            (STATE_SWITCH, vec![3071]),
        ];
        let mut other = Game::new(DIRECTOR, now, 4);
        other.pending = game.pending.clone();
        game.timed(&world.network, &mut world.instance, now + seconds(28.0));
        assert_eq!(commands(&world.sent(player)), off);
        other.states_off(&mut world.network.lock(), &world.instance);
        assert_eq!(commands(&world.sent(player)), off);
        assert!(other.pending.is_empty());
    }
}
