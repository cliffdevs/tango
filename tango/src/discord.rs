mod rpc;

#[allow(dead_code)]
use fluent_templates::Loader;

use crate::{game, i18n, sync};

const APP_ID: u64 = 974089681333534750;

pub struct GameInfo {
    pub title: String,
    pub family: String,
}

pub fn make_game_info(
    game: &'static (dyn game::Game + Send + Sync),
    patch: Option<(&str, &semver::Version)>,
    language: &unic_langid::LanguageIdentifier,
) -> GameInfo {
    let family = game.family_and_variant().0.to_string();
    let mut title = i18n::LOCALES.lookup(language, &format!("game-{}", family)).unwrap();
    if let Some((patch_name, patch_version)) = patch.as_ref() {
        title.push_str(&format!(" + {} v{}", patch_name, patch_version));
    }
    GameInfo { title, family }
}

pub fn make_base_activity(game_info: Option<GameInfo>) -> rpc::activity::Activity {
    rpc::activity::Activity {
        details: game_info.as_ref().map(|gi| gi.title.clone()),
        assets: Some(rpc::activity::Assets {
            small_image: Some("logo".to_string()),
            small_text: Some("Tango".to_string()),
            large_image: game_info.as_ref().map(|gi| gi.family.clone()),
            large_text: game_info.as_ref().map(|gi| gi.title.clone()),
        }),
        ..Default::default()
    }
}

pub fn make_looking_activity(
    link_code: &str,
    lang: &unic_langid::LanguageIdentifier,
    game_info: Option<GameInfo>,
) -> rpc::activity::Activity {
    rpc::activity::Activity {
        state: Some(i18n::LOCALES.lookup(lang, "discord-presence-looking").unwrap()),
        secrets: Some(rpc::activity::Secrets {
            join: Some(link_code.to_string()),
            ..Default::default()
        }),
        party: Some(rpc::activity::Party {
            id: Some(format!("party:{}", link_code)),
            size: Some([1, 2]),
        }),
        ..make_base_activity(game_info)
    }
}

pub fn make_single_player_activity(
    start_time: std::time::SystemTime,
    lang: &unic_langid::LanguageIdentifier,
    game_info: Option<GameInfo>,
) -> rpc::activity::Activity {
    rpc::activity::Activity {
        state: Some(i18n::LOCALES.lookup(lang, "discord-presence-in-single-player").unwrap()),
        timestamps: Some(rpc::activity::Timestamps {
            start: start_time
                .duration_since(std::time::UNIX_EPOCH)
                .ok()
                .map(|d| d.as_millis() as u64),

            end: None,
        }),
        ..make_base_activity(game_info)
    }
}

pub fn make_in_lobby_activity(
    link_code: &str,
    lang: &unic_langid::LanguageIdentifier,
    game_info: Option<GameInfo>,
) -> rpc::activity::Activity {
    rpc::activity::Activity {
        state: Some(i18n::LOCALES.lookup(lang, "discord-presence-in-lobby").unwrap()),
        party: Some(rpc::activity::Party {
            id: Some(format!("party:{}", link_code)),
            size: Some([2, 2]),
        }),
        ..make_base_activity(game_info)
    }
}

pub fn make_in_progress_activity(
    link_code: &str,
    start_time: std::time::SystemTime,
    lang: &unic_langid::LanguageIdentifier,
    game_info: Option<GameInfo>,
) -> rpc::activity::Activity {
    rpc::activity::Activity {
        state: Some(i18n::LOCALES.lookup(lang, "discord-presence-in-progress").unwrap()),
        party: Some(rpc::activity::Party {
            id: Some(format!("party:{}", link_code)),
            size: Some([2, 2]),
        }),
        timestamps: Some(rpc::activity::Timestamps {
            start: start_time
                .duration_since(std::time::UNIX_EPOCH)
                .ok()
                .map(|d| d.as_millis() as u64),

            end: None,
        }),
        ..make_base_activity(game_info)
    }
}

pub struct Client {
    rpc: std::sync::Arc<tokio::sync::Mutex<Option<rpc::Client>>>,
    current_activity: std::sync::Arc<tokio::sync::Mutex<Option<rpc::activity::Activity>>>,
    current_join_secret: std::sync::Arc<tokio::sync::Mutex<Option<String>>>,
}

impl Client {
    pub fn new() -> Self {
        let current_activity: std::sync::Arc<tokio::sync::Mutex<Option<rpc::activity::Activity>>> =
            std::sync::Arc::new(tokio::sync::Mutex::new(None));
        let current_join_secret = std::sync::Arc::new(tokio::sync::Mutex::new(None));
        let rpc = std::sync::Arc::new(tokio::sync::Mutex::new(None));

        {
            let rpc = rpc.clone();
            let current_activity = current_activity.clone();

            tokio::task::spawn(async move {
                loop {
                    {
                        // Try establish RPC connection, if not already open.
                        let mut rpc_guard = rpc.lock().await;
                        let current_activity = current_activity.clone();

                        if rpc_guard.is_none() {
                            *rpc_guard = match (|| async {
                                let rpc = rpc::Client::connect(APP_ID).await?;
                                rpc.subscribe(rpc::Event::ActivityJoin).await?;
                                Ok::<_, anyhow::Error>(rpc)
                            })()
                            .await
                            {
                                Ok(rpc) => {
                                    log::info!("connected to discord RPC");
                                    Some(rpc)
                                }
                                Err(err) => {
                                    log::warn!("did not open discord RPC client: {:?}", err);
                                    None
                                }
                            };
                        }

                        if let Some(rpc) = &*rpc_guard {
                            // Do stuff with RPC connection.
                            if let Err(err) = (|| async {
                                if let Some(activity) = current_activity.lock().await.as_ref() {
                                    rpc.set_activity(activity).await?;
                                }

                                Ok::<_, anyhow::Error>(())
                            })()
                            .await
                            {
                                log::warn!("discord RPC client encountered error: {:?}", err);
                                *rpc_guard = None;
                            }
                        }
                    }

                    tokio::time::sleep(std::time::Duration::from_secs(15)).await;
                }
            });
        }

        let client = Self {
            rpc,
            current_activity,
            current_join_secret,
        };
        client
    }

    pub fn set_current_activity(&self, activity: Option<rpc::activity::Activity>) {
        // RPC lock must be acquired first.
        let rpc = self.rpc.blocking_lock();

        let mut current_activity = self.current_activity.blocking_lock();
        if activity == *current_activity {
            return;
        }

        if let Some(activity) = activity.as_ref() {
            if let Some(rpc) = &*rpc {
                let _ = sync::block_on(rpc.set_activity(activity));
            }
        }

        *current_activity = activity;
    }

    pub fn has_current_join_secret(&self) -> bool {
        self.current_join_secret.blocking_lock().is_some()
    }

    pub fn take_current_join_secret(&self) -> Option<String> {
        self.current_join_secret.blocking_lock().take()
    }
}
