use std::sync::Arc;

use crossbeam_channel::Receiver;
use eframe::{
    egui::{
        self,
        plot::{self, Plot, PlotImage},
        CentralPanel, Color32, Context, DragValue, SidePanel,
    },
    emath::{Pos2, Rect},
    epaint::{ColorImage, ImageDelta, TextureHandle},
    App, CreationContext, Frame,
};
use glam::Vec3;
use rays_core::{
    material::{Lambertian, Metal},
    Camera, PathTracer, Pixel, Scene, SdfObject, Sphere,
};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct RaysApp {
    // this how you opt-out of serialization of a member
    //#[cfg_attr(feature = "persistence", serde(skip))]
    texture: TextureHandle,
    buffer: Vec<u8>,
    input_width: u32,
    input_height: u32,
    samples: usize,
    max_bounces: u8,
    grid: bool,
    receiver: Receiver<Pixel>,
    scene: Scene,
}
impl RaysApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        let input_width = 100u32;
        let input_height = 80u32;

        let texture = cc.egui_ctx.load_texture(
            "render area",
            ColorImage::new(
                [input_width as usize, input_height as usize],
                Color32::TRANSPARENT,
            ),
            egui::TextureFilter::Nearest,
        );

        let matl1 = Arc::new(Lambertian::new([0.99, 0.1, 0.1, 1.0].into()));
        let matl2 = Arc::new(Lambertian::new([0.1, 0.9, 0.2, 1.0].into()));
        let matl3 = Arc::new(Metal::new([0.1, 0.1, 0.9, 1.0].into()));
        let matl4 = Arc::new(Metal::new([0.3, 0.3, 0.3, 1.0].into()));

        let scene = Scene {
            camera: Camera::from_aspect_ratio(input_width as f32 / input_height as f32),
            objects: vec![
                SdfObject::new(Sphere::new(Vec3::new(0.0, 0.0, -1.0), 0.5), matl1),
                SdfObject::new(Sphere::new(Vec3::new(1.0, 0.0, -1.0), 0.5), matl3),
                SdfObject::new(Sphere::new(Vec3::new(-1.0, 0.0, -1.0), 0.5), matl4),
                SdfObject::new(Sphere::new(Vec3::new(0.0, -100.5, -1.0), 100.0), matl2),
            ],
            materials: vec![],
        };

        let samples = 32;
        let max_bounces = 16;

        RaysApp {
            texture,
            buffer: vec![0; (input_width * input_height * 4) as usize],
            grid: true,
            receiver: PathTracer::build([input_width, input_height]).run(
                scene.clone(),
                samples,
                max_bounces,
            ),
            scene,
            input_width,
            input_height,
            samples,
            max_bounces,
        }
    }
}
impl App for RaysApp {
    /// Called by the frame work to save state before shutdown.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, context: &Context, _frame: &mut Frame) {
        let RaysApp {
            texture,
            buffer,
            input_width,
            input_height,
            samples,
            max_bounces,
            grid,
            receiver,
            scene,
        } = self;

        update_texture(texture, buffer, receiver, context);

        // Build UI
        context.set_debug_on_hover(cfg!(debug_assertions));

        SidePanel::right("right panel").show(context, |ui| {
            ui.vertical(|ui| {
                ui.collapsing("Resolution", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Width");
                        ui.add(
                            DragValue::new(input_width)
                                .speed(10.0)
                                .fixed_decimals(0)
                                .clamp_range(1..=10_000usize),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Height");
                        ui.add(
                            DragValue::new(input_height)
                                .speed(10.0)
                                .fixed_decimals(0)
                                .clamp_range(1..=10_000usize),
                        );
                    });
                });
                ui.collapsing("Quality", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Max bounces:");
                        ui.add(
                            DragValue::new(max_bounces)
                                .speed(1.0)
                                .fixed_decimals(0)
                                .clamp_range(1..=255usize),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Samples:");
                        ui.add(
                            DragValue::new(samples)
                                .speed(1.0)
                                .fixed_decimals(0)
                                .clamp_range(1..=100_000usize),
                        );
                    });
                });

                ui.add_space(10.0);
                ui.checkbox(grid, "Grid");

                if ui.button("Render").clicked() {
                    buffer.clear();
                    scene.camera =
                        Camera::from_aspect_ratio(*input_width as f32 / *input_height as f32);
                    *receiver = PathTracer::build([*input_width, *input_height]).run(
                        scene.to_owned(),
                        *samples,
                        *max_bounces,
                    );
                    *texture = context.load_texture(
                        "render area",
                        ColorImage::new(
                            [*input_width as usize, *input_height as usize],
                            Color32::TRANSPARENT,
                        ),
                        egui::TextureFilter::Nearest,
                    );
                    *buffer = vec![0; (*input_width * *input_height * 4) as usize];
                };
            });
        });

        CentralPanel::default()
            .frame(egui::Frame {
                fill: context.style().visuals.extreme_bg_color,
                ..Default::default()
            })
            .show(context, |ui| {
                //CentralPanel::default().show_inside(ui, |ui| {
                let image = PlotImage::new(
                    texture.id(),
                    plot::PlotPoint::new(*input_width as f32 / 2.0, *input_height as f32 / 2.0),
                    [*input_width as f32, *input_height as f32],
                )
                .uv(Rect::from_min_max(Pos2::new(0.0, 1.0), Pos2::new(1.0, 0.0)));

                let plot = Plot::new("Image area")
                    .show_x(false)
                    .show_y(false)
                    .show_background(false)
                    .show_axes([*grid, *grid])
                    .data_aspect(1.0);
                plot.show(ui, |plot_ui| {
                    plot_ui.image(image.name("Render result"));
                });
            });
    }
}

fn update_texture(
    texture: &mut TextureHandle,
    buffer: &mut [u8],
    receiver: &mut Receiver<Pixel>,
    ctx: &egui::Context,
) {
    let width = texture.size()[0];
    let height = texture.size()[1];
    let updated = !receiver.is_empty();
    for pixel in receiver.try_iter() {
        let index = pixel.position[0] as usize * 4 + pixel.position[1] as usize * width * 4;
        buffer[index..(4 + index)].copy_from_slice(&pixel.color);
    }
    if updated {
        let image = ColorImage::from_rgba_unmultiplied([width, height], buffer);
        // TODO: only need to update a section of the texture as data is received. Should probably
        // change from updating per-pixel, to updating fixed-size chunks of the image as well.
        ctx.tex_manager().write().set(
            texture.id(),
            ImageDelta::full(image, egui::TextureFilter::Nearest),
        );
        ctx.request_repaint();
    }
}
