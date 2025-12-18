use std::time::Instant;

pub(crate) struct FpsCounter {
    last_time: Instant,
    frames: u32,
    fps: f32,
}

impl FpsCounter {
    pub fn new() -> FpsCounter {
        FpsCounter {
            last_time: Instant::now(),
            frames: 0,
            fps: 0.0,
        }
    }

    pub fn update(&mut self) -> bool {
        self.frames += 1;
        let current_time = Instant::now();

        let delta = current_time.duration_since(self.last_time).as_secs_f32();

        if delta >= 10.0 {
            self.fps = (self.frames as f32) / delta;
            self.frames = 0;
            self.last_time = current_time;

            true
        } else {
            false
        }
    }

    pub fn get_fps(&self) -> f32 {
        self.fps
    }
}
