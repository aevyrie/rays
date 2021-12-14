use eframe::{egui, epi};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
#[derive(Default)]
pub struct RaysApp {
    // this how you opt-out of serialization of a member
    //#[cfg_attr(feature = "persistence", serde(skip))]
    texture: Option<(egui::Vec2, egui::TextureId)>,
}

impl epi::App for RaysApp {
    fn name(&self) -> &str {
        "Rays"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }
    }

    /// Called by the frame work to save state before shutdown.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        egui::SidePanel::left("left panel").show(ctx, |ui| {
            ui.label("Some text");
        });
        egui::Area::new("main area").show(ctx, |ui| {
            if let Some(texture) = self.texture {
                ui.add(egui::Image::new(
                    texture.1,
                    [ctx.available_rect().width(), ctx.available_rect().height()],
                ));
            } else {
                let size =
                    egui::Vec2::new(ctx.available_rect().width(), ctx.available_rect().height());
                let pixels: Vec<_> = vec![100; size.x as usize * size.y as usize * 4]
                    .chunks_exact(4)
                    .enumerate()
                    .map(|(n, _p)| {
                        egui::Color32::from_rgba_unmultiplied(
                            (255.0 * (n as f32 / size.y) / size.y) as u8,
                            (255.0 * (n as f32 % size.x) / size.y) as u8,
                            127,
                            255,
                        )
                    })
                    .collect();
                let texture_size = size;

                // Allocate a texture:
                let texture_id = frame
                    .tex_allocator()
                    .alloc_srgba_premultiplied((size.x as usize, size.y as usize), &pixels);

                self.texture = Some((texture_size, texture_id));
            }
        });
    }
}
