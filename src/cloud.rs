// Copyright 2020-2022, Augustinas Lukauskas <augustinaslukauskas01@gmail.com>

use crate::vec3::Vec3;

#[derive(Debug, Clone)]
pub struct Cloud {
    pub min_height: f64,
    pub max_height: f64,
    threshold: f64,
    levels: u32,
    seed: i32,
    scale: f64
}

impl Cloud {
    pub fn new(min_height: f64, max_height: f64, threshold: f64, levels: u32, seed: i32, scale: f64) -> Self {
        Cloud { min_height, max_height, threshold, levels, seed, scale }
    }
    
    pub fn get_density(&self, position: Vec3) -> bool {
        let x = position.x;
        let y = position.y;
        let z = position.z;

        let divisor = (2_i32.pow(self.levels) - 1) as f64;
        let mut dividend = 2_i32.pow(self.levels - 1);
        let mut density = 0.0;

        for _ in 0..(self.levels - 1) {
            density += self.get_density_at_scale(dividend, x, y, z) * (dividend as f64 / divisor);
            if density < self.threshold - (dividend - 1) as f64 / divisor {
                return false;
            }
            if density >= self.threshold {
                return true;
            }

            dividend /= 2;
        }

        density += self.get_density_at_scale(1, x, y, z) * (1.0 / divisor);
        if density < self.threshold {
            return false;
        }
        true
    }

    fn get_density_at_scale(&self, scale2: i32, x: f64, y: f64, z: f64) -> f64 {
        let scale_combined = scale2 as f64 * self.scale;

        let qx = x.div_euclid(scale_combined);
        let qy = y.div_euclid(scale_combined);
        let qz = z.div_euclid(scale_combined);

        let x0 = qx as i32;
        let x1 = x0 + 1;
        let y0 = qy as i32;
        let y1 = y0 + 1;
        let z0 = qz as i32;
        let z1 = z0 + 1;

        let ax = x / scale_combined - qx;
        let ay = y / scale_combined - qy;
        let az = z / scale_combined - qz;

        Cloud::interpolate_3d(
            [
                self.hash(scale2, x0, y0, z0),
                self.hash(scale2, x1, y0, z0),
                self.hash(scale2, x0, y1, z0),
                self.hash(scale2, x1, y1, z0),
                self.hash(scale2, x0, y0, z1),
                self.hash(scale2, x1, y0, z1),
                self.hash(scale2, x0, y1, z1),
                self.hash(scale2, x1, y1, z1),
            ],
            [
                Cloud::extremify(ax),
                Cloud::extremify(ay),
                Cloud::extremify(az),
            ]
        )
    }

    fn hash(&self, scale2: i32, x: i32, y: i32, z: i32) -> f64 {
        let mut val = Cloud::hash_i32(self.seed);
        val = Cloud::hash_i32(val.wrapping_add(x));
        val = Cloud::hash_i32(val.wrapping_add(y));
        val = Cloud::hash_i32(val.wrapping_add(z));
        val = Cloud::hash_i32(val.wrapping_add(scale2));

        (val as f64 + 2147483648.0) / 4294967295.0
    }

    fn hash_i32(x: i32) -> i32 {
        ((x >> 16) ^ x).wrapping_mul(0x45d9f3b)
    }

    fn interpolate_3d(v: [f64; 8], p: [f64; 3]) -> f64 {
        let x: [f64; 4] = [
            Cloud::interpolate(v[0], v[1], p[0]),
            Cloud::interpolate(v[2], v[3], p[0]),
            Cloud::interpolate(v[4], v[5], p[0]),
            Cloud::interpolate(v[6], v[7], p[0]),
        ];

        let y: [f64; 2] = [
            Cloud::interpolate(x[0], x[1], p[1]),
            Cloud::interpolate(x[2], x[3], p[1]),
        ];

        Cloud::interpolate(y[0], y[1], p[2])
    }

    fn interpolate(min: f64, max: f64, amount: f64) -> f64 {
        min + (max - min) * amount
    }

    fn extremify(n: f64) -> f64 {
        //n * n * (3.0 - 2.0 * n)
        n * n * n * (n * (n * 6.0 - 15.0) + 10.0)
    }
}