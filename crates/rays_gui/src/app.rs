use crossbeam_channel::Receiver;
use eframe::{
    egui::{
        self,
        plot::{self, Plot, PlotImage},
        CentralPanel, Color32, CtxRef, DragValue, Layout, TextureId, TopBottomPanel,
    },
    epi,
};
use rays_core::{PathTracer, Pixel};

//mod event_system;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct RaysApp {
    // this how you opt-out of serialization of a member
    //#[cfg_attr(feature = "persistence", serde(skip))]
    texture: Option<TextureId>,
    buffer: Vec<Color32>,
    width: usize,
    height: usize,
    grid: bool,
    receiver: Option<Receiver<Pixel>>,
}
impl Default for RaysApp {
    fn default() -> Self {
        RaysApp {
            texture: None,
            buffer: Vec::new(),
            width: 800,
            height: 600,
            grid: false,
            receiver: None,
        }
    }
}

impl epi::App for RaysApp {
    fn name(&self) -> &str {
        "Rays"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &CtxRef,
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
        let RaysApp {
            texture,
            buffer,
            width,
            height,
            grid,
            receiver,
        } = self;

        update_texture(texture, buffer, width, height, receiver, frame, ctx);

        // Build UI
        ctx.set_debug_on_hover(cfg!(debug_assertions));

        TopBottomPanel::top("top panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Add widgets left to right, from the left edge
                ui.label("Resolution");
                ui.add(
                    DragValue::new(width)
                        .speed(1.0)
                        .fixed_decimals(0)
                        .clamp_range(1..=10_000),
                );
                ui.label("x");
                ui.add(
                    DragValue::new(height)
                        .speed(1.0)
                        .fixed_decimals(0)
                        .clamp_range(1..=10_000),
                );
                ui.add_space(10.0);
                ui.checkbox(grid, "Grid");
                // Add widgets right to left, from the right edge
                ui.expand_to_include_rect(ui.available_rect_before_wrap());
                ui.with_layout(Layout::right_to_left(), |ui| {
                    if ui.button("Render").clicked() {
                        buffer.clear();
                        *receiver = Some(PathTracer::build([*width, *height]).run());
                        *texture = None;
                        *buffer = vec![Color32::TRANSPARENT; *width * *height];
                    };
                });
            });
        });

        CentralPanel::default()
            .frame(egui::Frame {
                fill: ctx.style().visuals.extreme_bg_color,
                ..Default::default()
            })
            .show(ctx, |ui| {
                //CentralPanel::default().show_inside(ui, |ui| {
                if let Some(texture) = self.texture {
                    let image = PlotImage::new(
                        texture,
                        plot::Value::new(0.0, 0.0),
                        [*width as f32, *height as f32],
                    );
                    let plot = Plot::new("Image area")
                        .show_x(false)
                        .show_y(false)
                        .show_background(false)
                        .show_axes([*grid, *grid])
                        .data_aspect(1.0);
                    plot.show(ui, |plot_ui| {
                        plot_ui.image(image.name("Render result"));
                    })
                    .response;
                }
                //});
            });
    }
}

fn update_texture(
    texture: &mut Option<TextureId>,
    buffer: &mut Vec<Color32>,
    width: &mut usize,
    height: &mut usize,
    receiver: &mut Option<Receiver<Pixel>>,
    frame: &mut epi::Frame<'_>,
    ctx: &egui::CtxRef,
) {
    if let Some(ref rx) = receiver {
        for pixel in rx.try_iter() {
            let [r, g, b, a] = pixel.color;
            buffer[pixel.position[0] + pixel.position[1] * *width] =
                Color32::from_rgba_unmultiplied(r, g, b, a);
            *texture = None;
        }
    }
    *texture = if texture.is_some() {
        *texture
    } else if *width != 0 && *height != 0 && buffer.len() == *width * *height {
        // Allocate a texture:
        let texture_id = frame
            .tex_allocator()
            .alloc_srgba_premultiplied((*width, *height), &buffer);
        if let Some(texture) = texture {
            frame.tex_allocator().free(*texture)
        }
        *texture = Some(texture_id);
        ctx.request_repaint();
        Some(texture_id)
    } else {
        None
    };
}
