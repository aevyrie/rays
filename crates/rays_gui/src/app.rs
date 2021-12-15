use crossbeam_channel::Receiver;
use eframe::{
    egui::{self, Color32, TextureId, Vec2},
    epi,
};
use rays_core::Pixel;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
#[derive(Default)]
pub struct RaysApp {
    // this how you opt-out of serialization of a member
    //#[cfg_attr(feature = "persistence", serde(skip))]
    texture: Option<TextureId>,
    buffer: Vec<Color32>,
    size: Vec2,
    receiver: Option<Receiver<Pixel>>,
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
        if let Some(ref rx) = self.receiver {
            for pixel in rx.try_iter() {
                let [r, g, b, a] = pixel.color;
                self.buffer[pixel.position[0]
                    + (self.size.y as usize - pixel.position[1] - 1) * self.size.x as usize] =
                    Color32::from_rgba_unmultiplied(r, g, b, a);
                self.texture = None;
            }
        }
        self.texture = if self.texture.is_some() {
            self.texture
        } else if self.size != Vec2::ZERO {
            // Allocate a texture:
            let texture_id = frame.tex_allocator().alloc_srgba_premultiplied(
                (self.size.x as usize, self.size.y as usize),
                &self.buffer,
            );
            if let Some(texture) = self.texture {
                frame.tex_allocator().free(texture)
            }
            self.texture = Some(texture_id);
            ctx.request_repaint();
            Some(texture_id)
        } else {
            None
        };
        egui::SidePanel::left("left panel").show(ctx, |ui| {
            ui.label("Some text");
            if ui.button("Submit").clicked() {
                let (width, height) = (256, 256);
                self.buffer.clear();
                let rx = rays_core::PathTracer::build([width, height]).run();
                self.receiver = Some(rx);
                self.texture = None;
                self.buffer = vec![Color32::TRANSPARENT; width * height];
                self.size = Vec2::new(width as f32, height as f32);
            }
        });
        egui::Area::new("main area").show(ctx, |ui| {
            if let Some(texture) = self.texture {
                ui.add(egui::Image::new(
                    texture,
                    [ctx.available_rect().width(), ctx.available_rect().height()],
                ));
            }
        });
    }
}
