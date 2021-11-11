use std::time::Instant;
use slotmap::SlotMap;
use watertender::{prelude::*, trivial::Primitive};
use anyhow::{Result, ensure};
use watertender::defaults::FRAMES_IN_FLIGHT;
use crate::{App, DrawCmd, IndexBuffer, InstanceBuffer, Settings, Shader, Texture, VertexBuffer};

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

unsafe impl bytemuck::Zeroable for SceneData {}
unsafe impl bytemuck::Pod for SceneData {}

/// The engine object. Also known as the "Context" from within usercode.
pub struct Engine {
    vertex_bufs: SlotMap<VertexBuffer, FrameKeyed<ManagedBuffer>>,
    index_bufs: SlotMap<IndexBuffer, FrameKeyed<ManagedBuffer>>,
    instance_bufs: SlotMap<InstanceBuffer, FrameKeyed<ManagedBuffer>>,
    shaders: SlotMap<Shader, vk::Pipeline>,
    textures: SlotMap<Texture, FrameKeyed<ManagedImage>>,

    default_shader_key: Shader,

    descriptor_sets: Vec<vk::DescriptorSet>,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,

    pipeline_layout: vk::PipelineLayout,

    scene_ubo: FrameDataUbo<SceneData>,
    starter_kit: StarterKit,

    camera_prefix: [f32; 4 * 4],

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
    fn new(core: &SharedCore, mut platform: Platform<'_>, settings: Settings) -> Result<Self> {
        // Boilerplate
        let starter_kit = StarterKit::new(core.clone(), &mut platform)?;

        // Scene UBO
        let scene_ubo = FrameDataUbo::new(core.clone(), FRAMES_IN_FLIGHT)?;

        // Create descriptor set layout
        const FRAME_DATA_BINDING: u32 = 0;
        let bindings = [
            vk::DescriptorSetLayoutBindingBuilder::new()
                .binding(FRAME_DATA_BINDING)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::ALL_GRAPHICS),
        ];

        let descriptor_set_layout_ci =
            vk::DescriptorSetLayoutCreateInfoBuilder::new().bindings(&bindings);

        let descriptor_set_layout = unsafe {
            core.device
                .create_descriptor_set_layout(&descriptor_set_layout_ci, None, None)
        }
        .result()?;

        // Create descriptor pool
        let pool_sizes = [
            vk::DescriptorPoolSizeBuilder::new()
                ._type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(FRAMES_IN_FLIGHT as _),
        ];

        let create_info = vk::DescriptorPoolCreateInfoBuilder::new()
            .pool_sizes(&pool_sizes)
            .max_sets((FRAMES_IN_FLIGHT * 2) as _);

        let descriptor_pool =
            unsafe { core.device.create_descriptor_pool(&create_info, None, None) }.result()?;

        // Create descriptor sets
        let layouts = vec![descriptor_set_layout; FRAMES_IN_FLIGHT];
        let create_info = vk::DescriptorSetAllocateInfoBuilder::new()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);

        let descriptor_sets =
            unsafe { core.device.allocate_descriptor_sets(&create_info) }.result()?;

        // Write descriptor sets
        for (frame, &descriptor_set) in descriptor_sets.iter().enumerate() {
            let frame_data_bi = [scene_ubo.descriptor_buffer_info(frame)];
            let writes = [
                vk::WriteDescriptorSetBuilder::new()
                    .buffer_info(&frame_data_bi)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .dst_set(descriptor_set)
                    .dst_binding(FRAME_DATA_BINDING)
                    .dst_array_element(0),
            ];

            unsafe {
                core.device.update_descriptor_sets(&writes, &[]);
            }
        }

        let descriptor_set_layouts = [descriptor_set_layout];

        // Pipeline layout
        let push_constant_ranges = [vk::PushConstantRangeBuilder::new()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(std::mem::size_of::<[f32; 4 * 4]>() as u32)];

        let create_info = vk::PipelineLayoutCreateInfoBuilder::new()
            .push_constant_ranges(&push_constant_ranges)
            .set_layouts(&descriptor_set_layouts);

        let pipeline_layout =
            unsafe { core.device.create_pipeline_layout(&create_info, None, None) }.result()?;

        let mut shaders = SlotMap::with_key();

        let default_shader = shader(
            core,
            include_bytes!("shaders/unlit.vert.spv"),
            include_bytes!("shaders/unlit.frag.spv"),
            Primitive::Triangles.into(),
            starter_kit.render_pass,
            pipeline_layout,
        )?;

        let default_shader_key = shaders.insert(default_shader);

        Ok(Self {
            shaders,
            vertex_bufs: SlotMap::with_key(),
            index_bufs: SlotMap::with_key(),
            instance_bufs: SlotMap::with_key(),
            textures: SlotMap::with_key(),

            default_shader_key,

            descriptor_sets,
            descriptor_pool,
            descriptor_set_layout,
            pipeline_layout,

            scene_ubo,
            starter_kit,

            camera_prefix: [
                1., 0., 0., 0., //
                0., 1., 0., 0., //
                0., 0., 1., 0., //
                0., 0., 0., 1., //
            ],

            start_time: Instant::now(),
        })
    }

    fn frame(
        &mut self,
        packet: Vec<DrawCmd>,
        frame: Frame,
        core: &SharedCore,
        platform: Platform<'_>,
    ) -> Result<PlatformReturn> {
        let cmd = self.starter_kit.begin_command_buffer(frame)?;
        let command_buffer = cmd.command_buffer;

        unsafe {
            core.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                &[self.descriptor_sets[self.starter_kit.frame]],
                &[],
            );

            for cmd in packet {
                core.device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    *self.shaders.get(cmd.shader.unwrap_or(self.default_shader_key)).unwrap()
                );
            }
        }

        let (ret, cameras) = watertender::multi_platform_camera::platform_camera_prefix(&platform, self.camera_prefix)?;

        self.scene_ubo.upload(
            self.starter_kit.frame,
            &SceneData {
                cameras,
                time: self.start_time.elapsed().as_secs_f32(),
            },
        )?;

        // End draw cmds
        self.starter_kit.end_command_buffer(cmd)?;

        Ok(ret)
 
    }

    fn swapchain_resize(&mut self, images: Vec<vk::Image>, extent: vk::Extent2D) -> Result<()> {
        todo!()
    }

    fn winit_sync(&self) -> (vk::Semaphore, vk::Semaphore) {
        todo!()
    }
}