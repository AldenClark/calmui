use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, OnceLock};

use rust_embed::RustEmbed;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IconName {
    value: String,
}

impl IconName {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IconSource {
    Named(IconName),
}

impl IconSource {
    pub fn named(value: impl Into<String>) -> Self {
        Self::Named(IconName::new(value))
    }
}

#[derive(Clone, Debug, Default)]
struct PackIndex {
    names: BTreeMap<String, PathBuf>,
}

impl PackIndex {
    fn set_name_path(mut self, name: String, path: PathBuf) -> Self {
        self.names.entry(name).or_insert(path);
        self
    }

    fn add_alias(mut self, alias: String, path: PathBuf) -> Self {
        self.names.entry(alias).or_insert(path);
        self
    }

    fn resolve(&self, name: &str) -> Option<PathBuf> {
        self.names.get(name).cloned()
    }

    fn len(&self) -> usize {
        self.names.len()
    }
}

#[derive(Clone, Debug)]
struct RegistryInner {
    default_pack: String,
    packs: BTreeMap<String, PackIndex>,
}

#[derive(Clone, Debug)]
pub struct IconRegistry {
    inner: Arc<RegistryInner>,
}

impl Default for IconRegistry {
    fn default() -> Self {
        static DEFAULT_REGISTRY: OnceLock<IconRegistry> = OnceLock::new();
        DEFAULT_REGISTRY.get_or_init(Self::build_default).clone()
    }
}

impl IconRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    fn build_default() -> Self {
        let mut packs = BTreeMap::new();
        if let Some(tabler_root) = default_tabler_pack_root() {
            if let Ok(pack) = load_pack_from_root(&tabler_root) {
                packs.insert("tabler".to_string(), pack);
            }
        }

        Self {
            inner: Arc::new(RegistryInner {
                default_pack: "tabler".to_string(),
                packs,
            }),
        }
    }

    pub fn with_default_pack(mut self, pack: impl Into<String>) -> Self {
        let mut next = (*self.inner).clone();
        next.default_pack = pack.into();
        self.inner = Arc::new(next);
        self
    }

    pub fn register_embedded_pack<T: RustEmbed>(mut self, name: impl Into<String>) -> Self {
        let mut next = (*self.inner).clone();
        let pack_name = name.into();
        let extract_key = format!("custom-{pack_name}");
        if let Some(root) = extract_embedded_pack::<T>(&extract_key)
            && let Ok(pack) = load_pack_from_root(&root)
        {
            next.packs.insert(pack_name, pack);
        }
        self.inner = Arc::new(next);
        self
    }

    pub fn resolve(&self, source: &IconSource) -> Option<PathBuf> {
        match source {
            IconSource::Named(name) => self.resolve_named(name),
        }
    }

    pub fn resolve_named(&self, name: &IconName) -> Option<PathBuf> {
        let (pack_name, icon_name) = split_namespace(name.as_str(), &self.inner.default_pack);
        let pack = self.inner.packs.get(pack_name)?;

        pack.resolve(icon_name)
    }

    pub fn count(&self, pack: &str) -> usize {
        self.inner
            .packs
            .get(pack)
            .map(PackIndex::len)
            .unwrap_or_default()
    }

    pub fn packs(&self) -> Vec<String> {
        self.inner.packs.keys().cloned().collect()
    }
}

fn split_namespace<'a>(value: &'a str, default_pack: &'a str) -> (&'a str, &'a str) {
    if let Some((pack, icon)) = value.split_once(':') {
        if !pack.is_empty() && !icon.is_empty() {
            return (pack, icon);
        }
    }
    (default_pack, value)
}

fn load_pack_from_root(root: &Path) -> Result<PackIndex, std::io::Error> {
    let mut pack = PackIndex::default();
    let outline_root = root.join("outline");
    let filled_root = root.join("filled");

    if outline_root.exists() {
        for icon_name in read_icon_names(&outline_root)? {
            let path = outline_root.join(format!("{icon_name}.svg"));
            pack = pack.set_name_path(icon_name.clone(), path.clone());
            if !icon_name.ends_with("-outline") {
                pack = pack.add_alias(format!("{icon_name}-outline"), path);
            }
        }
    }

    if filled_root.exists() {
        for icon_name in read_icon_names(&filled_root)? {
            let path = filled_root.join(format!("{icon_name}.svg"));
            pack = pack.set_name_path(icon_name.clone(), path.clone());
            if !icon_name.ends_with("-filled") {
                pack = pack.add_alias(format!("{icon_name}-filled"), path);
            }
        }
    }
    Ok(pack)
}

fn read_icon_names(root: &Path) -> Result<Vec<String>, std::io::Error> {
    let mut names = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let is_svg = path
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.eq_ignore_ascii_case("svg"))
            .unwrap_or(false);
        if !is_svg {
            continue;
        }

        if let Some(stem) = path.file_stem().and_then(|value| value.to_str()) {
            names.push(stem.to_string());
        }
    }
    Ok(names)
}

fn default_tabler_pack_root() -> Option<PathBuf> {
    #[cfg(feature = "extend-icon")]
    {
        return extract_embedded_pack::<EmbeddedTablerExtended>("tabler-extended");
    }
    #[cfg(not(feature = "extend-icon"))]
    {
        extract_embedded_pack::<EmbeddedTablerBase>("tabler-base")
    }
}

fn extract_embedded_pack<T: RustEmbed>(folder_name: &str) -> Option<PathBuf> {
    let root = std::env::temp_dir()
        .join("calmui-icons")
        .join(env!("CARGO_PKG_VERSION"))
        .join(folder_name);
    let marker = root.join(".extract-ready");

    if marker.exists() && embedded_pack_is_complete::<T>(&root) {
        return Some(root);
    }

    let _ = fs::remove_dir_all(&root);
    if fs::create_dir_all(&root).is_err() {
        return None;
    }

    for relative in T::iter() {
        let relative = relative.as_ref();
        let Some(safe_relative) = sanitize_relative_path(relative) else {
            continue;
        };
        let Some(content) = T::get(relative) else {
            continue;
        };

        let destination = root.join(safe_relative);
        if let Some(parent) = destination.parent() {
            if fs::create_dir_all(parent).is_err() {
                return None;
            }
        }
        if fs::write(destination, content.data.as_ref()).is_err() {
            return None;
        }
    }

    if fs::write(marker, b"ok").is_err() {
        return None;
    }
    Some(root)
}

fn embedded_pack_is_complete<T: RustEmbed>(root: &Path) -> bool {
    T::iter().all(|relative| {
        let relative = relative.as_ref();
        let Some(safe_relative) = sanitize_relative_path(relative) else {
            return false;
        };
        root.join(safe_relative).is_file()
    })
}

fn sanitize_relative_path(input: &str) -> Option<PathBuf> {
    let mut output = PathBuf::new();
    for component in Path::new(input).components() {
        match component {
            Component::Normal(value) => output.push(value),
            _ => return None,
        }
    }
    Some(output)
}

#[derive(RustEmbed)]
#[folder = "assets/icons/tabler-base"]
#[allow(dead_code)]
struct EmbeddedTablerBase;

#[cfg(feature = "extend-icon")]
#[derive(RustEmbed)]
#[folder = "assets/icons/tabler"]
struct EmbeddedTablerExtended;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_contains_embedded_icons() {
        let registry = IconRegistry::new();
        assert!(registry.count("tabler") >= 5);
    }

    #[test]
    fn can_resolve_basic_icon() {
        let registry = IconRegistry::new();
        let icon = IconName::new("info-circle");
        let path = registry
            .resolve_named(&icon)
            .expect("info-circle should be resolvable");
        assert!(path.to_string_lossy().contains("info-circle.svg"));
    }

    #[test]
    fn can_resolve_triangle_icons() {
        let registry = IconRegistry::new();
        let up = IconName::new("triangle-up-filled");
        let down = IconName::new("triangle-down-filled");
        assert!(registry.resolve_named(&up).is_some());
        assert!(registry.resolve_named(&down).is_some());
    }

    #[test]
    fn can_resolve_filled_alias_for_shared_name() {
        let registry = IconRegistry::new();
        let icon = IconName::new("star-filled");
        assert!(registry.resolve_named(&icon).is_some());
    }

    #[cfg(feature = "extend-icon")]
    #[test]
    fn extended_pack_contains_full_tabler_counts() {
        let registry = IconRegistry::new();
        assert!(registry.count("tabler") >= 5_500);
    }
}
