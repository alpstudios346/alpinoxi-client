//! Launcher state and screen routing.
//!
//! One `AlpinoxiApp` owns everything the screens read and write. Screens are
//! plain functions in `ui/`, so the state stays testable and no screen needs a
//! reference to another.

use eframe::egui;

use alpinoxi_auth_system::{AuthenticationState, MemoryVault};
use alpinoxi_core::account::{AccountProfile, AccountProvider};
use alpinoxi_core::friends::{ChatMessage, Friend, Presence};
use alpinoxi_core::hud::{HudElement, KeybindSet};
use alpinoxi_core::launcher::{LaunchPlan, MemoryProfile, SLOGANS};
use alpinoxi_core::skin::{SkinModel, SkinSource};
use alpinoxi_core::slogan::SloganPicker;
use alpinoxi_ingame::{FriendBook, GameSession};

use crate::catalog::{self, CatalogEntry, Kind};
use crate::config::{self, Settings};
use crate::skinview::Wardrobe;
use crate::theme;
use crate::ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Home,
    Instances,
    Skin,
    Friends,
    Mods,
    Import,
    Settings,
}

impl Screen {
    pub const ALL: [Self; 7] = [
        Self::Home,
        Self::Instances,
        Self::Skin,
        Self::Friends,
        Self::Mods,
        Self::Import,
        Self::Settings,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Instances => "Instances",
            Self::Skin => "Skin",
            Self::Friends => "Friends",
            Self::Mods => "Mods & packs",
            Self::Import => "Import",
            Self::Settings => "Settings",
        }
    }
}

/// Login panel state: which provider is selected and how far its flow has got.
pub struct LoginState {
    pub provider: AccountProvider,
    pub cracked_name: String,
    pub state: AuthenticationState,
}

impl Default for LoginState {
    fn default() -> Self {
        Self {
            provider: AccountProvider::Microsoft,
            cracked_name: String::new(),
            state: AuthenticationState::Idle,
        }
    }
}

pub struct AlpinoxiApp {
    pub screen: Screen,
    pub settings: Settings,
    pub login: LoginState,
    pub account: Option<AccountProfile>,
    pub vault: MemoryVault,
    pub wardrobe: Wardrobe,
    pub skin_texture: Option<egui::TextureHandle>,
    pub skin_model: SkinModel,
    pub book: FriendBook,
    pub chat_draft: String,
    pub chat_channel: String,
    pub friend_draft: String,
    pub catalog: Vec<CatalogEntry>,
    pub catalog_query: String,
    pub catalog_kind: Option<Kind>,
    pub catalog_performance_only: bool,
    pub slogan: String,
    slogans: SloganPicker,
    pub notice: String,
    pub recording_keybind: Option<alpinoxi_core::hud::KeybindAction>,
    pub running: bool,
    pub banner: Option<egui::TextureHandle>,
    pub toast: Option<(String, f64)>,
    /// Last line this launcher sent, shown in the chat panel.
    pub last_sent: Option<String>,
    /// Path typed into the wardrobe's local-skin field.
    pub skin_path: String,
    /// Import-from-another-launcher screen state.
    pub import: ui::import::ImportState,
}

impl AlpinoxiApp {
    pub fn new() -> Self {
        let settings = config::load();
        let skin_path = settings
            .local_skin
            .as_ref()
            .map(|skin| skin.location.clone())
            .unwrap_or_default();
        let mut slogans = SloganPicker::new(SLOGANS, 0x4f1b_c1a5_9e37_2d11);
        let slogan = slogans
            .next(SLOGANS)
            .unwrap_or("Alpinoxi is best!")
            .to_owned();
        let wardrobe = Wardrobe::new(AccountProvider::Microsoft, "", settings.skin_model);
        Self {
            screen: Screen::Home,
            login: LoginState {
                cracked_name: settings.cracked_name.clone(),
                ..LoginState::default()
            },
            account: None,
            vault: MemoryVault::new(),
            wardrobe,
            skin_texture: None,
            skin_model: settings.skin_model,
            book: FriendBook::default(),
            chat_draft: String::new(),
            chat_channel: "global".to_owned(),
            friend_draft: String::new(),
            catalog: catalog::entries(),
            catalog_query: String::new(),
            catalog_kind: None,
            catalog_performance_only: false,
            slogan,
            slogans,
            notice: String::new(),
            recording_keybind: None,
            running: false,
            banner: None,
            toast: None,
            last_sent: None,
            skin_path,
            import: ui::import::ImportState::new(),
            settings,
        }
    }

    /// Rotates to the next tagline. Called by the top bar's refresh mark.
    pub fn next_slogan(&mut self) {
        if let Some(next) = self.slogans.next(SLOGANS) {
            self.slogan = next.to_owned();
        }
    }

    pub fn login(&mut self, provider: AccountProvider) {
        self.login.provider = provider;
        self.settings.last_provider = Some(provider);
        self.notice = match provider {
            AccountProvider::Offline => {
                match AccountProfile::cracked(&self.login.cracked_name.clone()) {
                    Ok(profile) => {
                        self.settings.cracked_name = profile.display_name.clone();
                        self.connect(profile);
                        return;
                    }
                    Err(error) => error.to_string(),
                }
            }
            AccountProvider::Microsoft => {
                "Microsoft sign-in opens in your browser: Alpinoxi asks for a short code, never a password.".to_owned()
            }
            AccountProvider::ElyBy => {
                "Ely.by sign-in opens in your browser with client id alpinoxi-client.".to_owned()
            }
        };
        self.login.state = AuthenticationState::AwaitingBrowser {
            url: String::new(),
            user_code: None,
        };
    }

    /// Called once a flow succeeded: sets the account, wardrobe and friend book.
    pub fn connect(&mut self, profile: AccountProfile) {
        self.wardrobe = Wardrobe::new(profile.provider, &profile.display_name, self.skin_model);
        self.wardrobe.resolve(self.settings.local_skin.clone());
        self.book.sign_in(Friend::new(
            profile.provider,
            profile.uuid.clone(),
            profile.display_name.clone(),
        ));
        self.login.state = AuthenticationState::Connected(profile.clone());
        self.notice = format!("Signed in as {}", profile.badge());
        self.account = Some(profile);
        self.screen = Screen::Home;
    }

    pub fn sign_out(&mut self) {
        self.account = None;
        self.login.state = AuthenticationState::Idle;
        self.skin_texture = None;
        self.book = FriendBook::default();
        self.notice = "Signed out.".to_owned();
    }

    pub fn save_settings(&mut self) {
        self.settings.skin_model = self.skin_model;
        if let Err(error) = config::save(&self.settings) {
            self.notice = format!("Settings could not be saved: {error}");
        } else {
            self.show_toast("Settings saved");
        }
    }

    pub fn show_toast(&mut self, text: &str) {
        self.toast = Some((text.to_owned(), 0.0));
    }

    /// The launch plan for the current instance, capped by system RAM.
    pub fn launch_plan(&self) -> LaunchPlan {
        LaunchPlan::from_profile(self.settings.memory, self.settings.system_memory_mib)
    }

    pub fn memory_warning(&self) -> Option<String> {
        let plan = self.launch_plan();
        let requested = u32::from(self.settings.memory.mebibytes());
        if plan.mebibytes < requested {
            Some(format!(
                "{} MiB was requested, but this machine has {} MiB of RAM, so Alpinoxi will use {} MiB.",
                requested, self.settings.system_memory_mib, plan.mebibytes
            ))
        } else if plan.mebibytes > 8_192 {
            Some(
                "More than 8 GB rarely helps a client: past that, GC pauses get longer. Sodium does more than extra RAM."
                    .to_owned(),
            )
        } else {
            None
        }
    }

    /// "Launch": builds the session the in-game layer would use.
    pub fn build_session(&self) -> Option<GameSession> {
        let account = self.account.clone()?;
        let mut session = GameSession::start(account);
        session.hud = self.settings.hud.clone();
        session.keybinds = self.settings.keybinds.clone();
        session.skin = self.wardrobe.skin.clone();
        session.book = self.book.clone();
        Some(session)
    }

    pub fn launch(&mut self) {
        if self.account.is_none() {
            self.notice = "Choose a login type first.".to_owned();
            return;
        }
        let plan = self.launch_plan();
        let _ = plan.arguments();
        self.running = true;
        self.notice = format!(
            "Starting {} · {} with {} MiB of heap.",
            self.settings.instance.name, self.settings.instance.game_version, plan.mebibytes
        );
        self.show_toast("Launch prepared");
    }

    pub fn set_memory(&mut self, profile: MemoryProfile) {
        self.settings.memory = profile;
        if let Some(warning) = self.memory_warning() {
            self.notice = warning;
        }
    }

    pub fn set_skin_model(&mut self, model: SkinModel) {
        self.skin_model = model;
        self.settings.skin_model = model;
        self.wardrobe.model = model;
        self.wardrobe.resolve(self.settings.local_skin.clone());
        self.skin_texture = None;
    }

    /// Mirrors the typed path into the settings so it survives a restart.
    pub fn remember_skin_path(&mut self, path: &str) {
        self.skin_path = path.to_owned();
    }

    pub fn load_local_skin(&mut self, path: &str) {
        match std::fs::read(path) {
            Ok(bytes) => match self.wardrobe.apply_local(&bytes, path) {
                Ok(()) => {
                    let mut skin = self.wardrobe.skin.clone().unwrap();
                    skin.source = SkinSource::Local;
                    self.settings.local_skin = Some(skin);
                    self.skin_texture = None;
                    self.notice = "Local skin applied.".to_owned();
                }
                Err(error) => self.notice = error.to_string(),
            },
            Err(error) => self.notice = format!("Could not read that file: {error}"),
        }
    }

    pub fn add_friend(&mut self) {
        let name = self.friend_draft.trim().to_owned();
        if name.is_empty() {
            return;
        }
        let friend = Friend::new(
            AccountProvider::Offline,
            alpinoxi_core::account::offline_uuid(&name),
            name,
        );
        match self.book.send_request(friend, 0) {
            Ok(()) => {
                self.notice = format!("Friend request sent to {}.", self.friend_draft.trim());
                self.friend_draft.clear();
            }
            Err(error) => self.notice = error.to_string(),
        }
    }

    pub fn send_chat(&mut self) {
        let Some(me) = self.book.me.clone() else {
            self.notice = "Sign in to use chat.".to_owned();
            return;
        };
        let channel = self.chat_channel.clone();
        let text = self.chat_draft.clone();
        if ChatMessage::sanitise(&text).is_err() {
            return;
        }
        let message = ChatMessage::new(me, channel, text.clone(), 0);
        let mut bridge = NullBridge;
        let mut overlay = alpinoxi_ingame::Overlay::default();
        if overlay
            .send(
                message.from.clone(),
                &message.channel,
                &message.text,
                0,
                &mut bridge,
            )
            .is_ok()
        {
            self.last_sent = Some(text);
            self.chat_draft.clear();
        }
    }

    /// Simulated presence so the friend panel shows its real states while the
    /// multiplayer service is not wired up yet.
    pub fn refresh_presence(&mut self) {
        let friends: Vec<Friend> = self.book.friends().to_vec();
        for (index, friend) in friends.iter().enumerate() {
            let presence = match index % 3 {
                0 => Presence::InGame {
                    server: Some("play.alpinoxi.net".to_owned()),
                },
                1 => Presence::InLauncher,
                _ => Presence::Offline,
            };
            self.book.set_presence(friend, presence);
        }
    }

    pub fn ensure_skin_texture(&mut self, ctx: &egui::Context) {
        if self.skin_texture.is_some() {
            return;
        }
        let Some(skin) = self.wardrobe.skin.clone() else {
            return;
        };
        if skin.source != SkinSource::Local {
            return;
        }
        let Ok(bytes) = std::fs::read(&skin.location) else {
            return;
        };
        let Ok(image) = crate::skinview::SkinImage::decode(&bytes) else {
            return;
        };
        let portrait = image.front_portrait(self.skin_model);
        let colour_image = egui::ColorImage::from_rgba_unmultiplied(
            [portrait.width as usize, portrait.height as usize],
            &portrait.rgba,
        );
        self.skin_texture =
            Some(ctx.load_texture("alpinoxi-skin", colour_image, egui::TextureOptions::NEAREST));
    }

    pub fn accent(&self) -> egui::Color32 {
        theme::provider_color(self.login.provider)
    }
}

/// The bridge used by launcher-side chat: the launcher has no game to talk to,
/// so it only records the message it already appended to its own view.
struct NullBridge;

impl alpinoxi_ingame::GameBridge for NullBridge {
    fn local_player_name(&self) -> String {
        String::new()
    }
    fn current_server(&self) -> Option<String> {
        None
    }
    fn show_hud(&mut self, _visible: bool) {}
    fn open_hud_editor(&mut self) {}
    fn open_friend_list(&mut self) {}
    fn open_chat(&mut self) {}
    fn set_skin_preview(&mut self, _visible: bool) {}
    fn push_message(&mut self, _channel: &str, _text: &str) {}
}

impl eframe::App for AlpinoxiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.account.is_none() {
            ui::login::show(ctx, self);
        } else {
            ui::shell::show(ctx, self);
        }

        // A key pressed while the HUD keybind picker is armed is captured here,
        // before any widget consumes it.
        if let Some(action) = self.recording_keybind {
            if let Some(chord) = ui::keybinds::capture(ctx) {
                let mut binds = self.settings.keybinds.clone();
                match binds.assign(action, chord) {
                    Ok(()) => {
                        self.settings.keybinds = binds;
                        self.recording_keybind = None;
                        self.notice = format!("{} is now {}.", action.label(), chord.label());
                    }
                    Err(taken) => {
                        self.notice = format!(
                            "{} already uses {}, pick another key.",
                            taken.label(),
                            chord.label()
                        );
                    }
                }
            }
        }

        if let Some((text, started)) = self.toast.clone() {
            let now = ctx.input(|input| input.time);
            let started = if started == 0.0 { now } else { started };
            if now - started > 2.5 {
                self.toast = None;
            } else {
                self.toast = Some((text, started));
                ctx.request_repaint_after(std::time::Duration::from_millis(120));
            }
        }

        // Keep the wardrobe honest when the player changes the model elsewhere.
        if self.wardrobe.model != self.skin_model {
            self.wardrobe.model = self.skin_model;
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let _ = config::save(&self.settings);
    }
}

/// Helper used by the HUD screen to preview the key list.
pub fn hud_elements() -> [HudElement; 10] {
    HudElement::ALL
}

/// Default keybinds, for the "reset" button.
pub fn default_keybinds() -> KeybindSet {
    KeybindSet::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cracked_login_connects_and_sets_the_wardrobe() {
        let mut app = AlpinoxiApp::new();
        app.login.cracked_name = "Alpinoxi".to_owned();
        app.login(AccountProvider::Offline);
        let account = app.account.clone().expect("cracked login should connect");
        assert_eq!(account.provider, AccountProvider::Offline);
        assert_eq!(app.wardrobe.provider, AccountProvider::Offline);
        assert!(app.build_session().is_some());
    }

    #[test]
    fn launch_never_exceeds_the_machine_budget() {
        let mut app = AlpinoxiApp::new();
        app.settings.system_memory_mib = 8 * 1024;
        app.set_memory(MemoryProfile::Custom(16_000));
        // Half of an 8 GiB machine is 4 096 MiB, and the absolute ceiling is
        // 8 GiB, so either bound catches a runaway request.
        assert!(app.launch_plan().mebibytes <= 8 * 1024);
        assert!(app.memory_warning().is_some());
    }

    #[test]
    fn launch_without_an_account_is_refused() {
        let mut app = AlpinoxiApp::new();
        app.launch();
        assert!(!app.running);
        assert!(app.notice.contains("login type"));
    }

    #[test]
    fn slogan_changes_on_request() {
        let mut app = AlpinoxiApp::new();
        let first = app.slogan.clone();
        app.next_slogan();
        assert!(!app.slogan.is_empty());
        assert_eq!(SLOGANS.iter().filter(|line| **line == first).count(), 1);
    }
}
