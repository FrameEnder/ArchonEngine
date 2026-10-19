use std::{io::Cursor, ptr::null_mut};

use binrw::{BinRead, BinWrite, binrw};
use glam::Vec3A;
use recastnavigation_sys::{
    DT_FAILURE, DT_PARTIAL_RESULT, DT_SUCCESS, dtAllocNavMesh, dtAllocNavMeshQuery, dtNavMesh,
    dtNavMesh_addTile, dtNavMesh_init, dtNavMeshParams, dtNavMeshQuery,
    dtNavMeshQuery_findNearestPoly, dtNavMeshQuery_findPath,
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
#[binrw]
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
        if let Ok(mut navmesh) = Self::read_le(&mut cursor) {
            navmesh.initialize();
            return Some(navmesh);
        }

        None
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

    /// Initializes Detour data.
    fn initialize(&mut self) {
        let navmesh_params = dtNavMeshParams {
            orig: self.nav_mesh_params.orig,
            tileWidth: self.nav_mesh_params.tile_width,
            tileHeight: self.nav_mesh_params.tile_height,
            maxTiles: self.nav_mesh_params.max_tiles,
            maxPolys: self.nav_mesh_params.max_polys,
        };

        unsafe {
            self.navmesh = dtAllocNavMesh();
            assert!(dtNavMesh_init(self.navmesh, &navmesh_params) == DT_SUCCESS);

            for tile in &mut self.tiles {
                // NOTE: At this point, Detour has a reference to our tile data! If it's moved or deleted, beware of confusing memory-related errors.
                assert!(
                    dtNavMesh_addTile(
                        self.navmesh,
                        tile.data.as_mut_ptr(),
                        tile.data.len() as i32,
                        0,
                        0,
                        null_mut()
                    ) == DT_SUCCESS
                );
            }

            self.navmesh_query = dtAllocNavMeshQuery();
            assert!(!self.navmesh_query.is_null());
            assert!(dtNavMeshQuery_init(self.navmesh_query, self.navmesh, 2048) == DT_SUCCESS);
        }
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
        let extents = [2.0, 4.0, 2.0];

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

            (nearest_ref, Vec3A::from_array(nearest_pt))
        }
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

            Some(Vec3A::from_array(random_pt))
        }
    }

    pub fn is_available(&self) -> bool {
        // NOTE: There might be more checks desired here, but this suffices for now
        !self.navmesh.is_null()
    }
}
