use std::{io::Cursor, ptr::null_mut};

use binrw::{BinRead, BinWrite, binrw};
use glam::Vec3A;
use recastnavigation_sys::{
    DT_FAILURE, DT_PARTIAL_RESULT, DT_SUCCESS, dtAllocNavMesh, dtAllocNavMeshQuery, dtFreeNavMesh,
    dtFreeNavMeshQuery, dtNavMesh, dtNavMesh_addTile, dtNavMesh_init, dtNavMeshParams,
    dtNavMeshQuery, dtNavMeshQuery_findNearestPoly, dtNavMeshQuery_findPath,
    dtNavMeshQuery_findRandomPointAroundCircle, dtNavMeshQuery_findStraightPath,
    dtNavMeshQuery_init, dtPolyRef, dtQueryFilter, dtQueryFilter_dtQueryFilter,
};

#[binrw]
#[derive(Default, Debug, Clone)]
pub struct NavmeshParams {
    pub orig: [f32; 3],
    pub tile_width: f32,
    pub tile_height: f32,
    pub max_tiles: i32,
    pub max_polys: i32,
}

/// Represents a navmesh tile for a zone.
#[binrw]
#[derive(Default, Debug, Clone)]
pub struct NavmeshTile {
    #[br(temp)]
    #[bw(calc = data.len() as u32)]
    data_size: u32,
    #[br(count = data_size)]
    pub data: Vec<u8>,
}

/// Represents a navmesh for a zone.
///
/// We reuse the .nvm file extension used by the retail server but these have no relations to ours.
///
/// Since 2026-11-10 the file starts with `KNV64`: its tiles are built with Detour's 64-bit polygon references
/// (recastnavigation-sys' `detour_large_nav_meshes`). With 32-bit ones a navmesh holds only 2^22 tiles times polygons a
/// tile (each rounded up to a power of two), and an open field's made with the simple shapes (2026-11-02) went over it:
/// Detour wouldn't take it and the world server panicked loading Coerthas Central Highlands (Bob's log of 2026-11-10).
/// The tiles' links hold polygon references, so an older file's tiles don't fit this build: it isn't read (no magic),
/// and it's made again with its collision (collision version 6.)
#[binrw]
#[brw(magic = b"KNV64")]
#[derive(Default, Debug, Clone)]
pub struct Navmesh {
    /// The zone ID that the navmesh was generated with. Not guaranteed to be valid, it's only here for informative purposes in tools.
    pub zone_id: u16,
    nav_mesh_params: NavmeshParams,
    #[br(temp)]
    #[bw(calc = tiles.len() as u32)]
    tile_count: u32,
    #[br(count = tile_count)]
    tiles: Vec<NavmeshTile>,

    #[bw(ignore)]
    #[br(default)]
    pub navmesh: *mut dtNavMesh,
    #[bw(ignore)]
    #[br(default)]
    navmesh_query: *mut dtNavMeshQuery,
}

// To send the pointers between threads.
unsafe impl Send for Navmesh {}
unsafe impl Sync for Navmesh {}

/// How far (in yalms) past the end of the navmesh an NPC walks straight on to reach someone it can't path to.
/// Detour gives a partial path (to the closest point it can reach) when the destination's polygon isn't connected to
/// the start's, e.g. a few yalms of missing mesh between two rocks. Without this the NPC stood at the edge for good,
/// out of range, re-pathing every tick (To Guard a Guardian, 2026-10-16 bundle: the allies stuck 9 yalms from the
/// Ixali they were after). Content passes no limit (see `calculate_path`).
pub const MAX_OFF_MESH_GAP: f32 = 15.0;

/// How far (half-sizes across, up, across, in yalms) a path's ends look for the navmesh, nearest first.
const PATH_END_SEARCH: [[f32; 3]; 3] = [[2.0, 4.0, 2.0], [6.0, 8.0, 6.0], [15.0, 15.0, 15.0]];

/// How many random spots `find_wander_position` tries for one within its radius.
const WANDER_TRIES: usize = 4;

unsafe extern "C" fn frand() -> f32 {
    fastrand::f32()
}

impl Navmesh {
    /// Creates a new Navmesh.
    pub fn new(zone_id: u16, nav_mesh_params: NavmeshParams, tiles: Vec<NavmeshTile>) -> Self {
        Self {
            zone_id,
            nav_mesh_params,
            tiles,
            navmesh: null_mut(),
            navmesh_query: null_mut(),
        }
    }

    /// Reads an existing NVM file.
    pub fn from_existing(buffer: &[u8]) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        let mut navmesh = Self::read_le(&mut cursor).ok()?;
        match navmesh.initialize() {
            Ok(()) => Some(navmesh),
            Err(why) => {
                tracing::warn!(
                    "Detour won't take the navmesh of zone {}: {why}",
                    navmesh.zone_id
                );
                None
            }
        }
    }

    /// Writes to the NVM file format.
    pub fn write_to_buffer(&self) -> Option<Vec<u8>> {
        let mut buffer = Vec::new();

        {
            let mut cursor = Cursor::new(&mut buffer);
            self.write_le(&mut cursor).ok()?;
        }

        Some(buffer)
    }

    /// Initializes Detour data, or says why Detour wouldn't take it (the navmesh is then left without it: no paths.)
    fn initialize(&mut self) -> Result<(), String> {
        let navmesh_params = dtNavMeshParams {
            orig: self.nav_mesh_params.orig,
            tileWidth: self.nav_mesh_params.tile_width,
            tileHeight: self.nav_mesh_params.tile_height,
            maxTiles: self.nav_mesh_params.max_tiles,
            maxPolys: self.nav_mesh_params.max_polys,
        };

        unsafe {
            let navmesh = dtAllocNavMesh();
            if navmesh.is_null() {
                return Err("out of memory".into());
            }
            let status = dtNavMesh_init(navmesh, &navmesh_params);
            if status != DT_SUCCESS {
                dtFreeNavMesh(navmesh);
                return Err(format!(
                    "init {status:#x} ({} tiles, {} polygons a tile at most)",
                    navmesh_params.maxTiles, navmesh_params.maxPolys
                ));
            }

            for (index, tile) in self.tiles.iter_mut().enumerate() {
                // NOTE: At this point, Detour has a reference to our tile data! If it's moved or deleted, beware of confusing memory-related errors.
                let status = dtNavMesh_addTile(
                    navmesh,
                    tile.data.as_mut_ptr(),
                    tile.data.len() as i32,
                    0,
                    0,
                    null_mut(),
                );
                if status != DT_SUCCESS {
                    dtFreeNavMesh(navmesh);
                    return Err(format!("tile {index}: addTile {status:#x}"));
                }
            }

            let query = dtAllocNavMeshQuery();
            if query.is_null() {
                dtFreeNavMesh(navmesh);
                return Err("out of memory".into());
            }
            let status = dtNavMeshQuery_init(query, navmesh, 2048);
            if status != DT_SUCCESS {
                dtFreeNavMeshQuery(query);
                dtFreeNavMesh(navmesh);
                return Err(format!("query init {status:#x}"));
            }
            self.navmesh = navmesh;
            self.navmesh_query = query;
        }
        Ok(())
    }

    /// Calculates a path from `start_pos` to `end_pos`. When the navmesh doesn't reach `end_pos`, the path walks as far
    /// as it does and then straight on, if that last stretch is at most `max_gap` yalms.
    pub fn calculate_path(&self, start_pos: Vec3A, end_pos: Vec3A, max_gap: f32) -> Vec<Vec3A> {
        unsafe {
            let mut filter = dtQueryFilter {
                m_areaCost: [1.0; 64],
                m_includeFlags: 0xffff,
                m_excludeFlags: 0,
            };
            dtQueryFilter_dtQueryFilter(&mut filter);

            let (start_poly, start_poly_pos) =
                Self::get_polygon_at_location(self.navmesh_query, start_pos, &filter);
            let (end_poly, end_poly_pos) =
                Self::get_polygon_at_location(self.navmesh_query, end_pos, &filter);

            let mut path = [0; 128];
            let mut path_count = 0;
            let status = dtNavMeshQuery_findPath(
                self.navmesh_query,
                start_poly,
                end_poly,
                &start_poly_pos.x,
                &end_poly_pos.x,
                &filter,
                path.as_mut_ptr(),
                &mut path_count,
                128,
            );
            if status & DT_FAILURE != 0 {
                tracing::warn!(
                    "Error in dtNavMeshQuery_findPath: {status:#x} start poly: {start_poly} end poly: {end_poly}"
                );
            }

            let mut straight_path = [0.0; 128 * 3];
            let mut straight_path_count = 0;

            // now calculate the positions in the path
            dtNavMeshQuery_findStraightPath(
                self.navmesh_query,
                &start_poly_pos.x,
                &end_poly_pos.x,
                path.as_ptr(),
                path_count,
                straight_path.as_mut_ptr(),
                null_mut(),
                null_mut(),
                &mut straight_path_count,
                128,
                0,
            );

            let mut path = Vec::new();
            for pos in straight_path[..straight_path_count as usize * 3].chunks(3) {
                path.push(Vec3A::from_slice(pos));
            }

            // The destination isn't reachable on the navmesh: walk the part that is, then straight on if it's close.
            if status & DT_PARTIAL_RESULT != 0 {
                let edge = path.last().copied().unwrap_or(start_pos);
                if Vec3A::distance(edge, end_pos) <= max_gap {
                    path.push(end_pos);
                }
            }

            path
        }
    }

    fn get_polygon_at_location(
        query: *const dtNavMeshQuery,
        position: Vec3A,
        filter: &dtQueryFilter,
    ) -> (dtPolyRef, Vec3A) {
        // The nearest navmesh first, then further afield: someone standing by a wall, or at a cliff's edge, is off
        // the navmesh (it keeps an NPC's width from walls and drops), and with nothing within 2 yalms the path failed
        // and the walker went straight there instead, through the air or up the wall (Bob's Sastasha bundle of
        // 2026-10-26: 40 paths from or to "poly 0" in a minute.)
        for extents in PATH_END_SEARCH {
            unsafe {
                let mut nearest_ref = 0;
                let mut nearest_pt = [0.0; 3];
                assert!(
                    dtNavMeshQuery_findNearestPoly(
                        query,
                        &position.x,
                        extents.as_ptr(),
                        filter,
                        &mut nearest_ref,
                        nearest_pt.as_mut_ptr()
                    ) == DT_SUCCESS
                );
                if nearest_ref != 0 {
                    return (nearest_ref, Vec3A::from_array(nearest_pt));
                }
            }
        }
        (0, position)
    }

    /// A random spot on the navmesh near `position`, or None when `position` isn't on the navmesh.
    pub fn find_wander_position(&mut self, position: Vec3A) -> Option<Vec3A> {
        unsafe {
            let mut filter = dtQueryFilter {
                m_areaCost: [0.0; 64],
                m_includeFlags: 0xffff,
                m_excludeFlags: 0,
            };
            dtQueryFilter_dtQueryFilter(&mut filter);

            let (start_poly, _) =
                Self::get_polygon_at_location(self.navmesh_query, position, &filter);
            if start_poly == 0 {
                return None;
            }

            // Retail strays a median 6 yalms (90 % within 9) from where it's first seen, so about 5 from home.
            let max_radius = 5.0;

            // Detour's point can be anywhere on a polygon that touches the circle, and open ground has big ones (17
            // yalms away on a flat test mesh): take one inside it, or the way to the last one, cut short.
            let mut spot = None;
            for _ in 0..WANDER_TRIES {
                let mut random_ref = 0;
                let mut random_pt = [0.0; 3];
                if dtNavMeshQuery_findRandomPointAroundCircle(
                    self.navmesh_query,
                    start_poly,
                    &position.x,
                    max_radius,
                    &filter,
                    Some(frand),
                    &mut random_ref,
                    random_pt.as_mut_ptr(),
                ) & DT_SUCCESS
                    == 0
                {
                    return None;
                }
                let point = Vec3A::from_array(random_pt);
                let distance = Vec3A::new(point.x - position.x, 0.0, point.z - position.z).length();
                if distance <= max_radius {
                    return Some(point);
                }
                spot = Some(position.lerp(point, max_radius * fastrand::f32() / distance));
            }

            // Cut short, it can be off the navmesh: stand it on the nearest point that isn't.
            let (spot_poly, spot) =
                Self::get_polygon_at_location(self.navmesh_query, spot?, &filter);
            (spot_poly != 0).then_some(spot)
        }
    }

    /// The navmesh's floor right under (or over) `position`: the nearest navmesh point within 3 yalms up or down whose
    /// spot is the same (a quarter yalm across), or None off the navmesh. For walking straight where there's no path,
    /// so the walker keeps to the ground (a straight line between two points goes through a slope or a bridge's hump.)
    pub fn ground_under(&self, position: Vec3A) -> Option<Vec3A> {
        if !self.is_available() || self.navmesh_query.is_null() {
            return None;
        }
        unsafe {
            let mut filter = dtQueryFilter {
                m_areaCost: [1.0; 64],
                m_includeFlags: 0xffff,
                m_excludeFlags: 0,
            };
            dtQueryFilter_dtQueryFilter(&mut filter);
            let extents = [0.5, 3.0, 0.5];
            let mut nearest_ref = 0;
            let mut nearest_pt = [0.0; 3];
            let status = dtNavMeshQuery_findNearestPoly(
                self.navmesh_query,
                &position.x,
                extents.as_ptr(),
                &filter,
                &mut nearest_ref,
                nearest_pt.as_mut_ptr(),
            );
            if status & DT_FAILURE != 0 || nearest_ref == 0 {
                return None;
            }
            let point = Vec3A::from_array(nearest_pt);
            let across = Vec3A::new(point.x - position.x, 0.0, point.z - position.z).length();
            (across <= 0.25).then_some(point)
        }
    }

    pub fn is_available(&self) -> bool {
        // NOTE: There might be more checks desired here, but this suffices for now
        !self.navmesh.is_null()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(max_tiles: i32, max_polys: i32) -> NavmeshParams {
        NavmeshParams {
            orig: [0.0; 3],
            tile_width: 32.0,
            tile_height: 32.0,
            max_tiles,
            max_polys,
        }
    }

    /// A navmesh too big for 32-bit polygon references (8192 tiles times 1024 polygons: 23 bits, where 22 fit) is
    /// taken: an open field's went over (Bob's log of 2026-11-10, Coerthas Central Highlands.)
    #[test]
    fn a_big_navmesh_is_taken() {
        let navmesh = Navmesh::new(155, params(5000, 600), Vec::new());
        let bytes = navmesh.write_to_buffer().unwrap();
        let read = Navmesh::from_existing(&bytes).unwrap();
        assert!(read.is_available());
    }

    /// A file from before `KNV64` (32-bit polygon references in its tiles' links) isn't read: it's made again.
    #[test]
    fn an_older_file_is_not_read() {
        let bytes = Navmesh::new(155, params(4, 16), Vec::new())
            .write_to_buffer()
            .unwrap();
        assert!(Navmesh::from_existing(&bytes).is_some());
        assert!(Navmesh::from_existing(&bytes[5..]).is_none());
    }

    /// One Detour won't take is left without paths (no panic: the world server panicked here.)
    #[test]
    fn one_detour_will_not_take_is_none() {
        let mut tiles = vec![NavmeshTile { data: vec![0; 64] }];
        let bytes = Navmesh::new(155, params(1, 1), std::mem::take(&mut tiles))
            .write_to_buffer()
            .unwrap();
        assert!(Navmesh::from_existing(&bytes).is_none());
    }
}
