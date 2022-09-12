use color::Color;
use crossbeam_channel::{unbounded, Receiver, Sender};
use dyn_clone::{clone_trait_object, DynClone};
use glam::{Mat4, Vec3, Vec3A, Vec4, Vec4Swizzles};
use material::Material;
use ray::Ray;
use rayon::prelude::*;
use std::{f32::consts::PI, sync::Arc};

pub mod color;
pub mod material;
pub mod ray;

pub struct PathTracer {
    size: [u32; 2],
    sender: Sender<Pixel>,
    receiver: Receiver<Pixel>,
}

pub struct Pixel {
    pub position: [u32; 2],
    pub color: [u8; 4],
}

impl PathTracer {
    pub fn build(size: [u32; 2]) -> PathTracer {
        let (sender, receiver) = unbounded();
        PathTracer {
            size,
            sender,
            receiver,
        }
    }

    pub fn run(self, scene: Scene, n_samples: usize, max_bounces: u8) -> Receiver<Pixel> {
        // TODO: Make the allocated `Vec`s thread-local to avoid reallocating
        std::thread::spawn(move || {
            let area = self.size[0] * self.size[1];
            let skip = (1..=self.size[0] / 3)
                .reduce(|div, i| {
                    if self.size[0] % i == 0 && self.size[1] % i == 0 {
                        i
                    } else {
                        div
                    }
                })
                .unwrap_or(1);

            (0..area).par_bridge().for_each(|index| {
                let scaled_i = index * skip;
                let j = (scaled_i % area) + (scaled_i / area);

                let y = j % self.size[1];
                let x = j / self.size[1];

                let position = [x, y];

                let mut color = Color::from(Vec4::ZERO);
                let mut last_luma = f32::INFINITY;
                let mut i = 0;

                for _ in 1..=n_samples {
                    let u = ((x as f32 + fastrand::f32()) / self.size[0] as f32) * 2.0 - 1.0;
                    let v = ((y as f32 + fastrand::f32()) / self.size[1] as f32) * 2.0 - 1.0;
                    let ray = Ray::from_uv(&scene.camera, u, v);
                    let new_color = ray.color(&scene, max_bounces);
                    if new_color.inner.is_finite() {
                        i += 1;
                        color += new_color;
                    } else {
                        continue;
                    }
                    // Early-out based on luminance convergence
                    if i % 64 == 0 {
                        let luma = (&color / i as f32).approx_luminance();
                        let delta = last_luma - luma;
                        if delta.abs() <= f32::EPSILON * 10.0 {
                            println!("early exit y: {y} n: {i}");
                            break;
                        }
                        last_luma = luma;
                    }
                }

                // Gamma correction
                let n = i as f32;
                color = color / n;
                color = Vec4::new(
                    color.r().sqrt(),
                    color.g().sqrt(),
                    color.b().sqrt(),
                    color.a(),
                )
                .into();

                // convert to u8 range
                let color = color.into_bytes();
                self.sender.send(Pixel { position, color }).ok();
            });
        });
        self.receiver
    }
}

#[derive(Clone)]
pub struct Camera {
    pub(crate) transform: Mat4,
    pub(crate) inv_transform: Mat4,
    pub(crate) inv_projection: Mat4,
}
impl Camera {
    /// Computes a ray in world space given the x and y coordinates of the target.
    #[inline(always)]
    pub fn from_aspect_ratio(aspect_ratio: f32) -> Self {
        let inv_projection =
            Mat4::perspective_infinite_reverse_rh(PI / 2.0, aspect_ratio, 1.0).inverse();
        let transform = Mat4::look_at_rh(Vec3::ZERO, -Vec3::Z, Vec3::Y);
        Self {
            inv_transform: transform.inverse(),
            transform,
            inv_projection,
        }
    }
}

#[derive(Clone)]
pub struct SdfObject {
    isosurface: Box<dyn Sdf>,
    material: Arc<dyn Material>,
}
impl Sdf for SdfObject {
    #[inline(always)]
    fn distance(&self, from: Vec3A) -> f32 {
        self.isosurface.distance(from)
    }

    #[inline(always)]
    fn normal(&self, ray_position: Vec3A) -> Vec3A {
        self.isosurface.normal(ray_position)
    }
}
impl SdfObject {
    pub fn new<S, M>(isosurface: S, material: Arc<M>) -> Self
    where
        S: 'static + Sdf,
        M: 'static + Material,
    {
        Self {
            isosurface: Box::new(isosurface),
            material,
        }
    }
}

clone_trait_object!(Sdf);

#[derive(Clone)]
pub struct Scene {
    pub camera: Camera,
    pub objects: Vec<SdfObject>,
    pub materials: Vec<Arc<dyn Material>>,
}

pub trait Sdf: Send + Sync + DynClone {
    fn distance(&self, ray_position: Vec3A) -> f32;
    fn normal(&self, ray_position: Vec3A) -> Vec3A;
}

#[derive(Clone)]
pub struct Sphere {
    /// Position and radius packed into a Vec4
    pos_rad: Vec4,
}
impl Sphere {
    pub fn new(position: Vec3, radius: f32) -> Self {
        Sphere {
            pos_rad: position.extend(radius),
        }
    }
}
impl Sdf for Sphere {
    #[inline(always)]
    fn distance(&self, ray_position: Vec3A) -> f32 {
        ray_position.distance(self.pos_rad.xyz().into()) - self.pos_rad.w
    }

    #[inline(always)]
    fn normal(&self, ray_position: Vec3A) -> Vec3A {
        (ray_position - Vec3A::from(self.pos_rad.xyz())).normalize()
    }
}
