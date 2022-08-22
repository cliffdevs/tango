use crate::game;

pub struct Gui {
    vbuf: Vec<u8>,
    vbuf_texture: Option<egui::TextureHandle>,
}

impl Gui {
    pub fn new() -> Self {
        Self {
            vbuf: vec![],
            vbuf_texture: None,
        }
    }

    pub fn draw(&mut self, ctx: &egui::Context, state: &mut game::State) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    if let Some(session) = &state.session {
                        // Apply stupid video scaling filter that only mint wants 🥴
                        let (vbuf_width, vbuf_height) = state.video_filter.output_size((
                            mgba::gba::SCREEN_WIDTH as usize,
                            mgba::gba::SCREEN_HEIGHT as usize,
                        ));

                        let make_vbuf_texture = || {
                            ctx.load_texture(
                                "vbuf",
                                egui::ColorImage::new(
                                    [vbuf_width, vbuf_height],
                                    egui::Color32::BLACK,
                                ),
                                egui::TextureFilter::Nearest,
                            )
                        };

                        let vbuf_texture = self.vbuf_texture.get_or_insert_with(make_vbuf_texture);
                        if vbuf_texture.size() != [vbuf_width, vbuf_height] {
                            *vbuf_texture = make_vbuf_texture();
                        }

                        if self.vbuf.len() != vbuf_width * vbuf_height * 4 {
                            self.vbuf = vec![0u8; vbuf_width * vbuf_height * 4];
                            log::info!("vbuf reallocated to ({}, {})", vbuf_width, vbuf_height);
                        }

                        state.video_filter.apply(
                            &session.lock_vbuf(),
                            &mut self.vbuf,
                            (
                                mgba::gba::SCREEN_WIDTH as usize,
                                mgba::gba::SCREEN_HEIGHT as usize,
                            ),
                        );

                        vbuf_texture.set(
                            egui::ColorImage::from_rgba_unmultiplied(
                                [vbuf_width, vbuf_height],
                                &self.vbuf,
                            ),
                            egui::TextureFilter::Nearest,
                        );

                        let scaling_factor = std::cmp::max_by(
                            std::cmp::min_by(
                                ui.available_width() / mgba::gba::SCREEN_WIDTH as f32,
                                ui.available_height() / mgba::gba::SCREEN_HEIGHT as f32,
                                |a, b| a.partial_cmp(b).unwrap(),
                            )
                            .floor(),
                            1.0,
                            |a, b| a.partial_cmp(b).unwrap(),
                        );
                        ui.image(
                            &*vbuf_texture,
                            egui::Vec2::new(
                                mgba::gba::SCREEN_WIDTH as f32 * scaling_factor as f32,
                                mgba::gba::SCREEN_HEIGHT as f32 * scaling_factor as f32,
                            ),
                        );
                    }
                },
            );
        });
    }
}
