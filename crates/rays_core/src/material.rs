use dyn_clone::{clone_trait_object, DynClone};
use glam::Vec3A;

use crate::{
    color::Color,
    ray::{self, RayHit},
    Sdf,
};

pub trait Material: Send + Sync + DynClone {
    /// Returns the scatter direction as a result of a ray hitting the surface of the material. This
    /// vector should **not be normalized**, as this is handled in the [`ray::Ray`]'s color
    /// function.
    fn scatter(&self, hit: &RayHit) -> Vec3A;
    fn attenuation(&self) -> &Color;
}

// Implements Clone for the boxed trait objects
clone_trait_object!(Material);
clone_trait_object!(Sdf);

#[derive(Clone, Debug)]
pub struct Lambertian {
    albedo: Color,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}
impl Material for Lambertian {
    fn scatter(&self, hit: &RayHit) -> Vec3A {
        hit.normal + ray::rand_on_unit_sphere()
    }

    fn attenuation(&self) -> &Color {
        &self.albedo
    }
}
