//! The mod and resource-pack catalogue.
//!
//! Alpinoxi links to official releases instead of redistributing files: each
//! entry carries the project's own page, and the launcher shows the licence
//! note next to it. The search and filter logic lives here so the UI stays thin.

use alpinoxi_core::launcher::PERFORMANCE_MODS;

/// Where a file comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Mod,
    ResourcePack,
}

impl Kind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Mod => "Mod",
            Self::ResourcePack => "Resource pack",
        }
    }
}

/// One catalogue entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogEntry {
    pub id: &'static str,
    pub name: &'static str,
    pub summary: &'static str,
    pub kind: Kind,
    pub url: &'static str,
    /// Whether the entry is part of the curated performance set.
    pub performance: bool,
    /// Shown in the UI so nobody has to guess how a pack or mod is licensed.
    pub licence: &'static str,
}

const MODRINTH: &str = "https://modrinth.com/mod/";
const PACK: &str = "https://modrinth.com/resourcepack/";

/// Hand-picked entries. The performance set is generated from `alpinoxi_core`
/// so the launcher and the README can never disagree about it.
pub fn entries() -> Vec<CatalogEntry> {
    let mut out: Vec<CatalogEntry> = PERFORMANCE_MODS
        .iter()
        .map(|mod_info| CatalogEntry {
            id: mod_info.id,
            name: mod_info.name,
            summary: mod_info.what_it_fixes,
            kind: Kind::Mod,
            url: leak(format!("{MODRINTH}{}", mod_info.slug)),
            performance: true,
            licence: "Open source, official release",
        })
        .collect();

    out.extend([
        CatalogEntry {
            id: "fabric-api",
            name: "Fabric API",
            summary: "Required by almost every Fabric mod",
            kind: Kind::Mod,
            url: "https://modrinth.com/mod/fabric-api",
            performance: false,
            licence: "Open source, official release",
        },
        CatalogEntry {
            id: "modmenu",
            name: "Mod Menu",
            summary: "In-game mod list and per-mod settings",
            kind: Kind::Mod,
            url: "https://modrinth.com/mod/modmenu",
            performance: false,
            licence: "Open source, official release",
        },
        CatalogEntry {
            id: "iris",
            name: "Iris Shaders",
            summary: "Shader support on top of Sodium",
            kind: Kind::Mod,
            url: "https://modrinth.com/mod/iris",
            performance: false,
            licence: "Open source, official release",
        },
        CatalogEntry {
            id: "alpinoxi-pack",
            name: "Alpinoxi HUD pack",
            summary: "Alpinoxi fonts and HUD icons as a resource pack",
            kind: Kind::ResourcePack,
            url: "https://github.com/alpstudios346/alpinoxi-client",
            performance: false,
            licence: "MIT, by AlpinoxiGD",
        },
        CatalogEntry {
            id: "fresh-animations",
            name: "Fresh Animations",
            summary: "Smooth mob animations",
            kind: Kind::ResourcePack,
            url: leak(format!("{PACK}fresh-animations")),
            performance: false,
            licence: "Check the author's terms on the project page",
        },
    ]);
    out
}

/// `&'static str` for entries built at run time. The set is tiny and built once
/// per process, so leaking is the simple, honest choice here.
fn leak(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

/// Case-insensitive search over name and summary, optionally filtered by kind.
pub fn search<'a>(
    entries: &'a [CatalogEntry],
    query: &str,
    kind: Option<Kind>,
    performance_only: bool,
) -> Vec<&'a CatalogEntry> {
    let needle = query.trim().to_ascii_lowercase();
    entries
        .iter()
        .filter(|entry| kind.is_none_or(|wanted| entry.kind == wanted))
        .filter(|entry| !performance_only || entry.performance)
        .filter(|entry| {
            needle.is_empty()
                || entry.name.to_ascii_lowercase().contains(&needle)
                || entry.summary.to_ascii_lowercase().contains(&needle)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn performance_entries_are_all_present() {
        let entries = entries();
        for mod_info in PERFORMANCE_MODS {
            assert!(entries.iter().any(|entry| entry.id == mod_info.id));
        }
    }

    #[test]
    fn search_filters_by_kind_and_query() {
        let entries = entries();
        let packs = search(&entries, "", Some(Kind::ResourcePack), false);
        assert!(packs.iter().all(|entry| entry.kind == Kind::ResourcePack));

        let sodium = search(&entries, "SODIUM", None, true);
        assert_eq!(sodium.len(), 1);
        assert_eq!(sodium[0].name, "Sodium");

        assert!(search(&entries, "nothing-matches-this", None, false).is_empty());
    }
}
