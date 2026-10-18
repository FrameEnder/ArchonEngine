# Changelog

Every change to the fork and its companion projects, newest first. **Add an entry with every change**, in the same
commit, under today's date. A commit can't contain its own hash, so write `—` for it; the next change fills it in.

## Documentation

| Date | Commit | Change |
|---|---|---|
| 2026-10-18 | — | Travel (teleport costs), items (sell price, wear, repair formula) from Bob's captures.
| 2026-10-18 | `3247ebd` | Combat (potencies measured from synced party members, statuses checked against retail, Raise); known issues.
| 2026-10-18 | `82a83ec` | Fixes from a review (combat, timing, opcodes).
| 2026-10-18 | `daec1d7` | Combat: no chains or synced-down kill EXP in duties; known issues.
| 2026-10-18 | `6e4487e` | Level sync (FATE rewards), quests (turn-in log lines and order), known issues (FATE and duty rewards).
| 2026-10-18 | `234465d` | Combat (enmity rules and lists, ground-targeted actions), fork opcodes (LootMessage, ActionRequestGroundTargeted; AoE layout, opcode 206, upstream's wrong sizes, dismount, Defeated, Commencing), timing (casts, duty clock, wipe), known issues (Limit Break, enmity, packets).
| 2026-10-18 | `3affb9a` | Combat (EXP: level penalty, chains, rested, packets; drops' LootMessage), quests (quest EXP's level), travel (weather) from Bob's captures.
| 2026-10-18 | `76bbc80` | Combat: combos, animation locks, casts landing 0.5 s early, recasts, auto-attack delay, over-time ticks from Bob's captures; known issues (combat timing, Raise).
| 2026-10-18 | `75a7c91` | Combat: enemies from Bob's captures (field HP by expansion, miss/parry, cast bars, speed, aggro, leash regeneration, Defeated, death and respawn timing) and regeneration; known issues.
| 2026-10-18 | `d0b51b7` | Combat: retail's max HP formula and its fitted per-level numbers (from Bob's own stats and synced party members); level sync and known issues updated.
| 2026-10-18 | `b65b947` | Retainers from Bob's 2026.10.06 capture: hiring and letting go at the vocate, entrusting/withdrawing/gil from the menu, price changes, every opcode confirmed (retainers, fork opcodes); the aethernet shard's scene (travel); known issues.
| 2026-10-18 | `7d32889` | Level Sync (new page): FATE and duty sync from Bob's captures, the stat caps; combat: gear (HQ), damage and healing modifiers, combos, animation lock, the class and role actions 1–50, combat items; known issues.
| 2026-10-18 | `bb5cd40` | Retainers (new page): the summoning bell, calling and sending away, entrusting, selling, the packets from Bob's captures; fork opcodes; `!retainer` (tools); known issues.
| 2026-10-18 | `2a6e491` | Claims, FirstAttack (combat).
| 2026-10-17 | `b0a07af` | Fixtures from Chronofoil, replay checking scenes, flags and effects, `battle-sim`, `lint`, CI (questkit); retail's scene flags and how searches play (quests); the quest battles' timings and health marks from the captures, adding a quest battle (quest battles); what the new checks can't see (known issues).
| 2026-10-17 | `5f16817` | Ul'dah's first five quests, attuning inside a quest, which scenes come back as yields (quests); Underneath the Sultantree (quest battles); combo steps without their step, the blanga's and the stranger's skills (combat); fixtures with layouts and yields, battle steps found by territory, composite scene flags (questkit); known issues.
| 2026-10-17 | `3f9a8bc` | Open-world spawns, Central Thanalan's monsters (combat); `questkit chronofoil --dropin` (questkit); Central Thanalan (known issues).
| 2026-10-17 | `7211858` | `master` fast-forwarded to `Latest-Quest` (changelog).
| 2026-10-17 | `ee5e162` | Marketboard: the search's category byte and flag, MarketBoardOpened, ItemSearchCategory per language (marketboard, fork opcodes, known issues).
| 2026-10-17 | `86bcfa2` | Marketboard (new page): what's listed, prices, listings, buying, the packets from Bob's captures; fork opcodes; Kawari Web's Marketboard page; questdump's item list (tools); known issues.
| 2026-10-17 | `7f95a59` | `player.scene_error_code`, event actions answered at once, The Gridanian Envoy's gangplanks (quests); fixture results with an error code (questkit).
| 2026-10-17 | `f4ffd30` | Cutscenes that end somewhere else (quests); Renewing the Covenant's ceremony (quests table).
| 2026-10-17 | `19ab1bb` | The early Gridania quests' notices checked against Bob's Chronofoil capture, notices numbered by todo (quests); Chasing Shadows' and Spirithold Broken's battle talk, bubbles and closing scenes, `complete_quest_battle`'s flags, NpcYell's fields (quest battles); battle talk known issue replaced by the heal line's.
| 2026-10-17 | `48e9522` | Regeneration (combat); in-combat HP regeneration (known issues).
| 2026-10-17 | `8e2be8d` | Leaving a zone with FATEs (travel); FATE packet order (known issues).
| 2026-10-17 | `32ba317` | Area attacks' edge, NPC moves, no wandering in content, straight paths when none (combat); To Guard a Guardian's sincallers and start facing, the quest battles' start spots (quest battles); the blanga's Terror Eye (known issues).
| 2026-10-17 | `8dd3b56` | Mapping the Realm, the whole-map reveal and shared discovery slots, the 31 unlocks in Bob's June–September sessions (character windows); To Guard a Guardian's sincallers, gargoyle, mage and scenery (quest battles); display flags and the gargoyle's wait (known issues).
| 2026-10-17 | `661aa3e` | Achievements: the list Types start in Key, Mapping the Realm on the first discovery, unique quests counted, and when retail completes them (character windows); ActorControl 518 (known issues).
| 2026-10-17 | `db6faaa` | To Guard a Guardian checked against Bob's Chronofoil capture (quest battles); `questkit chronofoil --battle` and who's in the fight (questkit); what the Recorder's raw packets can't be trusted for (recorder and diagnostics); packet layouts from retail captures (known issues).
| 2026-10-17 | `95ce15b` | Bob's Chronofoil captures and what's in them (outside sources, recorder and diagnostics); `questkit chronofoil` (questkit); packet sizes that differ on retail (known issues).
| 2026-10-17 | `b435ef4` | Outside Sources (new page): what to check when a recording leaves something unclear, the order of trust, Cactbot's ARR coverage; OverlayPlugin confirms the battle talk window's layout (quest battles).
| 2026-10-17 | `82f65de` | Placement, battle roles vs class roles (quest battles, combat, questkit).
| 2026-10-17 | `21d1075` | Roles (quest battles, questkit).
| 2026-10-17 | `c603df3` | The battle framework, `set_attack_range`, `keep_near` and `onUpdate`'s `info` (quest battles); staying near someone (combat); the outline and telegraphs (questkit); known issues.
| 2026-10-16 | `dbedea9` | Class rotations (combat); Recorder 2.7's class kits (recorder and diagnostics); the class profile (questkit); adding a quest battle; known issues.
| 2026-10-16 | `29ecfc9` | Recorder 2.6's battle files and sending to Kawari Web (recorder and diagnostics, Kawari Web); the report's new sections (questkit).
| 2026-10-16 | `ee05899` | Recorder 2.5's saved battles (recorder and diagnostics); `questkit battle` across runs (questkit).
| 2026-10-16 | `c2697fd` | Recorder 2.4 and what a recording can and can't tell (recorder and diagnostics); `questkit battle` (questkit, quest battles).
| 2026-10-16 | `9ef685a` | Combat: partial paths, no leashing in content; To Guard a Guardian's background fights; known issues.
| 2026-10-16 | `fe36e6c` | Quest battles: `warp_players`, To Guard a Guardian's second half, targeting and healing; known issues.
| 2026-10-16 | `6a4203e` | Achievements must be quick (character windows); connections desync when busy (known issues).
| 2026-10-16 | `de2d795` | Items That Teach Things (new page); achievements (Types 8, 20, counters); saving; known issues.
| 2026-10-16 | `53316cd` | Combat: leashing on the target's distance, the InCombat condition; the Adventurer Plate (character windows, opcodes, saving); known issues (teleport animation, the plate).
| 2026-10-16 | `feef465` | Quest unlock links (quests); how the client gates main menu commands, Recommendations and the Adventurer Plate (character windows, known issues).
| 2026-10-16 | `177bebd` | Debug commands only work in zone chat (tools, known issues).
| 2026-10-16 | `6d54f81` | Portraits: the window's disconnect was a server panic (Grand Company rank).
| 2026-10-16 | `048fde1` | Combat: recorded amounts and reading NPC-on-NPC damage from a duty capture; the two quest battles' targeting and healers; known issues.
| 2026-10-16 | `4416558` | Achievements, Portraits and Recommendations page; quest battles 12 and 14, the battle talk window; quests table (7 more); opcodes, saving, tools, known issues. |
| 2026-10-16 | `d69ba31` | questkit and CI pages; quests, overview, Kawari Web, diagnostics and known issues updated for them. |
| 2026-10-16 | `805fdbe` | This documentation section (`docs/src/fork/`), `CLAUDE.md`, and an updated `FORK_ROADMAP.md`. |

## Kawari Web (`FrameEnder/Kawari-Web`)

| Date | Commit | Change |
|---|---|---|
| 2026-10-17 | `4c80b86` | 2.15.0: Dashboard → Marketboard: every tab's rates, categories' changes, items' own prices and the listing settings, laid out like the board, written to `world.db`. |
| 2026-10-16 | `7133931` | 2.14.0: battles from Kawari Recorder (`POST /api/diag/battle`), kept and filed in the diagnostics repository's `battles/`; the Diagnostics page's Battles card; bundles name the battles from their window. |
| 2026-10-16 | `ad44611` | 2.13.0: Quest Lab's *Test this quest* (setup, capture on, a bundle tagged with the quest). |
| 2026-10-16 | `67b2f69` | README and CLAUDE.md point to this documentation. |
| 2026-10-16 | `c49ff92` | 2.12.0: the Shop (829 listings, gil and play coins, moogle mail delivery, admin editing). |
| 2026-10-15 | `5b8d9c4` | 2.11.0: quest progress tracker. |
| 2026-10-15 | `b3be94f` | 2.10.0. |

## kawari-diagnostics

| Date | Commit | Change |
|---|---|---|
| 2026-10-16 | `b699bca`, `b1d579e` | Recorder 2.7: the class mechanics in every action's info (combo predecessor, GCD group, level, self-buff, proc, charges) and each class's whole kit (actions and traits) in the battle. |
| 2026-10-16 | `25015ae`–`97e5538` | Recorder 2.6: hook health (in the tab and in every battle), your setup, each battle's own raw packets, enmity, tethers, event object states, positions every frame and fast moves, cast placement, the Action sheet's info per action, auto marks, Compare with the last run, field fights, and sending battles to Kawari Web. |
| 2026-10-16 | `b161d2b`, `b24206d` | Recorder 2.5: every battle saved to a file of its own (`battles/<start>-<territory>-<content>.json.gz`), kept until deleted, with labels, per-battle export and delete; exports take the battles not exported yet (or all); 2.4's `battles.json` is split into files. |
| 2026-10-16 | `ccedc94`–`efafdf2` | Recorder 2.4: the Battles tab (everyone's position, HP, target, casts and statuses every 250 ms, decoded actions, actor controls, casts, map effects, cutscenes, director events, marks, and the layout layers the NPCs came from with their AI settings); duty packets get their actor, the packets from zoning in and NPC/object spawns (up to 2.3 every packet read actor 0 and spawns weren't kept); `/krec mark`. |
| 2026-10-16 | `2ef6163` | CLAUDE.md: capture notes aren't written now; findings go in the fork's docs. |
| 2026-10-15 | `df8cec1`, `37d0eb7` | Palace of the Dead notes (recordings and published rules). |
| 2026-10-15 | `1c652c9` | Recorder 2.3: duty packets keep up to 1 KB. |
| 2026-10-15 | `9e37abb` | Blunderville notes. |
| 2026-10-15 | `0967111` | Recorder 2.2: every server packet inside duties. |
| 2026-10-14 | `6b1e4b5` | Recorder 2.1: the server packets that drive quests and duties. |
| 2026-10-14 | `f1cb42f` … `03a9b32` | Kawari Recorder 1.0–2.0 (replaces Bestiary). |
| 2026-10-14 | `f9618ac` … `2a50ef7` | Kawari Bestiary. |
| 2026-10-13 | `469e4fb` … `c1d4d56` | Repository, Kawari Probe 0.1–0.6.5. |

## Kawari (fork)

Commits on `Latest-Quest` since the fork point (`8f65e4b`, upstream v7.56.1). `master` was fast-forwarded to `Latest-Quest` on 2026-10-17 with Bob's OK (the commit that says so, right after `ee5e162`).

### 2026-10-18

| Commit | Change |
|---|---|
| — | Teleports cost gil as retail's client works it out (TerritoryTypeTelepo and TelepoRelay, exact for 87 teleports; favourites half, the free destination nothing), taken as the cast lands with LogMessage 4590, refused if unaffordable; NPCs pay a tenth more for materia; dying wears gear by 600. Workspace icarus features TelepoRelay and TerritoryTypeTelepo. |
| `3247ebd` | Class actions, statuses and Raise from Bob's Chronofoil captures: 24 traited actions' potencies set to retail's for levels 1–50 (measured from synced party members); Sheltron 1856, Temperance's party status 1873, no Rampart healing bonus below 94, Firestarter/Thunderhead permanent and Thunderhead only granted when absent, Shield Samba for the party, Cover's param, True Grit and Acorn Armageddon durations; self-status effects' 0x80, the target's EffectResult naming the target, ActorControl 20 without a source; Raise and Resurrection offer a revive and accepting it gets the player up in place with 20 % HP/MP and Weakness (client trigger 200's choice read). |
| `82a83ec` | Fixes from a review of today's changes: Defeated is actually sent (its task was being cancelled with the dead actor's); quest reward log lines only for items given, with the optional reward's count; ExecuteCombo only on players' actions (not auto-attacks or NPCs); patrols walk; idle enemy regeneration only out in the world; the duty clock stops when the duty is complete; mounts and items keep their half-second slidecast window; FATE rewards only for those who fought its enemies and are still there; enmity can't overflow. ActionRequestGroundTargeted's 846 (FFXIVOpcodes 7.57) marked current so it's used. |
| `daec1d7` | No kill chains in duties, and no kill EXP for players synced down in one (retail). |
| `6e4487e` | Rewards from Bob's Chronofoil captures: completed FATEs give whoever helped defeat their enemies retail's gold-rank EXP (ScaledQuestXP × QuestExpModifier at the FATE's level) and gil (twice its level); quest turn-ins send the gil and item log lines (LogMessage 1605/1606/1607) first and the EXP last, as retail. |
| `234465d` | Enmity and packets from Bob's Chronofoil captures: tank stances ×10 enmity, damage over time ticks credit their source, healing adds half its effective amount to every enemy with the healer on its list; the HaterList and EnmityList with real percentages, sent only when they change; ActionRequestGroundTargeted (fork opcode 846, layout from the captures) handled as an action on oneself; AoeEffect's effects at byte 42; PlayDismountAnimation with the landing position; ContentFinderCommencing with the duty ID; a duty's time left re-sent every 300 s. |
| `3affb9a` | EXP and weather from Bob's Chronofoil captures: quest EXP at no more than the quest's level; kill EXP reduced for enemies more than 2 levels below; chains of 0/20/25/30/40/50 % with 100/80/60/40/20/10 s windows and ActorControlSelf 12; retail's rested EXP formula; the EXP message's source enemy, SetEXP after a gain (stats and class info only on level-up), the level-up's effective level; drops obtained with LootMessage (fork opcode 190) and new drop chances; the weather forecast in exact integers with `<`, changing at each 1400 s boundary with a 20 s fade. |
| `76bbc80` | Combat timing from Bob's Chronofoil captures: combos changed only by actions in a combo (other GCDs don't break them), ExecuteCombo on every action (1 when continuing); per-action animation locks and 0.1 s for mounts; players' casts land 0.5 s before the bar ends (moving after that doesn't interrupt); ActorControlSelf 17 recasts sent for every action (GCD sped up like a cast time); auto-attacks every weapon delay × haste; DoT ticks as ActorControl 1541 and HoT ticks as 1540 (0, amount, source, critical), ±5 %. |
| `75a7c91` | Enemies and regeneration from Bob's Chronofoil captures: field enemies' HP by expansion (Heavensward 50–60, Shadowbringers 70–80); NPC misses by level gap (2/12/25/40 %), parries 0.83×; NPC cast bars 0.3 s shorter than the cast; running 6 and walking 2 yalms a second with the speed byte (players walk at 24); aggro 11 yalms all round or 15 in a ±45° cone, up to 8 levels below; no instant heal on a leash, enemies out of combat regain 10 % a tick; Defeated (ActorControl 6) 2 s after a death, fade at 8.05 s, delete 1.75 s later, respawns by level (15.7/30.9/61 s). Players: HP regen 1 % a tick in combat and floor(10 %) out of it, MP's Piety bonus from the level's main stat; UpdateHpMpTp's max MP. |
| `d0b51b7` | Player max HP as retail 7.x (`retail_max_hp`): base × job HP modifier + HP per Vitality above the level's main stat, the per-level numbers fitted from Bob's Chronofoil captures (his own PlayerStats on ten jobs and 82 level-synced party members whose capped gear gives their Vitality); tanks (ClassJob Role 1) get more per Vitality. Was about half of retail from 18 to 50. |
| `b65b947` | Retainers on 2026.10.06 (Bob's capture of 2026-10-18): hiring at the retainer vocate (its scenes and yields, SetRetainerAppearance 670, the name as an event text on 308, RetainerState hired, ActorControlSelf 152) and letting go (refused with items on the market); entrust/withdraw/gil from the retainer's menu (ItemOperation BASE+22/23/24, the server picking the slot) and client trigger 419; price changes (client trigger 400); RetainerStateNotice (144); RequestPlayerRetainerInfo 818 and RequestRetainerSaleHistory 274; the bell's condition 23. The aethernet shard's attunement scene with retail's flags and params. |
| `7d32889` | Level sync and the class kits. Level sync as retail (Bob's captures): UpdateClassInfo's current level is the synced one, a duty's own item level sync (ContentFinderCondition.ItemLevelSync), gear capped per attribute (ItemLevel × slot share × MeldParam, defense by item level ratio: exact at sync 50), FATE sync honouring on/off and the FATE's level, unsync (FateLevelSynced 0) when the FATE ends, HP/MP cut to the new maximum. Gear: HQ bonuses counted, PlayerStats' base values naked, the weapon's delay. Combat (`server/modifiers.rs`): ±5 % variance, critical and direct hits for players, damage/healing statuses (Fight or Flight, Rampart, Feint...), Tank Mastery, Hallowed Ground and Holmgang, Stun/Sleep/Bind stop enemies; combos only advance on spells and weaponskills (abilities don't break them), retail's animation locks, unscripted actions still answered, statuses count down in StatusEffectList and keep their source, interrupts only cancel casts. Scripts for every class/job action 1–50 and the role actions, `provoke`/`cleanse`, `player.is_player`/`hp`/`max_hp`; combat items (potions, ethers, elixirs, status removal, food). Workspace icarus feature `Status`. |
| `bb5cd40` | Retainers: a `retainer` table and `!retainer hire/list/release`; summoning bells list them (client trigger 9003 commands 2, 3 and 4), call one (RetainerState, their containers and market prices, their NPC, their menu) and send them away; entrusting, withdrawing and putting items up for sale work on their containers, which are saved after each change; PlayerRetainerInfo and an empty sale history. RetainerInfo's and RetainerInfoEnd's fields named from retail. Unhandled item operations warn instead of panicking. From Bob's Chronofoil captures of 2026-09-15; opcodes from FFXIVOpcodes, not yet seen working. |
| `2a6e491` | Claims: the first to attack an enemy (or an enemy a player) is sent to everyone nearby (FirstAttack, 374), right before the hit's results, and cleared when an enemy gives up a chase or a player's fight is over; the claimer is in CommonSpawn too. From Bob's Chronofoil captures; `first_attack_layout` test. |

### 2026-10-17

| Commit | Change |
|---|---|
| `b0a07af` | questkit holds every quest and quest battle to Bob's Chronofoil captures. `questkit chronofoil --quest` reads a quest's talks, scenes (flags, the client's answers), notices, updates, zone changes and unlocks; `questkit fixture --chronofoil` makes fixtures from them, and replay now fails on other scenes, other flags or other things done outside the quest. All 28 fixtures were remade from the 2026-10-14 to 17 captures, which found 15 scripts unlike retail, all fixed: cutscenes with `SET_BASE \| HIDE_HOTBAR` (0xF8402EFB) instead of `FADE_OUT \| CONDITION_CUTSCENE \| HIDE_UI` (Close to Home, Spirithold Broken, Leia's Legacy, On to Bentbranch, Chasing Shadows, Dread Is in the Air, To Guard a Guardian, Renewing the Covenant, The Gridanian Envoy); searches, aids and examinations play the object's scene, then the bar, then its "found it" scene (Surveying the Damage, Don't Look Down, In the Grim Darkness of the Forest, Migrant Marauders, Salvaging the Scene, A Soldier's Breakfast, Spirithold Broken, Chasing Shadows' second blade); The Gridanian Envoy moves you through a loading screen within a zone and stays put when the attendant gets a No; a few scenes' own flags. `questkit battle-sim` runs each quest battle's director against retail's run (battle fixtures in `tools/questkit/battles/`), which retimed Chasing Shadows and Spirithold Broken from the fight's start (they were 8-10 s late) and corrected Sapphire's health marks (Chasing Shadows' second wave 45 %; Spirithold Broken 70 % and 45 %), To Guard a Guardian's last line (42 %) and its win (2 s after the mage falls), and the lines after a group falls in Chasing Shadows and Underneath the Sultantree. `questkit lint` finds unscripted NPC actions, quest battle NPCs without timelines and checked scripts without fixtures: Chasing Shadows' frenzied oaks have a timeline now. All in CI; tests. |
| `5f16817` | Ul'dah's next quests, checked against Bob's 2026-10-17 retail run (Recorder export, Chronofoil session and the client's scripts from the Quest Lab): Close to Home (568) rewritten from upstream's stub, We Must Rebuild (595), Nothing to See Here (671) and Underneath the Sultantree (550) new; Coming to Ul'dah (594) already matched. Quest battle 37 (Underneath the Sultantree): the blanga, the stranger and three pairs of lesser blangas (drop-in layer `QB_ManWil005_001`, retail's spawns) at retail's health marks, battle talk and timings, with timelines from the capture's hits and three new action scripts (Bloody Claw, Rage of Halone, Spirits Within). NPC rotations use a combo step on its own when the step before it isn't in their kit (retail's stranger uses Rage of Halone without Riot Blade); test in `rotation.rs`. questkit: talk steps carry the target's layout ID (`target.object_id`), scenes can come back as yields, composite scene flags (`SET_BASE`) are read, fixtures find quest battle wins and arrivals by territory (tests). |
| `3f9a8bc` | Central Thanalan has its monsters: 632 spawn points (drop-in `wil_w1_planmap.json`) with retail's levels, HP, aggression, links, positions and facings, from Bob's Chronofoil sessions of 2026-10-17, and timelines (retail hitboxes, auto-attacks and damage where recorded) for the 17 kinds that had none. `questkit chronofoil --dropin TERRITORY` makes a zone's drop-in from captures (tested). |
| `7211858` | Fast-forward `master` to `Latest-Quest` (changelog only). |
| `ee5e162` | Marketboard fixes from the 2026-10-17 bundle: the board was empty (every search "No matching items", every item without listings) because the ItemSearchCategory sheet was read without a language and failed, leaving no item with a tab; it's read in the server's language now, and an index without categories isn't kept. A search's category is one byte followed by a flag (the weapons and armor tabs sent 271 for Conjurer's Arms, 15 plus a flag of 1). The packet the client sends as the window opens (fork opcode MarketBoardOpened, 683) is answered with an empty result, as retail does. |
| `86bcfa2` | The marketboard: every item with an ItemSearchCategory is listed, 50 listings each (10 HQ for items that can be, random stacks up to 99 or the stack size), priced from rates per tab (weapons and armor 100 a level, items 3, housing 50, plus a rarity bonus; HQ doubles the bonus), per search category or per item, which Kawari Web edits in `world.db` (migration `update-2026-10-17_marketboard`). Search, item history (real sales, then made-up ones) and buying (gil plus 5% tax; a bought listing is replaced). The packets were rewritten from Bob's Chronofoil captures (all 350 retail packets round-trip); two fork opcodes for buying (FFXIVOpcodes' 258 and 610). HQ and NQ no longer share an inventory stack. `kawari-questdump items` writes the marketboard's item fields and `marketboard-categories.json`. Tests in `marketboard.rs` and `database/marketboard.rs`. |
| `7f95a59` | Scripts' replies to server-side triggers go out at once: what a script queued from a server message (an event action's progress bar ending, a kill, a won quest battle) waited for the client's next packet, so objects answered several seconds after their bar. The Gridanian Envoy: boarding works (the gangplank's yes/no answers through its error code, 50 for No, and its "Yes" returns results `[0, 0]`, which the script took for No), with retail's check scene before the bar and its scene flags. Scripts can read `player.scene_error_code`; questkit fixtures can give a scene's error code, and 507's gangplank steps answer as retail did. |
| `f4ffd30` | Renewing the Covenant: the ceremony cutscene moves you to the Roost (POPRANGE0, territory 179) as retail does; it only finished the event, which left the screen black (the cutscene fades out and the client waits for the zone change), skipped or not. |
| `19ab1bb` | Quest progress pop-ups (notices) for Close to Home, Spirithold Broken, Surveying the Damage, Chasing Shadows, Eggs over Queasy and A Soldier's Breakfast, which sent none: retail's notices from Bob's Chronofoil capture of 2026-10-14 were added to their fixtures, and all 23 fixtures replay like retail. Chasing Shadows and Spirithold Broken show their battle talk in the battle talk window with retail's text IDs instead of `/say`; Yda's speech bubble (NpcYell 117) after Papalymo's; Spirithold Broken closes with retail's scene 9 as a cutscene (`complete_quest_battle` takes the closing scene's flags) instead of two chat lines and scene 6. |
| `48e9522` | Players regenerate MP in combat as on retail (200 a tick plus Piety's bonus; 600 out of combat), on a 3 s tick instead of every 500 ms: casters ran out of MP mid-fight and stayed out. |
| `8e2be8d` | Leaving a zone takes its FATEs down as retail does (2352 for each start NPC, the new `RemoveFateContext`, 2357, for every FATE, then the FATE director): only terminating the director crashed the client in its FATE update when teleporting from the Central Shroud to Gridania. |
| `32ba317` | Enemy area attacks count half a yalm past their edge, not a whole yalm (players who had left Terror Eye were still hit). NPCs are only sent an ActorMove when they move or turn (idle NPCs flickered between walking and standing, and flooded the open world); NPCs in content don't wander, and walk straight at a target they can't path to instead of dropping it (To Guard a Guardian's mage walking off, the second wave looping around the tree). To Guard a Guardian: sincallers kept back by range alone (they stood inside each other); all four quest battles start the player at retail's spot and facing (To Guard a Guardian's was nearly backwards). Test in `action.rs`. |
| `8dd3b56` | A Realm Reborn's field zones reveal their whole map on the first visit (LocationDiscovered part 164, as retail), which completes their Mapping the Realm; the rest need every part again, counted over every map sharing the discovery slot (Midas II and the like completed after one floor otherwise). Checked against the 31 unlocks in Bob's June–September sessions. To Guard a Guardian: the sincallers stay back behind Quzal, the gargoyle waits until you've loaded in, the mage also comes when the gargoyle falls, and the background fighters have retail's display flags (no health bar, not targetable; drop-in battle NPCs take `display_flags`). |
| `661aa3e` | Achievements read the sheet as retail's rows are laid out: Types 2, 6 and 9 list Key and then Data, Type 3 is class Key at level Data[0] (Types 3 and 6 never completed before); Mapping the Realm completes with the map's first discovered part, as on retail; Storyteller's counter (Key 214, unique quests) is counted from the completed quests. Checked against four unlocks in Bob's Chronofoil captures; tests in `achievement.rs`. |
| `db6faaa` | To Guard a Guardian from Bob's Chronofoil capture of the same run: sequence 4 at 34 %, Void Call at 27 % with sequence 5 and the blanga when it lands, the second wave's slowbeaks swapped, both sincallers start on Galfrid, the masked mage passive until hit (not hostile in the drop-in; Battle.lua's `passive`), Papalymo on the adds first and his Cure below 70 %. `questkit chronofoil --battle N.V` turns a capture's duty into a battle run (`chronofoil_battle.py`); `questkit battle` only counts NPCs in your fight and picks an engaged boss. |
| `95ce15b` | `questkit chronofoil`: reads Chronofoil captures (finished and unfinished), indexes each session's zones and duties with what happened there, and exports a visit as a Kawari packet capture; opcode tables for game versions 2026.06.18, 09.01 and 09.15 (`tools/questkit/questkit/data/chronofoil_opcodes.json`, derived in `chronofoil_opcodes.py`), checked against every packet's size in Bob's captures. CI installs zstandard. |
| `b435ef4` | Chasing Shadows' comment on battle talk brought up to date (the window is known; its text IDs aren't recorded yet). |
| `82f65de` | Placement: `keep_near` takes several anchors (Papalymo near Yda and you), `stand_behind` puts an NPC behind the enemy the tank is fighting (`server/placement.rs`); supports default to it. Roles: a battle role (`role`) apart from the class's (`class_role`): in Chasing Shadows Yda, a DPS, plays the tank, and Papalymo is a DPS with some support. questkit only calls an NPC a support when healing is a large part of what it does. |
| `21d1075` | Roles in the battle framework: tank (leads, pulls enemies off the others: supports first, then DPS, then the player), DPS (highest health first, two or more split the toughest enemies, can heal themselves), support (backline, near the tank, buffs DPS then tank, heals the tank first); enemies take roles too. `onUpdate`'s `info` gains each actor's target. `questkit battle`'s outline gives each ally's role, by class and by what it did. |
| `c603df3` | The battle framework (`content/lib/Battle.lua`, loaded before every quest battle's script): allies' roles (focus groups, order, range, staying near someone, heals), enemies' groups and first targets, context changes, timed steps and health/cleared phases. Chasing Shadows and To Guard a Guardian run on it (simulated: the same director calls and targets as before; Papalymo now casts from 10 yalms and stays within 15 of you). The director gets `set_attack_range` and `keep_near` (NPCs head back to their anchor), and `onUpdate` a fourth argument with everyone's HP, max HP and position. `questkit battle` drafts the director's outline per phase (focus, order, range, staying near, heals) and lists enemy telegraphs. |

### 2026-10-16

| Commit | Change |
|---|---|
| `dbedea9` | Class rotations for NPCs: a timeline's `rotation` (pace and priority-ordered actions with `every`, `refresh`, `then`, HP conditions), run every tick by `server/rotation.rs` with the Action sheet's GCD group, recast and combo predecessor (`GameData::get_action_mechanics`). `questkit battle` profiles each NPC by its class (pace, combos, abilities on cooldown, buff follow-ups, re-applied debuffs, unused kit) and writes its `rotation`. |
| `29ecfc9` | `questkit battle` reports Recorder 2.6's additions: failed hooks, your setup, action names, tethers, event objects, fast moves, auto marks, your enmity, and actor controls by category (named from `ActorControlCategory`). |
| `ee05899` | `questkit battle` reads any number of battles (exports, the Recorder's battles folder, single `.json.gz` files, 2.4's `battles.json`; labels from the index) and, for several runs of the same battle, adds up each NPC's evidence across them. |
| `c2697fd` | `questkit battle`: a report on a recorded duty or quest battle (Recorder 2.4's `battles.json`): who's in it with their layout layer and AI settings, waves and what preceded them, each NPC's targets with the explanations that fit every change (counted), actions, intervals and heals, and who never acted. Tested on made-up data. |
| `9ef685a` | To Guard a Guardian could never end: after Quzal fell, the last two Ixali were on a background guard 9 yalms past where the navmesh stops, and every ally stood at the edge (2026-10-16 bundle). NPCs given a partial path now walk straight on to their target (up to 15 yalms out in the world, any distance in content; Detour's success-but-partial status is no longer logged as an error), and enemies in content don't leash (those lostwings had leashed chasing Miraudont the long way round, then picked up the guard). The background fights are the layer's five groups (each sincaller on a guard, the melee Ixali in pairs), as the Recorder shows, so no Ixal or guard stands idle. |
| `fe36e6c` | To Guard a Guardian: the cutscene now ends with retail's loading screen into the second half (director `warp_players`: PrepareZoning to the same territory, then ActorSetPos), which the client waited for; Miraudont and the guards clear the adds before turning on Quzal; healing follows the Recorder's count of each NPC's Cures (the guards healed by the bow, Miraudont and Galfrid, Quzal by the sincallers, the player once by Papalymo), where before everyone also healed themselves and Miraudont was given the player's own Cures. |
| `6a4203e` | Fix players freezing (or reviving into endless loading) after finishing a quest or discovering a map part: the achievement check read the Map sheet for every Mapping the Realm achievement, taking long enough to desync the client's connection (2026-10-16 bundles). The Map sheet's discovery columns are read once now; a check over 50 ms is logged. |
| `de2d795` | Items that teach things work: mounts, unlock links (emotes, hairstyles…), barding, Triple Triad cards, fashion accessories and facewear are learned, as well as minions and orchestrion rolls (which no longer un-learn on a second use), and the item is used up. Achievements: Mapping the Realm (Type 8, the whole map discovered), Freebird (Type 20) and "Defeat N enemies" (counter 11), with counters saved (migration `update-2026-10-16-3_achievement_counters`); checked on discovering a map part, attuning aether currents and defeating an enemy. |
| `53316cd` | The adventurer plate migration is `update-2026-10-16-2_adventurer_plate`: as `update-2026-10-16_adventurer_plate` it had the same version as the achievements one (Diesel's version is the part before `_`), so it wouldn't have run on a database that already had that one, and a new database skipped the achievements migration. |
| `70fa284` | Adventurer plates are saved: the client's UpdateAdventurerPlate (opcode 795) is stored and sent back in AdventurerPlate (CharaCardData, migration `update-2026-10-16-2_adventurer_plate`); the plate shows the class/job's real level, and upstream's misplaced glasses/base plate are fixed. Enemies give up once the player they're after is more than 40 yalms from their spawn (enemies with no navmesh path never moved, so never leashed, and kept players in combat: no regeneration, 2026-10-16 bundle). The server sets the player's InCombat condition while enemies are after them, as retail does, so the client refuses Teleport and Return in combat. |
| `feef465` | Quests grant their unlock links (`REWARD…` parameters) when finished, and at login for quests already finished: Spirithold Broken's unlocks the Recommendations window (unlock link 9). |
| `177bebd` | `!achievement check` says how many achievements it completed (it was silent when there were none). |
| `6d54f81` | Fix the server panic (and the client's 90002 disconnect) when opening the Portraits window as a character in no Grand Company: the adventurer plate read the rank at index `0 - 1`. |
| `048fde1` | Quest battles as on retail: timelines' `recorded_amounts` (what each NPC's action did on retail to players and to other NPCs, damage and heals, read from the duty capture's ActionEffect packets), retail's targeting (hedge knights on Kuplu Kopo; Quzal, the Ixali and the guards on each other) and healers (Cure thresholds fitted to the recording) in Leia's Legacy and To Guard a Guardian; skill rates from the recorded use counts; scripts for the 24 actions those NPCs use that had none (enemy auto-attack 873 among them, so the bows, Galfrid and the hedge knights attack at all). |
| `4416558` | The rest of Gridania's story: Leia's Legacy (161), Dread Is in the Air (446), To Guard a Guardian (447), Festive Endeavors (448), Renewing the Covenant (449), The Gridanian Envoy (507) and Call of the Sea (674), with replay fixtures. Quest battles 14 (Leia's Legacy) and 12 (To Guard a Guardian) from the 2026-10-16 recording: drop-in layers `QB_SubFst055_001` and `QB_ManFst007_001`, 23 timelines. Directors: the battle talk window (`battle_talk`, DirectorPopupMessage, whose kind is now one byte), a finish scene per battle (`complete_quest_battle`'s third argument), `play_scene` with `onSceneFinished`, `despawn_bnpc`; `onActorDeath` is skipped for directors and FATEs that don't define it. Achievements: the window's list (server opcode 991), progress, near-completion replies, completion with the pop-up for quest/level/achievement Types, `!achievement`. Portraits: the adventurer plate's portrait is saved (client opcode 107) and sent back. Migration `update-2026-10-16_achievements_portrait`. |
| `d69ba31` | questkit (`tools/questkit`): bundle triage, quest replay tests with fixtures for all 16 scripted quests, the quest scaffolder, capture/retail comparison and a Lua syntax check; GitHub Actions checks (`.github/workflows/fork.yml`, `tools/ci/check-changes.sh`). Fixes the replay found: Close to Home counts its objectives in UI8AH and sets BitFlag8 1 when attuning, Chasing Shadows clears UI8AL as the battle starts, as retail does. Fixes upstream's `FesHlwTransform_00946.lua`, which didn't compile. |
| `805fdbe` | Document the fork: every feature and mechanic, in docs/src/fork |
| `a0aec12` | Save playtime with autosave; kawari-questdump items for Kawari Web's shop |

### 2026-10-15

| Commit | Change |
|---|---|
| `955b0ce` | Run an action's script tasks straight away, so Teleport warps when its cast ends |
| `2590f47` | Save progress every 30 seconds, and notice connections that dropped silently |
| `9e40e38` | Casts end with their cast bar, can be slidecast; enemy hits interrupt quest searches |
| `f92f557` | Script the Gridania quests up to Salvaging the Scene (379, 380, 381, 384, 387) |
| `5f27926` | Script Don't Look Down (quest 377) |
| `d28f9cf` | Script You Shall Not Trespass (quest 376), with the quest progress pop-up |
| `b073a3c` | Unlock Teleport once a second aetheryte is attuned |
| `ba3db92` | Merge opcode-scan (upstream 7.57 and the fork opcode scanner) into Latest-Quest |
| `8791c78` | questdump index: each quest's expansion, journal section and main scenario flag |
| `c320e5e` | questdump index: each quest's expansion, journal section and main scenario flag |
| `93bba11` | Read the fork's opcodes from the 7.57 executable: EventReturnHandler16 is 200 |
| `bc46eb3` | Read the fork's opcodes out of the game's executable at startup |
| `c08aeef` | Keep the fork's own opcodes in fork_opcodes.yml, with the version each was found on |

### 2026-10-14

| Commit | Change |
|---|---|
| `8e1a63e` | Merge upstream Kawari from Codeberg (through b1e7e2b, patch 7.57) |
| `74c308e` | Merge upstream (GitHub mirror, final commit before the move to Codeberg) |
| `da5ef28` | On to Bentbranch: accepting already starts at sequence 255 |
| `1b96208` | Script On to Bentbranch (quest 176) |
| `5040198` | Duty Complete as the clay golem falls; level-ups announce only the new levels |
| `6b304e0` | Spirithold Broken: talking to the Hearer at sequence 3 offers the quest battle |
| `58aae5e` | Define MAP_DISCOVERY_BITMASK_SIZE in constants.yml |
| `90b9858` | Save map discovery, and stop doubling rested EXP |
| `28f4635` | Script Spirithold Broken (quest 129) and its quest battle (15) |
| `8e46dbc` | Enemy area attacks hit every player inside them |
| `d9b9b68` | Enemy area attacks, aggro, leashing and the level gap |
| `cfb69d8` | Quest kill credit by BNpcBase, and event action casts finish on time |
| `24045b0` | Script A Soldier's Breakfast (quest 3855) |
| `2316f43` | Handle EventReturnHandler16 (client opcode 856), so Pauline takes the Surveying the Damage items |
| `2ab22c6` | Script Surveying the Damage (quest 175) |
| `888489f` | Attuning to aetherytes and aethernet shards plays the attunement animation and progress bar |
| `49c68f0` | Players deal and heal what retail does: main stats from the level's main modifier, retail damage formula |
| `2e21715` | Fill in what the live recording (first quest to Spirithold Broken) showed |
| `9517325` | Stop the world server dropping players whose message queue fills up |
| `acb93c2` | Enemies hit players for what they do on retail, with crits, and drains heal |
| `9bfdac6` | Central Shroud enemies from Kawari Bestiary's recording of retail |
| `731bf0f` | Give ordinary enemies Sapphire's per-level health |
| `92d04bc` | Scenario Guide: skip the other variants of a step once one is done |
| `bdbca42` | Scenario Guide: read ScenarioTree in a language it has, and log why quests are skipped |
| `c156537` | Log what the Scenario Guide is sent |
| `daf8d4a` | Fix striking dummies, sheathing during auto-attack, and the Scenario Guide |
| `95464be` | Give EXP and drops for defeated enemies |

### 2026-10-13

| Commit | Change |
|---|---|
| `7cac6c4` | Implement Eggs over Queasy, and let quests count defeated enemies |
| `38d4659` | Chasing Shadows: Yda and Papalymo's battle chatter |
| `9367259` | Chasing Shadows: Papalymo heals the player, and Yda's combo is a little slower |
| `a06e5ed` | Chasing Shadows: set the director's sequence, so losing can show "Duty Failed" |
| `a4a847e` | Add !directorscene, to look for a content director's generic scenes |
| `3e91c63` | questdump: extract game_script files by name alone |
| `6d62594` | questdump: skip game paths that don't exist when extracting |
| `0fd5256` | TEMPORARY: wait 60 seconds after falling in Chasing Shadows, to test the director's failure update |
| `2cb34d6` | Scale NPC damage against players down to retail-like amounts |
| `f5eb759` | Give Yda a Pugilist rotation and Papalymo a Thaumaturge one |
| `ff7926e` | Chasing Shadows: Yda takes the boss, Papalymo casts at the adds, and NPCs keep their stance |
| `8275c99` | Give NPCs enmity, a per-NPC attack timer, and stop them jumping when they change target |
| `71e37c9` | Fail Chasing Shadows when the player falls, and keep NPCs in their battle stance |
| `3414427` | Let Yda and Papalymo fight in Chasing Shadows |
| `f4e7ad8` | Answer the quest battle prompt's request so it opens by itself |
| `20538c9` | Let directors watch NPC health, and bring in Chasing Shadows' enemy waves |
| `e2e19a2` | Note what's known about the quest battle prompt request |
| `0663bfc` | Fix the world server crashing when a quest battle starts |
| `4895fa6` | Add !rawsweep debug command for finding unknown server opcodes |
| `6c9f669` | Answer the client's request to open the quest battle prompt |
| `5be9762` | End a quest's event action cast before continuing the event |
| `6506b09` | Implement Chasing Shadows |
| `11dc8b7` | Add quest battles and quest event actions |
| `76f6d79` | Put gear replaced from the main inventory into the Armoury Chest, like the client does |
| `3eb437d` | Give To the Bannock's reward the player picks |
| `ad4279e` | Implement Passing Muster, and give the optional quest reward the player picks |
| `905beaa` | Implement To the Bannock |
| `f5dc99f` | Add !rawpacket debug command for testing packet layouts |
| `3f2d23e` | Put quest variables where the client reads them |
| `7c139e3` | Implement Close to Home (Conjurer) |
| `9184958` | Don't remove instances while their navmesh is being generated |
| `3fb25fe` | Roll back the quest work to the White Mage state |
| `ff8090e` | Note packet capture and the diagnostics pipeline in the roadmap |
| `ed897d8` | Add optional packet capture to the world server |

### 2026-10-12

| Commit | Change |
|---|---|
| `bcf882f` | *(rolled back in `3fb25fe`)* Log why zone clients get dropped, add !questvar for testing |
| `02e22b5` | *(rolled back in `3fb25fe`)* Finish enter-territory events for scripts that don't handle them |
| `4070c7c` | *(rolled back in `3fb25fe`)* Only save the zone and quests on zone change |
| `4ed1233` | *(rolled back in `3fb25fe`)* Fix quest variables never reaching the client |
| `a8a7bf3` | *(rolled back in `3fb25fe`)* Log quest progress changes and implement //gm quest inspect |
| `a30ad51` | *(rolled back in `3fb25fe`)* Move players out of the opening area at login instead of mid-load |
| `d543ce9` | *(rolled back in `3fb25fe`)* Send players past Close to Home out of the Gridania opening area |
| `7486d2f` | *(rolled back in `3fb25fe`)* Stop the opening's city boundaries once the opening quest is done |
| `f29eede` | *(rolled back in `3fb25fe`)* Fix Gridania quests against the client's quest scripts, add the last four |
| `4556bf1` | *(kept by the rollback)* Add a quest index to kawari-questdump |
| `effe700` | *(kept by the rollback)* Add kawari-questdump for pulling quest data and client scripts from the game |

### 2026-10-11

| Commit | Change |
|---|---|
| `b27a8ce` | *(rolled back in `3fb25fe`)* Script 16 Gridania story quests after Close to Home |
| `164fa73` | *(rolled back in `3fb25fe`)* Add quest progression and finish Close to Home |
| `e164184` | Merge White Mage kit |
| `e36fa77` | Add a roadmap for the fork |
| `dc63634` | Add a full White Mage kit and the healer engine support it needs |
