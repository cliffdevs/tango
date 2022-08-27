use fluent_templates::Loader;

use crate::{games, gui, i18n};

pub struct State {
    selection: Option<(
        &'static (dyn games::Game + Send + Sync),
        Option<std::path::PathBuf>,
    )>,
}

impl State {
    pub fn new(
        selection: Option<(
            &'static (dyn games::Game + Send + Sync),
            Option<std::path::PathBuf>,
        )>,
    ) -> Self {
        Self { selection }
    }
}

pub struct SaveSelectWindow;

impl SaveSelectWindow {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        show: &mut Option<State>,
        selection: &mut Option<(&'static (dyn games::Game + Send + Sync), std::path::PathBuf)>,
        language: &unic_langid::LanguageIdentifier,
        saves_path: &std::path::Path,
        saves_list: gui::SavesListState,
    ) {
        let mut show_play_bool = show.is_some();
        egui::Window::new(format!(
            "{}",
            i18n::LOCALES.lookup(language, "select-save").unwrap()
        ))
        .id(egui::Id::new("select-save-window"))
        .open(&mut show_play_bool)
        .show(ctx, |ui| {
            let saves_list = saves_list.read();
            let games = games::sorted_games(language);
            if let Some((game, _)) = show.as_mut().unwrap().selection {
                let (family, variant) = game.family_and_variant();
                ui.heading(
                    i18n::LOCALES
                        .lookup(language, &format!("games.{}-{}", family, variant))
                        .unwrap(),
                );
            }

            ui.group(|ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                        if let Some((game, _)) = show.as_ref().unwrap().selection.clone() {
                            if ui
                                .selectable_label(
                                    false,
                                    format!(
                                        "⬅️ {}",
                                        i18n::LOCALES
                                            .lookup(language, "select-save.return-to-games-list")
                                            .unwrap()
                                    ),
                                )
                                .clicked()
                            {
                                show.as_mut().unwrap().selection = None;
                            }

                            if let Some(saves) = saves_list.saves.get(&game) {
                                for save in saves {
                                    if ui
                                        .selectable_label(
                                            selection
                                                .as_ref()
                                                .map(|(_, p)| save.as_path() == p)
                                                .unwrap_or(false),
                                            format!(
                                                "{}",
                                                save.as_path()
                                                    .strip_prefix(saves_path)
                                                    .unwrap_or(save.as_path())
                                                    .display()
                                            ),
                                        )
                                        .clicked()
                                    {
                                        *show = None;
                                        *selection = Some((game, save.as_path().to_path_buf()));
                                    }
                                }
                            }
                        } else {
                            for (available, game) in games
                                .iter()
                                .filter(|g| saves_list.roms.contains_key(*g))
                                .map(|g| (true, g))
                                .chain(
                                    games
                                        .iter()
                                        .filter(|g| !saves_list.roms.contains_key(*g))
                                        .map(|g| (false, g)),
                                )
                            {
                                let (family, variant) = game.family_and_variant();
                                let text = i18n::LOCALES
                                    .lookup(language, &format!("games.{}-{}", family, variant))
                                    .unwrap();

                                if available {
                                    if ui.selectable_label(false, text).clicked() {
                                        show.as_mut().unwrap().selection = Some((*game, None));
                                    }
                                } else {
                                    ui.weak(text);
                                }
                            }
                        }
                    });
                });
            });
        });

        if !show_play_bool {
            *show = None;
        }
    }
}
