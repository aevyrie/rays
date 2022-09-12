use dyn_clone::{clone_trait_object, DynClone};
use glam::Vec3A;

use crate::{
    color::Color,
    ray::{self, RayHit},
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

#[derive(Clone, Debug)]
pub struct Lambertian {
    albedo: Color,
}

impl Lambertian {
    #[inline(always)]
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}
impl Material for Lambertian {
    #[inline(always)]
    fn scatter(&self, hit: &RayHit) -> Vec3A {
        hit.normal + ray::rand_on_unit_sphere()
    }

    #[inline(always)]
    fn attenuation(&self) -> &Color {
        &self.albedo
    }
}

#[derive(Clone, Debug)]
pub struct Metal {
    albedo: Color,
}

impl Metal {
    #[inline(always)]
    pub fn new(albedo: Color) -> Metal {
        Metal { albedo }
    }
}

impl Material for Metal {
    #[inline(always)]
    fn scatter(&self, hit: &RayHit) -> Vec3A {
        hit.in_dir.reflect(hit.normal)
    }

    #[inline(always)]
    fn attenuation(&self) -> &Color {
        &self.albedo
    }
}
