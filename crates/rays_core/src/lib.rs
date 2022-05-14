use color::Color;
use crossbeam_channel::{unbounded, Receiver, Sender};
use dyn_clone::{clone_trait_object, DynClone};
use glam::{Mat4, Vec3, Vec3A, Vec4, Vec4Swizzles};
use material::Material;
use ray::Ray;
use rayon::prelude::*;
use ringbuffer::{ConstGenericRingBuffer, RingBuffer, RingBufferExt, RingBufferWrite};
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
        std::thread::spawn(move || {
            (0..self.size[0] * self.size[1])
                .par_bridge()
                .for_each(|index| {
                    let x = index % self.size[0];
                    let y = index / self.size[0];
                    let position = [x, y];

                    let mut n_actual = None;
                    let mut color = Color::from(Vec4::ZERO);
                    let mut luma = ConstGenericRingBuffer::<f32, 64>::new();

                    for i in 1..=n_samples {
                        let u = ((x as f32 + fastrand::f32()) / self.size[0] as f32) * 2.0 - 1.0;
                        let v = ((y as f32 + fastrand::f32()) / self.size[1] as f32) * 2.0 - 1.0;
                        let ray = Ray::from_uv(&scene.camera, u, v);
                        color += ray.color(&scene, max_bounces);
                        // Early-out based on luminance convergence
                        if matches!(i, 1..=64) || i % 32 == 0 {
                            luma.push(color.approx_luminance() / i as f32);
                            if !luma.is_full() {
                                continue;
                            }
                            let mean = luma.iter().sum::<f32>() / luma.capacity() as f32;
                            let sum_squares: f32 =
                                luma.iter().map(|x| f32::powi(x - mean, 2)).sum();
                            let variance = sum_squares / (luma.capacity() - 1) as f32;
                            if variance <= f32::EPSILON {
                                n_actual = Some(i);
                                break;
                            }
                        }
                    }

                    // Gamma correction
                    let n = n_actual.unwrap_or(n_samples) as f32;
                    color = Vec4::new(
                        (color.r() / n).sqrt(),
                        (color.g() / n).sqrt(),
                        (color.b() / n).sqrt(),
                        color.a() / n,
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
    fn distance(&self, from: Vec3A) -> f32 {
        self.isosurface.distance(from)
    }

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
    fn distance(&self, ray_position: Vec3A) -> f32 {
        ray_position.distance(self.pos_rad.xyz().into()) - self.pos_rad.w
    }

    fn normal(&self, ray_position: Vec3A) -> Vec3A {
        (ray_position - Vec3A::from(self.pos_rad.xyz())).normalize()
    }
}
