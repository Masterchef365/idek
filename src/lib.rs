use anyhow::{Result, ensure};
mod draw_cmd;
mod engine;
pub use engine::launch;
pub use draw_cmd::DrawCmd;
pub use watertender::vertex::Vertex;
pub use watertender::mainloop::PlatformEvent as Event;
pub use watertender::trivial::Primitive;

/// Commonly used items
pub mod prelude {
    pub use super::{Settings, launch, Vertex, VertexBuffer, Context, DrawCmd, App, Event};
    pub use anyhow::Result;
}

/// Launch settings
pub struct Settings {
    /// MSAA samples. Must be a power of two (up to 16)
    pub msaa_samples: u8,

    /// If true, use OpenXR to display in VR
    pub vr: bool,

    /// Application name
    pub name: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            msaa_samples: 4,
            vr: false,
            name: "Idek".to_string(),
        }
    }
}

/// An interface for applications
pub trait App: Sized {
    /// Initialization function, called once to construct the app
    fn init(ctx: &mut Context) -> Result<Self>;

    /// Called once per frame. Most app logic should live here.
    fn frame(&mut self, ctx: &mut Context) -> Result<Vec<DrawCmd>>;

    /// Called once per event
    fn event(&mut self, _event: Event) {
    }
}

/// A transform array in row-major order
pub type Transform = [f32; 2 * 3];

pub type VertexBuffer = ();
pub type IndexBuffer = ();
pub type InstanceBuffer = ();
pub type Shader = ();
pub type Texture = ();
pub type Instance = ();

/// Context with which to change the rendering environment from within an App
pub struct Context;

impl Context {
    pub fn vertices(&mut self, vertices: &[Vertex], dynamic: bool) -> Result<VertexBuffer> {
        todo!()
    }

    pub fn indices(&mut self, indices: &[u32], dynamic: bool) -> Result<IndexBuffer> {
        todo!()
    }

    pub fn instances(&mut self, instances: &[Instance], dynamic: bool) -> Result<InstanceBuffer> {
        todo!()
    }

    pub fn shader(&mut self, vertex: &[u8], fragment: &[u8], primitive: Primitive) -> Result<Shader> {
        todo!()
    }

    #[cfg(feature = "shaderc")]
    pub fn shader_glsl(&mut self, vertex: &str, fragment: &str, primitive: Primitive) -> Result<Shader> {
        todo!()
    }

    /// Create a new texture containing the specified data with the specified width. Data must be
    /// 8-bit RGBA (4 bytes per pixel), and must be in row-major order.
    pub fn texture(&mut self, data: &[u8], width: usize, dynamic: bool) -> Result<Texture> {
        ensure!(data.len() % 4 == 0, "Image data must be RGBA");
        let total_pixels = data.len() / 4;
        ensure!(total_pixels % width == 0, "Image data length must be a multiple of width");
        let image_height = total_pixels / width;
        todo!()
    }

    pub fn screen_size(&self) -> (u32, u32) {
        todo!()
    }

    pub fn update_vertices(&mut self, buffer: &mut VertexBuffer, vertices: &[Vertex]) -> Result<()> {
        todo!()
    }
}
