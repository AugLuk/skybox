// Copyright 2020-2022, Augustinas Lukauskas <augustinaslukauskas01@gmail.com>

use std::ops::Mul;
use crate:: vec3::Vec3;
use crate:: ray3::Ray3;
use crate:: color::Color;
use crate:: camera::Camera;
use crate:: cloud::Cloud;
use crate:: background::Background;
use crate:: fast_rng::Frng;

#[derive(Debug, Clone)]
pub struct Renderer {
    cloud: Cloud,
    slice_length: usize,
    color_byte_size: usize,
    image_width: usize,
    image_height: usize,
    min_fog_dist: f64,
    max_fog_dist: f64,
    step_size: f64,
    step_count: usize,
    pixel_width: usize,
    bundle_size: usize,
    background: Background,
    output_raw_color: bool,
    sun_brightness: f64,
}

impl Renderer {
    pub fn new(cloud: Cloud, slice_length: usize, color_byte_size: usize, image_width: usize, image_height: usize, min_fog_dist: f64, max_fog_dist: f64, step_size: f64, step_count: usize, pixel_width: usize, bundle_size: usize, background: Background, output_raw_color: bool, sun_brightness: f64) -> Self {
        Renderer { cloud, slice_length, color_byte_size, image_width, image_height, min_fog_dist, max_fog_dist, step_size, step_count, pixel_width, bundle_size, background, output_raw_color, sun_brightness }
    }

    pub fn render_slice(&self, camera: &Camera, min_py: usize, frng_seed: u64) -> Vec<u8> {
        let mut frng = Frng::new(frng_seed);

        let mut result = vec![0; self.slice_length];

        for color_index in 0..self.slice_length / self.color_byte_size {
            let px = color_index % self.image_width;
            let py = min_py + color_index / self.image_width;

            let color = self.trace_pixel(camera, px, py, &mut frng);

            let color2 = if self.output_raw_color {
                color.to_lin_48_u8()
            } else {
                color.mul(self.sun_brightness).to_srgb_48_u8()
            };

            let i1 = color_index * self.color_byte_size;
            let i2 = i1 + self.color_byte_size;

            result[i1..i2].copy_from_slice(&color2);
        }

        result
    }

    fn trace_pixel(&self, camera: &Camera, px: usize, py: usize, frng: &mut Frng) -> Color {
        let mut color_sum = Color::new(0.0, 0.0, 0.0);

        for spy in 0..self.pixel_width {
            for spx in 0..self.pixel_width {
                let mut ray = camera.get_ray(
                    (px as f64 + (spx as f64 + 0.5) / self.pixel_width as f64) / self.image_width as f64,
                    (py as f64 + (spy as f64 + 0.5) / self.pixel_width as f64) / self.image_height as f64,
                );

                color_sum = color_sum + self.trace_bundle(&mut ray, frng);
            }
        }

        color_sum / (self.pixel_width * self.pixel_width) as f64
    }

    fn trace_bundle(&self, ray: &mut Ray3, frng: &mut Frng) -> Color {
        if ray.direction.y <= 0.0 {
            return self.background.get_sky_color(ray.direction);
        }

        let dist_to_cloud = (self.cloud.min_height - ray.origin.y ) / ray.direction.y + frng.next_double(0.0, self.step_size);

        ray.origin = ray.origin + ray.direction * dist_to_cloud;

        if dist_to_cloud <= self.min_fog_dist {
            self.inner_trace_bundle(ray, frng)
        } else if dist_to_cloud >= self.max_fog_dist {
            self.background.get_sky_color(ray.direction)
        } else {
            let s_color = self.background.get_sky_color(ray.direction);
            let c_color = self.inner_trace_bundle(ray, frng);
            let mut fog_amount = (dist_to_cloud + ray.length - self.min_fog_dist) / self.max_fog_dist;
            if fog_amount > 1.0 {
                fog_amount = 1.0;
            }

            c_color.mix(s_color, fog_amount)
        }
    }

    fn inner_trace_bundle(&self, bundle: &mut Ray3, frng: &mut Frng) -> Color {
        let mut colors: Option<Vec<Color>> = None;

        for step in 0..self.step_count {
            if self.cloud.get_density(bundle.origin) {
                let mut colors_ = vec![Color::new(0.0, 0.0, 0.0); self.bundle_size as usize]; //@@@

                for ray_index in 0..colors_.len() {
                    let mut ray = bundle.clone();

                    let mut change;

                    loop {
                        change = Vec3::new(frng.next_double(-1.0, 1.0), frng.next_double(-1.0, 1.0), frng.next_double(-1.0, 1.0));

                        if change.mag_squared() < 1.0 {
                            break;
                        }
                    }

                    ray.direction = (ray.direction + change).normalize();

                    ray.origin = ray.origin + ray.direction * self.step_size;

                    colors_[ray_index] = self.trace_ray(&mut ray, frng, self.step_count - step - 1);
                }

                colors = Some(colors_);

                break;
            } else {
                bundle.origin = bundle.origin + bundle.direction * self.step_size;

                bundle.length += self.step_size;
            }
        }

        // compute color
        match colors {
            Some(colors) => {
                let mut color_sum = Color::new(0.0, 0.0, 0.0);

                for i in 0..colors.len() {
                    color_sum = color_sum + colors[i];
                }

                color_sum / colors.len() as f64
            },
            None => self.background.get_sky_color(bundle.direction),
        }
    }

    fn trace_ray(&self, ray: &mut Ray3, frng: &mut Frng, steps: usize) -> Color {
        for _step in 0..steps {
            if ray.origin.y < self.cloud.min_height || ray.origin.y > self.cloud.max_height {
                break;
            }

            if self.cloud.get_density(ray.origin) {
                let mut change;

                loop {
                    change = Vec3::new(frng.next_double(-1.0, 1.0), frng.next_double(-1.0, 1.0), frng.next_double(-1.0, 1.0));

                    if change.mag_squared() < 1.0 {
                        break;
                    }
                }

                ray.direction = (ray.direction + change).normalize();
            }

            ray.origin = ray.origin + ray.direction * self.step_size;
        }

        self.background.get_background_color(ray.direction)
    }
}