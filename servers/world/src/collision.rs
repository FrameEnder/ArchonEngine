//! A zone's collision as a player meets it: every collision mesh the navmesh is built from (the terrain's plates, the
//! bg parts' and collision boxes' meshes, and those in shared groups), kept as triangles, so that NPCs stand on the
//! floor a player stands on, stop at the walls a player stops at, and drop off a ledge as a player does.
//!
//! The navmesh alone left them in the air and under the floor: between two corners of a path, and wherever there's no
//! path at all (they walk straight there), nothing said where the ground was. Bob's Copperbell Mines bundle of 2026-10-26:
//! he dropped 30 yalms off a ledge; one bot stayed on it, going 3 yalms up and down the cliff, and two came down a
//! straight line through the air, 3 yalms every half second.
//!
//! `kawari-navimesh` writes it next to the navmesh (`<navmesh>.col`); the world server reads it with the navmesh.
//!
//! **Gimmicks.** The shared groups an event object controls (its LGB `bound_instance_id`: a duty's entrance barrier,
//! the Tam-Tara Deepcroft's barriers and boss-room walls) are there for players only while the object is: the client
//! takes them away with it when the server hides it (SetEventState). Their triangles carry the object's layout ID, and
//! each instance switches them off as its director hides the object (`Instance::set_gimmick_gone`), so bots and
//! enemies walk where players walk. The navmesh is built without them (they'd cut it for good.) Bob's bundles of
//! 2026-10-27: the Copperbell Mines' bots held up at the entrance, the Tam-Tara Deepcroft's for 42 and 81 s at the
//! first room while he walked on.
//!
//! **Materials.** Each triangle keeps its PCB polygon's material (since version 3), which says what the floor is: the
//! Thousand Maws of Toto-Rak's slime is its own surface, and standing on it gives a status (`server/status_floor.rs`).
//! Version 2 files (no materials) are still read; a zone that needs them has its navmesh made again.
//!
//! **Simple shapes.** Since version 4 the collision has the layouts' BgParts whose collision is a simple shape (a box,
//! a cylinder, a sphere or a plane: their analytic collider, 569 of Haukke Manor's 1199 BgParts with collision) and the
//! collision boxes of those shapes too; before, only meshes and boxes. Older files are still read, and each zone's
//! navmesh and collision are made again the first time it's entered (`is_older`.)
//!
//! **Open doors.** Since version 5 a door's shared group placed standing open leaves out its collision boxes (the
//! closed door's): before, every door was shut in the collision and the navmesh, though players walk through the open
//! ones (Bob's Haukke Manor bundle 20261103-051833: bots held at its open doors, and paths to him that went nowhere.)
//!
//! **Line of sight.** `sees` says whether a straight line between two points (at eye height) is clear of the
//! collision: walls, floors and ceilings, and the gimmicks the director put up; gimmicks it hasn't touched don't block
//! it (some aren't solid for players, as for walking.)

use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::{collections::HashMap, path::Path};

use glam::Vec3A;

/// The file's first bytes, and the layout's version.
const MAGIC: &[u8; 4] = b"KCOL";
const VERSION: u32 = 5;
/// The version with every door shut, still read (the same layout.)
const VERSION_WITH_DOORS_SHUT: u32 = 4;
/// The version before the simple shapes, still read (the same layout.)
const VERSION_WITHOUT_SHAPES: u32 = 3;
/// The version before materials, still read (without them.)
const VERSION_WITHOUT_MATERIALS: u32 = 2;

/// The grid the triangles are filed in (yalms a side.)
const CELL: f32 = 2.0;
/// The longest stretch walked at once before looking at the ground again (yalms.)
const SUBSTEP: f32 = 0.25;
/// The heights a walker's body meets walls at, over its climb (yalms): a step it can take is no wall.
const WALL_HEIGHTS: [f32; 2] = [0.1, 1.4];
/// How high over someone's feet their line of sight goes from and to (yalms; a guess, about a Hyur's eyes.)
pub const LINE_OF_SIGHT_HEIGHT: f32 = 1.5;
/// How far below its feet a walker looks for the floor it's over when it's put somewhere (yalms.)
const SETTLE_DEPTH: f32 = 200.0;
/// How far above its feet it looks when it's under the floor (it was walked straight through a slope, or spawned a
/// little under it.)
const SETTLE_RISE: f32 = 3.0;

/// The collision, ready to ask where the floor is and what's in the way. Cloning it is cheap (the triangles are
/// shared): each instance keeps its own copy of which gimmicks are gone.
#[derive(Debug, Default, Clone)]
pub struct Collision {
    /// How high a step a walker can take (yalms; the zone's RecastNavimesh AgentMaxClimb, which retail's navmeshes
    /// are built for.)
    pub climb: f32,
    /// How wide a walker is (yalms, its radius; RecastNavimesh AgentRadius.)
    pub radius: f32,
    /// A surface no steeper than this can be stood on (the cosine of RecastNavimesh AgentMaxSlope.)
    floor_cos: f32,
    origin: [f32; 2],
    columns: u32,
    rows: u32,
    vertices: Arc<[[f32; 3]]>,
    triangles: Arc<[[u32; 3]]>,
    /// For each cell (row by row), where its triangles start in `refs`; one more at the end.
    offsets: Arc<[u32]>,
    refs: Arc<[u32]>,
    /// Whether each triangle can be stood on (not steeper than `floor_cos`.)
    floors: Arc<[bool]>,
    /// The gimmicks: the layout ID of the event object each belongs to.
    gimmicks: Arc<[u32]>,
    /// For each triangle, its gimmick (an index into `gimmicks`, plus one), or 0 for the zone's own.
    owners: Arc<[u32]>,
    /// For each triangle, its PCB polygon's material; empty when the file has none (version 2.)
    materials: Arc<[u64]>,
    /// Which gimmicks are gone here (by index into `gimmicks`.)
    gone: Vec<bool>,
    /// Which gimmicks the director has put up here (shown: a boss room sealed), which nobody gets through.
    sealed: Vec<bool>,
    /// The file's version (`VERSION` for one built here.)
    version: u32,
}

/// Where a walk got to, and whether something stopped it before the end.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Walk {
    pub at: Vec3A,
    pub blocked: bool,
    /// When a gimmick stopped it (a door, a barrier, a boss room's wall): its event object's layout ID.
    pub gimmick: Option<u32>,
    /// Whether that gimmick is one the director put up (`set_gimmick_sealed`: a boss room sealed), which a player
    /// can't get through either. Other gimmicks standing can be ones players walk through (Bob's Copperbell Mines
    /// bundle of 2026-10-27: 9288834 by the first lift, never hidden, held the bots until they were put beside him.)
    pub sealed: bool,
}

/// Why a step can't be taken.
enum Stop {
    /// A wall, facing this way across the ground, and the gimmick it belongs to (its layout ID), if it does.
    Wall(Vec3A, Option<u32>),
    /// No floor to stand on: a step too high, a drop too deep, or nothing there at all.
    NoFloor,
}

impl Collision {
    /// Files `triangles` (indices into `vertices`) in a grid. `max_slope` is in degrees. `owners` is, for each
    /// triangle, the layout ID of the event object whose gimmick it belongs to, or 0 (missing ones are 0.)
    pub fn build(
        vertices: Vec<[f32; 3]>,
        triangles: Vec<[u32; 3]>,
        owners: Vec<u32>,
        materials: Vec<u64>,
        climb: f32,
        radius: f32,
        max_slope: f32,
    ) -> Self {
        let mut gimmicks: Vec<u32> = Vec::new();
        let kept: Vec<usize> = triangles
            .iter()
            .enumerate()
            .filter(|(_, tri)| {
                tri.iter().all(|i| (*i as usize) < vertices.len())
                    && normal(&vertices, tri).length_squared() > 1e-12
            })
            .map(|(index, _)| index)
            .collect();
        let materials: Vec<u64> = kept
            .iter()
            .map(|index| materials.get(*index).copied().unwrap_or(0))
            .collect();
        let (triangles, owners): (Vec<[u32; 3]>, Vec<u32>) = kept
            .into_iter()
            .map(|index| (index, triangles[index]))
            .map(|(index, tri)| {
                let layout = owners.get(index).copied().unwrap_or(0);
                if layout == 0 {
                    return (tri, 0);
                }
                let gimmick = match gimmicks.iter().position(|g| *g == layout) {
                    Some(at) => at,
                    None => {
                        gimmicks.push(layout);
                        gimmicks.len() - 1
                    }
                };
                (tri, gimmick as u32 + 1)
            })
            .unzip();
        let (mut min, mut max) = ([f32::MAX; 2], [f32::MIN; 2]);
        for tri in &triangles {
            for i in tri {
                let v = vertices[*i as usize];
                min = [min[0].min(v[0]), min[1].min(v[2])];
                max = [max[0].max(v[0]), max[1].max(v[2])];
            }
        }
        if triangles.is_empty() {
            (min, max) = ([0.0; 2], [0.0; 2]);
        }
        let columns = (((max[0] - min[0]) / CELL).floor() as u32 + 1).max(1);
        let rows = (((max[1] - min[1]) / CELL).floor() as u32 + 1).max(1);
        let mut collision = Self {
            climb,
            radius,
            floor_cos: max_slope.to_radians().cos(),
            origin: min,
            columns,
            rows,
            vertices: vertices.into(),
            gone: vec![false; gimmicks.len()],
            sealed: vec![false; gimmicks.len()],
            gimmicks: gimmicks.into(),
            owners: owners.into(),
            materials: materials.into(),
            version: VERSION,
            ..Default::default()
        };

        // Count, then fill (compressed rows: each cell's triangles side by side.)
        let mut counts = vec![0u32; (columns * rows) as usize];
        for tri in &triangles {
            for cell in collision.cells_over(&collision.bounds(tri)) {
                counts[cell] += 1;
            }
        }
        let mut offsets = Vec::with_capacity(counts.len() + 1);
        let mut total = 0u32;
        for count in &counts {
            offsets.push(total);
            total += count;
        }
        offsets.push(total);
        let mut next = offsets.clone();
        let mut refs = vec![0u32; total as usize];
        for (index, tri) in triangles.iter().enumerate() {
            for cell in collision.cells_over(&collision.bounds(tri)) {
                refs[next[cell] as usize] = index as u32;
                next[cell] += 1;
            }
        }
        collision.triangles = triangles.into();
        collision.offsets = offsets.into();
        collision.refs = refs.into();
        collision.find_floors();
        collision
    }

    fn find_floors(&mut self) {
        self.floors = self
            .triangles
            .iter()
            .map(|tri| {
                let n = normal(&self.vertices, tri).normalize_or_zero();
                n.y.abs() >= self.floor_cos
            })
            .collect::<Vec<_>>()
            .into();
    }

    /// The file's bytes (little-endian.)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&VERSION.to_le_bytes());
        let floor_slope = self.floor_cos.acos().to_degrees();
        for value in [
            self.climb,
            self.radius,
            floor_slope,
            self.origin[0],
            self.origin[1],
        ] {
            out.extend_from_slice(&value.to_le_bytes());
        }
        for value in [
            self.columns,
            self.rows,
            self.vertices.len() as u32,
            self.triangles.len() as u32,
            self.refs.len() as u32,
        ] {
            out.extend_from_slice(&value.to_le_bytes());
        }
        for v in self.vertices.iter() {
            for c in v {
                out.extend_from_slice(&c.to_le_bytes());
            }
        }
        for tri in self.triangles.iter() {
            for i in tri {
                out.extend_from_slice(&i.to_le_bytes());
            }
        }
        for value in self.offsets.iter().chain(self.refs.iter()) {
            out.extend_from_slice(&value.to_le_bytes());
        }
        out.extend_from_slice(&(self.gimmicks.len() as u32).to_le_bytes());
        for value in self.gimmicks.iter().chain(self.owners.iter()) {
            out.extend_from_slice(&value.to_le_bytes());
        }
        for index in 0..self.triangles.len() {
            let material = self.materials.get(index).copied().unwrap_or(0);
            out.extend_from_slice(&material.to_le_bytes());
        }
        out
    }

    /// Reads a file written by `to_bytes`, or None if it isn't one (or is another version.)
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut r = Reader { bytes, at: 0 };
        if r.take(4)? != MAGIC {
            return None;
        }
        let version = r.u32()?;
        if ![
            VERSION,
            VERSION_WITH_DOORS_SHUT,
            VERSION_WITHOUT_SHAPES,
            VERSION_WITHOUT_MATERIALS,
        ]
        .contains(&version)
        {
            return None;
        }
        let climb = r.f32()?;
        let radius = r.f32()?;
        let slope = r.f32()?;
        let origin = [r.f32()?, r.f32()?];
        let columns = r.u32()?;
        let rows = r.u32()?;
        let vertex_count = r.u32()? as usize;
        let triangle_count = r.u32()? as usize;
        let ref_count = r.u32()? as usize;
        let cells = (columns as usize).checked_mul(rows as usize)?;
        if columns == 0 || rows == 0 {
            return None;
        }
        let mut vertices = Vec::with_capacity(vertex_count);
        for _ in 0..vertex_count {
            vertices.push([r.f32()?, r.f32()?, r.f32()?]);
        }
        let mut triangles = Vec::with_capacity(triangle_count);
        for _ in 0..triangle_count {
            let tri = [r.u32()?, r.u32()?, r.u32()?];
            if tri.iter().any(|i| *i as usize >= vertex_count) {
                return None;
            }
            triangles.push(tri);
        }
        let mut offsets = Vec::with_capacity(cells + 1);
        for _ in 0..=cells {
            offsets.push(r.u32()?);
        }
        let mut refs = Vec::with_capacity(ref_count);
        for _ in 0..ref_count {
            let index = r.u32()?;
            if index as usize >= triangle_count {
                return None;
            }
            refs.push(index);
        }
        if offsets.windows(2).any(|pair| pair[0] > pair[1])
            || offsets.last().copied() != Some(ref_count as u32)
        {
            return None;
        }
        let gimmick_count = r.u32()? as usize;
        let mut gimmicks = Vec::with_capacity(gimmick_count);
        for _ in 0..gimmick_count {
            gimmicks.push(r.u32()?);
        }
        let mut owners = Vec::with_capacity(triangle_count);
        for _ in 0..triangle_count {
            let owner = r.u32()?;
            if owner as usize > gimmick_count {
                return None;
            }
            owners.push(owner);
        }
        let mut materials = Vec::new();
        if version >= VERSION_WITHOUT_SHAPES {
            materials.reserve(triangle_count);
            for _ in 0..triangle_count {
                materials.push(r.u64()?);
            }
        }
        let mut collision = Self {
            climb,
            radius,
            floor_cos: slope.to_radians().cos(),
            origin,
            columns,
            rows,
            vertices: vertices.into(),
            triangles: triangles.into(),
            offsets: offsets.into(),
            refs: refs.into(),
            floors: Arc::new([]),
            gone: vec![false; gimmicks.len()],
            sealed: vec![false; gimmicks.len()],
            gimmicks: gimmicks.into(),
            owners: owners.into(),
            materials: materials.into(),
            version,
        };
        collision.find_floors();
        Some(collision)
    }

    /// The collision file for a navmesh at `nvm_path`, shared by every instance of the zone. None if there's none
    /// yet (or it can't be read.)
    pub fn load_shared(nvm_path: &Path) -> Option<Arc<Self>> {
        static LOADED: OnceLock<Mutex<HashMap<String, Weak<Collision>>>> = OnceLock::new();
        let path = path_for(nvm_path);
        let key = path.to_string_lossy().to_string();
        let mut loaded = LOADED.get_or_init(Default::default).lock().ok()?;
        // One already loaded, unless it's an older one that's been made again since.
        if let Some(collision) = loaded.get(&key).and_then(Weak::upgrade)
            && (!collision.is_older() || !is_current(&path))
        {
            return Some(collision);
        }
        let bytes = std::fs::read(&path).ok()?;
        let Some(collision) = Self::from_bytes(&bytes) else {
            tracing::warn!("{path:?} isn't a collision file this build reads");
            return None;
        };
        tracing::info!(
            "Loaded the collision from {path:?} ({} triangles, {} of them the gimmicks of {} event objects)",
            collision.triangles.len(),
            collision.owners.iter().filter(|owner| **owner != 0).count(),
            collision.gimmicks.len()
        );
        let collision = Arc::new(collision);
        loaded.insert(key, Arc::downgrade(&collision));
        Some(collision)
    }

    fn bounds(&self, tri: &[u32; 3]) -> [f32; 4] {
        let mut b = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
        for i in tri {
            let v = self.vertices[*i as usize];
            b = [
                b[0].min(v[0]),
                b[1].min(v[2]),
                b[2].max(v[0]),
                b[3].max(v[2]),
            ];
        }
        b
    }

    /// The cells (indices) a box across the ground, [min x, min z, max x, max z], is over.
    fn cells_over(&self, b: &[f32; 4]) -> impl Iterator<Item = usize> + use<> {
        let clamp = |value: f32, origin: f32, count: u32| {
            (((value - origin) / CELL).floor().max(0.0) as u32).min(count - 1)
        };
        let (x0, x1) = (
            clamp(b[0], self.origin[0], self.columns),
            clamp(b[2], self.origin[0], self.columns),
        );
        let (z0, z1) = (
            clamp(b[1], self.origin[1], self.rows),
            clamp(b[3], self.origin[1], self.rows),
        );
        let columns = self.columns;
        (z0..=z1).flat_map(move |z| (x0..=x1).map(move |x| (z * columns + x) as usize))
    }

    fn cell_triangles(&self, cell: usize) -> &[u32] {
        &self.refs[self.offsets[cell] as usize..self.offsets[cell + 1] as usize]
    }

    fn triangle(&self, index: u32) -> [Vec3A; 3] {
        let tri = self.triangles[index as usize];
        tri.map(|i| Vec3A::from_array(self.vertices[i as usize]))
    }

    /// Whether (x, z) is inside the grid at all.
    fn covers(&self, x: f32, z: f32) -> bool {
        x >= self.origin[0]
            && z >= self.origin[1]
            && x < self.origin[0] + self.columns as f32 * CELL
            && z < self.origin[1] + self.rows as f32 * CELL
    }

    /// The highest floor at (x, z) between `bottom` and `top`, or None.
    pub fn floor(&self, x: f32, z: f32, top: f32, bottom: f32) -> Option<f32> {
        if !self.covers(x, z) {
            return None;
        }
        let cell = self.cells_over(&[x, z, x, z]).next()?;
        let mut best: Option<f32> = None;
        for index in self.cell_triangles(cell) {
            if !self.floors[*index as usize] || self.is_gone(*index) {
                continue;
            }
            let [a, b, c] = self.triangle(*index);
            if let Some(y) = height_at(a, b, c, x, z)
                && y <= top
                && y >= bottom
                && best.is_none_or(|best| y > best)
            {
                best = Some(y);
            }
        }
        best
    }

    /// Whether it's from an older file, made before the open doors were left open (version 4) or the simple shapes
    /// were in it (version 3 or 2): the zone's navmesh and collision are made again.
    pub fn is_older(&self) -> bool {
        self.version < VERSION
    }

    /// Whether the file had the triangles' materials (version 3 on.)
    pub fn has_materials(&self) -> bool {
        !self.materials.is_empty() || self.triangles.is_empty()
    }

    /// The highest floor at (x, z) between `bottom` and `top`, with its triangle's material (0 when unknown), or None.
    pub fn floor_material(&self, x: f32, z: f32, top: f32, bottom: f32) -> Option<(f32, u64)> {
        if !self.covers(x, z) {
            return None;
        }
        let cell = self.cells_over(&[x, z, x, z]).next()?;
        let mut best: Option<(f32, u64)> = None;
        for index in self.cell_triangles(cell) {
            if !self.floors[*index as usize] || self.is_gone(*index) {
                continue;
            }
            let [a, b, c] = self.triangle(*index);
            if let Some(y) = height_at(a, b, c, x, z)
                && y <= top
                && y >= bottom
                && best.is_none_or(|(best, _)| y > best)
            {
                let material = self.materials.get(*index as usize).copied().unwrap_or(0);
                best = Some((y, material));
            }
        }
        best
    }

    /// Whether triangle `index` belongs to a gimmick that's gone here.
    fn is_gone(&self, index: u32) -> bool {
        let owner = self.owners.get(index as usize).copied().unwrap_or(0);
        owner != 0 && self.gone.get(owner as usize - 1).copied().unwrap_or(false)
    }

    /// The layout ID of triangle `index`'s gimmick, if it belongs to one.
    fn gimmick_of(&self, index: u32) -> Option<u32> {
        let owner = self.owners.get(index as usize).copied().unwrap_or(0);
        (owner != 0).then(|| self.gimmicks[owner as usize - 1])
    }

    /// Whether the event object with this layout ID has a gimmick here.
    pub fn has_gimmick(&self, layout_id: u32) -> bool {
        self.gimmicks.contains(&layout_id)
    }

    /// Takes the gimmick of the event object with this layout ID away (`gone`), or puts it back. Whether that
    /// changed anything.
    pub fn set_gimmick_gone(&mut self, layout_id: u32, gone: bool) -> bool {
        let Some(index) = self.gimmicks.iter().position(|g| *g == layout_id) else {
            return false;
        };
        let changed = self.gone[index] != gone;
        self.gone[index] = gone;
        changed
    }

    /// Marks the gimmick of the event object with this layout ID as put up by the director (`sealed`) or not.
    pub fn set_gimmick_sealed(&mut self, layout_id: u32, sealed: bool) {
        if let Some(index) = self.gimmicks.iter().position(|g| *g == layout_id) {
            self.sealed[index] = sealed;
        }
    }

    fn is_sealed(&self, layout_id: u32) -> bool {
        self.gimmicks
            .iter()
            .position(|g| *g == layout_id)
            .is_some_and(|index| self.sealed[index])
    }

    /// Whether triangle `index` belongs to a gimmick the director put up (shown) here.
    fn is_sealed_triangle(&self, index: u32) -> bool {
        let owner = self.owners.get(index as usize).copied().unwrap_or(0);
        owner != 0
            && !self.is_gone(index)
            && self
                .sealed
                .get(owner as usize - 1)
                .copied()
                .unwrap_or(false)
    }

    /// Whether the straight line from `from` to `to` goes into a triangle that `counts`, looked for a cell's length at
    /// a time (a long line's whole box would be most of a room's cells.)
    fn line_meets(&self, from: Vec3A, to: Vec3A, counts: impl Fn(u32) -> bool) -> bool {
        let pieces = (Vec3A::new(to.x - from.x, 0.0, to.z - from.z).length() / CELL)
            .ceil()
            .max(1.0) as usize;
        let mut seen = std::collections::HashSet::new();
        for piece in 0..pieces {
            let a = from.lerp(to, piece as f32 / pieces as f32);
            let b = from.lerp(to, (piece + 1) as f32 / pieces as f32);
            let bounds = [a.x.min(b.x), a.z.min(b.z), a.x.max(b.x), a.z.max(b.z)];
            for cell in self.cells_over(&bounds) {
                for index in self.cell_triangles(cell) {
                    if !seen.insert(*index) || !counts(*index) {
                        continue;
                    }
                    let [p, q, r] = self.triangle(*index);
                    if segment_hits(from, to, p, q, r).is_some() {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Whether `from` and `to` (at eye height: `LINE_OF_SIGHT_HEIGHT` over each one's feet) see each other: nothing
    /// of the collision between them (walls, floors, ceilings) but gimmicks that are gone, or that the director hasn't
    /// put up (they stop no one for good: some aren't solid for players.)
    pub fn sees(&self, from: Vec3A, to: Vec3A) -> bool {
        let up = Vec3A::new(0.0, LINE_OF_SIGHT_HEIGHT, 0.0);
        !self.line_meets(from + up, to + up, |index| {
            self.gimmick_of(index).is_none() || self.is_sealed_triangle(index)
        })
    }

    /// Whether the way from `from` to `to` (across the ground, at a body's height) goes through a gimmick the director
    /// put up: a sealed room's wall, a door it shows as shut. A walker held up somewhere else never goes on past one.
    pub fn crosses_sealed(&self, from: Vec3A, to: Vec3A) -> bool {
        if !self.sealed.contains(&true) {
            return false;
        }
        WALL_HEIGHTS.iter().any(|height| {
            let up = Vec3A::new(0.0, self.climb + height, 0.0);
            self.line_meets(from + up, Vec3A::new(to.x, from.y, to.z) + up, |index| {
                self.is_sealed_triangle(index)
            })
        })
    }

    /// The floor a body stands on at (x, z) between `top` and `bottom`: the highest floor there, or, where that's a
    /// drop deeper than a step (or there's none), the highest floor under the body's edge, `radius` out: a body as wide
    /// as a player's doesn't fall through a gap narrower than itself. Bob's Copperbell Mines bundle of 2026-10-27: in
    /// Kottos' arena the bots dropped through the metal grating's holes onto the floor 2 yalms below, one step at a
    /// time, where he walked across it.
    fn body_floor(&self, x: f32, z: f32, top: f32, bottom: f32) -> Option<f32> {
        let centre = self.floor(x, z, top, bottom);
        let mut edge: Option<f32> = None;
        for k in 0..8 {
            let angle = k as f32 * std::f32::consts::FRAC_PI_4;
            let (px, pz) = (x + self.radius * angle.cos(), z + self.radius * angle.sin());
            if let Some(y) = self.floor(px, pz, top, bottom)
                && edge.is_none_or(|best| y > best)
            {
                edge = Some(y);
            }
        }
        match (centre, edge) {
            (Some(centre), Some(edge)) if edge - centre > self.climb => Some(edge),
            (None, edge) => edge,
            (centre, _) => centre,
        }
    }

    /// The first thing a straight line from `from` to `to` goes into: its surface's facing across the ground, and
    /// its gimmick's layout ID if it's one.
    fn hit(&self, from: Vec3A, to: Vec3A) -> Option<(Vec3A, Option<u32>)> {
        let b = [
            from.x.min(to.x),
            from.z.min(to.z),
            from.x.max(to.x),
            from.z.max(to.z),
        ];
        let mut nearest: Option<(f32, u32)> = None;
        let mut seen = Vec::new();
        for cell in self.cells_over(&b) {
            for index in self.cell_triangles(cell) {
                if seen.contains(index) {
                    continue;
                }
                seen.push(*index);
                if self.is_gone(*index) {
                    continue;
                }
                let [a, b, c] = self.triangle(*index);
                if let Some(t) = segment_hits(from, to, a, b, c)
                    && nearest.is_none_or(|(best, _)| t < best)
                {
                    nearest = Some((t, *index));
                }
            }
        }
        let (_, index) = nearest?;
        let n = normal(&self.vertices, &self.triangles[index as usize]);
        let across = Vec3A::new(n.x, 0.0, n.z).normalize_or_zero();
        // Facing back at the walker, whichever way the mesh was wound.
        let way = Vec3A::new(to.x - from.x, 0.0, to.z - from.z);
        let facing = if across.dot(way) > 0.0 {
            -across
        } else {
            across
        };
        Some((facing, self.gimmick_of(index)))
    }

    /// Where someone put at `position` stands: on the floor under them (or just over them, if they're a little under
    /// it), as they'd land there. Off the collision it stays as it is.
    pub fn settle(&self, position: Vec3A) -> Vec3A {
        let (x, y, z) = (position.x, position.y, position.z);
        let below = self.body_floor(x, z, y + self.climb, y - SETTLE_DEPTH);
        let above = self.floor(x, z, y + SETTLE_RISE, y + self.climb);
        // The nearer of the two: a little under a floor is on it, well under one is over the floor below.
        let floor = match (below, above) {
            (Some(below), Some(above)) => Some(if y - below <= above - y { below } else { above }),
            (below, above) => below.or(above),
        };
        match floor {
            Some(floor) => Vec3A::new(x, floor, z),
            None => position,
        }
    }

    /// One step of `way` (across the ground) from `at`, standing on the floor.
    fn step(&self, at: Vec3A, way: Vec3A, max_drop: f32) -> Result<Vec3A, Stop> {
        let next = at + way;
        let ahead = way.normalize_or_zero() * self.radius;
        for height in WALL_HEIGHTS {
            let up = Vec3A::new(0.0, self.climb + height, 0.0);
            if let Some((facing, gimmick)) = self.hit(at + up, next + ahead + up) {
                return Err(Stop::Wall(facing, gimmick));
            }
        }
        match self.body_floor(next.x, next.z, at.y + self.climb, at.y - max_drop) {
            Some(y) => Ok(Vec3A::new(next.x, y, next.z)),
            None => Err(Stop::NoFloor),
        }
    }

    /// Walks from `from` towards `to` across the ground as a player would: over the floor, up steps no higher than
    /// `climb`, down drops no deeper than `max_drop`, sliding along walls, and stopping where it can go no further.
    /// `to`'s height is ignored (the floor decides.)
    pub fn walk(&self, from: Vec3A, to: Vec3A, max_drop: f32) -> Walk {
        let mut at = self.settle(from);
        let way = Vec3A::new(to.x - at.x, 0.0, to.z - at.z);
        let distance = way.length();
        if distance < 1e-4 {
            return Walk {
                at,
                blocked: false,
                gimmick: None,
                sealed: false,
            };
        }
        let steps = (distance / SUBSTEP).ceil().max(1.0);
        let each = way / steps;
        for _ in 0..steps as usize {
            match self.step(at, each, max_drop) {
                Ok(next) => at = next,
                Err(Stop::Wall(facing, gimmick)) => {
                    // Along the wall: what's left of the step once the part into it is taken off.
                    let along = each - facing * each.dot(facing);
                    match (along.length() > each.length() * 0.2)
                        .then(|| self.step(at, along, max_drop))
                    {
                        Some(Ok(next)) => at = next,
                        _ => {
                            return Walk {
                                at,
                                blocked: true,
                                gimmick,
                                sealed: gimmick.is_some_and(|layout_id| self.is_sealed(layout_id)),
                            };
                        }
                    }
                }
                Err(Stop::NoFloor) => {
                    return Walk {
                        at,
                        blocked: true,
                        gimmick: None,
                        sealed: false,
                    };
                }
            }
        }
        Walk {
            at,
            blocked: false,
            gimmick: None,
            sealed: false,
        }
    }

    pub fn triangle_count(&self) -> usize {
        self.triangles.len()
    }
}

/// Reads a file's numbers in turn.
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let slice = self.bytes.get(self.at..self.at.checked_add(n)?)?;
        self.at += n;
        Some(slice)
    }

    fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }

    fn u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }

    fn f32(&mut self) -> Option<f32> {
        Some(f32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }
}

/// Whether the collision file at `path` is one this build reads (its first bytes say so; read cheaply, to wait for a
/// new one while an older one is being replaced.)
pub fn is_current(path: &Path) -> bool {
    use std::io::Read;
    let mut head = [0u8; 8];
    std::fs::File::open(path)
        .and_then(|mut file| file.read_exact(&mut head))
        .is_ok()
        && &head[..4] == MAGIC
        && head[4..] == VERSION.to_le_bytes()
}

/// Where the collision file for a navmesh at `nvm_path` goes.
pub fn path_for(nvm_path: &Path) -> std::path::PathBuf {
    let mut path = nvm_path.as_os_str().to_owned();
    path.push(".col");
    path.into()
}

fn normal(vertices: &[[f32; 3]], tri: &[u32; 3]) -> Vec3A {
    let [a, b, c] = tri.map(|i| Vec3A::from_array(vertices[i as usize]));
    (b - a).cross(c - a)
}

/// The height of triangle (a, b, c) over (x, z), if it's over it.
fn height_at(a: Vec3A, b: Vec3A, c: Vec3A, x: f32, z: f32) -> Option<f32> {
    let d = (b.x - a.x) * (c.z - a.z) - (c.x - a.x) * (b.z - a.z);
    if d.abs() < 1e-9 {
        return None;
    }
    let u = ((b.x - x) * (c.z - z) - (c.x - x) * (b.z - z)) / d;
    let v = ((c.x - x) * (a.z - z) - (a.x - x) * (c.z - z)) / d;
    let w = 1.0 - u - v;
    const EDGE: f32 = -1e-4;
    (u >= EDGE && v >= EDGE && w >= EDGE).then_some(u * a.y + v * b.y + w * c.y)
}

/// How far along `from`→`to` (0 to 1) it goes through triangle (a, b, c), from either side (Möller–Trumbore.)
fn segment_hits(from: Vec3A, to: Vec3A, a: Vec3A, b: Vec3A, c: Vec3A) -> Option<f32> {
    let dir = to - from;
    let e1 = b - a;
    let e2 = c - a;
    let p = dir.cross(e2);
    let det = e1.dot(p);
    if det.abs() < 1e-9 {
        return None;
    }
    let inv = 1.0 / det;
    let s = from - a;
    let u = s.dot(p) * inv;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = s.cross(e1);
    let v = dir.dot(q) * inv;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = e2.dot(q) * inv;
    (0.0..=1.0).contains(&t).then_some(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A quad from (x0, z0) to (x1, z1) at height y (two triangles.)
    fn floor_quad(
        v: &mut Vec<[f32; 3]>,
        t: &mut Vec<[u32; 3]>,
        x0: f32,
        z0: f32,
        x1: f32,
        z1: f32,
        y: f32,
    ) {
        let i = v.len() as u32;
        v.extend([[x0, y, z0], [x1, y, z0], [x1, y, z1], [x0, y, z1]]);
        t.extend([[i, i + 1, i + 2], [i, i + 2, i + 3]]);
    }

    /// A wall across x = `x`, from z0 to z1, from y0 to y1.
    fn wall_x(
        v: &mut Vec<[f32; 3]>,
        t: &mut Vec<[u32; 3]>,
        x: f32,
        z0: f32,
        z1: f32,
        y0: f32,
        y1: f32,
    ) {
        let i = v.len() as u32;
        v.extend([[x, y0, z0], [x, y0, z1], [x, y1, z1], [x, y1, z0]]);
        t.extend([[i, i + 1, i + 2], [i, i + 2, i + 3]]);
    }

    fn world(build: impl FnOnce(&mut Vec<[f32; 3]>, &mut Vec<[u32; 3]>)) -> Collision {
        let (mut v, mut t) = (Vec::new(), Vec::new());
        build(&mut v, &mut t);
        let collision = Collision::build(v, t, Vec::new(), Vec::new(), 0.75, 0.4, 55.0);
        // The file reads back as it was written.
        let again = Collision::from_bytes(&collision.to_bytes()).unwrap();
        assert_eq!(again.triangles, collision.triangles);
        assert_eq!(again.refs, collision.refs);
        assert_eq!(again.owners, collision.owners);
        again
    }

    #[test]
    fn follows_a_slope() {
        // A ramp rising 1 in 4 from x = 0 to x = 20 (5 yalms up.)
        let c = world(|v, t| {
            v.extend([
                [0.0, 0.0, -5.0],
                [20.0, 5.0, -5.0],
                [20.0, 5.0, 5.0],
                [0.0, 0.0, 5.0],
            ]);
            t.extend([[0, 1, 2], [0, 2, 3]]);
        });
        let walk = c.walk(Vec3A::new(1.0, 0.25, 0.0), Vec3A::new(17.0, 0.0, 0.0), 50.0);
        assert!(!walk.blocked);
        assert!((walk.at.x - 17.0).abs() < 1e-3);
        assert!((walk.at.y - 4.25).abs() < 1e-3, "{walk:?}");
    }

    #[test]
    fn stops_at_a_wall_and_slides_along_it() {
        let c = world(|v, t| {
            floor_quad(v, t, -10.0, -10.0, 10.0, 10.0, 0.0);
            wall_x(v, t, 3.0, -10.0, 10.0, 0.0, 4.0);
        });
        let walk = c.walk(Vec3A::ZERO, Vec3A::new(8.0, 0.0, 0.0), 50.0);
        assert!(walk.blocked);
        assert!(walk.at.x < 3.0 - 0.3 && walk.at.x > 2.0, "{walk:?}");
        assert_eq!(walk.at.y, 0.0);
        // At an angle it goes along the wall instead.
        let walk = c.walk(Vec3A::ZERO, Vec3A::new(8.0, 0.0, 8.0), 50.0);
        assert!(walk.at.x < 3.0 && walk.at.z > 6.0, "{walk:?}");
    }

    #[test]
    fn steps_up_a_step_but_not_a_cliff() {
        let c = world(|v, t| {
            floor_quad(v, t, -10.0, -10.0, 2.0, 10.0, 0.0);
            floor_quad(v, t, 2.0, -10.0, 6.0, 10.0, 0.5);
            floor_quad(v, t, 6.0, -10.0, 20.0, 10.0, 5.0);
            wall_x(v, t, 2.0, -10.0, 10.0, 0.0, 0.5);
            wall_x(v, t, 6.0, -10.0, 10.0, 0.5, 5.0);
        });
        let walk = c.walk(Vec3A::ZERO, Vec3A::new(4.0, 0.0, 0.0), 50.0);
        assert!(!walk.blocked);
        assert_eq!(walk.at.y, 0.5);
        let walk = c.walk(Vec3A::new(4.0, 0.5, 0.0), Vec3A::new(10.0, 0.0, 0.0), 50.0);
        assert!(walk.blocked && walk.at.x < 6.0, "{walk:?}");
        assert_eq!(walk.at.y, 0.5);
    }

    #[test]
    fn drops_off_a_ledge_onto_the_floor_below() {
        // A ledge 30 yalms up ends at x = 5; the beach below goes on (Sastasha.)
        let c = world(|v, t| {
            floor_quad(v, t, -10.0, -10.0, 5.0, 10.0, 24.0);
            floor_quad(v, t, -10.0, -10.0, 30.0, 10.0, -6.0);
            wall_x(v, t, 5.0, -10.0, 10.0, -6.0, 24.0);
        });
        let walk = c.walk(
            Vec3A::new(0.0, 24.0, 0.0),
            Vec3A::new(8.0, -6.0, 0.0),
            f32::INFINITY,
        );
        assert!(!walk.blocked);
        assert_eq!(walk.at.y, -6.0);
        assert!((walk.at.x - 8.0).abs() < 1e-3);
        // Not allowed to drop that far: it stops at the edge (its body over it by no more than its radius.)
        let walk = c.walk(Vec3A::new(0.0, 24.0, 0.0), Vec3A::new(8.0, -6.0, 0.0), 4.0);
        assert!(
            walk.blocked && walk.at.x <= 5.0 + c.radius && walk.at.y == 24.0,
            "{walk:?}"
        );
        // Back up, it can't climb the cliff.
        let walk = c.walk(
            Vec3A::new(8.0, -6.0, 0.0),
            Vec3A::new(0.0, 24.0, 0.0),
            f32::INFINITY,
        );
        assert!(
            walk.blocked && walk.at.x > 5.0 && walk.at.y == -6.0,
            "{walk:?}"
        );
    }

    #[test]
    fn settles_on_the_floor() {
        let c = world(|v, t| {
            floor_quad(v, t, -10.0, -10.0, 10.0, 10.0, 0.0);
            floor_quad(v, t, -10.0, -10.0, 10.0, 10.0, 10.0); // a floor above (a bridge)
        });
        // In the air between: down onto the floor below, not up onto the bridge.
        assert_eq!(c.settle(Vec3A::new(0.0, 5.0, 0.0)).y, 0.0);
        // A little under the bridge's floor: up onto it.
        assert_eq!(c.settle(Vec3A::new(0.0, 9.0, 0.0)).y, 10.0);
        assert_eq!(c.settle(Vec3A::new(0.0, 10.2, 0.0)).y, 10.0);
        // Off the collision it stays.
        assert_eq!(c.settle(Vec3A::new(50.0, 3.0, 0.0)).y, 3.0);
    }

    #[test]
    fn floors_keep_their_material() {
        // A floor (material 7), and a raised patch of slime on it (material 0x1234), as in Toto-Rak's last room.
        let (mut v, mut t) = (Vec::new(), Vec::new());
        floor_quad(&mut v, &mut t, -10.0, -10.0, 10.0, 10.0, 0.0);
        let floor = t.len();
        floor_quad(&mut v, &mut t, 2.0, 2.0, 4.0, 4.0, 0.2);
        let mut materials = vec![7; floor];
        materials.resize(t.len(), 0x1234);
        let built = Collision::build(v, t, Vec::new(), materials, 0.75, 0.4, 55.0);
        let c = Collision::from_bytes(&built.to_bytes()).unwrap();
        assert!(c.has_materials());
        assert_eq!(c.floor_material(0.0, 0.0, 0.5, -0.5), Some((0.0, 7)));
        assert_eq!(c.floor_material(3.0, 3.0, 0.7, -0.3), Some((0.2, 0x1234)));
        // In the air: no floor near enough.
        assert_eq!(c.floor_material(0.0, 0.0, 3.5, 2.5), None);

        // A version 2 file (no materials) still reads, without them.
        let mut old = built.to_bytes();
        old[4..8].copy_from_slice(&2u32.to_le_bytes());
        old.truncate(old.len() - 8 * built.triangles.len());
        let c = Collision::from_bytes(&old).unwrap();
        assert!(!c.has_materials());
        assert!(c.is_older());
        assert_eq!(c.floor_material(3.0, 3.0, 0.7, -0.3), Some((0.2, 0)));

        // A version 3 file (before the simple shapes) reads with its materials, and is made again.
        let mut three = built.to_bytes();
        three[4..8].copy_from_slice(&3u32.to_le_bytes());
        let c = Collision::from_bytes(&three).unwrap();
        assert!(c.has_materials() && c.is_older());
        // A version 4 file (every door shut) too.
        three[4..8].copy_from_slice(&4u32.to_le_bytes());
        assert!(Collision::from_bytes(&three).unwrap().is_older());
        assert!(!Collision::from_bytes(&built.to_bytes()).unwrap().is_older());
    }

    #[test]
    fn a_gimmick_stops_walkers_until_it_is_gone() {
        // A barrier (event object 4100000's gimmick) across the floor at x = 3.
        let (mut v, mut t) = (Vec::new(), Vec::new());
        floor_quad(&mut v, &mut t, -10.0, -10.0, 10.0, 10.0, 0.0);
        let floor = t.len();
        wall_x(&mut v, &mut t, 3.0, -10.0, 10.0, 0.0, 4.0);
        let mut owners = vec![0; floor];
        owners.resize(t.len(), 4100000);
        let built = Collision::build(v, t, owners, Vec::new(), 0.75, 0.4, 55.0);
        let mut c = Collision::from_bytes(&built.to_bytes()).unwrap();
        assert_eq!(c.owners, built.owners);
        assert!(c.has_gimmick(4100000) && !c.has_gimmick(1));

        let walk = c.walk(Vec3A::ZERO, Vec3A::new(8.0, 0.0, 0.0), 50.0);
        assert!(walk.blocked && walk.at.x < 3.0, "{walk:?}");
        assert_eq!(walk.gimmick, Some(4100000));

        // Hidden in this instance: gone for its walkers, still there for another instance's.
        let other = c.clone();
        assert!(c.set_gimmick_gone(4100000, true));
        assert!(!c.set_gimmick_gone(4100000, true));
        let walk = c.walk(Vec3A::ZERO, Vec3A::new(8.0, 0.0, 0.0), 50.0);
        assert!(!walk.blocked && (walk.at.x - 8.0).abs() < 1e-3, "{walk:?}");
        assert!(
            other
                .walk(Vec3A::ZERO, Vec3A::new(8.0, 0.0, 0.0), 50.0)
                .blocked
        );

        // Shown again, it stands again.
        assert!(c.set_gimmick_gone(4100000, false));
        assert!(c.walk(Vec3A::ZERO, Vec3A::new(8.0, 0.0, 0.0), 50.0).blocked);
    }

    #[test]
    fn a_body_does_not_fall_through_a_grating() {
        // A grating at y = 0 with a gap 0.4 wide across it at x = 4..4.4 (narrower than a body, 0.8 across), over a
        // floor 2 below.
        let c = world(|v, t| {
            floor_quad(v, t, -10.0, -10.0, 4.0, 10.0, 0.0);
            floor_quad(v, t, 4.4, -10.0, 20.0, 10.0, 0.0);
            floor_quad(v, t, -10.0, -10.0, 20.0, 10.0, -2.0);
        });
        let walk = c.walk(Vec3A::ZERO, Vec3A::new(10.0, 0.0, 0.0), f32::INFINITY);
        assert!(!walk.blocked);
        assert_eq!(walk.at.y, 0.0, "{walk:?}");
        // Put right over the gap, it stays on the grating.
        assert_eq!(c.settle(Vec3A::new(4.2, 0.1, 0.0)).y, 0.0);
        // A real edge still drops it, once it's past by more than its radius.
        let walk = c.walk(
            Vec3A::new(15.0, 0.0, 0.0),
            Vec3A::new(25.0, 0.0, 0.0),
            f32::INFINITY,
        );
        assert!(walk.at.y <= 0.0);
    }

    #[test]
    fn a_wall_or_a_floor_blocks_the_line_of_sight() {
        // A room's floor with a wall at x = 3 (from z -10 to 0 only), a floor 6 above, and a door (gimmick 77) at x = -3.
        let (mut v, mut t) = (Vec::new(), Vec::new());
        floor_quad(&mut v, &mut t, -10.0, -10.0, 10.0, 10.0, 0.0);
        wall_x(&mut v, &mut t, 3.0, -10.0, 0.0, 0.0, 4.0);
        floor_quad(&mut v, &mut t, -10.0, -10.0, 10.0, 10.0, 6.0);
        let plain = t.len();
        wall_x(&mut v, &mut t, -3.0, -10.0, 10.0, 0.0, 4.0);
        let mut owners = vec![0; plain];
        owners.resize(t.len(), 77);
        let mut c = Collision::build(v, t, owners, Vec::new(), 0.75, 0.4, 55.0);
        let here = Vec3A::new(0.0, 0.0, -5.0);
        // Through the wall: no; past its end: yes; through the floor above: no.
        assert!(!c.sees(here, Vec3A::new(8.0, 0.0, -5.0)));
        assert!(c.sees(here, Vec3A::new(8.0, 0.0, 9.0)));
        assert!(!c.sees(here, Vec3A::new(0.0, 6.0, -5.0)));
        // A long way across a room, cell by cell (more than one of them.)
        assert!(c.sees(Vec3A::new(-2.0, 0.0, -9.0), Vec3A::new(2.0, 0.0, 9.0)));
        // A door the director hasn't touched doesn't block it, nor block the way; one it shut does both.
        let beyond = Vec3A::new(-8.0, 0.0, -5.0);
        assert!(c.sees(here, beyond) && !c.crosses_sealed(here, beyond));
        c.set_gimmick_sealed(77, true);
        assert!(!c.sees(here, beyond) && c.crosses_sealed(here, beyond));
        assert!(!c.crosses_sealed(here, Vec3A::new(8.0, 0.0, 9.0)));
        // Gone (opened), it's no wall at all.
        c.set_gimmick_gone(77, true);
        assert!(c.sees(here, beyond) && !c.crosses_sealed(here, beyond));
    }

    #[test]
    fn only_a_sealed_gimmick_says_so() {
        let (mut v, mut t) = (Vec::new(), Vec::new());
        floor_quad(&mut v, &mut t, -10.0, -10.0, 10.0, 10.0, 0.0);
        let floor = t.len();
        wall_x(&mut v, &mut t, 3.0, -10.0, 10.0, 0.0, 4.0);
        let mut owners = vec![0; floor];
        owners.resize(t.len(), 77);
        let mut c = Collision::build(v, t, owners, Vec::new(), 0.75, 0.4, 55.0);
        let walk = c.walk(Vec3A::ZERO, Vec3A::new(8.0, 0.0, 0.0), 50.0);
        assert_eq!((walk.gimmick, walk.sealed), (Some(77), false));
        c.set_gimmick_sealed(77, true);
        let walk = c.walk(Vec3A::ZERO, Vec3A::new(8.0, 0.0, 0.0), 50.0);
        assert_eq!((walk.gimmick, walk.sealed), (Some(77), true));
    }
}
