//! Party bots' rotations, one Lua script per job (`resources/scripts/rotations/`): what a bot uses next, decided the way
//! a player's rotation is, from its level, its buffs and procs, its target's damage over time, its combo, its
//! cooldowns, its MP and job gauge, and how many enemies are around. The scripts' priorities come from the community's
//! open rotation logic (Boss Mod's standard rotations, awgil/ffxiv_bossmod, BSD-3), the jobs' guides, and what retail's
//! players used at each level in Bob's Chronofoil captures; each script says where its numbers came from. A job
//! without a script keeps the heuristic rotation (`party_bot::build_rotation`).
//!
//! A script returns a table: `classjobs` (the ClassJob rows it plays: a job and its base class) and `choose(b)`, called
//! each tick the bot could act, with `b` (see `Rotation.lua`) the bot's situation; it returns the action to use and,
//! optionally, `"self"` for one used on the bot itself. A table `b.mem` is kept for each bot between calls.

use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use mlua::{Function, Lua, RegistryKey, Table, Value};

use kawari::config::get_config;

/// Everything a rotation script sees about a bot (built each time it may act.)
#[derive(Debug, Clone, Default)]
pub struct BotView {
    pub bot_id: u32,
    pub classjob: u8,
    pub level: u8,
    pub mp: u32,
    pub max_mp: u32,
    /// Its own and its target's health, in %.
    pub hp: f32,
    pub target_hp: f32,
    /// Seconds until its GCD comes round again (0: ready), since its last GCD action, and the abilities used since.
    pub gcd_left: f32,
    pub since_gcd: f32,
    pub weaves: u8,
    /// Its last GCD action, if a combo could still follow it (0 otherwise.)
    pub combo: u32,
    /// The actions it knows at its level that the server has scripts for: (action, on the GCD, seconds until ready,
    /// the sheet's MP cost.)
    pub known: Vec<(u32, bool, f32, u32)>,
    /// Its statuses: (status, seconds left (999 for one that never runs out), param.)
    pub statuses: Vec<(u16, f32, u16)>,
    /// Its target's statuses: (status, seconds left, whether it put it there.)
    pub target_statuses: Vec<(u16, f32, bool)>,
    /// Living enemies within 5 yalms of its target, and of itself (each counting the target if it's there.)
    pub near_target: u32,
    pub near_self: u32,
    /// How far its target is (ground distance between the edges of their hitboxes isn't kept: middle to middle.)
    pub distance: f32,
    /// Its job gauge, by name (`JobGauge::named`.)
    pub gauge: Vec<(&'static str, i32)>,
    /// Seconds since the fight began for it (since it last had no target.)
    pub in_combat: f32,
}

struct Rotations {
    lua: Lua,
    /// The `choose` function for each ClassJob.
    by_classjob: HashMap<u8, RegistryKey>,
    /// Each bot's `mem` table.
    memory: RegistryKey,
}

static ROTATIONS: OnceLock<Mutex<Option<Rotations>>> = OnceLock::new();

fn rotations() -> &'static Mutex<Option<Rotations>> {
    ROTATIONS.get_or_init(|| {
        Mutex::new(
            load()
                .map_err(|err| tracing::error!("Bot rotations: {err}"))
                .ok(),
        )
    })
}

/// Where the rotation scripts are: the additional resource paths' first, then the built-in ones.
fn rotation_dirs() -> Vec<String> {
    let mut dirs: Vec<String> = get_config()
        .filesystem
        .additional_resource_paths
        .iter()
        .map(|path| format!("{path}/scripts/rotations"))
        .collect();
    dirs.push("resources/scripts/rotations".to_string());
    // The tests run from the crate's folder.
    #[cfg(test)]
    dirs.push(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../resources/scripts/rotations"
        )
        .to_string(),
    );
    dirs
}

fn load() -> mlua::Result<Rotations> {
    let lua = Lua::new();
    let dirs = rotation_dirs();
    // The shared helpers (`b`'s methods), from the first folder that has them.
    let library = dirs
        .iter()
        .map(|dir| format!("{dir}/lib/Rotation.lua"))
        .find(|path| std::path::Path::new(path).exists())
        .ok_or_else(|| mlua::Error::runtime("no rotations/lib/Rotation.lua"))?;
    lua.load(std::fs::read(&library)?)
        .set_name(format!("@{library}"))
        .exec()?;

    let mut by_classjob = HashMap::new();
    let mut seen = std::collections::HashSet::new();
    for dir in &dirs {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        let mut paths: Vec<_> = entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|x| x.to_str()) == Some("lua"))
            .collect();
        paths.sort();
        for path in paths {
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            // An additional path's script of the same name takes the built-in one's place.
            if !seen.insert(name.clone()) {
                continue;
            }
            let script: Table = match lua
                .load(std::fs::read(&path)?)
                .set_name(format!("@{}", path.display()))
                .eval()
            {
                Ok(script) => script,
                Err(err) => {
                    tracing::error!("Bot rotation {name}: {err}");
                    continue;
                }
            };
            let classjobs: Vec<u8> = script.get("classjobs").unwrap_or_default();
            let choose: Function = match script.get("choose") {
                Ok(choose) => choose,
                Err(_) => {
                    tracing::error!("Bot rotation {name} has no choose function");
                    continue;
                }
            };
            for classjob in classjobs {
                by_classjob.insert(classjob, lua.create_registry_value(choose.clone())?);
            }
        }
    }
    tracing::info!(
        "Bot rotations for {} classes and jobs: {:?}",
        by_classjob.len(),
        {
            let mut ids: Vec<_> = by_classjob.keys().copied().collect();
            ids.sort();
            ids
        }
    );
    let memory = lua.create_registry_value(lua.create_table()?)?;
    Ok(Rotations {
        lua,
        by_classjob,
        memory,
    })
}

/// Whether a rotation script plays this class or job.
pub fn has(classjob: u8) -> bool {
    rotations()
        .lock()
        .ok()
        .and_then(|guard| {
            guard
                .as_ref()
                .map(|r| r.by_classjob.contains_key(&classjob))
        })
        .unwrap_or(false)
}

/// Forgets a bot's `mem` (it left, or the party wiped.)
pub fn forget(bot_id: u32) {
    if let Ok(guard) = rotations().lock()
        && let Some(r) = guard.as_ref()
        && let Ok(memory) = r.lua.registry_value::<Table>(&r.memory)
    {
        let _ = memory.raw_set(bot_id, Value::Nil);
    }
}

/// What the bot's job script picks: the action, and whether it's used on the bot itself.
pub fn choose(view: &BotView) -> Option<(u32, bool)> {
    let guard = rotations().lock().ok()?;
    let r = guard.as_ref()?;
    let key = r.by_classjob.get(&view.classjob)?;
    let result = (|| -> mlua::Result<Option<(u32, bool)>> {
        let choose: Function = r.lua.registry_value(key)?;
        let b = build(&r.lua, r, view)?;
        let make: Function = r.lua.globals().get("BotView")?;
        let b: Table = make.call(b)?;
        let (action, on): (Option<u32>, Option<String>) = choose.call(b)?;
        Ok(action
            .filter(|action| *action != 0)
            .map(|action| (action, on.as_deref() == Some("self"))))
    })();
    match result {
        Ok(choice) => choice,
        Err(err) => {
            tracing::error!("Bot rotation for ClassJob {}: {err}", view.classjob);
            None
        }
    }
}

fn build(lua: &Lua, r: &Rotations, view: &BotView) -> mlua::Result<Table> {
    let b = lua.create_table()?;
    b.set("id", view.bot_id)?;
    b.set("classjob", view.classjob)?;
    b.set("level", view.level)?;
    b.set("mp", view.mp)?;
    b.set("max_mp", view.max_mp)?;
    b.set("hp", view.hp)?;
    b.set("target_hp", view.target_hp)?;
    b.set("gcd_left", view.gcd_left)?;
    b.set("since_gcd", view.since_gcd)?;
    b.set("weaves", view.weaves)?;
    b.set("combo", view.combo)?;
    b.set("near_target", view.near_target)?;
    b.set("near_self", view.near_self)?;
    b.set("distance", view.distance)?;
    b.set("in_combat", view.in_combat)?;

    let known = lua.create_table()?;
    let gcd = lua.create_table()?;
    let costs = lua.create_table()?;
    for (action, on_gcd, left, cost) in &view.known {
        known.set(*action, *left)?;
        gcd.set(*action, *on_gcd)?;
        costs.set(*action, *cost)?;
    }
    b.set("known", known)?;
    b.set("gcd_actions", gcd)?;
    b.set("costs", costs)?;

    let statuses = lua.create_table()?;
    let params = lua.create_table()?;
    for (id, left, param) in &view.statuses {
        statuses.set(*id, *left)?;
        params.set(*id, *param)?;
    }
    b.set("statuses", statuses)?;
    b.set("params", params)?;

    let mine = lua.create_table()?;
    let any = lua.create_table()?;
    for (id, left, own) in &view.target_statuses {
        let best: f32 = any.get::<Option<f32>>(*id)?.unwrap_or(0.0).max(*left);
        any.set(*id, best)?;
        if *own {
            mine.set(*id, *left)?;
        }
    }
    b.set("target_mine", mine)?;
    b.set("target_any", any)?;

    let gauge = lua.create_table()?;
    for (name, value) in &view.gauge {
        gauge.set(*name, *value)?;
    }
    b.set("gauge", gauge)?;

    let memory: Table = lua.registry_value(&r.memory)?;
    let mem: Table = match memory.raw_get::<Option<Table>>(view.bot_id)? {
        Some(mem) => mem,
        None => {
            let mem = lua.create_table()?;
            memory.raw_set(view.bot_id, mem.clone())?;
            mem
        }
    };
    b.set("mem", mem)?;
    Ok(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every rotation script loads, and each picks something sensible for a fresh bot of its jobs at a few levels.
    #[test]
    fn every_rotation_script_runs() {
        let r = load().expect("rotations load");
        assert!(!r.by_classjob.is_empty());
        for classjob in r.by_classjob.keys() {
            for level in [1u8, 15, 30, 50] {
                let view = BotView {
                    bot_id: 1,
                    classjob: *classjob,
                    level,
                    mp: 10000,
                    max_mp: 10000,
                    hp: 100.0,
                    target_hp: 100.0,
                    near_target: 1,
                    near_self: 1,
                    distance: 3.0,
                    // Everything known and ready: the script still has to pick by level itself.
                    known: Vec::new(),
                    ..Default::default()
                };
                let b = build(&r.lua, &r, &view).unwrap();
                let make: Function = r.lua.globals().get("BotView").unwrap();
                let b: Table = make.call(b).unwrap();
                let choose: Function = r.lua.registry_value(&r.by_classjob[classjob]).unwrap();
                let _: (Option<u32>, Option<String>) = choose
                    .call(b)
                    .unwrap_or_else(|err| panic!("ClassJob {classjob} at {level}: {err}"));
            }
        }
    }
}
