use anyhow::Result;
mod draw_cmd;
mod engine;
pub use draw_cmd::DrawCmd;
pub use engine::launch;
pub use watertender::mainloop::{Platform, PlatformEvent as Event};
use watertender::nalgebra::{Matrix4, Vector4};
pub use watertender::vertex::Vertex;

/// An arcball camera for windowed mode
pub use watertender::winit_arcball::WinitArcBall;

/// An arcball in windowed contexts, and a system-specific camera in VR contexts
pub use watertender::multi_platform_camera::MultiPlatformCamera;

/// Primitive to draw with
pub use watertender::trivial::Primitive;

pub use watertender::nalgebra;
#[cfg(feature = "openxr")]
pub use watertender::openxr;
pub use watertender::winit;

pub static DEFAULT_VERTEX_SHADER: &[u8] = include_bytes!("shaders/unlit.vert.spv");
pub static DEFAULT_FRAGMENT_SHADER: &[u8] = include_bytes!("shaders/unlit.frag.spv");

/// Commonly used items
pub mod prelude {
    pub use super::{
        launch, App, Context, DrawCmd, Event, IndexBuffer, MultiPlatformCamera, Platform,
        Primitive, Settings, Shader, Vertex, VertexBuffer, DEFAULT_FRAGMENT_SHADER,
        DEFAULT_VERTEX_SHADER,
    };
    pub use anyhow::Result;
}

/// Launch settings
pub struct Settings<Args = ()> {
    /// MSAA samples. Must be a power of two (up to 16)
    pub msaa_samples: u8,

    /// If true, use OpenXR to display in VR
    pub vr: bool,

    /// Application name
    pub name: String,

    /// Maximum number of transforms able to be used at once
    pub max_transforms: usize,

    /// User-defined arguments
    pub args: Args,
}

impl<T: Default> Default for Settings<T> {
    fn default() -> Self {
        Self {
            msaa_samples: 4,
            vr: false,
            name: "Idek".to_string(),
            max_transforms: 10_000,
            args: Default::default(),
        }
    }
}

impl<Args> Settings<Args> {
    /// Optionally enable VR
    pub fn vr(mut self, vr: bool) -> Self {
        self.vr = vr;
        self
    }

    /// Set application name
    pub fn name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    /// Enable VR if there are any command line arguments. Useful for debugging or simple use-cases
    pub fn vr_if_any_args(mut self) -> Self {
        self.vr = std::env::args().skip(1).next().is_some();
        self
    }

    /// Set the number of MSAA samples
    pub fn msaa_samples(mut self, samples: u8) -> Self {
        self.msaa_samples = samples;
        self
    }

    /// Enable VR if there are any command line arguments
    pub fn max_transforms(mut self, max_transforms: usize) -> Self {
        self.max_transforms = max_transforms;
        self
    }

    /// Set the user args for your app
    pub fn args(mut self, args: Args) -> Self {
        self.args = args;
        self
    }
}

/// An interface for applications
pub trait App<Args = ()>: Sized {
    /// Initialization function, called once to construct the app
    fn init(ctx: &mut Context, platform: &mut Platform, args: Args) -> Result<Self>;

    /// Called once per frame. Most app logic should live here.
    fn frame(&mut self, ctx: &mut Context, platform: &mut Platform) -> Result<Vec<DrawCmd>>;

    /// Called once per event
    fn event(&mut self, _ctx: &mut Context, platform: &mut Platform, event: Event) -> Result<()> {
        Ok(close_when_asked(platform, &event))
    }
}

/// Close the winit window when asked
pub fn close_when_asked(platform: &mut Platform, event: &Event) {
    match (event, platform) {
        (
            Event::Winit(winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            }),
            Platform::Winit { control_flow, .. },
        ) => **control_flow = winit::event_loop::ControlFlow::Exit,
        _ => (),
    }
}

/// A transform array in row-major order
pub type Transform = [[f32; 4]; 4];

slotmap::new_key_type! {
    /// A vertex buffer
    pub struct VertexBuffer;
    /// An index buffer
    pub struct IndexBuffer;
    /// A shader
    pub struct Shader;
    /// A texture
    pub struct Texture;
    // /// An instance buffer
    // pub struct InstanceBuffer;
}

/// Context with which to change the rendering environment from within an App
pub use engine::Engine;
pub type Context = engine::Engine;

/// Return a camera prefix matrix which keeps (-1, 1) on XY visible and at a 1:1 aspect ratio
pub fn simple_ortho_cam((width, height): (u32, u32)) -> Matrix4<f32> {
    let (width, height) = (width as f32, height as f32);
    let diag = match width < height {
        true => Vector4::new(1., width / height, 1., 1.),
        false => Vector4::new(height / width, 1., 1., 1.),
    };
    Matrix4::from_diagonal(&diag)
}

/// Same as `simple_ortho_cam` but using the builtin inputs
pub fn simple_ortho_cam_ctx(ctx: &mut Context, platform: &mut Platform) {
    if !platform.is_vr() {
        ctx.set_camera_prefix(simple_ortho_cam(ctx.screen_size()));
    }
}
