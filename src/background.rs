// Copyright 2020-2022, Augustinas Lukauskas <augustinaslukauskas01@gmail.com>

use crate::vec3::Vec3;
use crate::color::Color;

#[derive(Debug, Clone)]
pub struct Background {
    sun_size: f64,
    sun_color: Color,
    sky_colors: Vec<Color>,
    ground_color: Color,
    sun_x: f64,
    sun_y: f64,
    sun_z: f64,
}

impl Background {
    pub fn new(sun_size: f64, sun_color: Color, sky_colors: Vec<Color>, ground_color: Color, phi: f64, theta: f64) -> Self {
        let sun_x = phi.sin() * theta.cos();
        let sun_y = phi.sin() * theta.sin();
        let sun_z = phi.cos();

        Background { sun_size, sun_color, sky_colors, ground_color, sun_x, sun_y, sun_z }
    }

    pub fn get_background_color(&self, direction: Vec3) -> Color {
        if direction.y < 0.0 {
            return self.ground_color;
        }

        self.get_sky_color(direction)
    }

    pub fn get_sky_color(&self, direction: Vec3) -> Color {
        let dist = ((direction.x - self.sun_x).powi(2) + (direction.y - self.sun_y).powi(2) + (direction.z - self.sun_z).powi(2)).sqrt();

        if dist <= self.sun_size {
            return self.sun_color;
        }

        let temp = (dist - self.sun_size) / (2.0 - self.sun_size) * (self.sky_colors.len() as f64 - 1.01);
        let disk = temp as usize;
        let disk_position = temp - temp.floor();

        self.sky_colors[disk].mix(self.sky_colors[disk+1], disk_position)
    }
}