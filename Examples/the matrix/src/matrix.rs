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
}

pub(crate) struct Stream {
    decay: f32,
    pos: glam::Vec2,
    vel: f32,
    size: f32,
    symbols: Vec<Symbol>,
    top_symbol: usize,
    length: usize,
}

pub(crate) struct Matrix {
    streams: Vec<Stream>,
}

const MAX_LENGTH: u8 = 255;


impl Default for Symbol {
    fn default() -> Self {
        Self {
            char: ' ',
            changing: false,
        }
    }
}

impl Symbol {
    fn update(&mut self) {
        if self.changing {
            self.char = random::<char>();
        }
    }
    fn new_rand() -> Self {
        Self {
            char: random::<char>(),
            changing: random::<u8>() < 5,
        }
    }
}

impl Default for Stream {
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
        }
    }
}

impl Stream {
    fn init(&mut self) {
        self.decay = 0.05;
        self.pos = glam::Vec2::new(0.0, 0.0);
        self.vel = 2.0;
        self.size = 0.01;
        self.top_symbol = 0;
        self.length = (random::<u8>() % MAX_LENGTH) as usize;

        self.symbols.resize_with(1, Symbol::default);
    }

    fn update(&mut self) {
        if self.top_symbol >= self.length {
            return;
        }

        self.top_symbol += 1;
        self.symbols[self.top_symbol] = Symbol::new_rand();
    }
}

impl Matrix {
    pub fn new() -> Self {
        let mut streams = vec! {
            Stream::default()
        };
        streams.iter_mut().for_each(|stream| stream.init());

        Self {
            streams,
        }
    }

    pub fn update(&mut self) {
        for stream in self.streams.iter_mut() {
            stream.update();
        }
    }
    pub fn geometry(&self, vb: &mut Vec<Vertex>, ib: &mut Vec<u16>) {
        vb.clear();
        ib.clear();

        for stream in self.streams.iter() {
            let scale = 0.9;
            let offset = glam::vec2(0.5, 0.5);
            for _symbol in stream.symbols[0..stream.top_symbol + 1].iter() {
                vb.push(Vertex {
                    pos: offset + scale * glam::Vec2::new(-0.5, -0.5),
                    uv: glam::Vec2::new(0.0, 1.0),
                    color: glam::Vec2::new(1.0, 1.0),
                });
                vb.push(Vertex {
                    pos: offset + scale * glam::Vec2::new(-0.5, 0.5),
                    uv: glam::Vec2::new(0.0, 0.0),
                    color: glam::Vec2::new(1.0, 1.0),
                });
                vb.push(Vertex {
                    pos: offset + scale * glam::Vec2::new(0.5, -0.5),
                    uv: glam::Vec2::new(1.0, 1.0),
                    color: glam::Vec2::new(1.0, 1.0),
                });
                vb.push(Vertex {
                    pos: offset + scale * glam::Vec2::new(0.5, 0.5),
                    uv: glam::Vec2::new(1.0, 0.0),
                    color: glam::Vec2::new(1.0, 1.0),
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