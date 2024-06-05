#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct MvpPushConst {
    pub mvp: glam::Mat4,
}

impl MvpPushConst {
    pub fn size_in_bytes() -> u32 {
        std::mem::size_of::<Self>() as u32
    }
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}
