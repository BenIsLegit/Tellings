//! Setups a person can start from.
//!
//! - **Presets** ship with the tool. They are the files in `presets/`, built in so
//!   they are always there, and they cannot be changed from inside the tool.
//! - **Saved setups** are a person's own. They live in the app's folder as plain
//!   JSON files, so they can be copied, shared and edited by hand.
//!
//! Either kind can start from another with `"extends"`: "the Joyful setup, but
//! with this voice".

use crate::error::{Error, Result};
use crate::paths::{slug, Paths};
use crate::setup::{from_doc, merge_docs, Setup};
use serde::Serialize;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

/// The presets, in the order they are shown.
const PRESETS: &[(&str, &str)] = &[
    ("today", include_str!("../../../presets/today.json")),
    ("luminous", include_str!("../../../presets/luminous.json")),
    ("joyful", include_str!("../../../presets/joyful.json")),
    ("sorrowful", include_str!("../../../presets/sorrowful.json")),
    ("glorious", include_str!("../../../presets/glorious.json")),
    ("st-benedict", include_str!("../../../presets/st-benedict.json")),
    ("follow-along", include_str!("../../../presets/follow-along.json")),
    ("voice-drops-out", include_str!("../../../presets/voice-drops-out.json")),
    ("words-only", include_str!("../../../presets/words-only.json")),
    ("one-decade", include_str!("../../../presets/one-decade.json")),
    ("phone", include_str!("../../../presets/phone.json")),
    ("tv", include_str!("../../../presets/tv.json")),
    ("light", include_str!("../../../presets/light.json")),
    ("wooden-beads", include_str!("../../../presets/wooden-beads.json")),
    ("walk", include_str!("../../../presets/walk.json")),
    ("guide", include_str!("../../../presets/guide.json")),
];

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    Preset,
    Saved,
    File,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    /// What to type to use it: `luminous`, `my-joyful`.
    pub id: String,
    pub name: String,
    pub description: String,
    pub kind: Kind,
    pub extends: Option<String>,
}

/// A setup as it was found, before its `extends` chain is folded in.
#[derive(Clone, Debug)]
pub struct Found {
    pub entry: Entry,
    pub doc: Value,
    /// The folder the file sits in, for pictures named beside it.
    pub dir: Option<PathBuf>,
}

fn entry(id: &str, kind: Kind, doc: &Value) -> Entry {
    Entry {
        id: id.to_string(),
        name: doc["name"].as_str().filter(|s| !s.trim().is_empty()).unwrap_or(id).to_string(),
        description: doc["description"].as_str().unwrap_or("").to_string(),
        kind,
        extends: doc["extends"].as_str().map(String::from),
    }
}

fn parse(text: &str, what: &str) -> Result<Value> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let v: Value = serde_json::from_str(text).map_err(|e| Error::setup(format!("{what} is not valid JSON: {e}")))?;
    if !v.is_object() {
        return Err(Error::setup(format!("{what} has to be a JSON object: {{ ... }}.")));
    }
    Ok(v)
}

pub fn presets() -> Vec<Entry> {
    PRESETS.iter().map(|(id, text)| entry(id, Kind::Preset, &parse(text, id).expect("built-in preset"))).collect()
}

pub fn saved(paths: &Paths) -> Vec<Entry> {
    let mut out = Vec::new();
    let Ok(dir) = std::fs::read_dir(paths.setups()) else { return out };
    for item in dir.flatten() {
        let path = item.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Some(id) = path.file_stem().and_then(|s| s.to_str()) else { continue };
        if let Ok(doc) = std::fs::read_to_string(&path).map_err(Error::from).and_then(|t| parse(&t, id)) {
            out.push(entry(id, Kind::Saved, &doc));
        }
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}

/// Find a setup by what a person typed: a preset, a saved setup, or a file.
/// A saved setup wins over a preset of the same name, so a person can make
/// "luminous" their own.
pub fn find(paths: &Paths, reference: &str) -> Result<Found> {
    let reference = reference.trim();
    let as_path = Path::new(reference);
    if reference.ends_with(".json") || as_path.is_file() {
        let text = std::fs::read_to_string(as_path).map_err(|e| Error::setup(format!("Could not open the setup file {reference}: {e}")))?;
        let doc = parse(&text, reference)?;
        let id = as_path.file_stem().and_then(|s| s.to_str()).unwrap_or("setup");
        let dir = as_path.parent().filter(|p| !p.as_os_str().is_empty()).map(Path::to_path_buf).or_else(|| std::env::current_dir().ok());
        return Ok(Found { entry: entry(id, Kind::File, &doc), doc, dir });
    }
    let id = slug(reference);
    let file = paths.setups().join(format!("{id}.json"));
    if file.is_file() {
        let doc = parse(&std::fs::read_to_string(&file)?, &id)?;
        return Ok(Found { entry: entry(&id, Kind::Saved, &doc), doc, dir: Some(paths.setups()) });
    }
    if let Some((pid, text)) = PRESETS.iter().find(|(pid, _)| *pid == id) {
        let doc = parse(text, pid)?;
        return Ok(Found { entry: entry(pid, Kind::Preset, &doc), doc, dir: None });
    }
    // Last try: match on the name shown in the list, so "Joyful Mysteries" works too.
    let wanted = reference.to_lowercase();
    if let Some(e) = saved(paths).into_iter().chain(presets()).find(|e| e.name.to_lowercase() == wanted) {
        return find(paths, &e.id);
    }
    let names: Vec<String> = presets().into_iter().chain(saved(paths)).map(|e| e.id).collect();
    Err(Error::setup(format!("There is no setup called \"{reference}\". The setups are: {}.", names.join(", "))))
}

/// Fold in whatever a setup extends, oldest first, so the setup itself has the last word.
pub fn resolve(paths: &Paths, doc: &Value) -> Result<Value> {
    let mut chain: Vec<Value> = vec![doc.clone()];
    let mut seen: Vec<String> = Vec::new();
    let mut next = doc["extends"].as_str().map(String::from);
    while let Some(parent) = next {
        if seen.contains(&parent) || seen.len() > 8 {
            return Err(Error::setup(format!("Setups extend each other in a circle: {} -> {parent}.", seen.join(" -> "))));
        }
        let found = find(paths, &parent)?;
        next = found.doc["extends"].as_str().map(String::from);
        chain.push(found.doc);
        seen.push(parent);
    }
    let mut out = Value::Object(Map::new());
    for layer in chain.iter().rev() {
        out = merge_docs(&out, layer);
    }
    if let Some(o) = out.as_object_mut() {
        o.remove("extends");
    }
    Ok(out)
}

/// A setup found by name and filled in, ready to plan.
#[derive(Debug)]
pub struct Loaded {
    pub entry: Entry,
    /// The sparse setup with its `extends` chain folded in.
    pub doc: Value,
    pub setup: Setup,
    pub dir: Option<PathBuf>,
}

pub fn load(paths: &Paths, reference: &str) -> Result<Loaded> {
    let found = find(paths, reference)?;
    let doc = resolve(paths, &found.doc)?;
    let setup = from_doc(&doc)?;
    Ok(Loaded { entry: found.entry, doc, setup, dir: found.dir })
}

/// Save a setup under a name. Returns its id. The name in the file is set to match.
pub fn save(paths: &Paths, name: &str, doc: &Value) -> Result<Entry> {
    let name = name.trim();
    if name.is_empty() {
        return Err(Error::setup("A saved setup needs a name."));
    }
    // Make sure it is a real setup before it lands on disk.
    from_doc(&resolve(paths, doc)?)?;
    let mut doc = doc.clone();
    doc.as_object_mut().expect("checked by from_doc").insert("name".into(), Value::String(name.to_string()));
    // Keep the name at the top of the file, where a person looks first.
    let mut ordered = Map::new();
    ordered.insert("name".into(), doc["name"].clone());
    for (k, v) in doc.as_object().unwrap() {
        if k != "name" {
            ordered.insert(k.clone(), v.clone());
        }
    }
    let id = slug(name);
    std::fs::create_dir_all(paths.setups())?;
    std::fs::write(paths.setups().join(format!("{id}.json")), serde_json::to_string_pretty(&Value::Object(ordered.clone()))? + "\n")?;
    Ok(entry(&id, Kind::Saved, &Value::Object(ordered)))
}

pub fn delete(paths: &Paths, id: &str) -> Result<()> {
    let file = paths.setups().join(format!("{}.json", slug(id)));
    if !file.is_file() {
        return Err(Error::setup(format!("There is no saved setup called \"{id}\". Presets cannot be deleted.")));
    }
    std::fs::remove_file(file)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn temp() -> Paths {
        let dir = std::env::temp_dir().join(format!("pvc-lib-{}-{:?}", std::process::id(), std::thread::current().id()));
        let _ = std::fs::remove_dir_all(&dir);
        Paths::new(dir)
    }

    #[test]
    fn every_preset_is_a_working_setup() {
        let paths = temp();
        assert_eq!(presets().len(), 16);
        for p in presets() {
            let loaded = load(&paths, &p.id).unwrap_or_else(|e| panic!("{}: {e}", p.id));
            let ctx = crate::plan::Context { today: chrono::NaiveDate::from_ymd_opt(2026, 11, 19).unwrap(), search: vec![] };
            let plan = crate::plan::build(&loaded.setup, &ctx).unwrap_or_else(|e| panic!("{}: {e}", p.id));
            assert!(plan.prayer_count >= 12, "{}", p.id);
            assert!(!p.description.is_empty(), "{} needs a description", p.id);
        }
    }

    #[test]
    fn the_luminous_preset_is_the_first_video() {
        let s = load(&temp(), "luminous").unwrap().setup;
        assert_eq!(s.voice.name, "Imperial - Male");
        assert_eq!(s.screen.size().unwrap(), (2076, 2152));
        assert_eq!(s.rules.len(), 5);
        assert_eq!(s.rules[3].set["picture"]["zoom"], 1.2);
        assert_eq!(s.rules[1].set["picture"]["credit"], "Giotto, Marriage at Cana (detail), Scrovegni Chapel, c. 1305");
    }

    #[test]
    fn save_then_start_from_it_and_change_one_thing() {
        let paths = temp();
        let saved_entry = save(&paths, "My Joyful", &json!({ "extends": "joyful", "voice": { "name": "Bella" } })).unwrap();
        assert_eq!(saved_entry.id, "my-joyful");
        assert_eq!(saved(&paths).len(), 1);

        let child = json!({ "extends": "my-joyful", "screen": { "preset": "tv-1080p" } });
        let setup = from_doc(&resolve(&paths, &child).unwrap()).unwrap();
        assert_eq!(setup.voice.name, "Bella", "from the saved setup");
        assert_eq!(setup.prayers.mysteries, crate::setup::Mysteries::Joyful, "from the preset under it");
        assert_eq!(setup.screen.size().unwrap(), (1920, 1080), "the one change");

        // Found by the name on the card, too.
        assert_eq!(find(&paths, "My Joyful").unwrap().entry.id, "my-joyful");
        delete(&paths, "my-joyful").unwrap();
        assert!(find(&paths, "my-joyful").is_err());
        let _ = std::fs::remove_dir_all(&paths.data);
    }

    #[test]
    fn a_circle_of_setups_is_caught() {
        let paths = temp();
        std::fs::create_dir_all(paths.setups()).unwrap();
        std::fs::write(paths.setups().join("a.json"), r#"{ "extends": "b" }"#).unwrap();
        std::fs::write(paths.setups().join("b.json"), r#"{ "extends": "a" }"#).unwrap();
        assert!(load(&paths, "a").unwrap_err().to_string().contains("circle"));
        let _ = std::fs::remove_dir_all(&paths.data);
    }

    #[test]
    fn an_unknown_name_lists_the_choices() {
        let err = load(&temp(), "nope").unwrap_err().to_string();
        assert!(err.contains("luminous") && err.contains("joyful"), "{err}");
    }
}
