mod munger;
mod offsets;

use byteorder::ByteOrder;

use crate::{battle, game, lockstep, replayer, session, shadow, sync};

pub struct Hooks {
    offsets: &'static offsets::Offsets,
}

impl Hooks {
    fn munger(&self) -> munger::Munger {
        munger::Munger { offsets: self.offsets }
    }
}

pub static AE2E_00: Hooks = Hooks {
    offsets: &offsets::AE2E_00,
};

pub static AE2J_01: Hooks = Hooks {
    offsets: &offsets::AE2J_01,
};

fn random_background(rng: &mut impl rand::Rng) -> u8 {
    const BATTLE_BACKGROUNDS: &[u8] = &[0x00, 0x01, 0x02, 0x03, 0x05, 0x08, 0x15, 0x18];
    BATTLE_BACKGROUNDS[rng.gen_range(0..BATTLE_BACKGROUNDS.len())]
}

fn step_rng(seed: u32) -> u32 {
    let seed = std::num::Wrapping(seed);
    ((seed << 1) + (seed >> 0x1f) + std::num::Wrapping(1)).0 ^ 0x873ca9e5
}

fn generate_rng_state(rng: &mut impl rand::Rng) -> u32 {
    let mut rng_state = 0xa338244f;
    for _ in 0..rng.gen_range(0..0x10000) {
        rng_state = step_rng(rng_state);
    }
    rng_state
}

const INIT_RX: [u8; 16] = [
    0x00, 0x04, 0x00, 0xff, 0xff, 0xff, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04,
];

impl game::Hooks for Hooks {
    fn common_traps(&self) -> Vec<(u32, Box<dyn Fn(mgba::core::CoreMutRef)>)> {
        vec![
            (self.offsets.rom.start_screen_jump_table_entry, {
                let munger = self.munger();
                Box::new(move |core| {
                    munger.skip_logo(core);
                })
            }),
            (
                self.offsets.rom.start_screen_play_music_call,
                Box::new(move |mut core| {
                    let pc = core.as_ref().gba().cpu().thumb_pc() as u32;
                    core.gba_mut().cpu_mut().set_thumb_pc(pc + 4);
                }),
            ),
            (self.offsets.rom.start_screen_sram_unmask_ret, {
                let munger = self.munger();

                Box::new(move |core| {
                    munger.continue_from_title_menu(core);
                })
            }),
            (self.offsets.rom.game_load_ret, {
                let munger = self.munger();

                Box::new(move |core| {
                    munger.open_comm_menu_from_overworld(core);
                })
            }),
        ]
    }

    fn primary_traps(
        &self,
        joyflags: std::sync::Arc<std::sync::atomic::AtomicU32>,
        match_: std::sync::Arc<tokio::sync::Mutex<Option<std::sync::Arc<battle::Match>>>>,
        completion_token: session::CompletionToken,
    ) -> Vec<(u32, Box<dyn Fn(mgba::core::CoreMutRef)>)> {
        let make_send_and_receive_call_hook = || {
            let match_ = match_.clone();
            Box::new(move |mut core: mgba::core::CoreMutRef| {
                let pc = core.as_ref().gba().cpu().thumb_pc();
                core.gba_mut().cpu_mut().set_thumb_pc(pc + 4);

                let match_ = sync::block_on(match_.lock());
                match &*match_ {
                    Some(match_) => match_,
                    _ => {
                        core.gba_mut().cpu_mut().set_gpr(0, 0);
                        return;
                    }
                };
                core.gba_mut().cpu_mut().set_gpr(0, 3);
            })
        };
        vec![
            (self.offsets.rom.comm_menu_init_ret, {
                let match_ = match_.clone();
                let munger = self.munger();
                Box::new(move |core| {
                    let match_ = sync::block_on(match_.lock());
                    let match_ = match &*match_ {
                        Some(match_) => match_,
                        _ => {
                            return;
                        }
                    };

                    let mut rng = sync::block_on(match_.lock_rng());
                    let offerer_rng_state = generate_rng_state(&mut *rng);
                    let answerer_rng_state = generate_rng_state(&mut *rng);
                    munger.set_rng_state(
                        core,
                        if match_.is_offerer() {
                            offerer_rng_state
                        } else {
                            answerer_rng_state
                        },
                    );
                    munger.start_battle_from_comm_menu(core, random_background(&mut *rng));
                })
            }),
            (
                self.offsets.rom.match_end_ret,
                Box::new(move |_core| {
                    completion_token.complete();
                }),
            ),
            (self.offsets.rom.round_end_set_win, {
                let match_ = match_.clone();
                Box::new(move |_| {
                    let match_ = sync::block_on(match_.lock());
                    let match_ = match &*match_ {
                        Some(match_) => match_,
                        _ => {
                            return;
                        }
                    };

                    let mut round_state = sync::block_on(match_.lock_round_state());
                    round_state.set_last_result(battle::BattleResult::Win);
                })
            }),
            (self.offsets.rom.round_end_set_loss, {
                let match_ = match_.clone();
                Box::new(move |_| {
                    let match_ = sync::block_on(match_.lock());
                    let match_ = match &*match_ {
                        Some(match_) => match_,
                        _ => {
                            return;
                        }
                    };

                    let mut round_state = sync::block_on(match_.lock_round_state());
                    round_state.set_last_result(battle::BattleResult::Loss);
                })
            }),
            (self.offsets.rom.round_end_damage_judge_set_win, {
                let match_ = match_.clone();
                Box::new(move |_| {
                    let match_ = sync::block_on(match_.lock());
                    let match_ = match &*match_ {
                        Some(match_) => match_,
                        _ => {
                            return;
                        }
                    };

                    let mut round_state = sync::block_on(match_.lock_round_state());
                    round_state.set_last_result(battle::BattleResult::Win);
                })
            }),
            (self.offsets.rom.round_end_damage_judge_set_loss, {
                let match_ = match_.clone();
                Box::new(move |_| {
                    let match_ = sync::block_on(match_.lock());
                    let match_ = match &*match_ {
                        Some(match_) => match_,
                        _ => {
                            return;
                        }
                    };

                    let mut round_state = sync::block_on(match_.lock_round_state());
                    round_state.set_last_result(battle::BattleResult::Loss);
                })
            }),
            (self.offsets.rom.round_end_damage_judge_set_draw, {
                let match_ = match_.clone();
                Box::new(move |_| {
                    let match_ = sync::block_on(match_.lock());
                    let match_ = match &*match_ {
                        Some(match_) => match_,
                        _ => {
                            return;
                        }
                    };

                    let mut round_state = sync::block_on(match_.lock_round_state());
                    let result = {
                        let round = round_state.round.as_ref().expect("round");
                        round.on_draw_result()
                    };
                    round_state.set_last_result(result);
                })
            }),
            (self.offsets.rom.round_ending_entry1, {
                let match_ = match_.clone();

                Box::new(move |mut _core| {
                    let match_ = sync::block_on(match_.lock());
                    let match_ = match &*match_ {
                        Some(match_) => match_,
                        None => {
                            return;
                        }
                    };

                    // This is level-triggered because otherwise it's a massive pain to deal with.
                    let mut round_state = sync::block_on(match_.lock_round_state());
                    if round_state.round.is_none() {
                        return;
                    }

                    sync::block_on(round_state.end_round()).expect("end round");
                    sync::block_on(match_.advance_shadow_until_round_end()).expect("advance shadow");
                })
            }),
            (self.offsets.rom.round_ending_entry2, {
                let match_ = match_.clone();
                Box::new(move |mut _core| {
                    let match_ = sync::block_on(match_.lock());
                    let match_ = match &*match_ {
                        Some(match_) => match_,
                        None => {
                            return;
                        }
                    };

                    // This is level-triggered because otherwise it's a massive pain to deal with.
                    let mut round_state = sync::block_on(match_.lock_round_state());
                    if round_state.round.is_none() {
                        return;
                    }

                    sync::block_on(round_state.end_round()).expect("end round");
                    sync::block_on(match_.advance_shadow_until_round_end()).expect("advance shadow");
                })
            }),
            {
                let match_ = match_.clone();
                (
                    self.offsets.rom.round_start_ret,
                    Box::new(move |_core| {
                        let match_ = sync::block_on(match_.lock());
                        let match_ = match &*match_ {
                            Some(match_) => match_,
                            _ => {
                                return;
                            }
                        };
                        sync::block_on(match_.start_round()).expect("start round");
                    }),
                )
            },
            (self.offsets.rom.link_is_p2_ret, {
                let match_ = match_.clone();
                Box::new(move |mut core| {
                    let match_ = sync::block_on(match_.lock());
                    let match_ = match &*match_ {
                        Some(match_) => match_,
                        _ => {
                            return;
                        }
                    };

                    let round_state = sync::block_on(match_.lock_round_state());
                    let round = match round_state.round.as_ref() {
                        Some(round) => round,
                        None => {
                            return;
                        }
                    };

                    core.gba_mut().cpu_mut().set_gpr(0, round.local_player_index() as i32);
                })
            }),
            (self.offsets.rom.main_read_joyflags, {
                let match_ = match_.clone();
                let munger = self.munger();
                Box::new(move |core| {
                    let match_ = sync::block_on(match_.lock());
                    let match_ = match &*match_ {
                        Some(match_) => match_,
                        _ => {
                            return;
                        }
                    };

                    let mut round_state = sync::block_on(match_.lock_round_state());

                    let round = match round_state.round.as_mut() {
                        Some(round) => round,
                        None => {
                            return;
                        }
                    };

                    if !munger.is_linking(core) {
                        return;
                    }

                    if !round.has_committed_state() {
                        let mut rng = sync::block_on(match_.lock_rng());
                        let rng_state = generate_rng_state(&mut *rng);
                        munger.set_rng_state(core, rng_state);

                        round.set_first_committed_state(
                            core.save_state().expect("save state"),
                            sync::block_on(match_.advance_shadow_until_first_committed_state())
                                .expect("shadow save state"),
                            &munger.tx_packet(core),
                        );
                        log::info!("primary rng state: {:08x}", munger.rng_state(core));
                        log::info!("battle state committed on {}", round.current_tick());
                    }

                    'abort: loop {
                        if let Err(e) = sync::block_on(round.add_local_input_and_fastforward(
                            core,
                            joyflags.load(std::sync::atomic::Ordering::Relaxed) as u16,
                        )) {
                            log::error!("failed to add local input: {}", e);
                            break 'abort;
                        }
                        return;
                    }
                    match_.cancel();
                })
            }),
            (
                self.offsets.rom.handle_input_custom_send_and_receive_call,
                make_send_and_receive_call_hook(),
            ),
            (
                self.offsets.rom.handle_input_in_turn_send_and_receive_call,
                make_send_and_receive_call_hook(),
            ),
            (self.offsets.rom.comm_menu_send_and_receive_call, {
                let munger = self.munger();

                Box::new(move |mut core| {
                    let pc = core.as_ref().gba().cpu().thumb_pc();
                    core.gba_mut().cpu_mut().set_thumb_pc(pc + 4);
                    core.gba_mut().cpu_mut().set_gpr(0, 3);
                    munger.set_rx_packet(core, 0, &INIT_RX);
                    munger.set_rx_packet(core, 1, &INIT_RX);
                })
            }),
            (
                self.offsets.rom.init_sio_call,
                Box::new(move |mut core| {
                    let pc = core.as_ref().gba().cpu().thumb_pc();
                    core.gba_mut().cpu_mut().set_thumb_pc(pc + 4);
                }),
            ),
            (self.offsets.rom.round_call_jump_table_ret, {
                let match_ = match_.clone();

                Box::new(move |_| {
                    let match_ = sync::block_on(match_.lock());
                    let match_ = match &*match_ {
                        Some(match_) => match_,
                        _ => {
                            return;
                        }
                    };

                    let mut round_state = sync::block_on(match_.lock_round_state());

                    let round = match round_state.round.as_mut() {
                        Some(round) => round,
                        None => {
                            return;
                        }
                    };

                    if !round.has_committed_state() {
                        return;
                    }

                    round.increment_current_tick();
                })
            }),
        ]
    }

    fn shadow_traps(&self, shadow_state: shadow::State) -> Vec<(u32, Box<dyn Fn(mgba::core::CoreMutRef)>)> {
        let make_send_and_receive_call_hook = || {
            let shadow_state = shadow_state.clone();
            let munger = self.munger();

            Box::new(move |mut core: mgba::core::CoreMutRef| {
                let pc = core.as_ref().gba().cpu().thumb_pc();
                core.gba_mut().cpu_mut().set_thumb_pc(pc + 4);

                let mut round_state = shadow_state.lock_round_state();
                let round = match round_state.round.as_mut() {
                    Some(round) => round,
                    None => {
                        core.gba_mut().cpu_mut().set_gpr(0, 0);
                        return;
                    }
                };
                core.gba_mut().cpu_mut().set_gpr(0, 3);

                let ip = if let Some(ip) = round.take_shadow_input() {
                    ip
                } else {
                    return;
                };

                // HACK: This is required if the emulator advances beyond read joyflags and runs this function again, but is missing input data.
                // We permit this for one tick only, but really we should just not be able to get into this situation in the first place.
                if ip.local.local_tick + 1 == round.current_tick() {
                    return;
                }

                if ip.local.local_tick != ip.remote.local_tick {
                    shadow_state.set_anyhow_error(anyhow::anyhow!(
                        "copy input data: local tick != remote tick (in battle tick = {}): {} != {}",
                        round.current_tick(),
                        ip.local.local_tick,
                        ip.remote.local_tick
                    ));
                    return;
                }

                if ip.local.local_tick != round.current_tick() {
                    shadow_state.set_anyhow_error(anyhow::anyhow!(
                        "copy input data: input tick != in battle tick: {} != {}",
                        ip.local.local_tick,
                        round.current_tick(),
                    ));
                    return;
                }

                let remote_packet = round.peek_remote_packet().unwrap();
                if remote_packet.tick != round.current_tick() {
                    shadow_state.set_anyhow_error(anyhow::anyhow!(
                        "copy input data: local packet tick != in battle tick: {} != {}",
                        remote_packet.tick,
                        round.current_tick(),
                    ));
                    return;
                }

                munger.set_rx_packet(
                    core,
                    round.local_player_index() as u32,
                    &ip.local.packet.try_into().unwrap(),
                );
                munger.set_rx_packet(
                    core,
                    round.remote_player_index() as u32,
                    &remote_packet.packet.clone().try_into().unwrap(),
                );
                round.set_remote_packet(round.current_tick() + 1, munger.tx_packet(core).to_vec());
                round.set_input_injected();
            })
        };

        vec![
            (self.offsets.rom.comm_menu_init_ret, {
                let munger = self.munger();
                let shadow_state = shadow_state.clone();
                Box::new(move |core| {
                    let mut rng = shadow_state.lock_rng();
                    let offerer_rng_state = generate_rng_state(&mut *rng);
                    let answerer_rng_state = generate_rng_state(&mut *rng);
                    munger.set_rng_state(
                        core,
                        if shadow_state.is_offerer() {
                            answerer_rng_state
                        } else {
                            offerer_rng_state
                        },
                    );
                    munger.start_battle_from_comm_menu(core, random_background(&mut *rng));
                })
            }),
            (self.offsets.rom.round_start_ret, {
                let shadow_state = shadow_state.clone();
                Box::new(move |_| {
                    shadow_state.start_round();
                })
            }),
            (self.offsets.rom.round_end_set_win, {
                let shadow_state = shadow_state.clone();
                Box::new(move |_| {
                    let mut round_state = shadow_state.lock_round_state();
                    round_state.set_last_result(battle::BattleResult::Loss);
                })
            }),
            (self.offsets.rom.round_end_set_loss, {
                let shadow_state = shadow_state.clone();
                Box::new(move |_| {
                    let mut round_state = shadow_state.lock_round_state();
                    round_state.set_last_result(battle::BattleResult::Win);
                })
            }),
            (self.offsets.rom.round_end_damage_judge_set_win, {
                let shadow_state = shadow_state.clone();
                Box::new(move |_| {
                    let mut round_state = shadow_state.lock_round_state();
                    round_state.set_last_result(battle::BattleResult::Loss);
                })
            }),
            (self.offsets.rom.round_end_damage_judge_set_loss, {
                let shadow_state = shadow_state.clone();
                Box::new(move |_| {
                    let mut round_state = shadow_state.lock_round_state();
                    round_state.set_last_result(battle::BattleResult::Win);
                })
            }),
            (self.offsets.rom.round_end_damage_judge_set_draw, {
                let shadow_state = shadow_state.clone();
                Box::new(move |_| {
                    let mut round_state = shadow_state.lock_round_state();
                    let result = {
                        let round = round_state.round.as_mut().expect("round");
                        round.on_draw_result()
                    };
                    round_state.set_last_result(result);
                })
            }),
            (self.offsets.rom.round_end_entry, {
                let shadow_state = shadow_state.clone();
                Box::new(move |core| {
                    shadow_state.end_round();
                    shadow_state.set_applied_state(core.save_state().expect("save state"), 0);
                })
            }),
            (self.offsets.rom.link_is_p2_ret, {
                let shadow_state = shadow_state.clone();
                Box::new(move |mut core| {
                    let round_state = shadow_state.lock_round_state();
                    let round = match round_state.round.as_ref() {
                        Some(round) => round,
                        None => {
                            return;
                        }
                    };

                    core.gba_mut().cpu_mut().set_gpr(0, round.remote_player_index() as i32);
                })
            }),
            (self.offsets.rom.main_read_joyflags, {
                let shadow_state = shadow_state.clone();
                let munger = self.munger();
                Box::new(move |mut core| {
                    let mut round_state = shadow_state.lock_round_state();
                    let round = match round_state.round.as_mut() {
                        Some(round) => round,
                        None => {
                            return;
                        }
                    };

                    if !munger.is_linking(core) && !round.has_first_committed_state() {
                        let mut rng = shadow_state.lock_rng();
                        let rng_state = generate_rng_state(&mut *rng);
                        munger.set_rng_state(core, rng_state);
                        return;
                    }

                    if !round.has_first_committed_state() {
                        round
                            .set_first_committed_state(core.save_state().expect("save state"), &munger.tx_packet(core));
                        log::info!("shadow rng state: {:08x}", munger.rng_state(core));
                        log::info!("shadow state committed on {}", round.current_tick());
                        return;
                    }

                    if let Some(ip) = round.peek_shadow_input().clone() {
                        if ip.local.local_tick != ip.remote.local_tick {
                            shadow_state.set_anyhow_error(anyhow::anyhow!(
                                "read joyflags: local tick != remote tick (in battle tick = {}): {} != {}",
                                round.current_tick(),
                                ip.local.local_tick,
                                ip.remote.local_tick
                            ));
                            return;
                        }

                        if ip.local.local_tick != round.current_tick() {
                            shadow_state.set_anyhow_error(anyhow::anyhow!(
                                "read joyflags: input tick != in battle tick: {} != {}",
                                ip.local.local_tick,
                                round.current_tick(),
                            ));
                            return;
                        }

                        core.gba_mut()
                            .cpu_mut()
                            .set_gpr(4, (ip.remote.joyflags | 0xfc00) as i32);
                    }

                    if round.take_input_injected() {
                        shadow_state.set_applied_state(core.save_state().expect("save state"), round.current_tick());
                    }
                })
            }),
            (
                self.offsets.rom.handle_input_custom_send_and_receive_call,
                make_send_and_receive_call_hook(),
            ),
            (
                self.offsets.rom.handle_input_in_turn_send_and_receive_call,
                make_send_and_receive_call_hook(),
            ),
            (self.offsets.rom.comm_menu_send_and_receive_call, {
                let munger = self.munger();
                Box::new(move |mut core| {
                    let pc = core.as_ref().gba().cpu().thumb_pc();
                    core.gba_mut().cpu_mut().set_thumb_pc(pc + 4);
                    core.gba_mut().cpu_mut().set_gpr(0, 3);
                    munger.set_rx_packet(core, 0, &INIT_RX);
                    munger.set_rx_packet(core, 1, &INIT_RX);
                })
            }),
            (
                self.offsets.rom.init_sio_call,
                Box::new(move |mut core| {
                    let pc = core.as_ref().gba().cpu().thumb_pc();
                    core.gba_mut().cpu_mut().set_thumb_pc(pc + 4);
                }),
            ),
            (self.offsets.rom.round_call_jump_table_ret, {
                let shadow_state = shadow_state.clone();
                Box::new(move |_core| {
                    let mut round_state = shadow_state.lock_round_state();
                    let round = match round_state.round.as_mut() {
                        Some(round) => round,
                        None => {
                            return;
                        }
                    };
                    if !round.has_first_committed_state() {
                        return;
                    }
                    round.increment_current_tick();
                })
            }),
        ]
    }

    fn replayer_traps(&self, replayer_state: replayer::State) -> Vec<(u32, Box<dyn Fn(mgba::core::CoreMutRef)>)> {
        let make_send_and_receive_call_hook = || {
            let munger = self.munger();
            let replayer_state = replayer_state.clone();
            Box::new(move |mut core: mgba::core::CoreMutRef| {
                let mut replayer_state = replayer_state.lock_inner();

                let pc = core.as_ref().gba().cpu().thumb_pc();
                core.gba_mut().cpu_mut().set_thumb_pc(pc + 4);
                core.gba_mut().cpu_mut().set_gpr(0, 3);

                if replayer_state.is_round_ending() {
                    return;
                }

                let current_tick = replayer_state.current_tick();

                let ip = match replayer_state.pop_input_pair() {
                    Some(ip) => ip,
                    None => {
                        let mut rx = [
                            0x05, 0x00, 0x00, 0xfc, 0x00, 0x00, 0x00, 0xfc, 0x00, 0xfc, 0x00, 0x00, 0xff, 0xff, 0xff,
                            0xff,
                        ];
                        byteorder::LittleEndian::write_u32(&mut rx[0xc..0x10], munger.packet_seqnum(core));
                        munger.set_rx_packet(core, 0, &rx);
                        munger.set_rx_packet(core, 1, &rx);
                        return;
                    }
                };

                if ip.local.local_tick != ip.remote.local_tick {
                    replayer_state.set_anyhow_error(anyhow::anyhow!(
                        "copy input data: local tick != remote tick (in battle tick = {}): {} != {}",
                        current_tick,
                        ip.local.local_tick,
                        ip.remote.local_tick
                    ));
                    return;
                }

                if ip.local.local_tick != current_tick {
                    replayer_state.set_anyhow_error(anyhow::anyhow!(
                        "copy input data: input tick != in battle tick: {} != {}",
                        ip.local.local_tick,
                        current_tick,
                    ));
                    return;
                }

                let local_packet = replayer_state.peek_local_packet().unwrap().clone();
                if local_packet.tick != current_tick {
                    replayer_state.set_anyhow_error(anyhow::anyhow!(
                        "copy input data: local packet tick != in battle tick: {} != {}",
                        local_packet.tick,
                        current_tick,
                    ));
                    return;
                }

                munger.set_rx_packet(
                    core,
                    replayer_state.local_player_index() as u32,
                    &local_packet.packet.clone().try_into().unwrap(),
                );
                munger.set_rx_packet(
                    core,
                    replayer_state.remote_player_index() as u32,
                    &replayer_state
                        .apply_shadow_input(lockstep::Pair {
                            local: ip.local.with_packet(local_packet.packet),
                            remote: ip.remote,
                        })
                        .expect("apply shadow input")
                        .try_into()
                        .unwrap(),
                );
                replayer_state.set_local_packet(current_tick + 1, munger.tx_packet(core).to_vec());
            })
        };

        vec![
            (self.offsets.rom.battle_start_play_music_call, {
                let replayer_state = replayer_state.clone();
                Box::new(move |mut core| {
                    let replayer_state = replayer_state.lock_inner();
                    if !replayer_state.disable_bgm() {
                        return;
                    }
                    let pc = core.as_ref().gba().cpu().thumb_pc() as u32;
                    core.gba_mut().cpu_mut().set_thumb_pc(pc + 4);
                })
            }),
            (self.offsets.rom.link_is_p2_ret, {
                let replayer_state = replayer_state.clone();
                Box::new(move |mut core| {
                    let replayer_state = replayer_state.lock_inner();
                    core.gba_mut()
                        .cpu_mut()
                        .set_gpr(0, replayer_state.local_player_index() as i32);
                })
            }),
            (self.offsets.rom.round_ending_entry1, {
                let replayer_state = replayer_state.clone();
                Box::new(move |_core| {
                    let mut replayer_state = replayer_state.lock_inner();
                    if replayer_state.is_round_ending() {
                        return;
                    }
                    replayer_state.set_round_ending();
                })
            }),
            (self.offsets.rom.round_ending_entry2, {
                let replayer_state = replayer_state.clone();
                Box::new(move |_core| {
                    let mut replayer_state = replayer_state.lock_inner();
                    if replayer_state.is_round_ending() {
                        return;
                    }
                    replayer_state.set_round_ending();
                })
            }),
            (self.offsets.rom.round_end_entry, {
                let replayer_state = replayer_state.clone();
                Box::new(move |_core| {
                    let mut replayer_state = replayer_state.lock_inner();
                    replayer_state.set_round_ended();
                })
            }),
            (self.offsets.rom.main_read_joyflags, {
                let replayer_state = replayer_state.clone();
                Box::new(move |mut core| {
                    let mut replayer_state = replayer_state.lock_inner();
                    let current_tick = replayer_state.current_tick();

                    if current_tick == replayer_state.commit_tick() {
                        replayer_state.set_committed_state(core.save_state().expect("save committed state"));
                    }

                    let ip = match replayer_state.peek_input_pair() {
                        Some(ip) => ip.clone(),
                        None => {
                            return;
                        }
                    };

                    if ip.local.local_tick != ip.remote.local_tick {
                        replayer_state.set_anyhow_error(anyhow::anyhow!(
                            "read joyflags: local tick != remote tick (in battle tick = {}): {} != {}",
                            current_tick,
                            ip.local.local_tick,
                            ip.remote.local_tick
                        ));
                        return;
                    }

                    if ip.local.local_tick != current_tick {
                        replayer_state.set_anyhow_error(anyhow::anyhow!(
                            "read joyflags: input tick != in battle tick: {} != {}",
                            ip.local.local_tick,
                            current_tick,
                        ));
                        return;
                    }

                    core.gba_mut().cpu_mut().set_gpr(4, (ip.local.joyflags | 0xfc00) as i32);

                    if current_tick == replayer_state.dirty_tick() {
                        replayer_state.set_dirty_state(core.save_state().expect("save dirty state"));
                    }
                })
            }),
            (
                self.offsets.rom.handle_input_custom_send_and_receive_call,
                make_send_and_receive_call_hook(),
            ),
            (
                self.offsets.rom.handle_input_in_turn_send_and_receive_call,
                make_send_and_receive_call_hook(),
            ),
            (self.offsets.rom.round_call_jump_table_ret, {
                let replayer_state = replayer_state.clone();
                Box::new(move |_| {
                    let mut replayer_state = replayer_state.lock_inner();
                    replayer_state.increment_current_tick();
                })
            }),
            (self.offsets.rom.round_end_set_win, {
                let replayer_state = replayer_state.clone();
                Box::new(move |_| {
                    let mut replayer_state = replayer_state.lock_inner();
                    replayer_state.set_round_result(replayer::BattleResult::Win);
                })
            }),
            (self.offsets.rom.round_end_set_loss, {
                let replayer_state = replayer_state.clone();
                Box::new(move |_| {
                    let mut replayer_state = replayer_state.lock_inner();
                    replayer_state.set_round_result(replayer::BattleResult::Loss);
                })
            }),
            (self.offsets.rom.round_end_damage_judge_set_win, {
                let replayer_state = replayer_state.clone();
                Box::new(move |_| {
                    let mut replayer_state = replayer_state.lock_inner();
                    replayer_state.set_round_result(replayer::BattleResult::Win);
                })
            }),
            (self.offsets.rom.round_end_damage_judge_set_loss, {
                let replayer_state = replayer_state.clone();
                Box::new(move |_| {
                    let mut replayer_state = replayer_state.lock_inner();
                    replayer_state.set_round_result(replayer::BattleResult::Loss);
                })
            }),
            (self.offsets.rom.round_end_damage_judge_set_draw, {
                let replayer_state = replayer_state.clone();
                Box::new(move |_| {
                    let mut replayer_state = replayer_state.lock_inner();
                    replayer_state.set_round_result(replayer::BattleResult::Draw);
                })
            }),
        ]
    }

    fn predict_rx(&self, rx: &mut Vec<u8>) {
        match rx[0] {
            0x05 => {
                let tick = byteorder::LittleEndian::read_u32(&rx[0xc..0x10]);
                byteorder::LittleEndian::write_u32(&mut rx[0xc..0x10], tick + 1);
            }
            _ => {}
        }
    }

    fn prepare_for_fastforward(&self, mut core: mgba::core::CoreMutRef) {
        core.gba_mut()
            .cpu_mut()
            .set_thumb_pc(self.offsets.rom.main_read_joyflags);
    }
}
