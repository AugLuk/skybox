// Copyright 2020-2022, Augustinas Lukauskas <augustinaslukauskas01@gmail.com>

pub struct Frng {
    state: u64,
}

impl Frng {
    pub fn new(seed: u64) -> Frng {
        let mut result = Frng {state: seed};

        result.next();
        result.next();
        result.next();

        result
    }

    fn next(&mut self) {
        self.state = self.state.wrapping_mul(44485709377909);
    }

    fn next_short(&mut self) -> u16 {
        self.next();
        (self.state >> 48) as u16
    }

    pub fn next_double_default(&mut self) -> f64 {
        self.next_short() as f64 / 65536.00001
    }

    pub fn next_double(&mut self, min: f64, max: f64) -> f64 {
        self.next_double_default() * (max - min) + min
    }
}