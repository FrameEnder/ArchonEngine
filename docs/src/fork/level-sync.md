# Level Sync

How a character is brought down to a FATE's or a duty's level, as retail does it. From Bob's Chronofoil captures: 5
FATE syncs (a level 87 White Mage synced to 18 and 50) and about 60 duty entries (White Mage 80–88, a new Conjurer
6–15).

* Server: `zone_connection/stats.rs` (`set_fate_level_sync`, `base_parameters`, `update_class_info`),
  `GameData::equipped_item_stats` / `find_content_synced_item_level`, `server/mod.rs` (the duty's sync,
  `SetNewStatValues`), `main.rs` (FateLevelSync, SyncMaxLevel, FATEComplete).

## FATEs

* The client sends client trigger **813** (`FateLevelSync`): `fate, 1, 0, 0, 1`, the second number 1 to sync (0 to
  unsync). Before it, 812 (`EnteredRunningFate`) up to three times.
* The server syncs only if the character is **above** the FATE's `ClassJobLevelMax`, then sends (retail, ~0.05 s later):
  ActorGauge (zeroed), PlayerStats (synced), StatusEffectList (HP **cut to the new maximum**), UpdateClassInfo (current
  level = the synced level, class level = the real one, synced level), SetItemLevel (unchanged), actor controls 257 and
  260, then **FateLevelSynced** (2362) with the FATE.
* The item level is ParamGrow[synced level].ItemLevelSync (18 → 18, 50 → 130).
* SetItemLevel (ActorControlSelf 521) stays the uncapped average under sync: 509 synced to 18, 31, 36, 50 and 80 in
  27 syncs (Bob's Chronofoil captures), as Kawari sends it (`EquippedStorage::calculate_item_level`, which matched
  the first SetItemLevel of all 41 sessions). Other players' Examine window shows the capped average (see
  [Party bots](party-bots.md)).
* **Unsyncing** happened when the FATE ended, every time in the captures: the same packets unsynced, FateLevelSynced
  with **0**, and current HP **kept** (not refilled). Kawari unsyncs when the FATE the player synced to ends
  (`FromServer::FATEComplete(fate)`), or when they ask, or change zones.
* No status is gained for the sync (the 1050 in the list is the EXP bonus). FATE EXP while synced showed +150 %
  (29,250 for a level-50 FATE at 87).

## Duties

* **Quest battles are duties too** (2026-10-27): each has a ContentFinderCondition, the row whose Content is the
  QuestBattle and whose TerritoryType is where it's fought (`GameData::quest_battle_content_finder`; retail's ZoneInit
  named 349 for Trial by Wind's 54 and 300 for Trial by Water's 16, Bob's Chronofoil capture b23b91ef). The battle's
  instance takes it, so ZoneInit carries it and the player is synced to its ClassJobLevelSync like any duty's. Before,
  quest battles had none and synced nobody: a level 21 conjurer fought Trial by Wind at 21 (Bob's bundle of
  2026-10-27), where retail had him at 14.
* In the zone-in packets: PlayerStats (synced), then UpdateClassInfo (synced). Leaving: the unsynced UpdateClassInfo
  before the next zone.
* Level: ContentFinderCondition.ClassJobLevelSync. **Item level: ContentFinderCondition.ItemLevelSync** when it isn't 0
  (420, 440, 470, 505 and 60 seen: HP 28138 → 22178 at 420), otherwise ParamGrow[level].ItemLevelSync. Kawari used the
  ParamGrow value always.
* Unrestricted parties aren't synced (Kawari's existing rule). Retail's max HP there was ×4 (136724 = 4 × 34181); not
  explained by the captures, not done.

## Stats under sync

Per item and per attribute (`GameData::equipped_item_stats`), when the item's level is above the sync item level S:

```text
cap = round(ItemLevel[S][attribute] × BaseParam[attribute].<slot>Percent × BaseParam[attribute].MeldParam[item's BaseParamModifier] / 100000)
attribute = min(item's value (HQ bonus included), cap)
defense = round(defense × ItemLevel[S].Defense / ItemLevel[item level].Defense)   (magic defense likewise)
```

ItemLevel's columns are in BaseParam ID order. This matches retail exactly at sync 50 (Vitality 373, Mind 387, Piety
238, Critical Hit 207, Determination 131, Spell Speed 148 from the gear) and within one point at sync 18; the MeldParam
factor is what makes Vitality 90 % on a healer's gear. Kawari used to cap by the attribute's **position** on the item
(the first attribute by the Strength cap...), without the slot share or MeldParam, and didn't cap substats or defense.

The naked stats follow the synced level (`level_main`, which matched at 87 and 50). Substats start from ParamGrow
BaseSpeed. Traits follow the synced level already (`has_trait` uses the current level).

## Actions above the synced level

Retail sends nothing for them: the client locks and downgrades actions from UpdateClassInfo's **current level** (a
synced White Mage sent Stone, not Glare). Kawari sent the real level there, now the synced one.

## FATE rewards

A completed FATE (progress 100) gives everyone who helped defeat its enemies (`fate_participants`) its gold-rank
rewards (`get_fate_rewards`): **EXP** ParamGrow's ScaledQuestXP × QuestExpModifier at the FATE's level, **gil** twice
its level, as retail in Bob's Chronofoil captures (FATE 222, level 13: 1320 EXP and 26 gil; FATE 1409, level 50: 11700
and 100). Retail also gave about 5.7 Grand Company seals a level and sometimes items, in this order: ActorControl 2353
(the FATE, 7), the EXP, the item and gil and seals, then its reward window (opcode 291 on 2026.09.15). Kawari gave
nothing before.

## Not done

* Weapon damage under sync (presumably ItemLevel[S]'s Physical/MagicalDamage) isn't capped.
* StatusEffectList's header bytes 2–3 (the real level and the synced level on retail) aren't sent that way.
* Leaving a FATE's area while synced (not in the captures), FATE EXP under sync.
