use std::{
    collections::HashMap,
    path::PathBuf,
    ptr::{null, null_mut},
};

use glam::{Affine3A, Vec3A};
use icarus::RecastNavimesh::RecastNavimeshSheet;
use icarus::TerritoryType::TerritoryTypeSheet;
use kawari::config::get_config;
use kawari_world::{Navmesh, NavmeshParams, NavmeshTile, collision::Collision};
use physis::{
    Language,
    layer::{Layer, LayerEntryData, ModelCollisionType, TriggerBoxShape},
    lgb::Lgb,
    lvb::Lvb,
    pcb::{ListEntry, Node, Pcb, Polygon},
    resource::{ResourceResolver, SqPackResource, UnpackedResource},
    sgb::Sgb,
};
use recastnavigation_sys::{
    CreateContext, dtCreateNavMeshData, dtNavMeshCreateParams, rcAllocCompactHeightfield,
    rcAllocContourSet, rcAllocHeightfield, rcAllocPolyMesh, rcAllocPolyMeshDetail,
    rcBuildCompactHeightfield, rcBuildContours, rcBuildContoursFlags_RC_CONTOUR_TESS_WALL_EDGES,
    rcBuildDistanceField, rcBuildPolyMesh, rcBuildPolyMeshDetail, rcBuildRegions, rcCalcGridSize,
    rcCreateHeightfield, rcErodeWalkableArea, rcFilterLedgeSpans,
    rcFilterLowHangingWalkableObstacles, rcFilterWalkableLowHeightSpans, rcHeightfield,
    rcMarkWalkableTriangles, rcRasterizeTriangles,
};

fn main() {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = std::env::args().collect();
    let zone_id: u16 = args[1].parse().unwrap();
    let destination_path: String = args[2].parse().unwrap();

    let config = get_config();

    tracing::info!(
        "Generating navmesh for zone {zone_id}, writing to {destination_path}, this will take a while!"
    );

    let mut resolver = ResourceResolver::new();
    for path in &config.filesystem.additional_search_paths {
        let unpacked_resource = UnpackedResource::from_existing(path);
        resolver.add_source(unpacked_resource);
    }
    resolver.add_source(SqPackResource::from_existing(
        &config.filesystem.game_path(),
    ));

    let sheet = TerritoryTypeSheet::read_from(&mut resolver, Language::None).unwrap();
    let Some(row) = sheet.row(zone_id as u32) else {
        tracing::error!("Invalid zone id {zone_id}!");
        return;
    };

    // e.g. ffxiv/fst_f1/fld/f1f3/level/f1f3
    let bg_path = row.Bg;
    let name = row.Name;

    let path = format!("bg/{}.lvb", bg_path);
    let lvb = resolver.parsed::<Lvb>(&path).unwrap();

    let navimesh_sheet = RecastNavimeshSheet::read_from(&mut resolver, Language::None).unwrap();
    // Use default settings unless overriden
    let mut navimesh_row = navimesh_sheet
        .row(0)
        .expect("No default row in RecastNavimesh sheet?");
    for (_, row) in navimesh_sheet.into_iter().flatten_subrows() {
        if row.Name == name {
            tracing::info!("Using navimesh settings override for this zone!");
            navimesh_row = row;
        }
    }

    let context;
    unsafe {
        context = CreateContext(true);
    }

    let scene = &lvb.sections[0];

    let Pcb::List(pcblist) = resolver
        .parsed::<Pcb>(&format!(
            "{}/collision/list.pcb",
            scene.general.bg_path.value
        ))
        .unwrap()
    else {
        panic!("list.pcb isn't a list?!");
    };

    // Every collision mesh in the zone, as a player meets it: the terrain's plates, then the layers' bg parts,
    // collision boxes and shared groups.
    let mut geometry = Geometry::default();
    let lgbs: Vec<(String, Lgb)> = scene
        .lgb_paths
        .iter()
        .filter_map(|path| match resolver.parsed::<Lgb>(path) {
            Ok(lgb) => Some((path.clone(), lgb)),
            Err(e) => {
                tracing::error!(
                    "Failed to parse {path}: {e}, this is most likely a bug in Physis and should be reported somewhere!"
                );
                None
            }
        })
        .collect();
    // The shared groups an event object controls (its bound instance: a barrier, a door, a boss room's wall), by
    // their instance ID: the object's layout ID. They're there for players only while the object is (the client
    // takes them away when the server hides it), so their triangles carry it (kawari_world::collision's gimmicks)
    // and the navmesh is built without them.
    let mut bound: HashMap<u32, u32> = HashMap::new();
    for (_, lgb) in &lgbs {
        for chunk in &lgb.chunks {
            for layer in &chunk.layers {
                for object in &layer.objects {
                    if let LayerEntryData::EventObject(eobj) = &object.data
                        && eobj.bound_instance_id != 0
                    {
                        bound.insert(eobj.bound_instance_id, object.instance_id);
                    }
                }
            }
        }
    }
    // Which layers are this territory's (its LVB layer set), as the world server spawns from: counted only, to see
    // how much collision comes from the others.
    let layer_set = scene
        .layer_sets
        .layer_sets
        .iter()
        .find(|set| set.territory_type_id == zone_id)
        .map(|set| set.id);
    for entry in &pcblist.entries {
        add_plate(
            entry,
            &scene.general.bg_path.value,
            &mut resolver,
            &mut geometry,
        );
    }

    let mut outside_layer_set = 0;
    for (_, lgb) in &lgbs {
        for chunk in &lgb.chunks {
            for layer in &chunk.layers {
                // Exclude festival objects which *usually* don't have collision, but this is to exclude non-Global festivals that include non-existent SGBs.
                if layer.header.festival_id != 0 {
                    continue;
                }

                let before = geometry.triangles.len();
                let transform = Affine3A::default();
                walk_layer(&mut resolver, layer, &transform, &bound, 0, &mut geometry);
                if layer_set.is_some_and(|id| !layer.header.has_layer_set(id as u32)) {
                    outside_layer_set += geometry.triangles.len() - before;
                }
            }
        }
    }
    let gimmick_triangles = geometry.owners.iter().filter(|owner| **owner != 0).count();
    let mut gimmicks: Vec<u32> = geometry
        .owners
        .iter()
        .copied()
        .filter(|owner| *owner != 0)
        .collect();
    gimmicks.sort_unstable();
    gimmicks.dedup();
    tracing::info!(
        "{} collision triangles from {} vertices; {gimmick_triangles} of them the gimmicks of {} event objects {gimmicks:?} (left out of the navmesh); {outside_layer_set} from layers outside the territory's layer set {layer_set:?}",
        geometry.triangles.len(),
        geometry.vertices.len(),
        gimmicks.len()
    );

    // The area to cover: the geometry's own bounds, all of it (list.pcb's leave out shared groups' meshes; and the
    // heights were cut at 100 yalms up and down, so anything higher or lower had no navmesh.)
    let (mut min_bounds, mut max_bounds) = ([f32::MAX; 3], [f32::MIN; 3]);
    for v in &geometry.vertices {
        for axis in 0..3 {
            min_bounds[axis] = min_bounds[axis].min(v[axis]);
            max_bounds[axis] = max_bounds[axis].max(v[axis]);
        }
    }
    if geometry.vertices.is_empty() {
        min_bounds = pcblist.bounds.min;
        max_bounds = pcblist.bounds.max;
    }
    let min_x = min_bounds[0];
    let min_z = min_bounds[2];
    let min_y = min_bounds[1] - 1.0;
    let max_y = max_bounds[1] + navimesh_row.AgentHeight + 1.0;
    min_bounds[1] = min_y;
    max_bounds[1] = max_y;

    let tile_origin_x = min_x;
    let tile_origin_y = min_z;

    let mut size_x: i32 = 0;
    let mut size_z: i32 = 0;

    let cell_size = navimesh_row.CellSize;
    let cell_height = navimesh_row.CellHeight;

    unsafe {
        rcCalcGridSize(
            min_bounds.as_ptr(),
            max_bounds.as_ptr(),
            cell_size,
            &mut size_x,
            &mut size_z,
        );
    }

    let tile_size = navimesh_row.TileSize as i32;
    let tile_width = (size_x + tile_size - 1) / tile_size;
    let tile_height = (size_z + tile_size - 1) / tile_size;

    let walkable_radius = (navimesh_row.AgentRadius / cell_size).ceil() as i32;
    let border_size = walkable_radius + 3;
    let tile_cell_size = tile_size as f32 * cell_size;
    let max_slope = navimesh_row.AgentMaxSlope;
    // Recast counts climbs in cell heights (it was in cell sizes.)
    let walkable_climb = (navimesh_row.AgentMaxClimb / cell_height).floor() as i32;
    let walkable_height = (navimesh_row.AgentHeight / cell_height).ceil() as i32;

    // Each tile's triangles (those over it, border included), so a tile only rasterizes what's on it.
    let border = border_size as f32 * cell_size;
    let tile_of = |value: f32, origin: f32, count: i32| {
        (((value - origin) / tile_cell_size).floor() as i32).clamp(0, count - 1)
    };
    let mut tile_triangles: Vec<Vec<i32>> = vec![Vec::new(); (tile_width * tile_height) as usize];
    for (tri, owner) in geometry.triangles.iter().zip(&geometry.owners) {
        // Gimmicks come and go: the navmesh goes through them (the collision stops walkers while they stand.)
        if *owner != 0 {
            continue;
        }
        let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
        for i in tri {
            let v = geometry.vertices[*i as usize];
            lo = [lo[0].min(v[0]), lo[1].min(v[2])];
            hi = [hi[0].max(v[0]), hi[1].max(v[2])];
        }
        for z in tile_of(lo[1] - border, min_z, tile_height)
            ..=tile_of(hi[1] + border, min_z, tile_height)
        {
            for x in tile_of(lo[0] - border, min_x, tile_width)
                ..=tile_of(hi[0] + border, min_x, tile_width)
            {
                tile_triangles[(z * tile_width + x) as usize].extend(tri.iter().map(|i| *i as i32));
            }
        }
    }

    let mut tiles = Vec::new();
    for z in 0..tile_height {
        for x in 0..tile_width {
            let indices = &tile_triangles[(z * tile_width + x) as usize];
            if indices.is_empty() {
                continue;
            }
            // Step 1: Create a heightfield
            unsafe {
                let min_bounds = [
                    (min_x + ((x as f32) * tile_cell_size)) - (border_size as f32 * cell_size),
                    min_y,
                    (min_z + ((z as f32) * tile_cell_size)) - (border_size as f32 * cell_size),
                ];
                let max_bounds = [
                    (min_x + ((x as f32 + 1.0) * tile_cell_size))
                        + (border_size as f32 * cell_size),
                    max_y,
                    (min_z + ((z as f32 + 1.0) * tile_cell_size))
                        + (border_size as f32 * cell_size),
                ];

                let width = tile_size + border_size * 2;
                let height = tile_size + border_size * 2;

                let height_field = rcAllocHeightfield();
                assert!(rcCreateHeightfield(
                    context,
                    height_field,
                    width,
                    height,
                    min_bounds.as_ptr(),
                    max_bounds.as_ptr(),
                    cell_size,
                    cell_height
                ));

                // Step 2: insert the tile's geometry into the heightfield
                let ntris = indices.len() as i32 / 3;
                let mut tri_area_ids: Vec<u8> = vec![0; ntris as usize];
                let vertices = geometry.vertices.as_ptr() as *const f32;
                rcMarkWalkableTriangles(
                    context,
                    max_slope,
                    vertices,
                    geometry.vertices.len() as i32,
                    indices.as_ptr(),
                    ntris,
                    tri_area_ids.as_mut_ptr(),
                );
                assert!(rcRasterizeTriangles(
                    context,
                    vertices,
                    geometry.vertices.len() as i32,
                    indices.as_ptr(),
                    tri_area_ids.as_ptr(),
                    ntris,
                    height_field,
                    walkable_climb,
                ));

                tiles.push(Tile { x, z, height_field });
            }
        }
    }

    let mut navmesh_tiles = Vec::new();
    let mut max_polys = 0;

    for tile in tiles {
        unsafe {
            // Step 2b: Some easy filtering
            rcFilterLowHangingWalkableObstacles(context, walkable_climb, tile.height_field);
            // No navmesh along a drop deeper than a step (a cliff's edge): Recast's usual filtering.
            rcFilterLedgeSpans(context, walkable_height, walkable_climb, tile.height_field);
            rcFilterWalkableLowHeightSpans(context, walkable_height, tile.height_field);

            // Step 3: Build a compact heightfield out of the normal heightfield
            let compact_heightfield = rcAllocCompactHeightfield();
            assert!(!tile.height_field.is_null());
            assert!(rcBuildCompactHeightfield(
                context,
                walkable_height, // In VX units
                walkable_climb,  // In VX units
                tile.height_field,
                compact_heightfield
            ));
            if (*compact_heightfield).spanCount == 0 {
                continue;
            }
            assert!((*compact_heightfield).spanCount > 0);

            assert!(rcErodeWalkableArea(
                context,
                walkable_radius, // In VX units
                compact_heightfield
            ));

            assert!(rcBuildDistanceField(context, compact_heightfield));

            let min_region_area = navimesh_row.RegionMinSize;
            let merge_region_area = navimesh_row.RegionMergedSize;
            assert!(rcBuildRegions(
                context,
                compact_heightfield,
                border_size,
                (min_region_area.powi(2)) as i32,   // In VX units
                (merge_region_area.powi(2)) as i32, // In VX units
            ));

            // Step 4: Build the contour set from the compact heightfield
            let contour_set = rcAllocContourSet();
            let max_error = navimesh_row.MaxEdgeError;
            let max_edge_len = (navimesh_row.MaxEdgeLength / cell_size) as i32; // in VX units
            let build_flags = rcBuildContoursFlags_RC_CONTOUR_TESS_WALL_EDGES as i32;
            assert!(rcBuildContours(
                context,
                compact_heightfield,
                max_error,
                max_edge_len,
                contour_set,
                build_flags
            ));
            if (*contour_set).nconts <= 0 {
                continue;
            }
            assert!((*contour_set).nconts > 0);

            // Step 5: Build the polymesh out of the contour set
            let poly_mesh = rcAllocPolyMesh();
            let nvp = navimesh_row.VertsPerPoly as i32;
            assert!(rcBuildPolyMesh(context, contour_set, nvp, poly_mesh));
            assert!(!(*poly_mesh).verts.is_null());
            assert!((*poly_mesh).nverts > 0);

            let flags =
                std::slice::from_raw_parts_mut((*poly_mesh).flags, (*poly_mesh).npolys as usize);
            for flag in flags {
                *flag = 1;
            }

            // Step 6: Build the polymesh detail
            let poly_mesh_detail = rcAllocPolyMeshDetail();
            let sample_dist = cell_size * navimesh_row.DetailMeshSampleDistance;
            let sample_max_error = cell_height * navimesh_row.DetailMeshMaxSampleError;
            assert!(rcBuildPolyMeshDetail(
                context,
                poly_mesh,
                compact_heightfield,
                sample_dist,
                sample_max_error,
                poly_mesh_detail
            ));

            let mut create_params = dtNavMeshCreateParams {
                // Polygon Mesh Attributes
                verts: (*poly_mesh).verts,
                vertCount: (*poly_mesh).nverts,
                polys: (*poly_mesh).polys,
                polyFlags: (*poly_mesh).flags,
                polyAreas: (*poly_mesh).areas,
                polyCount: (*poly_mesh).npolys,
                nvp: (*poly_mesh).nvp,

                // Height Detail Attributes
                detailMeshes: (*poly_mesh_detail).meshes,
                detailVerts: (*poly_mesh_detail).verts,
                detailVertsCount: (*poly_mesh_detail).nverts,
                detailTris: (*poly_mesh_detail).tris,
                detailTriCount: (*poly_mesh_detail).ntris,

                // Off-Mesh Connections Attributes
                offMeshConVerts: null(),
                offMeshConRad: null(),
                offMeshConFlags: null(),
                offMeshConAreas: null(),
                offMeshConDir: null(),
                offMeshConUserID: null(),
                offMeshConCount: 0,

                // Tile Attributes
                userId: 0,
                tileX: tile.x,
                tileY: tile.z,
                tileLayer: 0,
                bmin: (*poly_mesh).bmin,
                bmax: (*poly_mesh).bmax,

                // General Configuration Attributes
                walkableHeight: navimesh_row.AgentHeight,
                walkableRadius: navimesh_row.AgentRadius,
                walkableClimb: navimesh_row.AgentMaxClimb,
                cs: cell_size,
                ch: cell_height,
                buildBvTree: true,
            };

            let mut out_data: *mut u8 = null_mut();
            let mut out_data_size = 0;
            assert!(dtCreateNavMeshData(
                &mut create_params,
                &mut out_data,
                &mut out_data_size
            ));
            assert!(!out_data.is_null());
            assert!(out_data_size > 0);

            let out_data = core::slice::from_raw_parts(out_data, out_data_size as usize);
            let mut data = vec![0; out_data_size as usize];
            data.copy_from_slice(out_data);

            navmesh_tiles.push(NavmeshTile { data });
            max_polys = max_polys.max((*poly_mesh).npolys);
        }
    }

    let navmesh = Navmesh::new(
        zone_id,
        NavmeshParams {
            orig: [tile_origin_x, min_y, tile_origin_y],
            tile_width: tile_size as f32 * cell_size,
            tile_height: tile_size as f32 * cell_size,
            max_tiles: navmesh_tiles.len() as i32,
            max_polys,
        },
        navmesh_tiles,
    );

    let serialized_navmesh = navmesh.write_to_buffer().unwrap();

    let path = PathBuf::from(&destination_path);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap(); // create directory structure
    write_whole(&path, &serialized_navmesh);

    // The collision NPCs walk on (kawari_world::collision), written last: the world server waits for it.
    let collision = Collision::build(
        geometry.vertices,
        geometry.triangles,
        geometry.owners,
        geometry.materials,
        navimesh_row.AgentMaxClimb,
        navimesh_row.AgentRadius,
        navimesh_row.AgentMaxSlope,
    );
    write_whole(
        &kawari_world::collision::path_for(&path),
        &collision.to_bytes(),
    );
    tracing::info!("Wrote {destination_path} and its collision");
}

/// Writes `bytes` to `path` whole: to a temporary file first, then renamed, so the world server never reads half.
fn write_whole(path: &std::path::Path, bytes: &[u8]) {
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(".part");
    std::fs::write(&temporary, bytes).unwrap();
    std::fs::rename(&temporary, path).unwrap();
}

/// The zone's collision triangles, in world space.
#[derive(Default)]
struct Geometry {
    vertices: Vec<[f32; 3]>,
    triangles: Vec<[u32; 3]>,
    /// For each triangle, the layout ID of the event object whose gimmick it belongs to, or 0.
    owners: Vec<u32>,
    /// For each triangle, its PCB polygon's material (what the floor is: the slime in the Thousand Maws of Toto-Rak.)
    materials: Vec<u64>,
}

/// Represents the heightfield of a tile.
#[derive(Debug)]
struct Tile {
    x: i32,
    z: i32,
    height_field: *mut rcHeightfield,
}

/// Walk each node, add it's collision model to the scene (as `owner`'s gimmick, if it isn't 0.)
fn walk_node(node: &Node, transform: &Affine3A, owner: u32, geometry: &mut Geometry) {
    if !node.vertices.is_empty() {
        let first = geometry.vertices.len() as u32;
        for vertex in &node.vertices {
            let transformed = transform.transform_point3(Vec3A::from_slice(vertex).into());
            geometry
                .vertices
                .push([transformed.x, transformed.y, transformed.z]);
        }
        for polygon in &node.polygons {
            let indices = &polygon.vertex_indices;
            assert!(indices.len() == 3);
            geometry.triangles.push([
                first + indices[0] as u32,
                first + indices[1] as u32,
                first + indices[2] as u32,
            ]);
            geometry.owners.push(owner);
            geometry.materials.push(polygon.material);
        }
    }

    for child in &node.children {
        walk_node(child, transform, owner, geometry);
    }
}

fn add_plate(
    entry: &ListEntry,
    tera_path: &str,
    sqpack_resource: &mut ResourceResolver,
    geometry: &mut Geometry,
) {
    let pcb_path = format!("{}/collision/tr{:04}.pcb", tera_path, entry.mesh_id,);
    let Ok(file) = sqpack_resource.parsed::<Pcb>(&pcb_path) else {
        tracing::warn!("Failed to find {pcb_path}?! This may be a bug.");
        return;
    };
    let Pcb::Mesh(mdl) = file else {
        panic!("Mesh pcb isn't a mesh?!");
    };
    let transform = Affine3A::default();
    walk_node(&mdl, &transform, 0, geometry);
}

/// Walk each layer, add it's collision model to the scene. `bound` names the shared groups event objects control
/// (see `main`); `owner` is the gimmick a shared group's layers belong to (0 at the top.)
fn walk_layer(
    resolver: &mut ResourceResolver,
    layer: &Layer,
    transform: &Affine3A,
    bound: &HashMap<u32, u32>,
    owner: u32,
    geometry: &mut Geometry,
) {
    for object in &layer.objects {
        let child_transform: Affine3A = transform * Affine3A::from(object.transform);
        let owner = if owner != 0 {
            owner
        } else {
            bound.get(&object.instance_id).copied().unwrap_or(0)
        };

        match &object.data {
            LayerEntryData::BgPart(bg_part) => {
                if !bg_part.collision_asset_path.value.is_empty()
                    && bg_part.collision_type == ModelCollisionType::Replace
                {
                    let Pcb::Mesh(pcb) = resolver
                        .parsed::<Pcb>(&bg_part.collision_asset_path.value)
                        .unwrap()
                    else {
                        panic!("Mesh pcb isn't a mesh?!");
                    };

                    walk_node(&pcb, &child_transform, owner, geometry);
                }
            }
            LayerEntryData::CollisionBox(collision_box) => {
                if !collision_box.parent_data.enabled {
                    continue;
                }

                if !collision_box.collision_asset_path.value.is_empty() {
                    let Pcb::Mesh(pcb) = resolver
                        .parsed::<Pcb>(&collision_box.collision_asset_path.value)
                        .unwrap()
                    else {
                        panic!("Mesh pcb isn't a mesh?!");
                    };

                    walk_node(&pcb, &child_transform, owner, geometry);
                } else {
                    match collision_box.parent_data.trigger_box_shape {
                        TriggerBoxShape::None => unreachable!(),
                        TriggerBoxShape::Box => {
                            // TODO: It might be nice to have this as a helper somewhere?
                            let Pcb::Mesh(mesh) = Pcb::new_from_vertices(
                                &[
                                    [-1.0, -1.0, 1.0],
                                    [1.0, -1.0, 1.0],
                                    [-1.0, 1.0, 1.0],
                                    [1.0, 1.0, 1.0],
                                    [-1.0, -1.0, -1.0],
                                    [1.0, -1.0, -1.0],
                                    [-1.0, 1.0, -1.0],
                                    [1.0, 1.0, -1.0],
                                ],
                                &[
                                    Polygon {
                                        vertex_indices: [2, 6, 7],
                                        material: 0,
                                    },
                                    Polygon {
                                        vertex_indices: [2, 3, 7],
                                        material: 0,
                                    },
                                    Polygon {
                                        vertex_indices: [0, 4, 5],
                                        material: 0,
                                    },
                                    Polygon {
                                        vertex_indices: [0, 1, 5],
                                        material: 0,
                                    },
                                    Polygon {
                                        vertex_indices: [0, 2, 6],
                                        material: 0,
                                    },
                                    Polygon {
                                        vertex_indices: [0, 4, 6],
                                        material: 0,
                                    },
                                    Polygon {
                                        vertex_indices: [1, 3, 7],
                                        material: 0,
                                    },
                                    Polygon {
                                        vertex_indices: [1, 5, 7],
                                        material: 0,
                                    },
                                    Polygon {
                                        vertex_indices: [0, 2, 3],
                                        material: 0,
                                    },
                                    Polygon {
                                        vertex_indices: [0, 1, 3],
                                        material: 0,
                                    },
                                    Polygon {
                                        vertex_indices: [4, 6, 7],
                                        material: 0,
                                    },
                                    Polygon {
                                        vertex_indices: [4, 5, 7],
                                        material: 0,
                                    },
                                ],
                            ) else {
                                panic!("This can never happen!!");
                            };

                            walk_node(&mesh, &child_transform, owner, geometry);
                        }
                        TriggerBoxShape::Sphere => {
                            tracing::warn!("Sphere collision is not yet supported!")
                        }
                        TriggerBoxShape::Cylinder => {
                            tracing::warn!("Cylinder collision is not yet supported!")
                        }
                        TriggerBoxShape::Plane => {
                            tracing::warn!("Plane collision is not yet supported!")
                        }
                        TriggerBoxShape::Mesh => unreachable!(),
                        TriggerBoxShape::PlaneTwoSided => {
                            tracing::warn!("Plane two-sided collision is not yet supported!")
                        }
                    }
                }
            }
            LayerEntryData::SharedGroup(sgb) => {
                match resolver.parsed::<Sgb>(&sgb.asset_path.value) {
                    Ok(sgb) => {
                        for group in &sgb.sections[0].layer_groups {
                            for layer in &group.layers {
                                walk_layer(
                                    resolver,
                                    layer,
                                    &child_transform,
                                    bound,
                                    owner,
                                    geometry,
                                );
                            }
                        }
                    }
                    Err(err) => {
                        tracing::warn!("Failed to load sgb: {} {err:?}", sgb.asset_path.value);
                    }
                }
            }
            _ => {}
        }
    }
}
