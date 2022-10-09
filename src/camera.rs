// Copyright 2020-2022, Augustinas Lukauskas <augustinaslukauskas01@gmail.com>

use crate::vec3::Vec3;
use crate::ray3::Ray3;

#[derive(Debug, Clone)]
pub struct Camera {
    origin: Vec3,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
    dist_to_display: f64,
    width: f64,
    height: f64,
}

impl Camera {
    pub fn new(forward: Vec3, up: Vec3) -> Camera {
        Camera {
            origin: Vec3::new(0.0, 0.0, 0.0),
            forward,
            right: up.cross(forward),
            up,
            dist_to_display: 1.0,
            width: 2.0,
            height: 2.0
        }
    }

    pub fn get_ray(&self, x: f64, y:f64) -> Ray3 {
        let x = (x - 0.5) * self.width;
        let y = -(y - 0.5) * self.height;

        let ray_origin = self.origin + self.forward * self.dist_to_display + self.right * x + self.up * y;

        Ray3::new(ray_origin, (ray_origin - self.origin).normalize())
    }
}