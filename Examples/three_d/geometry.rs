use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vert3D {
    pos: [f32; 4],
    uw: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Circle {
    vertices: Vec<Vert3D>,
    indices: Vec<u32>,
}

impl Circle {
    pub fn from_radius_segments(radius: f32, segments: u32) -> Self {
        let mut vertices = Vec::with_capacity(segments as usize + 1);
        vertices.push(Vert3D {
            pos: [0.0, 0.0, 0.0, 1.0],
            uw: [0.5, 0.5],
        });
        for i in 0..=segments {
            let angle = 2.0 * std::f32::consts::PI * (i as f32) / (segments as f32);
            vertices.push(Vert3D {
                pos: [angle.cos() * radius, angle.sin() * radius, 0.0, 1.0],
                uw: [0.5 + 0.5 * angle.cos(), 0.5 + 0.5 * angle.sin()],
            });
        }

        let mut indices = Vec::with_capacity(segments as usize * 3);
        for i in 0..segments {
            indices.push(0);
            indices.push(i + 1);
            indices.push(i + 2);
        }

        Self { vertices, indices }
    }
    pub fn from_radius_tolerance(radius: f32, tolerance: f32) -> Self {
        let theta = 2.0 * (1.0 - tolerance / radius).acos();
        let segments = (2.0 * std::f32::consts::PI / theta).ceil() as u32;

        Self::from_radius_segments(radius, segments)
    }

    pub fn vertex_size() -> usize {
        std::mem::size_of::<Vert3D>()
    }
    pub fn vertex_count(&self) -> u32 {
        self.vertices.len() as u32
    }
    pub fn index_count(&self) -> u32 {
        self.indices.len() as u32
    }

    pub fn vertex_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.vertices)
    }
    pub fn index_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.indices)
    }
}
