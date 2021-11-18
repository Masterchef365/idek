use anyhow::Result;
mod draw_cmd;
mod engine;
pub use engine::launch;
pub use draw_cmd::DrawCmd;
pub use watertender::vertex::Vertex;
pub use watertender::mainloop::{PlatformEvent as Event, Platform};
pub use watertender::trivial::Primitive;

pub use watertender::winit;
pub use watertender::openxr;

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
    fn frame(&mut self, ctx: &mut Context, platform: &mut Platform) -> Result<Vec<DrawCmd>>;

    /// Called once per event
    fn event(&mut self, event: Event, platform: &mut Platform) {
        match (event, platform) {
            (
                Event::Winit(winit::event::Event::WindowEvent { event: winit::event::WindowEvent::CloseRequested, .. }),
                Platform::Winit { control_flow, .. }
            ) => **control_flow = winit::event_loop::ControlFlow::Exit,
            _ => (),
        }
    }
}

/// A transform array in row-major order
pub type Transform = [f32; 4 * 4];

slotmap::new_key_type! {
    pub struct VertexBuffer;
    pub struct IndexBuffer;
    pub struct InstanceBuffer;
    pub struct Shader;
    pub struct Texture;
}

/// Context with which to change the rendering environment from within an App
pub type Context = engine::Engine;
