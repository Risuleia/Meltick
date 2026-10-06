pub struct Spring {
    pub x: f32,
    pub v: f32
}

impl Spring {
    pub fn new() -> Self {
        Self { x: 0.0, v: 0.0 }
    }

    pub fn step(&mut self, dt: f32, k: f32, c: f32) {
        let n = ((dt / 0.004).ceil() as i32).max(1);
        let h = dt / n as f32;

        for _ in 0..n {
            let a = -k * self.x - c * self.v;
            self.v += a * h;
            self.x += self.v * h;
        }
    }
}