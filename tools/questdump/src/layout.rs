//! A territory's layout as JSON (`kawari-questdump layout`): its LVB's layer groups (the LGBs), each layer's header
//! (whether it's active by default, its layer set filter, its festival) and its objects (instance ID, name, kind,
//! where, and everything physis reads of them), for looking at what the client places and how (ArchonEngine
//! ControlPanel's harness serves it). The bg, vfx and sound LGBs are left out, as the server leaves them.

use physis::Language;
use physis::layer::{LayerEntryData, LayerSetReferencedType};
use physis::lgb::Lgb;
use physis::lvb::Lvb;
use physis::resource::ResourceResolver;
use serde_json::{Value, json};

/// The most objects one call returns.
pub const MAX_OBJECTS: usize = 2000;

/// What to keep: objects by instance ID, layers by name (a part of it), objects by kind (`EventNPC`, a part of it.)
#[derive(Default)]
pub struct Filter {
    pub objects: Vec<u32>,
    pub layer: Option<String>,
    pub kind: Option<String>,
}

impl Filter {
    /// `--where` terms: `object=ID[,ID]`, `layer=NAME`, `kind=KIND`.
    pub fn add(&mut self, spec: &str) -> Result<(), String> {
        let (key, value) = spec
            .split_once('=')
            .ok_or_else(|| format!("--where {spec:?} isn't KEY=VALUE"))?;
        match key.trim() {
            "object" => {
                for id in value.split(',').map(str::trim).filter(|id| !id.is_empty()) {
                    self.objects.push(
                        id.parse()
                            .map_err(|_| format!("{id:?} isn't an instance ID"))?,
                    );
                }
            }
            "layer" => self.layer = Some(value.trim().to_lowercase()),
            "kind" => self.kind = Some(value.trim().to_lowercase()),
            other => return Err(format!("--where {other}: object, layer or kind")),
        }
        Ok(())
    }
}

/// The object's kind, as physis names it (`EventNPC`.)
pub fn kind(data: &LayerEntryData) -> String {
    let text = format!("{data:?}");
    text.split(['(', ' ', '{']).next().unwrap_or("").to_string()
}

pub fn dump(
    resolver: &mut ResourceResolver,
    territory: u32,
    filter: &Filter,
) -> Result<Value, String> {
    let sheet = icarus::TerritoryType::TerritoryTypeSheet::read_from(resolver, Language::None)
        .map_err(|err| format!("couldn't read the TerritoryType sheet: {err:?}"))?;
    let row = sheet
        .row(territory)
        .ok_or_else(|| format!("no TerritoryType {territory}"))?;
    let bg = row.Bg.to_string();
    if bg.is_empty() {
        return Err(format!("TerritoryType {territory} has no background"));
    }
    let lvb_path = format!("bg/{bg}.lvb");
    let lvb = resolver
        .parsed::<Lvb>(&lvb_path)
        .map_err(|err| format!("couldn't read {lvb_path}: {err}"))?;
    let section = lvb.sections.first().ok_or("the LVB has no section")?;
    let layer_sets: Vec<Value> = section
        .layer_sets
        .layer_sets
        .iter()
        .map(|set| json!({ "id": set.id, "territory": set.territory_type_id }))
        .collect();

    let mut groups = Vec::new();
    let mut count = 0;
    let mut more = false;
    for path in &section.lgb_paths {
        if path.ends_with("bg.lgb") || path.ends_with("vfx.lgb") || path.ends_with("sound.lgb") {
            continue;
        }
        let lgb = match resolver.parsed::<Lgb>(path) {
            Ok(lgb) => lgb,
            Err(err) => {
                groups.push(json!({ "path": path, "error": err.to_string() }));
                continue;
            }
        };
        let mut layers = Vec::new();
        for chunk in &lgb.chunks {
            for layer in &chunk.layers {
                let header = &layer.header;
                let name = header.name.value.clone();
                if filter
                    .layer
                    .as_ref()
                    .is_some_and(|wanted| !name.to_lowercase().contains(wanted))
                {
                    continue;
                }
                let mut objects = Vec::new();
                for object in &layer.objects {
                    let object_kind = kind(&object.data);
                    if !filter.objects.is_empty() && !filter.objects.contains(&object.instance_id) {
                        continue;
                    }
                    if filter
                        .kind
                        .as_ref()
                        .is_some_and(|wanted| !object_kind.to_lowercase().contains(wanted))
                    {
                        continue;
                    }
                    if count == MAX_OBJECTS {
                        more = true;
                        break;
                    }
                    count += 1;
                    objects.push(json!({
                        "instance_id": object.instance_id,
                        "name": object.name.value,
                        "kind": object_kind,
                        "position": object.transform.translation,
                        "rotation": object.transform.rotation,
                        "scale": object.transform.scale,
                        "data": format!("{:?}", object.data),
                    }));
                }
                // With an object or kind asked for, only the layers that have one.
                if objects.is_empty() && (!filter.objects.is_empty() || filter.kind.is_some()) {
                    continue;
                }
                let filter_list = &header.layer_set_referenced_list;
                layers.push(json!({
                    "layer_id": header.layer_id,
                    "name": name,
                    "active": header.active,
                    "unk1": header.unk1,
                    "unk2": header.unk2,
                    "unk3": header.unk3,
                    "unk4": header.unk4,
                    "unk5": header.unk5,
                    "unk6": header.unk6,
                    "festival": [header.festival_id, header.festival_phase_id],
                    "filter": {
                        "type": match filter_list.referenced_type {
                            LayerSetReferencedType::All => "All",
                            LayerSetReferencedType::Include => "Include",
                            LayerSetReferencedType::Exclude => "Exclude",
                            _ => "Other",
                        },
                        "layer_sets": filter_list.layer_set_ids,
                    },
                    "object_count": layer.objects.len(),
                    "objects": objects,
                }));
            }
        }
        groups.push(json!({ "path": path, "layers": layers }));
    }
    Ok(json!({
        "territory": territory,
        "bg": bg,
        "layer_sets": layer_sets,
        "groups": groups,
        "objects": count,
        "more": more,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn where_terms() {
        let mut filter = Filter::default();
        filter.add("object=9283423,9083081").unwrap();
        filter.add("layer=LVD_REMAKE_GIMMICK").unwrap();
        filter.add("kind=EventNPC").unwrap();
        assert_eq!(filter.objects, vec![9283423, 9083081]);
        assert_eq!(filter.layer.as_deref(), Some("lvd_remake_gimmick"));
        assert_eq!(filter.kind.as_deref(), Some("eventnpc"));
        assert!(filter.add("colour=red").is_err());
        assert!(filter.add("object=x").is_err());
    }

    #[test]
    fn kinds_are_physis_names() {
        assert_eq!(kind(&LayerEntryData::Unknown), "Unknown");
    }
}
