# Quest Dumps

`kawari-questdump` pulls everything about a quest out of your own game install, so quests can be scripted from what the client actually does instead of guesswork:

* **quest.json** - the quest's row from the Quest sheet: every script variable (`ACTOR0`, `SEQ_1_ACTOR2`, `EOBJECT0`...) with NPC/object/enemy names where known, the journal objectives (ToDo), the listeners that tell the client which actors matter in each sequence, and the rewards.
* **&lt;script&gt;.luab** - the client's compiled quest script.
* **&lt;script&gt;.txt** - a readable listing of that script: every scene function (`OnScene00000`, ...), and the methods, globals, fields and constants each one uses (for example which quest variables it reads with `GetQuestUI8AL`.) Add `--listing` for every instruction.

**Note:** The `.luab` files and listings come from Square Enix's game files. Keep dumps to yourself, and don't commit them (`quest-dumps/` is in `.gitignore`.)

## Usage

It finds the game the same way the server does (`config.yaml` in the current folder, or auto-detection), or you can point it at the game's `game` folder (the one containing `sqpack`) with `--game`.

```shell
# Specific quests, by ID, script ID or name
kawari-questdump dump 3854 XxaFst031_03855 "Surveying the Damage"

# The next 20 quests after Close to Home, in the same journal genre (the main story here)
kawari-questdump chain 124 20

# Any game file, e.g. an opening script
kawari-questdump extract game_script/opening/OpeningGridania.luab

# Read a script you already have
kawari-questdump disasm quest-dumps/XxaFst034_03854/XxaFst034_03854.luab --listing

# The whole quest graph in one file: names, levels, prerequisites, issuers and areas
kawari-questdump index
```

`index` writes `quest-index.json`. Each quest's `previous` lists the quests that unlock it, and
`previous_join` says how they combine: `2` means any one of them (the story quests that follow the three
city starts), anything else means all of them.

Everything is written to `quest-dumps/` (change it with `--out`), with an `index.txt` summarizing what was dumped.

## Reading a listing

Each function gets a header and a summary:

```text
== XxaFst034_03854:OnScene00001  [function 0.1, 3 params, 0 upvalues, stack 10, 23 instructions]
   methods  GetQuestUI8AL, SetQuestUI8AL, PlayCutScene
   numbers  3, 62
```

* **methods** are calls made with `:`, which is how the client scripts talk to the game (talking, cutscenes, quest variables.)
* **fields** are named lookups like `self.ACTOR0`, which line up with the variables in `quest.json`.
* The scene number is in the function's name, and is what a Kawari quest script passes to `play_scene`.

The client scripts only cover what's shown to the player. Which scene plays for which NPC in which sequence is decided by the server, so `quest.json`'s listeners and named variables (`SEQ_1_ACTOR2 = 2` means "talking to ACTOR2 during sequence 1 plays scene 2") fill in the rest.

## From ArchonEngine ControlPanel

ArchonEngine ControlPanel's **Quest Lab** (Dashboard → Quests) runs this tool for you, using the `kawari-questdump` from
the active build against the server's own game install, and adds the parts around it:

* browse dumps, with each quest's `quest.json` laid out, its listing, and the fork's own script side by side
* check a script against the dump: which of the client's scenes it plays, and any quest variable it names
  that the Quest sheet doesn't have
* download a bundle of dumps and current scripts to hand to whoever is writing the next quests
* put a character at any point in a quest: prerequisites done, quest accepted at a chosen sequence
