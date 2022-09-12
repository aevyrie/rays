use glam::Vec3A;
use std::sync::Arc;

use crate::{Camera, Color, Material, Scene, Sdf};

const DIST_EPSILON: f32 = 0.0001;
const RAY_OFFSET: f32 = DIST_EPSILON * 10.0;
const MAX_DIST: f32 = 100000000.0;

#[derive(Debug, Clone)]
pub struct Ray {
    pub origin: Vec3A,
    pub direction: Vec3A,
}
impl Ray {
    #[inline(always)]
    pub fn at(&self, t: f32) -> Vec3A {
        self.origin + self.direction * t
    }

    #[inline(always)]
    pub fn reflect(&self, normal: Vec3A) -> Vec3A {
        self.direction - 2.0 * self.direction.dot(normal) * normal
    }

    #[inline(always)]
    pub fn color(&self, scene: &Scene, max_bounces: u8) -> Color {
        if max_bounces == 0 {
            return [0.0, 0.0, 0.0, 1.0].into();
        }

        if let Some((hit, material)) = self.closest_hit(scene) {
            let scatter_dir = material.scatter(&hit);
            // Prevent NaN/inf errors by checking the direction can be normalized
            let scatter_dir = scatter_dir.try_normalize().unwrap_or(hit.normal);
            let mut scatter_ray = Ray {
                origin: hit.position,
                direction: scatter_dir,
            };
            // Move the ray away from the surface to prevent artifacts
            scatter_ray.origin = scatter_ray.at(RAY_OFFSET);
            material.attenuation() * scatter_ray.color(scene, max_bounces - 1)
        } else {
            let t = 0.5 * (self.direction.y + 1.0);
            let color = (1.0 - t) + t * Vec3A::new(0.5, 0.7, 1.0);
            [color.x, color.y, color.z, 1.0].into()
        }
    }

    #[inline(always)]
    fn closest_hit(&self, scene: &Scene) -> Option<(RayHit, Arc<dyn Material>)> {
        let mut ray_pos = self.origin;
        for _ in 0..10_000_000 {
            let (index, distance) = scene
                .objects
                .iter()
                .enumerate()
                .map(|(i, obj)| (i, obj.distance(ray_pos)))
                .reduce(
                    |(i, accum), (j, item)| {
                        if item < accum {
                            (j, item)
                        } else {
                            (i, accum)
                        }
                    },
                )
                .unwrap();

            if distance <= DIST_EPSILON {
                return Some((
                    RayHit {
                        position: ray_pos,
                        normal: scene.objects[index].normal(ray_pos),
                        in_dir: self.to_owned(),
                    },
                    scene.objects[index].material.clone(),
                ));
            } else if ray_pos.length_squared() > MAX_DIST {
                break;
            }
            ray_pos += self.direction * distance;
        }
        None
    }

    #[inline(always)]
    pub fn from_uv(camera: &Camera, u: f32, v: f32) -> Ray {
        let base_ray = camera
            .inv_projection
            .transform_point3a(Vec3A::new(u, v, 0.0));
        let direction = camera.transform.transform_point3a(base_ray).normalize();
        let origin = camera.inv_transform.transform_point3a(Vec3A::ZERO);
        Ray { origin, direction }
    }
}

pub struct RayHit {
    pub in_dir: Ray,
    pub position: Vec3A,
    pub normal: Vec3A,
}

/// Returns a random point from the surface of a sphere.
#[inline(always)]
pub fn rand_on_unit_sphere() -> Vec3A {
    loop {
        let p = Vec3A::new(fastrand::f32(), fastrand::f32(), fastrand::f32()) * 2.0 - 1.0;
        if p.length_squared() >= 1.0 {
            continue;
        };
        return p.normalize();
    }
}
