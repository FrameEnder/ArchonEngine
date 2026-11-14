//! Side talk (`resources/data/quest_side_talks.json`, written by `questkit sidetalk`): what a quest's NPCs say when
//! talked to outside the quest's steps, in each sequence the client offers them. A quest script with entries gets
//! `events/quest/SideTalk.lua` loaded after it, which plays the line when the script itself does nothing with the
//! talk (`events/lua.rs`). See `docs/src/fork/quests.md` (Side talk).
//!
//! A recording has the steps a player took, so a transcribed script answered those and ended every other talk
//! without a word; retail's NPCs have a line in between (Bob, 2026-11-14, of Dressed to Deceive's imperials). The
//! scenes are worked out from the client's quest script and the Quest sheet's listeners, numbers only.

use std::{collections::HashMap, sync::OnceLock};

fn table() -> &'static HashMap<String, serde_json::Value> {
    static TABLE: OnceLock<HashMap<String, serde_json::Value>> = OnceLock::new();
    TABLE.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../../../resources/data/quest_side_talks.json"
        ))
        .expect("resources/data/quest_side_talks.json doesn't read")
    })
}

/// A quest's side talks (its number, not its event ID), as `SideTalk.lua` reads them.
pub fn entries(quest: u32) -> Option<&'static serde_json::Value> {
    table().get(&quest.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SideTalk.lua as the server loads it (`events/lua.rs`: `to_value` of the entries, then the file after the
    /// quest's script), with a script that only ends the talk, as a transcribed one does for an NPC who isn't a
    /// step: Biggs (1006560) in Every Little Thing She Does Is Magitek's sequence 4 says his line (scene 11, the
    /// client's script) and the talk ends when it has played. A talk the script answers is the script's, and so is
    /// an NPC or a sequence with no line.
    #[test]
    fn a_line_is_played_when_the_script_is_silent() {
        use mlua::{Lua, LuaSerdeExt};
        let lua = Lua::new();
        let entries = entries(1005).expect("1005 has side talks");
        let biggs = entries
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["actor"] == 1006560)
            .unwrap();
        assert_eq!(biggs["scenes"]["4"], 11);
        lua.globals()
            .set("QUEST_SIDE_TALKS", lua.to_value(entries).unwrap())
            .unwrap();
        lua.globals().set("EVENT_ID", 0x10000 + 1005).unwrap();
        lua.globals().set("BASE_ID", 1006560).unwrap();
        lua.globals().set("HIDE_HOTBAR", 0x2000).unwrap();
        lua.load(
            r#"
            LOG = {}
            QUEUED = {}
            ANSWERS = false
            function onTalk(target, player)
                if ANSWERS then
                    player:play_scene(9, 0, {})
                else
                    player:finish_event()
                end
            end
            function onReturn(scene, results, player)
                table.insert(LOG, "script's return " .. scene)
            end
            player = {
                get_quest_sequence = function(_, id) return SEQUENCE end,
                play_scene = function(_, scene, flags, params)
                    table.insert(QUEUED, "scene")
                    table.insert(LOG, "scene " .. scene .. " " .. flags)
                end,
                finish_event = function(_)
                    table.insert(QUEUED, "finish")
                    table.insert(LOG, "finish")
                end,
                -- As LuaPlayer::event_unanswered.
                event_unanswered = function(_)
                    for _, task in ipairs(QUEUED) do
                        if task ~= "finish" then return false end
                    end
                    QUEUED = {}
                    table.insert(LOG, "taken back")
                    return true
                end,
            }
            "#,
        )
        .exec()
        .unwrap();
        let side_talk = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../resources/scripts/events/quest/SideTalk.lua"
        ))
        .unwrap();
        lua.load(side_talk).exec().unwrap();
        lua.load(
            r#"
            SEQUENCE = 4
            onTalk(nil, player)
            onReturn(11, { 0 }, player)
            QUEUED = {}
            ANSWERS = true
            onTalk(nil, player)
            onReturn(9, { 0 }, player)
            QUEUED = {}
            ANSWERS = false
            SEQUENCE = 10
            onTalk(nil, player)
            "#,
        )
        .exec()
        .unwrap();
        let log: Vec<String> = lua.globals().get("LOG").unwrap();
        assert_eq!(
            log,
            [
                "finish",
                "taken back",
                "scene 11 8192",
                "finish",
                "scene 9 0",
                "script's return 9",
                "finish"
            ]
        );
        assert!(super::entries(1).is_none());
    }

    /// A line read by hand with one for after (`tools/questkit/sidetalk_hand.json`): the flame private second class
    /// (1006648) of 3872's sequence 2 says scene 9 until saluted and 11 after, which BitFlag8's first flag (0x80 of
    /// UI8F, as the script's onEmote sets it) tells; a soldier's corpse of 1358's sequence 2 has its pop-up message
    /// played with the flags its scenes in sequence 3 had on retail (0x2001).
    #[test]
    fn a_line_for_after_and_other_flags() {
        use mlua::{Lua, LuaSerdeExt};
        for (quest, actor, sequence, var, expected) in [
            (3872, 1006648, 2, 0x00, "scene 9 8192"),
            (3872, 1006648, 2, 0x80, "scene 11 8192"),
            (3872, 1006648, 2, 0x20, "scene 9 8192"),
            (1358, 1008684, 2, 0x00, "scene 10 8193"),
        ] {
            let lua = Lua::new();
            lua.globals()
                .set(
                    "QUEST_SIDE_TALKS",
                    lua.to_value(entries(quest).unwrap()).unwrap(),
                )
                .unwrap();
            lua.globals().set("EVENT_ID", 0x10000 + quest).unwrap();
            lua.globals().set("BASE_ID", actor).unwrap();
            lua.globals().set("HIDE_HOTBAR", 0x2000).unwrap();
            lua.globals().set("SEQUENCE", sequence).unwrap();
            lua.globals().set("VAR5", var).unwrap();
            lua.load(
                r#"
                LOG = {}
                player = {
                    get_quest_sequence = function(_, id) return SEQUENCE end,
                    get_quest_var = function(_, id, index) return index == 5 and VAR5 or 0 end,
                    play_scene = function(_, scene, flags, params)
                        table.insert(LOG, "scene " .. scene .. " " .. flags)
                    end,
                    finish_event = function(_) end,
                    event_unanswered = function(_) return true end,
                }
                "#,
            )
            .exec()
            .unwrap();
            let side_talk = std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../resources/scripts/events/quest/SideTalk.lua"
            ))
            .unwrap();
            lua.load(side_talk).exec().unwrap();
            lua.load("onTalk(nil, player)").exec().unwrap();
            let log: Vec<String> = lua.globals().get("LOG").unwrap();
            assert_eq!(log, [expected], "quest {quest}, {actor}");
        }
    }

    /// What SideTalk.lua asks of the player: a talk the script only ended is unanswered and its end taken back; one
    /// it did anything else with isn't, and keeps what it queued.
    #[test]
    fn only_an_ended_event_is_unanswered() {
        use crate::lua::{LuaPlayer, LuaTask};
        let mut player = LuaPlayer::default();
        assert!(player.event_unanswered());
        player.finish_event();
        assert!(player.event_unanswered());
        assert!(player.queued_tasks.is_empty());
        player.queued_tasks.push(LuaTask::Warp { warp_id: 1 });
        player.finish_event();
        assert!(!player.event_unanswered());
        assert_eq!(player.queued_tasks.len(), 2);
    }
}
