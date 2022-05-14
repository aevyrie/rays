use glam::Vec3A;
use std::sync::Arc;

use crate::{Camera, Color, Material, Scene, Sdf};

const EPSILON: f32 = 0.001;

#[derive(Debug, Clone)]
pub struct Ray {
    pub origin: Vec3A,
    pub direction: Vec3A,
}
impl Ray {
    pub fn at(&self, t: f32) -> Vec3A {
        self.origin + self.direction * t
    }
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
            let screen_dist = (self.origin - scatter_ray.origin).length();
            // Move the ray away from the surface to prevent artifacts
            scatter_ray.origin = scatter_ray.at(EPSILON * screen_dist);
            material.attenuation() * scatter_ray.color(scene, max_bounces - 1)
        } else {
            let t = 0.5 * (self.direction.y + 1.0);
            let color = (1.0 - t) + t * Vec3A::new(0.5, 0.7, 1.0);
            [color.x, color.y, color.z, 1.0].into()
        }
    }

    fn closest_hit(&self, scene: &Scene) -> Option<(RayHit, Arc<dyn Material>)> {
        let mut ray_pos = self.origin;
        let mut steps = 1;
        while let Some((index, distance)) = scene
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
        {
            let screen_dist = (ray_pos - self.origin).length();
            if distance.abs() <= EPSILON * screen_dist {
                return Some((
                    RayHit {
                        position: ray_pos,
                        normal: scene.objects[index].normal(ray_pos),
                    },
                    scene.objects[index].material.clone(),
                ));
            } else if steps >= 1024 || ray_pos.length_squared() > 100000000.0 {
                return None;
            }
            ray_pos += self.direction * distance;
            steps += 1;
        }
        unreachable!();
    }

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
    pub position: Vec3A,
    pub normal: Vec3A,
}

/// Returns a random point from the surface of a sphere.
pub fn rand_on_unit_sphere() -> Vec3A {
    loop {
        let p = Vec3A::new(fastrand::f32(), fastrand::f32(), fastrand::f32()) * 2.0 - 1.0;
        if p.length_squared() >= 1.0 {
            continue;
        };
        return p.normalize();
    }
}
