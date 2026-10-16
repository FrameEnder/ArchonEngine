# Combat and NPCs

What the fork changed about fighting: how damage is worked out, how enemies behave, what they give when they die, and
the White Mage kit. Numbers come from retail recordings (Kawari Bestiary, then Kawari Recorder) unless a line says
otherwise.

## Player stats and damage

**Stats** (`zone_connection/stats.rs`, `use_retail_formula`): a player's main stats start from the level's **main
modifier** (`level_main`, Sapphire's table, levels 1–10 checked against retail) instead of ParamGrow `BaseSpeed`.
Determination starts there too; Critical Hit and Direct Hit at `BaseSpeed`, as retail shows. Before this a level 7
Conjurer had 81 Mind (retail 35–38) and killed everything in one or two hits. HP is unchanged (upstream's formula).

**Damage and healing** use retail's formula as Sapphire has it:

```text
damage = potency / 100 × f(WD) × f(AP) × f(DET)

f(WD)  = floor(level_main × class modifier for the stat / 1000) + weapon damage (physical or magic)
f(AP)  = floor(125 × (attack power − level_main) / level_div + 100) / 100      (ATTACK_POWER_SCALE = 125)
f(DET) = floor(130 × (determination − level_main) / level_div + 1000) / 1000  (DETERMINATION_SCALE = 130)
auto-attacks: f(WD) × weapon delay / 3 s, potency 90
```

`level_div` is ParamGrow's `LevelModifier`. Attack power is Strength, or Dexterity for Dexterity classes; magic attack
is Intelligence, or **Mind for healers**; healing potency is Mind. Stone, Aero, Cure and auto-attacks land within a few
percent of the recording at levels 6–9. Variance, criticals and traits are applied by the caller.

## Enemy damage to players

Enemies still compute damage like players do (stat × potency), which is far too high, so it is scaled
(`server/action.rs`, `npc_damage_scale`):

* If the enemy's **timeline** records its retail auto-attack damage by level (`damage_to_players` in
  `resources/timelines/<Name>_<BNpcBase>.json`), its auto-attack hits for that, and its other actions for that ×
  their potency / 100. Other levels scale with the per-level HP table. Filled in for the Central Shroud monsters.
* Otherwise it deals 12 % (`NPC_DAMAGE_TO_PLAYERS_PERCENT`). Retail's enemy formula isn't published.
* Every hit on a player varies ±5 %, crits 5 % of the time for 1.5×, misses 8 %, is parried 5 % for 80 % damage.
* **Recorded amounts** come first: a timeline's `recorded_amounts` (`{"<action>": {"players": avg, "npcs": avg}}`)
  says what that NPC's action did on retail to a player and to another NPC. The hit is scaled to it (then varied,
  missed, parried or critical as above), and a heal (Cure) is set to it ±5 %. Used for quest battle allies and
  enemies (Leia's Legacy, To Guard a Guardian), so they wear each other down, and heal, as fast as on retail.
* NPCs hitting other NPCs without a recorded amount are not scaled.
* Drains (`heal_self` effect, e.g. Blood Drain) heal the attacker by what they took; the attacker's HP is sent after.

Timeline fields the fork uses: `hitbox_radius` (reach includes it), `autoattack_action_id` (870/871/872 per kind),
`attack_range` (default 5 yalms; Papalymo 10), `damage_to_players` (`{"level": damage}`), `recorded_amounts` (above),
and `timepoints` (the
skills an enemy uses in turn). A trailing no-op timepoint keeps the real ones evenly spaced because the loop hits its
last point half as often. Sprites, roselets and floating eyes cast their spell instead of auto-attacking.

### Reading NPC-on-NPC damage from a recording

The Recorder's Monsters tab only keeps hits on the player, but its duty capture (`content_packets.json`) keeps every
ActionEffect1 (189) inside a duty, so who hit whom for how much can be read from it. Two fields are obfuscated per
session: the **action ID** has a key added (25 and 183 in the two 2026-10-16 runs: find it as the offset that turns
the most actions into enemy auto-attacks, 870–873), and each effect's **amount** (u16 at byte 6 of the 8-byte effect)
is **XORed** with a key, which is what the empty effect slots hold (`0xF8C5`, `0x843B`). Checked against the
Recorder's own damage-to-player numbers. The layout from the IPC data: animation target (8), action ID (8), global
sequence (12), …, effects from byte 42 (8 × 8: kind, then the amount at +6), target ID at byte 112. Targets are
told apart by their maximum HP (EffectResultBasic, 549: target at byte 8, HP at 12); attackers by which action they
use (e.g. 873 on Kuplu Kopo can only be the hedge knights), and one action two NPCs share splits into two clear
clusters (Galfrid's auto-attack on Quzal ~76, the bow's ~12).

## Enemy health

Enemies without a set HP use Sapphire's per-level **BnpcBaseHp** table (`BNPC_BASE_HP` in `server/actor.rs`, 44 at
level 1 to 2589 at level 50); it matched retail at every level recorded except 43 (retail 1891). A drop-in's `hp`
overrides it (Halitostroper 7983, Wood Wailer Sentries 6100, quest enemies).

## Enemy behaviour (`server/npc_behavior.rs`)

| Mechanic | How it works |
|---|---|
| **Enmity** | Each NPC keeps an enmity table. Damage adds to it; the NPC attacks whoever it hates most. Directors add enmity with `set_hate`. Before this, NPCs switched to whoever hit them last. |
| **Auto-attacks** | Every 3 s (recorded 3.0–3.1 s), timed per NPC from when it picked up its target. With a cast time they show a cast bar. |
| **Stance** | NPCs draw their weapon when they pick up a target and again 0.5 s after each action lands; they sheathe 2 s after losing their target. |
| **Pathing** | A new path starts from where the NPC actually is, and changing target drops the old path (no more 14–16 yalm jumps). |
| **Aggro** | Aggressive enemies notice players within 15 yalms (`AGGRO_RANGE`), while idle *or* wandering, and ignore players 7+ levels above them (`AGGRO_LEVEL_GAP`). |
| **Leashing** | Chased more than 40 yalms from spawn (`LEASH_DISTANCE`), an enemy drops all enmity, heals to full and walks home ignoring everyone until within 3 yalms. Enemies wander around their spawn. |
| **Area attacks** | Cones, circles and lines from the Action sheet (`CastType`, `EffectRange`, `XAxisModifier`; cone angle from the omen, e.g. `gl_fan090` = 90°). Aimed where the target was and the way the enemy faced when the cast started; the enemy stands still while casting; ground-targeted omens show where the target stood. They land only on players still inside when the cast ends, each getting their own roll (damage, variance, miss/parry/crit, statuses). The ActionEffect packet still shows only the main target (only the one-target opcode is known). |
| **Striking dummies** | Keep attackers on their enmity list but don't move, turn or fight back; resetting enmity clears it. |

Player side: sheathing stops auto-attacking (the client doesn't always ask).

## Defeating enemies

When a battle NPC dies, everyone on its enmity list, and their party in the same instance, gets:

* **Quest credit**: each of their active quests' `onBNpcKill(base_id, layout_id, player)` (see [Quests](quests.md)).
* **EXP**: ParamGrow's `ScaledQuestXP` for the enemy's level (= 45 + 5 × level, exactly what retail gave). **Kill
  chains** for enemies at or above your level within 60 s: +20 %, +25 %, then +30 %. **Rested EXP** adds 50 % to kill
  and gathering EXP only (a rounding bug once made it +100 %). Bonuses show in the floating EXP.
* **Drops** (`resources/scripts/battle/Drops.lua`, by BNpcName): items from Garland Tools' data; chance and amount
  from recordings where known (anole 67 % ×3, ground squirrel 17 % ×2, trickster imp 11 %), otherwise one 50 % roll
  (a guess). Central Shroud only so far.

Level-ups announce only the new levels (the message's third value is the previous level).

## Healer engine and the White Mage kit

From the first fork merge (`dc63634`):

* Heals restore HP (Benediction heals to full). Spells cost MP from the Action sheet; Thin Air, Freecure and Swiftcast
  are consumed; Swiftcast makes the next cast instant and Presence of Mind cuts cast times by 20 %, on top of upstream's
  adjusted cast times.
* Area actions hit every enemy or party member in range. DoTs, HoTs and MP-over-time tick every 3 s.
* The Lily gauge (Lilies, Blood Lily) with `spend_lily` / `spend_blood_lily`.
* Action scripts can see the caster's statuses (`get_effect`); refreshing a status no longer makes it expire early.
* All 36 Conjurer/White Mage actions and 6 role actions (`resources/scripts/actions/`), with 7.56 potencies;
  Global.lua has their status/trait IDs and `whmPotency` (Maim and Mend +10 %/+30 %) and `traitPotency`.
* Fixed: every action wiped the instance's queued tasks (an inverted `retain`).

Enemy skills added for the Central Shroud and the quest battles: Frost Breath, Ice Spikes, Blood Drain, Acorn
Armageddon (Heavy ~14 s), True Grit, Plaincracker, Hammer Beak, Snap Jaw and others (see `actions/003`–`actions/009`),
plus True Strike, Snap Punch, Fire, Thunder for the quest battle allies.

## Adding a monster or skill

1. Record it (Kawari Recorder → Monsters tab: level, HP per level, skills, damage to players at your level).
2. Timeline: `resources/timelines/<Name>_<BNpcBase>.json` with its hitbox, auto-attack, `damage_to_players`,
   timepoints.
3. Skill script: `resources/scripts/actions/<id / 100>/<Name>_<id>.lua`, potency relative to its auto-attack.
4. Drops: `Drops.lua` by BNpcName.
5. Fixed spawns: the zone's drop-in.
