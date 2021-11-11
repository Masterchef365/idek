use std::time::Instant;

use slotmap::SlotMap;
use watertender::{prelude::*, trivial::Primitive};
use anyhow::{Result, ensure};
use watertender::defaults::FRAMES_IN_FLIGHT;
use crate::{DrawCmd, IndexBuffer, InstanceBuffer, Shader, Texture, VertexBuffer, App, Settings};

/// Launch an App
pub fn launch<A: App + 'static>(settings: crate::Settings) -> Result<()> {
    let info = AppInfo::default().name(settings.name.clone())?;
    watertender::starter_kit::launch::<EngineWrapper<A>, _>(info, settings.vr, settings)
}

/// A wrapper type to aid in managing the App and Engine lifetimes. The EngineWrapper is the actual main loop target, but it defers to the actual Engine object for everything outside of calling the App.
struct EngineWrapper<A> {
    app: A,
    engine: Engine,
}

impl<A: App> MainLoop<Settings> for EngineWrapper<A> {
    fn new(core: &SharedCore, platform: Platform<'_>, settings: Settings) -> Result<Self> {
        let mut engine = Engine::new(core, platform, settings)?;
        let app = A::init(&mut engine)?;
        Ok(Self {
            app,
            engine
        })
    }

    fn frame(
        &mut self,
        frame: Frame,
        core: &SharedCore,
        platform: Platform<'_>,
    ) -> Result<PlatformReturn> {
        let frame_packet = self.app.frame(&mut self.engine)?;
        self.engine.frame(frame_packet, frame, core, platform)
    }

    fn swapchain_resize(&mut self, images: Vec<vk::Image>, extent: vk::Extent2D) -> Result<()> {
        self.engine.swapchain_resize(images, extent)
    }

    fn event(
        &mut self,
        event: PlatformEvent<'_, '_>,
        _core: &Core,
        _platform: Platform<'_>,
    ) -> Result<()> {
        Ok(self.app.event(event))
    }
}

impl<A: App> SyncMainLoop<Settings> for EngineWrapper<A> {
    fn winit_sync(&self) -> (vk::Semaphore, vk::Semaphore) {
        self.engine.winit_sync()
    }
}

/// Wrapper to handle dynamic buffers with greater ease
enum FrameKeyed<T> {
    Singular(T),
    Dynamic([T; FRAMES_IN_FLIGHT]),
}

impl<T> FrameKeyed<T> {
    pub fn get(&self, frame: usize) -> &T {
        match self {
            FrameKeyed::Singular(v) => v,
            FrameKeyed::Dynamic(v) => &v[frame],
        }
    }

    pub fn get_mut(&mut self, frame: usize) -> &mut T {
        match self {
            FrameKeyed::Singular(v) => v,
            FrameKeyed::Dynamic(v) => &mut v[frame],
        }
    }
}

/// All data inside the scene UBO
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct SceneData {
    cameras: [f32; 4 * 4 * 2],
    time: f32,
}

/// The engine object. Also known as the "Context" from within usercode.
pub struct Engine {
    vertex_bufs: SlotMap<VertexBuffer, FrameKeyed<ManagedBuffer>>,
    index_bufs: SlotMap<IndexBuffer, FrameKeyed<ManagedBuffer>>,
    instance_bufs: SlotMap<InstanceBuffer, FrameKeyed<ManagedBuffer>>,
    shaders: SlotMap<Shader, vk::Pipeline>,
    textures: SlotMap<Texture, FrameKeyed<ManagedImage>>,

    descriptor_sets: Vec<vk::DescriptorSet>,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,

    scene_ubo: FrameDataUbo<SceneData>,
    starter_kit: StarterKit,

    start_time: Instant,
}

type Instance = (); // TODO

// Public functions ("Context")
impl Engine {
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

    pub fn update_vertices(&mut self, buffer: VertexBuffer, vertices: &[Vertex]) -> Result<()> {
        todo!()
    }

    pub fn update_indices(&mut self, buffer: VertexBuffer, vertices: &[u32]) -> Result<()> {
        todo!()
    }
}

impl Engine {
    fn new(core: &SharedCore, platform: Platform<'_>, settings: Settings) -> Result<Self> {
        todo!()
    }

    fn frame(
        &mut self,
        packet: Vec<DrawCmd>,
        frame: Frame,
        core: &SharedCore,
        platform: Platform<'_>,
    ) -> Result<PlatformReturn> {
        todo!()
    }

    fn swapchain_resize(&mut self, images: Vec<vk::Image>, extent: vk::Extent2D) -> Result<()> {
        todo!()
    }

    fn winit_sync(&self) -> (vk::Semaphore, vk::Semaphore) {
        todo!()
    }
}