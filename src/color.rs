// Copyright 2020-2022, Augustinas Lukauskas <augustinaslukauskas01@gmail.com>

use std::ops::{Add, Div, Mul};

#[derive(Debug, Copy, Clone)]
pub struct Color {
    r: f64,
    g: f64,
    b: f64,
}

impl Color {
    pub fn new(r: f64, g: f64, b: f64) -> Color {
        Color {r, g, b}
    }

    pub fn from_str(v: &str) -> Color {
        assert_eq!(v.len(), 6);
        let input = [
            u8::from_str_radix(&v[0..2], 16).unwrap(),
            u8::from_str_radix(&v[2..4], 16).unwrap(),
            u8::from_str_radix(&v[4..6], 16).unwrap(),
        ];

        let mut output = [0.0; 3];
        for (c_lin, c_s_rgb_8) in output.iter_mut().zip(input.iter()) {
            let c_s_rgb = *c_s_rgb_8 as f64 / 255.0;

            *c_lin = if c_s_rgb <= 0.04045 {
                c_s_rgb / 12.92
            } else {
                ((c_s_rgb + 0.055) / 1.055).powf(2.4)
            }
        }

        Color {
            r: output[0],
            g: output[1],
            b: output[2]
        }
    }

    pub fn mix(self, c: Color, amount: f64) -> Color {
        Color {
            r: (1.0-amount) * self.r + amount*c.r,
            g: (1.0-amount) * self.g + amount*c.g,
            b: (1.0-amount) * self.b + amount*c.b,
        }
    }

    pub fn to_lin_48_u8(self) -> [u8; 6] {
        let input = [self.r, self.g, self.b];
        let mut output = [0; 6];
        for (c_out, c_in) in output.chunks_mut(2).zip(input.iter()) {
            let c_lin = c_in.clamp(0.0, 1.0);

            let val = (c_lin * 65535.0).round() as u16;

            c_out[0] = (val >> 8) as u8;
            c_out[1] = val as u8;
        }

        output
    }

    pub fn to_srgb_48_u8(self) -> [u8; 6] {
        let input = [self.r, self.g, self.b];
        let mut output = [0; 6];
        for (c_out, c_in) in output.chunks_mut(2).zip(input.iter()) {
            let c_lin = c_in.clamp(0.0, 1.0);

            let val = if c_lin <= 0.0031308 {
                (12.92 * c_lin * 65535.0).round() as u16
            } else {
                ((1.055 * c_lin.powf(1.0 / 2.4) - 0.055) * 65535.0).round() as u16
            };

            c_out[0] = (val >> 8) as u8;
            c_out[1] = val as u8;
        }

        output
    }
}

impl Add<Color> for Color {
    type Output = Color;

    #[inline]
    fn add(self, other: Color) -> Color {
        Color {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
        }
    }
}

impl Div<f64> for Color {
    type Output = Color;

    #[inline]
    fn div(self, other: f64) -> Color {
        Color {
            r: self.r / other,
            g: self.g / other,
            b: self.b / other,
        }
    }
}

impl Mul<f64> for Color {
    type Output = Color;

    #[inline]
    fn mul(self, other: f64) -> Color {
        Color {
            r: self.r * other,
            g: self.g * other,
            b: self.b * other,
        }
    }
}