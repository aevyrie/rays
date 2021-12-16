use std::{any::Any, collections::HashMap};

use crossbeam_channel::Receiver;
use eframe::{
    egui::{self, Color32, TextureId},
    epi,
};
use rays_core::{PathTracer, Pixel};

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
    receiver: Option<Receiver<Pixel>>,
    events: EventQueue,
    zoom: f32,
}
impl Default for RaysApp {
    fn default() -> Self {
        RaysApp {
            texture: None,
            buffer: Vec::new(),
            width: 800,
            height: 600,
            receiver: None,
            events: EventQueue::default(),
            zoom: 1.0,
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
        let RaysApp {
            texture,
            buffer,
            width,
            height,
            receiver,
            events,
            zoom,
        } = self;

        ctx.set_debug_on_hover(cfg!(debug_assertions));

        update_texture(texture, buffer, width, height, receiver, frame, ctx);

        egui::TopBottomPanel::top("top panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Submit").clicked() {
                    buffer.clear();
                    *receiver = Some(PathTracer::build([*width, *height]).run());
                    *texture = None;
                    *buffer = vec![Color32::TRANSPARENT; *width * *height];
                }
                ui.add_space(20.0);
                ui.label("Zoom");
                if ui.button("Fit").clicked() {
                    events.notify(Zoom::Fit)
                }
                if ui.button("Fill").clicked() {
                    events.notify(Zoom::Fill)
                }
                if ui.button("1:1").clicked() {
                    events.notify(Zoom::One)
                }
            });
        });
        egui::SidePanel::left("left panel").show(ctx, |ui| {
            // = egui::vec2(20.0, 20.0);
            ui.heading("Render Settings");
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label("Width");
                ui.add(
                    egui::DragValue::new(width)
                        .speed(1.0)
                        .fixed_decimals(0)
                        .clamp_range(1..=10_000),
                );
            });
            ui.horizontal(|ui| {
                ui.label("Height");
                ui.add(
                    egui::DragValue::new(height)
                        .speed(1.0)
                        .fixed_decimals(0)
                        .clamp_range(1..=10_000),
                );
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                if let Some(texture) = self.texture {
                    let padding = 5.0;
                    let h_zoom = (ctx.available_rect().height() - padding) / *height as f32;
                    let w_zoom = (ctx.available_rect().width() - padding) / *width as f32;
                    for event in events.read() {
                        match event {
                            Zoom::Fit => *zoom = w_zoom.min(h_zoom) * 0.95,
                            Zoom::Fill => *zoom = w_zoom.max(h_zoom) * 0.95,
                            Zoom::One => *zoom = 1.0,
                        }
                    }
                    let scaled_img_size = (*width as f32 * *zoom, *height as f32 * *zoom);
                    ui.centered_and_justified(|ui| {
                        ui.image(texture, scaled_img_size);
                    });
                }
            })
        });
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
enum Zoom {
    Fit,
    Fill,
    One,
}

#[derive(Default)]
struct EventQueue {
    events: HashMap<String, Box<dyn Any>>,
}
impl EventQueue {
    fn notify<T: 'static>(&mut self, event: T) {
        match self.events.get_mut(std::any::type_name::<T>()) {
            Some(list) => {
                let list = list.downcast_mut::<Vec<T>>().unwrap();
                list.push(event);
            }
            None => {
                self.events
                    .insert(std::any::type_name::<T>().into(), Box::new(vec![event]));
            }
        }
    }
    fn read<T: 'static + Clone>(&mut self) -> std::vec::IntoIter<T> {
        match self.events.get_mut(std::any::type_name::<T>()) {
            Some(list) => list
                .downcast_mut::<Vec<T>>()
                .unwrap()
                .drain(0..)
                .collect::<Vec<T>>()
                .into_iter(),

            None => Vec::new().into_iter(),
        }
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
