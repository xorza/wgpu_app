use bytemuck::{Pod, Zeroable};
use rand::random;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub(crate) struct Vertex {
    pos: glam::Vec2,   // xy
    uv: glam::Vec2,    // uv
    color: glam::Vec2, // green, alpha
}

struct Symbol {
    char: char,
    changing: bool,
    opacity: f32,
}

pub(crate) struct Thread {
    decay: f32,
    pos: glam::Vec2,
    vel: f32,
    size: f32,
    symbols: Vec<Symbol>,
    top_symbol: usize,
    length: usize,
    time_accum: f32,
}

pub(crate) struct Matrix {
    threads: Vec<Thread>,
    prev_time: f32,
}

const MAX_LENGTH: u8 = 255;

impl Default for Symbol {
    fn default() -> Self {
        Self {
            char: ' ',
            changing: false,
            opacity: 1.0,
        }
    }
}

impl Symbol {
    fn new_rand() -> Self {
        Self {
            char: random::<char>(),
            changing: random::<u8>() < 5,
            opacity: 1.0,
        }
    }
}

impl Default for Thread {
    fn default() -> Self {
        let symbols = Vec::with_capacity(MAX_LENGTH as usize);

        Self {
            decay: 0.0,
            pos: glam::Vec2::new(0.0, 0.0),
            vel: 0.0,
            size: 0.0,
            symbols,
            top_symbol: 0,
            length: 0,
            time_accum: 0.0,
        }
    }
}

impl Thread {
    fn init(&mut self) {
        self.decay = random::<f32>() * 0.3 + 0.01;
        self.pos = glam::Vec2::new(random::<f32>(), random::<f32>() * 1.8 - 0.5);
        self.vel = random::<f32>() * 7.0 + 0.3;
        self.size = random::<f32>() * 0.03 + 0.002;
        self.top_symbol = 0;
        self.length = (random::<u8>() % MAX_LENGTH) as usize;

        self.symbols.resize_with(1, Symbol::default);
    }

    fn update(&mut self, delta: f32) -> bool {
        self.time_accum += delta;

        for symbol in self.symbols[0..self.top_symbol + 1].iter_mut() {
            if symbol.changing {
                symbol.char = random::<char>();
            }
            symbol.opacity = (symbol.opacity - self.decay * delta).max(0.0);
        }

        if self.top_symbol < self.length && self.time_accum * self.vel >= 1.0 {
            self.time_accum -= 1.0 / self.vel;
            self.top_symbol += 1;
            self.symbols.push(Symbol::new_rand());
        }

        self.symbols[self.top_symbol].opacity > 0.0
    }
}

impl Matrix {
    pub fn new() -> Self {
        let mut threads: Vec<Thread> = vec![];
        for _ in 0..250 {
            let mut thread = Thread::default();
            thread.init();
            threads.push(thread);
        }

        Self {
            threads,
            prev_time: 0.0,
        }
    }

    pub fn update(&mut self, time: f32) {
        let delta = time - self.prev_time;
        self.prev_time = time;

        for stream in self.threads.iter_mut() {
            stream.update(delta);
        }

        self.threads
            .sort_unstable_by(|a, b| a.size.partial_cmp(&b.size).unwrap());
    }
    pub fn geometry(&self, vb: &mut Vec<Vertex>, ib: &mut Vec<u16>) {
        vb.clear();
        ib.clear();

        for thread in self.threads.iter() {
            let scale = thread.size;
            let iter = thread.symbols[0..thread.top_symbol + 1]
                .iter()
                .enumerate()
                .filter_map(|(idx, symbol)| {
                    let pos = thread.pos - glam::Vec2::new(0.0, thread.size * idx as f32)
                        + scale * glam::Vec2::new(-0.5, -0.5);

                    if symbol.opacity > 0.0 && pos.y >= 0.0 && pos.y <= 1.0 {
                        Some((symbol, pos))
                    } else {
                        None
                    }
                });

            for (symbol, pos) in iter {
                vb.push(Vertex {
                    pos: pos + scale * glam::Vec2::new(-0.5, -0.5),
                    uv: glam::Vec2::new(0.0, 1.0),
                    color: glam::Vec2::new(1.0, symbol.opacity),
                });
                vb.push(Vertex {
                    pos: pos + scale * glam::Vec2::new(-0.5, 0.5),
                    uv: glam::Vec2::new(0.0, 0.0),
                    color: glam::Vec2::new(1.0, symbol.opacity),
                });
                vb.push(Vertex {
                    pos: pos + scale * glam::Vec2::new(0.5, -0.5),
                    uv: glam::Vec2::new(1.0, 1.0),
                    color: glam::Vec2::new(1.0, symbol.opacity),
                });
                vb.push(Vertex {
                    pos: pos + scale * glam::Vec2::new(0.5, 0.5),
                    uv: glam::Vec2::new(1.0, 0.0),
                    color: glam::Vec2::new(1.0, symbol.opacity),
                });

                ib.push((vb.len() - 2) as u16);
                ib.push((vb.len() - 3) as u16);
                ib.push((vb.len() - 4) as u16);

                ib.push((vb.len() - 1) as u16);
                ib.push((vb.len() - 3) as u16);
                ib.push((vb.len() - 2) as u16);
            }
        }
    }
}

impl Vertex {
    pub fn size_in_bytes() -> usize {
        std::mem::size_of::<Self>()
    }
}
