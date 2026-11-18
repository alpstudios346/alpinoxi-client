use oneclient_content::packages::ProviderId;

use crate::i18n::t;

#[derive(Debug, Clone, Copy)]
pub struct EssentialPackage {
    pub provider: ProviderId,
    pub project_id: &'static str,
    pub name: &'static str,
    pub disable_body: fn() -> &'static str,
    pub remove_body: fn() -> &'static str,
}

impl PartialEq for EssentialPackage {
    fn eq(&self, other: &Self) -> bool {
        self.provider == other.provider && self.project_id == other.project_id
    }
}

impl Eq for EssentialPackage {}

pub const POLYPLUS: EssentialPackage = EssentialPackage {
    provider: ProviderId::Modrinth,
    project_id: "Iw9mZi4a",
    name: "Poly+",
    disable_body: || t("oneclient.essential.polyplus.disable_body"),
    remove_body: || t("oneclient.essential.polyplus.remove_body"),
};

pub const ESSENTIAL: EssentialPackage = EssentialPackage {
    provider: ProviderId::Modrinth,
    project_id: "k2ZPuTBm",
    name: "Essential",
    disable_body: || t("oneclient.essential.essential.disable_body"),
    remove_body: || t("oneclient.essential.essential.remove_body"),
};

pub const CUSTOM_SKIN_LOADER: EssentialPackage = EssentialPackage {
    provider: ProviderId::Modrinth,
    project_id: "idMHQ4n2",
    name: "CustomSkinLoader",
    disable_body: || t("oneclient.essential.customskinloader.disable_body"),
    remove_body: || t("oneclient.essential.customskinloader.remove_body"),
};

pub const ESSENTIAL_PACKAGES: &[EssentialPackage] = &[ESSENTIAL, CUSTOM_SKIN_LOADER];

pub fn lookup(provider: ProviderId, package_id: &str) -> Option<&'static EssentialPackage> {
    ESSENTIAL_PACKAGES
        .iter()
        .find(|package| package.provider == provider && package.project_id == package_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_mods_are_essential_on_modrinth_only() {
        assert_eq!(
            lookup(ProviderId::Modrinth, "k2ZPuTBm").map(|p| p.name),
            Some("Essential")
        );
        assert_eq!(
            lookup(ProviderId::Modrinth, "idMHQ4n2").map(|p| p.name),
            Some("CustomSkinLoader")
        );
        assert!(lookup(ProviderId::CurseForge, "k2ZPuTBm").is_none());
        assert!(lookup(ProviderId::Local, "k2ZPuTBm").is_none());
        assert!(lookup(ProviderId::Modrinth, "Iw9mZi4a").is_none());
    }

    #[test]
    fn ordinary_packages_are_not_guarded() {
        assert!(lookup(ProviderId::Modrinth, "AibBIVmj").is_none());
    }
}
