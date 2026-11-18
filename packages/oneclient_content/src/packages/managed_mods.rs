use std::collections::HashSet;

use oneclient_common::domain::{ContentType, GameLoader, ProviderId};

use crate::ctx::ContentCtx;
use crate::error::{ContentError, ContentResult};
use crate::packages::local_manifest::{read_jar_dependencies, read_jar_mod_id};
use crate::packages::{
    DisableRoot, PackageProvider, PackageStore, disable_impact, ensure_cluster_dependencies,
    pick_version, resolve_required,
};

const ESSENTIAL: ManagedMod = ManagedMod {
    project_id: "k2ZPuTBm",
    name: "Essential Mod",
    account_support: AccountSupport::Microsoft,
};
const CUSTOM_SKIN_LOADER: ManagedMod = ManagedMod {
    project_id: "idMHQ4n2",
    name: "CustomSkinLoader",
    account_support: AccountSupport::Any,
};
const QUICK_SKIN: ManagedMod = ManagedMod {
    project_id: "zAIE84Ch",
    name: "Quick Skin",
    account_support: AccountSupport::Alternate,
};
const WORLD_HOST: ManagedMod = ManagedMod {
    project_id: "2AIZDfYo",
    name: "World Host",
    account_support: AccountSupport::Alternate,
};
const IN_GAME_ACCOUNT_SWITCHER: ManagedMod = ManagedMod {
    project_id: "cudtvDnd",
    name: "In-Game Account Switcher",
    account_support: AccountSupport::Alternate,
};
const E4ALL: ManagedMod = ManagedMod {
    project_id: "fB1gJLGX",
    name: "e4all",
    account_support: AccountSupport::Alternate,
};

const MANAGED_MODS: &[ManagedMod] = &[
    ESSENTIAL,
    CUSTOM_SKIN_LOADER,
    QUICK_SKIN,
    WORLD_HOST,
    IN_GAME_ACCOUNT_SWITCHER,
    E4ALL,
];

#[derive(Clone, Copy)]
struct ManagedMod {
    project_id: &'static str,
    name: &'static str,
    account_support: AccountSupport,
}

#[derive(Clone, Copy)]
enum AccountSupport {
    Any,
    Microsoft,
    Alternate,
}

/// Removes OneConfig and Poly+ content, then installs Alpinoxi's default mods.
/// Modpack-linked instances are left unchanged.
pub async fn apply_alpinoxi_mod_policy(
    cluster_id: i64,
    alternate_auth: bool,
    offline: bool,
    ctx: &ContentCtx,
) -> ContentResult<()> {
    let cluster = PackageStore::get_cluster(cluster_id, ctx).await?;
    if cluster.linked_modpack_hash.is_some() {
        return Ok(());
    }

    remove_oneconfig_and_polyplus(cluster_id, offline, ctx).await?;

    let loader = GameLoader::from_repr(cluster.mc_loader as u8).unwrap_or(GameLoader::Vanilla);
    if !loader.is_modded() {
        return Ok(());
    }

    if offline {
        tracing::warn!(
            cluster_id,
            "offline launch cannot download missing Alpinoxi-managed mods"
        );
        return Ok(());
    }

    let provider = ctx.providers.get(ProviderId::Modrinth)?;
    for managed in MANAGED_MODS
        .iter()
        .filter(|managed| match managed.account_support {
            AccountSupport::Any => true,
            AccountSupport::Microsoft => !alternate_auth,
            AccountSupport::Alternate => alternate_auth,
        })
    {
        ensure_managed_mod(provider, *managed, cluster_id, ctx).await?;
    }

    Ok(())
}

async fn ensure_managed_mod(
    provider: &dyn PackageProvider,
    managed: ManagedMod,
    cluster_id: i64,
    ctx: &ContentCtx,
) -> ContentResult<()> {
    let existing = PackageStore::list_linked_artifacts(cluster_id, ctx).await?;
    if existing.iter().any(|artifact| {
        artifact.content_type == ContentType::Mod
            && artifact.provider == Some(ProviderId::Modrinth)
            && artifact.project_id.as_deref() == Some(managed.project_id)
    }) {
        return Ok(());
    }

    let cluster = PackageStore::get_cluster(cluster_id, ctx).await?;
    let Some(version) = pick_version(provider, managed.project_id, &cluster, ctx).await? else {
        tracing::info!(
            cluster_id,
            mod_name = managed.name,
            mc_version = %cluster.mc_version,
            loader = ?GameLoader::from_repr(cluster.mc_loader as u8),
            "no compatible release of a managed mod is available"
        );
        return Ok(());
    };

    let project = provider.get_project(managed.project_id, ctx).await?;
    if project.content_type != ContentType::Mod {
        return Err(ContentError::InvalidData {
            reason: format!("{} is not a mod project", managed.name),
        });
    }

    let version = provider
        .get_version(&project.id, &version.version_id, ctx)
        .await?;
    let dependencies = resolve_required(ProviderId::Modrinth, &version, cluster_id, ctx).await?;
    if !dependencies.unresolved.is_empty() {
        return Err(ContentError::InvalidData {
            reason: format!(
                "Could not resolve required dependencies for {}: {}",
                managed.name,
                dependencies.unresolved.join(", ")
            ),
        });
    }

    for dependency in dependencies.install.into_iter().rev() {
        PackageStore::install_to_cluster(
            ProviderId::Modrinth,
            &dependency.project,
            &dependency.version,
            cluster_id,
            false,
            false,
            None,
            ctx,
        )
        .await?;
    }

    PackageStore::install_to_cluster(
        ProviderId::Modrinth,
        &project,
        &version,
        cluster_id,
        false,
        false,
        None,
        ctx,
    )
    .await?;

    tracing::info!(
        cluster_id,
        mod_name = managed.name,
        "installed Alpinoxi-managed mod"
    );
    Ok(())
}

async fn remove_oneconfig_and_polyplus(
    cluster_id: i64,
    offline: bool,
    ctx: &ContentCtx,
) -> ContentResult<()> {
    if !offline {
        ensure_cluster_dependencies(cluster_id, ctx).await?;
    }

    let linked = PackageStore::list_linked_artifacts(cluster_id, ctx).await?;
    let mut roots = Vec::new();
    let mut remove_hashes = HashSet::new();

    for artifact in linked
        .iter()
        .filter(|artifact| artifact.content_type == ContentType::Mod)
    {
        let Some(cached) = PackageStore::cached_artifact(&artifact.hash, ctx).await? else {
            continue;
        };
        let mod_id = read_jar_mod_id(&cached.path).await.map(|mod_id| mod_id.id);
        let dependencies = read_jar_dependencies(&cached.path).await;

        if is_oneconfig_or_polyplus(
            artifact.provider,
            artifact.project_id.as_deref(),
            Some(&artifact.file_name),
            artifact.display_name.as_deref(),
            mod_id.as_deref(),
        ) {
            roots.push(DisableRoot::Artifact(artifact.hash.clone()));
            remove_hashes.insert(artifact.hash.clone());
        }

        if dependencies
            .requires
            .iter()
            .any(|id| is_oneconfig_or_polyplus_mod_id(id))
        {
            roots.push(DisableRoot::Artifact(artifact.hash.clone()));
            remove_hashes.insert(artifact.hash.clone());
        }
    }

    for root in [
        DisableRoot::Package(ProviderId::Modrinth, "AibBIVmj".into()),
        DisableRoot::Package(ProviderId::CurseForge, "1148006".into()),
        DisableRoot::Package(ProviderId::Modrinth, "Iw9mZi4a".into()),
    ] {
        roots.push(root);
    }

    for root in &roots {
        let impact = disable_impact(cluster_id, std::slice::from_ref(root), ctx).await?;
        remove_hashes.extend(impact.required_hashes());
    }

    let mut remove_hashes: Vec<_> = remove_hashes.into_iter().collect();
    remove_hashes.sort();
    for hash in remove_hashes {
        crate::bundles::remove_artifact_from_cluster(cluster_id, &hash, true, ctx).await?;
    }

    Ok(())
}

fn is_oneconfig_or_polyplus(
    provider: Option<ProviderId>,
    project_id: Option<&str>,
    file_name: Option<&str>,
    display_name: Option<&str>,
    mod_id: Option<&str>,
) -> bool {
    (provider == Some(ProviderId::Modrinth) && matches!(project_id, Some("AibBIVmj" | "Iw9mZi4a")))
        || (provider == Some(ProviderId::CurseForge) && project_id == Some("1148006"))
        || file_name.is_some_and(is_oneconfig_or_polyplus_mod_id)
        || display_name.is_some_and(is_oneconfig_or_polyplus_mod_id)
        || mod_id.is_some_and(is_oneconfig_or_polyplus_mod_id)
}

fn is_oneconfig_or_polyplus_mod_id(value: &str) -> bool {
    let value = value.trim().to_ascii_lowercase();
    if value == "poly+" || value.starts_with("poly+.") {
        return true;
    }

    let normalized: String = value
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|character| character.to_ascii_lowercase())
        .collect();
    matches!(normalized.as_str(), "oneconfig" | "polyplus")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifies_oneconfig_and_polyplus_artifacts_by_provider_and_metadata() {
        assert!(is_oneconfig_or_polyplus(
            Some(ProviderId::Modrinth),
            Some("AibBIVmj"),
            None,
            None,
            None
        ));
        assert!(is_oneconfig_or_polyplus(
            Some(ProviderId::CurseForge),
            Some("1148006"),
            None,
            None,
            None
        ));
        assert!(is_oneconfig_or_polyplus(
            None,
            None,
            Some("Poly+.jar"),
            None,
            None
        ));
        assert!(!is_oneconfig_or_polyplus(
            Some(ProviderId::Modrinth),
            Some("not-oneconfig"),
            Some("Polymer.jar"),
            Some("Polymer"),
            Some("polymer")
        ));
    }

    #[test]
    fn recognizes_required_local_mod_ids_case_insensitively() {
        assert!(is_oneconfig_or_polyplus_mod_id("OneConfig"));
        assert!(is_oneconfig_or_polyplus_mod_id("polyplus"));
        assert!(!is_oneconfig_or_polyplus_mod_id("polymer"));
    }
}
