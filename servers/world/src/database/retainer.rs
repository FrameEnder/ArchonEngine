//! The retainer table (see `crate::retainer` and docs/src/fork/retainers.md.)

use diesel::prelude::*;

use super::{WorldDatabase, schema, unixepoch};
use crate::retainer::{
    RETAINER_LIST_SLOTS, Retainer, RetainerInventory, customize_from_hex, customize_to_hex,
};

type RetainerRow = (
    i64,
    i64,
    i32,
    String,
    i32,
    i32,
    i32,
    String,
    i32,
    String,
    i64,
);

fn from_row(row: RetainerRow) -> Option<Retainer> {
    let (
        id,
        content_id,
        display_order,
        name,
        classjob,
        level,
        town,
        customize,
        voice,
        inventory,
        hired_at,
    ) = row;
    let Some(customize) = customize_from_hex(&customize) else {
        tracing::warn!("Retainer {id} has an unreadable appearance, leaving them out!");
        return None;
    };
    Some(Retainer {
        id: id as u64,
        content_id: content_id as u64,
        display_order: display_order as u8,
        name,
        classjob: classjob as u8,
        level: level as u8,
        town: town as u8,
        customize,
        voice: voice as u8,
        // A retainer whose items can't be read keeps their slot but starts empty, rather than taking the bell down.
        inventory: serde_json::from_str(&inventory).unwrap_or_else(|_| {
            tracing::warn!("Retainer {id} has an unreadable inventory, treating it as empty!");
            RetainerInventory::default()
        }),
        hired_at,
    })
}

impl WorldDatabase {
    /// A character's retainers, in the bell's order.
    pub fn retainers(&mut self, owner: u64) -> Vec<Retainer> {
        use schema::retainer::dsl::*;

        retainer
            .filter(content_id.eq(owner as i64))
            .order(display_order.asc())
            .load::<RetainerRow>(&mut self.connection)
            .unwrap_or_default()
            .into_iter()
            .filter_map(from_row)
            .collect()
    }

    /// One retainer, if `owner` has them.
    pub fn retainer(&mut self, owner: u64, wanted: u64) -> Option<Retainer> {
        use schema::retainer::dsl::*;

        retainer
            .filter(id.eq(wanted as i64))
            .filter(content_id.eq(owner as i64))
            .first::<RetainerRow>(&mut self.connection)
            .ok()
            .and_then(from_row)
    }

    /// Hires a new retainer for `owner` in their first free slot. Returns them, or None if all ten slots are taken.
    pub fn hire_retainer(
        &mut self,
        owner: u64,
        new_name: &str,
        new_classjob: u8,
        new_level: u8,
        new_town: u8,
        new_customize: &physis::savedata::chardat::CustomizeData,
        new_voice: u8,
    ) -> Option<Retainer> {
        use schema::retainer::dsl::*;

        let taken: Vec<i32> = retainer
            .filter(content_id.eq(owner as i64))
            .select(display_order)
            .load(&mut self.connection)
            .unwrap_or_default();
        let slot = (0..RETAINER_LIST_SLOTS as i32).find(|slot| !taken.contains(slot))?;

        let new_id = Retainer::new_id();
        let new_inventory = serde_json::to_string(&RetainerInventory::default()).unwrap();
        diesel::insert_into(retainer)
            .values((
                id.eq(new_id as i64),
                content_id.eq(owner as i64),
                display_order.eq(slot),
                name.eq(new_name),
                classjob.eq(new_classjob as i32),
                level.eq(new_level as i32),
                town.eq(new_town as i32),
                customize.eq(customize_to_hex(new_customize)),
                voice.eq(new_voice as i32),
                inventory.eq(new_inventory),
                hired_at.eq(unixepoch()),
            ))
            .execute(&mut self.connection)
            .ok()?;
        self.retainer(owner, new_id)
    }

    /// Saves what a retainer holds.
    pub fn save_retainer_inventory(&mut self, wanted: u64, contents: &RetainerInventory) {
        use schema::retainer::dsl::*;

        if let Err(err) = diesel::update(retainer.filter(id.eq(wanted as i64)))
            .set(inventory.eq(serde_json::to_string(contents).unwrap()))
            .execute(&mut self.connection)
        {
            tracing::warn!("Couldn't save retainer {wanted}'s inventory: {err}");
        }
    }

    /// Lets a retainer go, with everything they hold.
    pub fn release_retainer(&mut self, owner: u64, wanted: u64) -> bool {
        use schema::retainer::dsl::*;

        diesel::delete(
            retainer
                .filter(id.eq(wanted as i64))
                .filter(content_id.eq(owner as i64)),
        )
        .execute(&mut self.connection)
        .is_ok_and(|count| count > 0)
    }

    /// Removes a deleted character's retainers.
    pub fn delete_retainers(&mut self, owner: u64) {
        use schema::retainer::dsl::*;

        let _ = diesel::delete(retainer.filter(content_id.eq(owner as i64)))
            .execute(&mut self.connection);
    }
}

#[cfg(test)]
mod tests {
    use diesel_migrations::MigrationHarness;

    use super::*;
    use crate::database::MIGRATIONS;

    fn database() -> WorldDatabase {
        let mut connection = SqliteConnection::establish(":memory:").unwrap();
        connection.run_pending_migrations(MIGRATIONS).unwrap();
        diesel::sql_query("PRAGMA foreign_keys = OFF")
            .execute(&mut connection)
            .unwrap();
        WorldDatabase { connection }
    }

    fn customize() -> physis::savedata::chardat::CustomizeData {
        // A retail retainer's appearance (Bob's 2026-09-15 capture.)
        customize_from_hex("0401016407046700a95d07001107005d85030282070003641a2a").unwrap()
    }

    #[test]
    fn hire_list_and_release() {
        let mut database = database();
        let first = database
            .hire_retainer(7, "Abc", 0, 1, 3, &customize(), 1)
            .unwrap();
        let second = database
            .hire_retainer(7, "Defg", 16, 10, 1, &customize(), 2)
            .unwrap();
        assert_eq!(first.display_order, 0);
        assert_eq!(second.display_order, 1);
        assert_eq!(
            customize_to_hex(&second.customize),
            "0401016407046700a95d07001107005d85030282070003641a2a"
        );

        let list = database.retainers(7);
        assert_eq!(
            list.iter().map(|r| r.name.as_str()).collect::<Vec<_>>(),
            ["Abc", "Defg"]
        );
        assert!(database.retainers(8).is_empty());
        assert!(database.retainer(8, first.id).is_none());

        let mut inventory = first.inventory.clone();
        inventory.pages[0].slots[3].item_id = 5;
        inventory.pages[0].slots[3].quantity = 2;
        database.save_retainer_inventory(first.id, &inventory);
        assert_eq!(
            database
                .retainer(7, first.id)
                .unwrap()
                .inventory
                .item_count(),
            1
        );

        // A freed slot is taken by the next one hired.
        assert!(database.release_retainer(7, first.id));
        let third = database
            .hire_retainer(7, "Hij", 0, 1, 2, &customize(), 1)
            .unwrap();
        assert_eq!(third.display_order, 0);

        for i in 0..8 {
            assert!(
                database
                    .hire_retainer(7, &format!("Name{i}"), 0, 1, 1, &customize(), 1)
                    .is_some()
            );
        }
        assert!(
            database
                .hire_retainer(7, "Toomany", 0, 1, 1, &customize(), 1)
                .is_none()
        );
    }
}
