include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));

pub const GL_LINES: u32 = 0x0001;
pub const GL_LINE_STRIP: u32 = 0x0003;
pub const GL_MODELVIEW: u32 = 0x1700;
pub const GL_PROJECTION: u32 = 0x1701;
pub const GL_COLOR_BUFFER_BIT: u32 = 0x4000;
