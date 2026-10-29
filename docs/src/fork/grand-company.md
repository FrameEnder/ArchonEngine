# Grand Companies

Enlisting is a quest (683-685, [Quests](quests.md)); after it the player's company has three people in its headquarters
Kawari plays out: the personnel officer, the quartermaster and, for the company's chocobo, its stable's attendant
(My Little Chocobo). The company and its rank are kept per character (`grand_company`: the company, a rank for each)
and sent to the client in GrandCompanyInfo; each company's seals have a currency slot of their own (slot 1 the Storm
Seals, 2 the Serpent Seals, 3 the Flame Seals).

**Sources**: Bob's Chronofoil captures of 2026-10-29: 9c4e397d (Limsa Lominsa, a Maelstrom private: the officer's menu
twice, three purchases, My Little Chocobo accepted), bab8c255 (Ul'dah, the Immortal Flames: the officer, two purchases,
the whole quest) and 0845b9a2 (Gridania, the Twin Adders: the officer, three purchases including the Serpent Chocobo
Issuance, the whole quest, the whistle used); the client's scripts ComDefGrandCompanyOfficer_00073 (Bob's Script Check
bundle of 2026-10-27) and SubFst120_00700, SubSea910_00701, SubWil160_00702 (his quest dump of 2026-10-29); the
CustomTalk, GCShop, GCScripShopItem, GCScripShopCategory and GrandCompanyRank sheets.

## The personnel officer

CustomTalk 720969, `events/custom/000/ComDefGrandCompanyOfficer_00073.lua`. Every scene has flags 0x2000.

* **Scene 0** greets: 1 for a member of the officer's company (the menu follows), 2 for a member of another company
  (changing companies, scene 7, isn't done: the event ends), 0 for someone in none (it ends).
* **Scene 1** is the menu: 1 supply and provisioning missions, 2 the rank, 3 the officer's explanation, 4 leave. After
  each the menu played again on retail.
* **Supply and provisioning missions** are CustomTalk 720971 (`ComDefGCSupplyDuty_00075.lua`), started nested
  (EventStart type 7, its scene 0, flags 0x2000). The client builds the list from its own sheets: nothing came from
  the server while it was open, and closing it returned -1. The officer's menu comes back through Kawari's
  return-from-nesting (`CONTINUES_AFTER_NESTED`, its scene 255).
* **Scene 2** tells the rank. The client checks a promotion itself (GetGcRankUpCondition: the rank-up quest, the
  Hunting Log, the seals) and, when it can be had, asks for it (GrandCompanyRankUp) and returns 1; otherwise 0, as in
  all three captures. On 1 the server promotes (`player:promote_grand_company()`, `ZoneConnection::promote_grand_company`):
  the rank's RequiredSeals paid from the company's seals (GrandCompanyRank: 2,000 from Private Third Class, then
  3,000, 4,000…), the rank raised, GrandCompanyInfo sent. Then **scene 4**, the ceremony, which returns the reward the
  new rank brings (the client's IsRankReward): 1 expert delivery (rank 6, scene 5), 2 the level 30 company leves
  (rank 5, scene 6), 3 the level 45 leves (rank 8, scene 9), 4 changing companies (rank 9, scene 10), 0 none; the
  menu plays after.
* **Scene 3** is the officer's explanation (it returns nothing).

## The quartermaster

GCShop events 1441793 (the Maelstrom's), 1441794 (the Twin Adders') and 1441795 (the Immortal Flames'),
`events/gc_shop.rs`:

1. Scene 0 (flags 0x2001, no parameters), which the client ends with no results; then scene 10, the list (0x2001).
2. A purchase is scene 10 returning 7 results: 0, the item's place in its tab, the item, its price in seals for the
   count, the company, the count, the tab. The tab is GCScripShopItem's row, a GCScripShopCategory: the company,
   tier and kind (1 the Maelstrom's Materiel, 10 the Twin Adders', 19 the Flames'). The server checks the item, the
   company, the rank (RequiredGrandCompanyRank) and the seals, then as retail: EventLogMessage 1783 from the shop
   (item, count, seals), the seals' slot, InventoryActionAck (0xFFFFFFFF, 6), the item's slot, and scene 10 again with
   one parameter, 0.
3. Closing returns 1 first: the event ends.

Retail also sent three InventoryTransactions after each purchase (the seals and the item, as moves); Kawari doesn't.

Retail's purchases, all from the Materiel tab: items 5594 ×3 for 36 seals, 5595 for 34 and 5596 for 70 (Limsa);
6019 for 200 and 5595 for 34 (Ul'dah); 6018 for 200, 5596 for 70 and 5594 for 12 (Gridania).

## My Little Chocobo

Quests 700 (the Twin Adders'), 701 (the Maelstrom's) and 702 (the Immortal Flames'), `events/quest/007/`, one script
with each company's people, Chocobo Issuance (6018, 6017, 6019: 200 seals at the quartermaster), chocobo (ENpcs
1006001-1006003) and cutscene (NCUT0 237, 239, 238):

* ACTOR0 (Vorsaile Heuloix, R'ashaht Rhiki, Swift) offers it (scene 0, a yield): sequence 1.
* ACTOR1 (Cingur, Fraegeim, Mimigun) takes the Issuance (scene 1, an NpcTrade yield), the cutscene (scene 2, 0xF8402FFF), and as it
  ends the Issuance is taken, notice 0, sequence 2.
* The chocobo asks for its name (scene 4, the client's NameMyChocobo). The client answers with a text yield (yield 231)
  in a packet sized to the name: **EventYieldText16** for up to 9 bytes (opcode 945), **EventYieldText24** for 10 to
  17 (137): the handler, scene, yield ID and the text from byte 7. The server takes it (`player:name_companion(scene,
  yield)`, `ZoneConnection::name_companion`): the companion's name kept, **CompanionName** (157: 0, then the name),
  then the yield resumed with 1. As scene 4 ends, notice 1, sequence 255.
* ACTOR1 rewards it (scene 6, a yield): the Chocobo Whistle (6001). Using the whistle (ItemAction 324, type
  1322) teaches mount 1, the company chocobo: ActorControlSelf 903 [0, 1, 1] on retail, Kawari's existing item lesson.

The Maelstrom's quest was only accepted on retail (160 Storm Seals, too few for its Issuance), so after that it's the
other two's runs with its own constants.
