//! The `kawari_extended` object of an imported Auracite backup, kept whole (`crate::auracite`,
//! docs/src/fork/auracite.md.)

use diesel::prelude::*;

use super::{WorldDatabase, schema};

impl WorldDatabase {
    /// Keeps a character's `kawari_extended` object (JSON text), replacing one imported before.
    pub fn save_auracite_extended(&mut self, owner: u64, json: &str) {
        use schema::auracite_extended::dsl::*;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or_default();
        let result = diesel::replace_into(auracite_extended)
            .values((
                content_id.eq(owner as i64),
                data.eq(json),
                imported_at.eq(now),
            ))
            .execute(&mut self.connection);
        if let Err(error) = result {
            tracing::warn!("Couldn't keep the backup's kawari_extended for {owner}: {error}");
        }
    }

    /// A character's kept `kawari_extended` object, if one was imported.
    pub fn auracite_extended(&mut self, owner: u64) -> Option<String> {
        use schema::auracite_extended::dsl::*;

        auracite_extended
            .filter(content_id.eq(owner as i64))
            .select(data)
            .first::<String>(&mut self.connection)
            .ok()
    }

    pub fn delete_auracite_extended(&mut self, owner: u64) {
        use schema::auracite_extended::dsl::*;

        let _ = diesel::delete(auracite_extended.filter(content_id.eq(owner as i64)))
            .execute(&mut self.connection);
    }
}
