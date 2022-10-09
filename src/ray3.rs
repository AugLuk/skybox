// Copyright 2020-2022, Augustinas Lukauskas <augustinaslukauskas01@gmail.com>

use crate::vec3::Vec3;

#[derive(Debug, Clone)]
pub struct Ray3 {
    pub origin: Vec3,
    pub direction: Vec3,
    pub length: f64,
}

impl Ray3 {
    pub fn new(origin: Vec3, direction: Vec3) -> Ray3 {
        Ray3 {origin, direction, length: 0.0}
    }
}