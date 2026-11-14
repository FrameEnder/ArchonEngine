//! Quest travel (`resources/data/quest_travel.json`, written by `questkit travel`): the NPCs a quest offers to take the
//! player somewhere, in every sequence the client offers them, sequence 0 (before it's accepted) included. A quest
//! script with entries gets `events/quest/Travel.lua` loaded after it, which answers those talks (`events/lua.rs`).
//! See `docs/src/fork/quests.md` (Quest travel).
//!
//! A recording catches an NPC's ride in one sequence; the client's script has the same question in the others, and a
//! transcribed script answered only the recorded one: Slafborn wouldn't take the player back into Cid's workshop
//! before Every Little Thing She Does Is Magitek (1005) was accepted (Bob's bundle 20261114-024122), as the airship
//! ticketers wouldn't before Lady of the Vortex (Bob's bundle 20261111-101422).

use std::{collections::HashMap, sync::OnceLock};

fn table() -> &'static HashMap<String, serde_json::Value> {
    static TABLE: OnceLock<HashMap<String, serde_json::Value>> = OnceLock::new();
    TABLE.get_or_init(|| {
        serde_json::from_str(include_str!("../../../resources/data/quest_travel.json"))
            .expect("resources/data/quest_travel.json doesn't read")
    })
}

/// A quest's travel entries (its number, not its event ID), as `Travel.lua` reads them.
pub fn entries(quest: u32) -> Option<&'static serde_json::Value> {
    table().get(&quest.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Slafborn (1006530) takes the player into Cid's workshop before 1005 is accepted (scene 2), in sequence 1 (4)
    /// and 255 (51), where retail's ride in sequence 3 (scene 8) did.
    #[test]
    fn slafborn_in_every_sequence() {
        let ridden = entries(1005).expect("1005 has travel");
        let slafborn = ridden
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["actor"] == 1006530)
            .unwrap();
        assert_eq!(slafborn["scenes"]["0"], 2);
        assert_eq!(slafborn["scenes"]["1"], 4);
        assert_eq!(slafborn["scenes"]["255"], 51);
        assert_eq!(slafborn["move"][0], "change_territory");
        assert_eq!(slafborn["move"][1], 156);
        assert!(entries(1).is_none());
    }

    /// Travel.lua as the server loads it (`events/lua.rs`: `to_value` of the entries, then the file after the quest's
    /// script): Slafborn before 1005 is accepted plays scene 2, and yes moves the player as retail's ride did; any
    /// other talk goes to the script.
    #[test]
    fn travel_lua_answers_the_ride() {
        use mlua::{Lua, LuaSerdeExt};
        let lua = Lua::new();
        lua.globals()
            .set(
                "QUEST_TRAVEL",
                lua.to_value(entries(1005).unwrap()).unwrap(),
            )
            .unwrap();
        lua.globals().set("EVENT_ID", 0x10000 + 1005).unwrap();
        lua.globals().set("BASE_ID", 1006530).unwrap();
        lua.load(
            r#"
            LOG = {}
            function onTalk(target, player) table.insert(LOG, "script") end
            player = {
                get_quest_sequence = function(_, id) return SEQUENCE end,
                play_scene = function(_, scene, flags, params) table.insert(LOG, "scene " .. scene .. " " .. flags) end,
                finish_event = function(_) table.insert(LOG, "finish") end,
                change_territory = function(_, territory, position, rotation)
                    table.insert(LOG, string.format("to %d %.2f %.2f %.2f %.3f", territory, position.x, position.y,
                        position.z, rotation))
                end,
            }
            "#,
        )
        .exec()
        .unwrap();
        let travel = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../resources/scripts/events/quest/Travel.lua"
        ))
        .unwrap();
        lua.load(travel).exec().unwrap();
        lua.load(
            r#"
            SEQUENCE = nil
            onTalk(nil, player)
            onYield(2, 0, { 1 }, player)
            onReturn(2, { 1 }, player)
            SEQUENCE = 3
            onTalk(nil, player)
            "#,
        )
        .exec()
        .unwrap();
        let log: Vec<String> = lua.globals().get("LOG").unwrap();
        assert_eq!(
            log,
            [
                "scene 2 8192",
                "finish",
                "to 156 1.36 -158.58 11.22 3.118",
                "script"
            ]
        );
    }
}
