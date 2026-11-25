use alpinoxi_auth_system::{begin, AuthenticationState};
use alpinoxi_core::{AccountProvider, MemoryProfile};
use eframe::egui::{self, Color32, RichText, Vec2};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(Vec2::new(1120.0, 720.0))
            .with_min_inner_size(Vec2::new(920.0, 600.0))
            .with_title("Alpinoxi Client"),
        ..Default::default()
    };

    eframe::run_native(
        "Alpinoxi Client",
        options,
        Box::new(|creation_context| {
            configure_theme(&creation_context.egui_ctx);
            Ok(Box::<AlpinoxiApp>::default())
        }),
    )
}

fn configure_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = Color32::from_rgb(12, 24, 47);
    visuals.panel_fill = Color32::from_rgb(8, 17, 35);
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(16, 35, 66);
    visuals.selection.bg_fill = Color32::from_rgb(41, 143, 230);
    visuals.hyperlink_color = Color32::from_rgb(87, 214, 255);
    ctx.set_visuals(visuals);
}

#[derive(Default)]
struct AlpinoxiApp {
    selected_provider: Option<AccountProvider>,
    memory: MemoryProfile,
    notice: String,
}

impl eframe::App for AlpinoxiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("navigation")
            .resizable(false)
            .exact_width(220.0)
            .show(ctx, |ui| {
                ui.add_space(24.0);
                ui.heading(RichText::new("ALPINOXI").color(Color32::from_rgb(91, 216, 255)).size(24.0));
                ui.label(RichText::new("CLIENT").strong().color(Color32::from_rgb(102, 139, 198)));
                ui.add_space(42.0);
                for label in ["Home", "Instances", "Accounts", "Library", "Settings"] {
                    let selected = label == "Home";
                    ui.add_sized(
                        [190.0, 38.0],
                        egui::Button::new(label).selected(selected),
                    );
                    ui.add_space(4.0);
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.label(RichText::new("v0.1.0 • Rust native").color(Color32::GRAY));
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(20.0);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("WELCOME BACK").strong().color(Color32::from_rgb(91, 216, 255)));
                    ui.heading(RichText::new("Ready for your next world?").size(34.0));
                    ui.label(RichText::new("Choose an account, tune the instance, then launch when Minecraft setup is ready.").color(Color32::from_rgb(170, 192, 225)));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    ui.add_sized([150.0, 42.0], egui::Button::new(RichText::new("Setup required").strong()));
                });
            });
            ui.add_space(26.0);

            ui.group(|ui| {
                ui.set_min_width(ui.available_width());
                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("DEFAULT INSTANCE").strong().color(Color32::from_rgb(91, 216, 255)));
                        ui.heading("Vanilla 1.21.1");
                        ui.label("No game files installed yet");
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_sized([155.0, 42.0], egui::Button::new(RichText::new("Install Minecraft").strong()));
                    });
                });
                ui.add_space(14.0);
            });

            ui.add_space(20.0);
            ui.columns(2, |columns| {
                account_card(&mut columns[0], self);
                memory_card(&mut columns[1], self);
            });
            ui.add_space(18.0);
            ui.group(|ui| {
                ui.add_space(10.0);
                ui.label(RichText::new("ALPINOXI STATUS").strong().color(Color32::from_rgb(91, 216, 255)));
                ui.label(if self.notice.is_empty() { "No account action selected." } else { &self.notice });
                ui.add_space(10.0);
            });
        });
    }
}

fn account_card(ui: &mut egui::Ui, app: &mut AlpinoxiApp) {
    ui.group(|ui| {
        ui.add_space(12.0);
        ui.label(RichText::new("ACCOUNT").strong().color(Color32::from_rgb(91, 216, 255)));
        ui.heading("Choose how you play");
        ui.label("Every profile stays clearly labelled.");
        ui.add_space(8.0);
        for provider in [AccountProvider::Microsoft, AccountProvider::ElyBy, AccountProvider::Offline] {
            let label = format!("{}\n{}", provider.label(), provider.detail());
            if ui.add_sized([ui.available_width(), 54.0], egui::Button::new(label)).clicked() {
                app.selected_provider = Some(provider);
                app.notice = match begin(provider) {
                    AuthenticationState::AwaitingBrowser(_) => format!("{} sign-in will open once authentication is configured.", provider.label()),
                    AuthenticationState::Failed(error) => error.to_string(),
                    _ => "Account state updated.".to_owned(),
                };
            }
            ui.add_space(5.0);
        }
        ui.add_space(8.0);
    });
}

fn memory_card(ui: &mut egui::Ui, app: &mut AlpinoxiApp) {
    ui.group(|ui| {
        ui.add_space(12.0);
        ui.label(RichText::new("MEMORY").strong().color(Color32::from_rgb(91, 216, 255)));
        ui.heading("Keep RAM intentional");
        ui.label("This limit applies to Minecraft, not to Alpinoxi Client.");
        ui.add_space(12.0);
        for (name, profile, text) in [
            ("Balanced", MemoryProfile::Balanced, "2 GB • lower-RAM systems"),
            ("Performance", MemoryProfile::Performance, "4 GB • modded gameplay"),
        ] {
            let selected = app.memory == profile;
            if ui.selectable_label(selected, format!("{name}\n{text}")).clicked() {
                app.memory = profile;
            }
            ui.add_space(6.0);
        }
        ui.label(format!("Selected: {} MiB", app.memory.mebibytes()));
        ui.add_space(12.0);
    });
}
