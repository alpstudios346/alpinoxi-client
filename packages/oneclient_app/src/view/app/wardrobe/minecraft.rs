use freya::prelude::*;
use uuid::Uuid;

use crate::components::{Button, Icon, IconType, PlayerModel, ScrollArea, centered_spinner};
use crate::hooks::wardrobe::{
    MojangLookQuery, WardrobeAction, active_cape, active_skin, use_mojang_look, use_wardrobe_action,
};
use crate::hooks::{
    query_error, query_is_loading, settled_or_loading, use_current_account, use_dispatch,
};
use crate::i18n::t;
use crate::theme::{colors, rounded, text_size};
use crate::ui::centered_note;
use crate::view::app::wardrobe::page::perform;

#[derive(PartialEq)]
pub struct Wardrobe;

impl Component for Wardrobe {
    fn render(&self) -> impl IntoElement {
        let current = use_current_account();
        let account = settled_or_loading(&current).flatten();

        match account {
            Some(account) if account.is_microsoft() => MinecraftWardrobe {
                account_id: account.id,
                profile_key: account.skin_profile_key(),
            }
            .into_element(),
            Some(account) if account.is_elyby() => ElybyWardrobe {
                username: account.username,
            }
            .into_element(),
            Some(account) if account.is_offline() => OfflineWardrobe.into_element(),
            Some(_) => account_prompt(),
            None if query_is_loading(&current) => {
                centered_spinner(t("oneclient.wardrobe.account.loading"))
            }
            None => account_prompt(),
        }
    }
}

fn account_prompt() -> Element {
    centered_note(t("oneclient.wardrobe.account.signed_out"))
}

#[derive(PartialEq)]
struct OfflineWardrobe;

impl Component for OfflineWardrobe {
    fn render(&self) -> impl IntoElement {
        let dispatch = use_dispatch();
        let open_accounts = move |_| {
            let _ = freya::router::RouterContext::get().push(crate::Route::SettingsAccounts {});
            dispatch.close_account_switcher();
        };
        let open_switcher = move |_| dispatch.open_account_switcher();

        rect()
            .width(Size::fill())
            .height(Size::fill())
            .center()
            .child(
                rect()
                    .max_width(Size::px(520.))
                    .vertical()
                    .cross_align(Alignment::Center)
                    .spacing(16.)
                    .padding(Gaps::new_all(28.))
                    .corner_radius(rounded(18.))
                    .background(colors::component_bg())
                    .child(Icon::new(IconType::User01).size(32.).color(colors::brand()))
                    .child(
                        label()
                            .text(t("oneclient.wardrobe.account.offline.title"))
                            .font_size(text_size(22.))
                            .font_weight(FontWeight::BOLD)
                            .color(colors::fg_primary()),
                    )
                    .child(
                        label()
                            .text(t("oneclient.wardrobe.account.offline.body"))
                            .width(Size::fill())
                            .text_align(TextAlign::Center)
                            .font_size(text_size(14.))
                            .color(colors::fg_secondary()),
                    )
                    .child(
                        rect()
                            .horizontal()
                            .spacing(10.)
                            .child(
                                Button::new()
                                    .primary()
                                    .text(t("oneclient.wardrobe.account.offline.switch"))
                                    .on_press(open_switcher),
                            )
                            .child(
                                Button::new()
                                    .secondary()
                                    .text(t("oneclient.wardrobe.account.offline.manage"))
                                    .on_press(open_accounts),
                            ),
                    ),
            )
            .into_element()
    }
}

#[derive(PartialEq)]
struct ElybyWardrobe {
    username: String,
}

impl Component for ElybyWardrobe {
    fn render(&self) -> impl IntoElement {
        let message = use_state(|| None::<String>);
        let auto_opened = use_state(|| false);
        let mut opened = auto_opened;
        let auto_open_status = message;

        use_side_effect(move || {
            if !*opened.peek() {
                opened.set(true);
                open_elyby_skins(auto_open_status);
            }
        });

        let mut status = message;
        let open = move |_| open_elyby_skins(status);
        rect()
            .width(Size::fill())
            .height(Size::fill())
            .center()
            .child(
                rect()
                    .max_width(Size::px(520.))
                    .vertical()
                    .cross_align(Alignment::Center)
                    .spacing(16.)
                    .padding(Gaps::new_all(28.))
                    .corner_radius(rounded(18.))
                    .background(colors::component_bg())
                    .child(
                        Icon::new(IconType::ExternalLink)
                            .size(32.)
                            .color(colors::brand()),
                    )
                    .child(
                        label()
                            .text(t("oneclient.wardrobe.account.elyby.title"))
                            .font_size(text_size(22.))
                            .font_weight(FontWeight::BOLD)
                            .color(colors::fg_primary()),
                    )
                    .child(
                        label()
                            .text(t("oneclient.wardrobe.account.elyby.body"))
                            .width(Size::fill())
                            .text_align(TextAlign::Center)
                            .font_size(text_size(14.))
                            .color(colors::fg_secondary()),
                    )
                    .child(
                        label()
                            .text(self.username.clone())
                            .font_size(text_size(13.))
                            .color(colors::fg_secondary()),
                    )
                    .child(
                        Button::new()
                            .primary()
                            .text(t("oneclient.wardrobe.account.elyby.open"))
                            .on_press(open),
                    )
                    .maybe_child(status.read().clone().map(|message| {
                        label()
                            .text(message)
                            .font_size(text_size(12.))
                            .color(colors::fg_secondary())
                            .into_element()
                    })),
            )
            .into_element()
    }
}

fn open_elyby_skins(mut status: State<Option<String>>) {
    spawn(async move {
        match tokio::task::spawn_blocking(|| open::that("https://ely.by/skins")).await {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                tracing::warn!(%error, "could not open Ely.by skin settings");
                status.set(Some(
                    t("oneclient.wardrobe.account.elyby.open_failed").into(),
                ));
            }
            Err(error) => {
                tracing::warn!(%error, "could not open Ely.by skin settings");
                status.set(Some(
                    t("oneclient.wardrobe.account.elyby.open_failed").into(),
                ));
            }
        }
    });
}

#[derive(PartialEq)]
struct MinecraftWardrobe {
    account_id: Uuid,
    profile_key: String,
}

impl Component for MinecraftWardrobe {
    fn render(&self) -> impl IntoElement {
        let profile_query = use_mojang_look(self.account_id);
        let profile = settled_or_loading(&profile_query);
        let action = use_wardrobe_action();
        let mut status = use_state(|| None::<String>);
        let mut open_status = status.clone();
        let account_id = self.account_id;
        let profile_key = self.profile_key.clone();

        if query_is_loading(&profile_query) {
            return centered_spinner(t("oneclient.wardrobe.page.loading"));
        }

        let mut content = rect()
            .width(Size::fill())
            .vertical()
            .spacing(18.)
            .padding(Gaps::new_all(24.))
            .child(
                label()
                    .text(t("oneclient.wardrobe.minecraft.title"))
                    .font_size(text_size(28.))
                    .font_weight(FontWeight::BOLD)
                    .color(colors::fg_primary()),
            )
            .child(
                label()
                    .text(t("oneclient.wardrobe.minecraft.body"))
                    .font_size(text_size(14.))
                    .color(colors::fg_secondary()),
            )
            .child(
                Button::new()
                    .secondary()
                    .child(Icon::new(IconType::ExternalLink).size(15.))
                    .text(t("oneclient.wardrobe.minecraft.manage"))
                    .on_press(move |_| {
                        let mut status = open_status;
                        spawn(async move {
                            let result = tokio::task::spawn_blocking(|| {
                                open::that("https://www.minecraft.net/msaprofile")
                            })
                            .await;
                            let failed = match result {
                                Ok(Ok(())) => false,
                                Ok(Err(error)) => {
                                    tracing::warn!(%error, "could not open Microsoft skin profile");
                                    true
                                }
                                Err(error) => {
                                    tracing::warn!(%error, "could not open Microsoft skin profile");
                                    true
                                }
                            };
                            if failed {
                                status.set(Some(
                                    t("oneclient.wardrobe.minecraft.open_failed").into(),
                                ));
                            }
                        });
                    }),
            );

        match (profile, query_error(&profile_query)) {
            (Some(profile), _) => {
                if let Some(skin) = active_skin(&profile) {
                    content = content.child(
                        rect()
                            .height(Size::px(300.))
                            .corner_radius(rounded(16.))
                            .background(colors::component_bg())
                            .child(PlayerModel::new(profile_key.clone())),
                    );
                    content = content.child(
                        label()
                            .text(match &skin.variant {
                                oneclient_mc::SkinVariant::Classic => {
                                    t("oneclient.wardrobe.minecraft.skin.classic")
                                }
                                oneclient_mc::SkinVariant::Slim => {
                                    t("oneclient.wardrobe.minecraft.skin.slim")
                                }
                            })
                            .font_size(text_size(13.))
                            .color(colors::fg_secondary()),
                    );
                }

                content = content.child(
                    label()
                        .text(t("oneclient.wardrobe.minecraft.capes"))
                        .font_size(text_size(18.))
                        .font_weight(FontWeight::BOLD)
                        .color(colors::fg_primary()),
                );

                if profile.capes.is_empty() {
                    content = content.child(
                        label()
                            .text(t("oneclient.wardrobe.minecraft.no_capes"))
                            .font_size(text_size(13.))
                            .color(colors::fg_secondary()),
                    );
                } else {
                    for cape in &profile.capes {
                        let active =
                            active_cape(&profile).is_some_and(|active| active.id == cape.id);
                        let name = cape.alias.clone();
                        let cape_id = cape.id.clone();
                        let action = action.clone();
                        let status = status.clone();
                        let name_for_action = name.clone();
                        let mut button = Button::new().secondary();
                        button = button.text(if active {
                            t("oneclient.wardrobe.minecraft.cape.active")
                        } else {
                            t("oneclient.wardrobe.minecraft.cape.equip")
                        });
                        content = content.child(
                            rect()
                                .horizontal()
                                .cross_align(Alignment::Center)
                                .spacing(12.)
                                .padding(Gaps::new_symmetric(10., 14.))
                                .corner_radius(rounded(10.))
                                .background(colors::component_bg())
                                .child(
                                    label()
                                        .text(name.clone())
                                        .width(Size::flex(1.))
                                        .font_size(text_size(14.))
                                        .color(colors::fg_primary()),
                                )
                                .child(button.on_press(move |_| {
                                    perform(
                                        action,
                                        status,
                                        WardrobeAction::MojangCape {
                                            account: account_id,
                                            cape: (!active).then_some(cape_id),
                                            name: name_for_action,
                                        },
                                    );
                                })),
                        );
                    }
                }
            }
            (None, Some(error)) => {
                content = content.child(
                    label()
                        .text(error)
                        .font_size(text_size(13.))
                        .color(colors::error()),
                );
            }
            (None, None) => {}
        }

        content = content.maybe_child(status.read().clone().map(|message| {
            label()
                .text(message)
                .font_size(text_size(13.))
                .color(colors::fg_secondary())
                .into_element()
        }));

        ScrollArea::new()
            .width(Size::fill())
            .height(Size::fill())
            .child(content)
    }
}
