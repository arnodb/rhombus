include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));

// Clear
pub const GL_DEPTH_BUFFER_BIT: u32 = 0x0100;
pub const GL_COLOR_BUFFER_BIT: u32 = 0x4000;

// Begin
pub const GL_LINES: u32 = 0x0001;
pub const GL_LINE_LOOP: u32 = 0x0002;
//pub const GL_LINE_STRIP: u32 = 0x0003;
pub const GL_TRIANGLE_FAN: u32 = 0x0006;

// Enable
pub const GL_DEPTH_TEST: u32 = 0x0B71;

// MatrixMode
pub const GL_MODELVIEW: u32 = 0x1700;
pub const GL_PROJECTION: u32 = 0x1701;
