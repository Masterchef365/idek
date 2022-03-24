use crate::{App, DrawCmd, IndexBuffer, Settings, Shader, VertexBuffer, Blend, Transform};
use crate::{DEFAULT_FRAGMENT_SHADER, DEFAULT_VERTEX_SHADER};
use anyhow::{ensure, Result};
use slotmap::SlotMap;
use std::marker::PhantomData;
use std::time::Instant;
use watertender::defaults::FRAMES_IN_FLIGHT;
use watertender::{
    memory::UsageFlags, nalgebra::Matrix4, prelude::*, trivial::Primitive, vk::CommandBuffer,
    erupt,
};
use erupt::{utils, vk};
use std::ffi::CString;


pub const TRANSFORM_IDENTITY: Transform = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// Launch an App
pub fn launch<Args: 'static, A: App<Args> + 'static>(
    settings: crate::Settings<Args>,
) -> Result<()> {
    let info = AppInfo::default()
        .validation(cfg!(debug_assertions))
        .name(settings.name.clone())?;
    watertender::starter_kit::launch::<EngineWrapper<Args, A>, _>(info, settings.vr, settings)
}

/// A wrapper type to aid in managing the App and Engine lifetimes. The EngineWrapper is the actual main loop target, but it defers to the actual Engine object for everything outside of calling the App.
struct EngineWrapper<Args, A: App<Args>> {
    app: A,
    engine: Engine,
    _phantomdata: PhantomData<Args>,
}

// Implement the MainLoop trait for the wrapper
impl<Args, A: App<Args>> MainLoop<Settings<Args>> for EngineWrapper<Args, A> {
    fn new(
        core: &SharedCore,
        mut platform: Platform<'_>,
        settings: Settings<Args>,
    ) -> Result<Self> {
        let mut engine = Engine::new(core, &mut platform, &settings)?;

        let app = A::init(&mut engine, &mut platform, settings.args)?;

        Ok(Self {
            app,
            engine,
            _phantomdata: PhantomData,
        })
    }

    fn frame(
        &mut self,
        frame: Frame,
        core: &SharedCore,
        mut platform: Platform,
    ) -> Result<PlatformReturn> {
        let frame_packet = self.app.frame(&mut self.engine, &mut platform)?;

        self.engine.frame(frame_packet, frame, core, &mut platform)
    }

    fn swapchain_resize(&mut self, images: Vec<vk::Image>, extent: vk::Extent2D) -> Result<()> {
        self.engine.swapchain_resize(images, extent)
    }

    fn event(
        &mut self,
        event: PlatformEvent<'_, '_>,
        _core: &Core,
        mut platform: Platform,
    ) -> Result<()> {
        self.app.event(&mut self.engine, &mut platform, event)
    }
}

impl<Args, A: App<Args>> SyncMainLoop<Settings<Args>> for EngineWrapper<Args, A> {
    fn winit_sync(&self) -> (vk::Semaphore, vk::Semaphore) {
        self.engine.winit_sync()
    }
}

enum UploadBuffer {
    Static(ManagedBuffer),
    Dynamic(Vec<ManagedBuffer>),
}

impl UploadBuffer {
    /// Create a new buffer initialized with `data`.
    pub fn new(core: &SharedCore, data: &[u8], dynamic: bool) -> Result<Self> {
        let mut instance = Self::new_empty(core, data.len() as _, dynamic)?;
        match &mut instance {
            Self::Static(buf) => buf.write_bytes(0, data)?,
            Self::Dynamic(bufs) => {
                for buf in bufs {
                    buf.write_bytes(0, data)?;
                }
            }
        }
        Ok(instance)
    }

    /// Write to a dynamic buffer. Panics if the buffer is not dynamic
    pub fn write(&mut self, frame: usize, data: &[u8]) -> Result<()> {
        match self {
            Self::Static(_) => panic!("Attempted to write to a static buffer"),
            Self::Dynamic(bufs) => bufs[frame].write_bytes(0, data),
        }
    }

    /// Get the internal buffer for the current frame
    pub fn buffer(&self, frame: usize) -> vk::Buffer {
        match self {
            Self::Static(buf) => buf.buffer(),
            Self::Dynamic(bufs) => bufs[frame].buffer(),
        }
    }

    pub fn new_empty(core: &SharedCore, size: u64, dynamic: bool) -> Result<Self> {
        let ci = vk::BufferCreateInfoBuilder::new()
            .usage(vk::BufferUsageFlags::TRANSFER_SRC)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&[])
            .size(size);
        let make_buf = || ManagedBuffer::new(core.clone(), ci, UsageFlags::UPLOAD);
        Ok(match dynamic {
            true => Self::Dynamic(
                (0..FRAMES_IN_FLIGHT)
                    .map(|_| make_buf())
                    .collect::<Result<_>>()?,
            ),
            false => Self::Static(make_buf()?),
        })
    }
}

enum QueuedUpload {
    VertexBuffer(VertexBuffer),
    IndexBuffer(IndexBuffer),
    //Texture(Texture),
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

/// CPU-GPU synchronized memory. Might be dynamic.
struct SyncMemory {
    /// GPU-side memory (FAST_DEVICE_ACCESS)
    gpu: ManagedBuffer,
    /// CPU-side memory (UPLOAD)
    cpu: UploadBuffer,
    /// Size in bytes
    size_bytes: u64,
    /// Length (# of vertices, indices, instances)
    length: u32,
}

/// The engine object. Also known as the "Context" from within usercode.
pub struct Engine {
    vertex_bufs: SlotMap<VertexBuffer, SyncMemory>,
    index_bufs: SlotMap<IndexBuffer, SyncMemory>,
    //instance_bufs: SlotMap<InstanceBuffer, SyncMemory>,
    shaders: SlotMap<Shader, vk::Pipeline>,
    //textures: SlotMap<Texture, (ManagedImage, UploadBuffer)>,
    /// Trivial built-in shader
    default_shader_key: Shader,

    descriptor_sets: Vec<vk::DescriptorSet>,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,

    pipeline_layout: vk::PipelineLayout,

    scene_ubo: FrameDataUbo<SceneData>,
    starter_kit: StarterKit,

    transforms: Vec<ManagedBuffer>,

    camera_prefix: Matrix4<f32>,

    /// Uploads to be completed during the next frame
    queued_uploads: Vec<QueuedUpload>,

    start_time: Instant,
}

// type Instance = ();

// Public functions ("Context")
impl Engine {
    /// Upload a set of vertices
    pub fn vertices(&mut self, vertices: &[Vertex], dynamic: bool) -> Result<VertexBuffer> {
        let size_bytes = std::mem::size_of_val(vertices) as u64;
        let ci = vk::BufferCreateInfoBuilder::new()
            .usage(vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&[])
            .size(size_bytes);

        let gpu_buf = ManagedBuffer::new(
            self.starter_kit.core.clone(),
            ci,
            UsageFlags::FAST_DEVICE_ACCESS,
        )?;

        let upload_buf = UploadBuffer::new(
            &self.starter_kit.core,
            bytemuck::cast_slice(vertices),
            dynamic,
        )?;

        let key = self.vertex_bufs.insert(SyncMemory {
            cpu: upload_buf,
            gpu: gpu_buf,
            size_bytes,
            length: vertices.len() as _,
        });

        self.queued_uploads.push(QueuedUpload::VertexBuffer(key));

        Ok(key)
    }

    /// Upload a set of indices
    pub fn indices(&mut self, indices: &[u32], dynamic: bool) -> Result<IndexBuffer> {
        let size_bytes = std::mem::size_of_val(indices) as u64;
        let ci = vk::BufferCreateInfoBuilder::new()
            .usage(vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&[])
            .size(size_bytes);

        let gpu_buf = ManagedBuffer::new(
            self.starter_kit.core.clone(),
            ci,
            UsageFlags::FAST_DEVICE_ACCESS,
        )?;

        let upload_buf = UploadBuffer::new(
            &self.starter_kit.core,
            bytemuck::cast_slice(indices),
            dynamic,
        )?;

        let key = self.index_bufs.insert(SyncMemory {
            cpu: upload_buf,
            gpu: gpu_buf,
            size_bytes,
            length: indices.len() as _,
        });

        self.queued_uploads.push(QueuedUpload::IndexBuffer(key));

        Ok(key)
    }

    /*pub fn instances(&mut self, instances: &[Instance], dynamic: bool) -> Result<InstanceBuffer> {
        todo!()
    }*/

    /// Upload a shader
    pub fn shader(
        &mut self,
        vertex: &[u8],
        fragment: &[u8],
        primitive: Primitive,
        blend: Blend,
    ) -> Result<Shader> {
        Ok(self.shaders.insert(shader(
            &self.starter_kit.core,
            vertex,
            fragment,
            primitive.into(),
            self.starter_kit.render_pass,
            self.pipeline_layout,
            self.starter_kit.msaa_samples,
            blend,
        )?))
    }

    /// Compile and upload the given shader source
    #[cfg(feature = "shaderc")]
    pub fn shader_glsl(
        &mut self,
        vertex: &str,
        fragment: &str,
        primitive: Primitive,
    ) -> Result<Shader> {
        todo!()
    }

    /*
    /// Create a new texture containing the specified data with the specified width. Data must be
    /// 8-bit RGBA (4 bytes per pixel), and must be in row-major order.
    pub fn texture(&mut self, data: &[u8], width: usize, dynamic: bool) -> Result<Texture> {
        ensure!(data.len() % 4 == 0, "Image data must be RGBA");
        let total_pixels = data.len() / 4;
        ensure!(
            total_pixels % width == 0,
            "Image data length must be a multiple of width"
        );
        let image_height = total_pixels / width;
        todo!()
    }
    */

    /// Returns the current screen size in pixels
    /// (width, height)
    pub fn screen_size(&self) -> (u32, u32) {
        let extent = self.starter_kit.framebuffer.extent();
        (extent.width, extent.height)
    }

    /// Return the time since the engine started
    pub fn start_time(&self) -> Instant {
        self.start_time
    }

    /// Set the camera prefix. This transformation is applied to each vertex. In the OpenXR backend,
    /// this is applied before the camera view and projection matrices
    pub fn set_camera_prefix(&mut self, matrix: Matrix4<f32>) {
        self.camera_prefix = matrix;
    }

    /// Dynamically upload vertices. Possibly only if the buffer was created as dynamic
    pub fn update_vertices(&mut self, handle: VertexBuffer, vertices: &[Vertex]) -> Result<()> {
        let memory = self.vertex_bufs.get_mut(handle).unwrap();
        let bytes = bytemuck::cast_slice(vertices);
        //assert_eq!(bytes.len() as u64, memory.size, "Must write exactly as many vertices as the original buffer");
        memory.cpu.write(self.starter_kit.frame, bytes)?;
        self.queued_uploads.push(QueuedUpload::VertexBuffer(handle));
        Ok(())
    }

    /// Dynamically upload indices. Possibly only if the buffer was created as dynamic
    pub fn update_indices(&mut self, handle: IndexBuffer, indices: &[u32]) -> Result<()> {
        let memory = self.index_bufs.get_mut(handle).unwrap();
        let bytes = bytemuck::cast_slice(indices);
        //assert_eq!(bytes.len() as u64, memory.size, "Must write exactly as many vertices as the original buffer");
        memory.cpu.write(self.starter_kit.frame, bytes)?;
        self.queued_uploads.push(QueuedUpload::IndexBuffer(handle));
        Ok(())
    }
}

fn create_transform_buffers(
    core: &SharedCore,
    max_transforms: usize,
) -> Result<Vec<ManagedBuffer>> {
    let total_size = std::mem::size_of::<Transform>() * max_transforms;
    let ci = vk::BufferCreateInfoBuilder::new()
        .size(total_size as u64)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .usage(vk::BufferUsageFlags::STORAGE_BUFFER);
    (0..FRAMES_IN_FLIGHT)
        .map(|_| ManagedBuffer::new(core.clone(), ci, watertender::memory::UsageFlags::UPLOAD))
        .collect::<Result<Vec<_>>>()
}

impl Engine {
    fn new<Args>(
        core: &SharedCore,
        platform: &mut Platform<'_>,
        settings: &Settings<Args>,
    ) -> Result<Self> {
        // Boilerplate
        let starter_kit = StarterKit::new(
            core.clone(),
            platform,
            watertender::starter_kit::Settings {
                msaa_samples: settings.msaa_samples as _,
                ..Default::default()
            },
        )?;

        // Scene UBO
        let scene_ubo = FrameDataUbo::new(core.clone(), FRAMES_IN_FLIGHT)?;

        // Transforms data
        // TODO: Auto-resize! (Use a deletion queue)
        let transforms = create_transform_buffers(core, settings.max_transforms)?;

        // Create descriptor set layout
        const FRAME_DATA_BINDING: u32 = 0;
        const TRANSFORM_BINDING: u32 = 1;
        let bindings = [
            vk::DescriptorSetLayoutBindingBuilder::new()
                .binding(FRAME_DATA_BINDING)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::ALL_GRAPHICS),
            vk::DescriptorSetLayoutBindingBuilder::new()
                .binding(TRANSFORM_BINDING)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
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
            vk::DescriptorPoolSizeBuilder::new()
                ._type(vk::DescriptorType::STORAGE_BUFFER)
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
            let transform_bi = [vk::DescriptorBufferInfoBuilder::new()
                .buffer(transforms[frame].buffer())
                .offset(0)
                .range(vk::WHOLE_SIZE)];

            let writes = [
                vk::WriteDescriptorSetBuilder::new()
                    .buffer_info(&frame_data_bi)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .dst_set(descriptor_set)
                    .dst_binding(FRAME_DATA_BINDING)
                    .dst_array_element(0),
                vk::WriteDescriptorSetBuilder::new()
                    .buffer_info(&transform_bi)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .dst_set(descriptor_set)
                    .dst_binding(TRANSFORM_BINDING)
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
            .size(std::mem::size_of::<u32>() as u32)];

        let create_info = vk::PipelineLayoutCreateInfoBuilder::new()
            .push_constant_ranges(&push_constant_ranges)
            .set_layouts(&descriptor_set_layouts);

        let pipeline_layout =
            unsafe { core.device.create_pipeline_layout(&create_info, None, None) }.result()?;

        let mut shaders = SlotMap::with_key();

        let default_shader = shader(
            core,
            DEFAULT_VERTEX_SHADER,
            DEFAULT_FRAGMENT_SHADER,
            Primitive::Triangles.into(),
            starter_kit.render_pass,
            pipeline_layout,
            starter_kit.msaa_samples,
            Blend::Opaque,
        )?;

        let default_shader_key = shaders.insert(default_shader);

        Ok(Self {
            shaders,
            vertex_bufs: SlotMap::with_key(),
            index_bufs: SlotMap::with_key(),
            //instance_bufs: SlotMap::with_key(),
            //textures: SlotMap::with_key(),
            default_shader_key,

            transforms,

            queued_uploads: vec![],

            descriptor_sets,
            descriptor_pool,
            descriptor_set_layout,
            pipeline_layout,

            scene_ubo,
            starter_kit,

            camera_prefix: Matrix4::identity(),

            start_time: Instant::now(),
        })
    }

    fn frame(
        &mut self,
        packet: Vec<DrawCmd>,
        frame: Frame,
        core: &SharedCore,
        platform: &mut Platform,
    ) -> Result<PlatformReturn> {
        let cmd = self.starter_kit.begin_command_buffer(&frame)?;
        let command_buffer = cmd.command_buffer;

        unsafe {
            // Upload buffers
            for job in self.queued_uploads.drain(..) {
                match job {
                    QueuedUpload::VertexBuffer(key) => {
                        let memory = self.vertex_bufs.get(key).unwrap();
                        write_cpu_gpu_copy(
                            &self.starter_kit.core,
                            command_buffer,
                            memory,
                            self.starter_kit.frame,
                        );
                    }
                    QueuedUpload::IndexBuffer(key) => {
                        let memory = self.index_bufs.get(key).unwrap();
                        write_cpu_gpu_copy(
                            &self.starter_kit.core,
                            command_buffer,
                            memory,
                            self.starter_kit.frame,
                        );
                    }
                }
            }

            // Make sure buffer uploads are synchronized
            let buf_upload_mem_barrier = vk::MemoryBarrierBuilder::new()
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ);

            core.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::ALL_GRAPHICS,
                None,
                &[buf_upload_mem_barrier],
                &[],
                &[],
            );

            self.starter_kit.begin_render_pass(&frame);
            self.starter_kit.set_viewport();

            // Bind UBO
            core.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                &[self.descriptor_sets[self.starter_kit.frame]],
                &[],
            );

            let mut transforms = vec![TRANSFORM_IDENTITY];

            // Draw frame packet
            for cmd in packet {
                // Bind current shader, or default if None
                core.device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    *self
                        .shaders
                        .get(cmd.shader.unwrap_or(self.default_shader_key))
                        .unwrap(),
                );

                // Add transform to the buffer if present; otherwise use the default (identity) transform.
                let transform_index;
                match cmd.transform {
                    Some(transform) => {
                        transform_index = transforms.len() as u32;
                        transforms.push(transform);
                    }
                    None => transform_index = 0,
                };

                // Transform index is conveyed via push constant
                let push_const = [transform_index];
                core.device.cmd_push_constants(
                    command_buffer,
                    self.pipeline_layout,
                    vk::ShaderStageFlags::VERTEX,
                    0,
                    std::mem::size_of_val(&push_const) as u32,
                    push_const.as_ptr() as _,
                );

                // Bind vertex buffers
                let vertex_memory = self.vertex_bufs.get(cmd.vertices).unwrap();
                core.device.cmd_bind_vertex_buffers(
                    command_buffer,
                    0,
                    &[vertex_memory.gpu.buffer()],
                    &[0],
                );

                // Draw indexed if there are indices, otherwise draw only by vertex order
                if let Some(indices) = cmd.indices {
                    let index_memory = self.index_bufs.get(indices).unwrap();
                    core.device.cmd_bind_index_buffer(
                        command_buffer,
                        index_memory.gpu.buffer(),
                        0,
                        vk::IndexType::UINT32,
                    );

                    let n_indices = cmd
                        .limit
                        .map(|limit| index_memory.length.min(limit))
                        .unwrap_or(index_memory.length);
                    core.device
                        .cmd_draw_indexed(command_buffer, n_indices, 1, 0, 0, 0)
                } else {
                    let n_vertices = cmd
                        .limit
                        .map(|limit| vertex_memory.length.min(limit))
                        .unwrap_or(vertex_memory.length);
                    core.device.cmd_draw(command_buffer, n_vertices, 1, 0, 0);
                }
            }

            // Write transforms data
            let bytes = bytemuck::cast_slice(&transforms);
            let frame = self.starter_kit.frame;
            let buffer = &mut self.transforms[frame];
            ensure!(
                (bytes.len() as u64) < buffer.memory.as_ref().unwrap().size(),
                "Maximum transforms exceeded"
            );
            buffer.write_bytes(0, bytes)?;
        }

        let (ret, cameras) = watertender::multi_platform_camera::platform_camera_prefix(
            platform,
            self.camera_prefix,
        )?;

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
        self.starter_kit.swapchain_resize(images, extent)
    }

    fn winit_sync(&self) -> (vk::Semaphore, vk::Semaphore) {
        self.starter_kit.winit_sync()
    }
}

fn write_cpu_gpu_copy(
    core: &Core,
    command_buffer: CommandBuffer,
    memory: &SyncMemory,
    frame: usize,
) {
    let region = vk::BufferCopyBuilder::new()
        .size(memory.size_bytes)
        .src_offset(0)
        .dst_offset(0);

    unsafe {
        core.device.cmd_copy_buffer(
            command_buffer,
            memory.cpu.buffer(frame),
            memory.gpu.buffer(),
            &[region],
        )
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        unsafe {
            let core = &self.starter_kit.core;
            core.device
                .device_wait_idle()
                .expect("Failed to idle the device");
            core.device
                .destroy_descriptor_pool(Some(self.descriptor_pool), None);
            core.device
                .destroy_descriptor_set_layout(Some(self.descriptor_set_layout), None);
        }
    }
}

// Build a graphics pipeline compatible with `Vertex` which renders the given primitive
pub fn shader(
    prelude: &Core,
    vertex_src: &[u8],
    fragment_src: &[u8],
    primitive: vk::PrimitiveTopology,
    render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
    msaa_samples: vk::SampleCountFlagBits,
    blend: Blend,
) -> Result<vk::Pipeline> {
    // Create shader modules
    let vert_decoded = utils::decode_spv(vertex_src)?;
    let create_info = vk::ShaderModuleCreateInfoBuilder::new().code(&vert_decoded);
    let vertex = unsafe {
        prelude
            .device
            .create_shader_module(&create_info, None, None)
    }
    .result()?;

    let frag_decoded = utils::decode_spv(fragment_src)?;
    let create_info = vk::ShaderModuleCreateInfoBuilder::new().code(&frag_decoded);
    let fragment = unsafe {
        prelude
            .device
            .create_shader_module(&create_info, None, None)
    }
    .result()?;

    let attribute_descriptions = Vertex::get_attribute_descriptions();
    let binding_descriptions = [Vertex::binding_description()];

    // Build pipeline
    let vertex_input = vk::PipelineVertexInputStateCreateInfoBuilder::new()
        .vertex_attribute_descriptions(&attribute_descriptions[..])
        .vertex_binding_descriptions(&binding_descriptions);

    let input_assembly = vk::PipelineInputAssemblyStateCreateInfoBuilder::new()
        .topology(primitive)
        .primitive_restart_enable(false);

    let viewport_state = vk::PipelineViewportStateCreateInfoBuilder::new()
        .viewport_count(1)
        .scissor_count(1);

    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state =
        vk::PipelineDynamicStateCreateInfoBuilder::new().dynamic_states(&dynamic_states);

    let rasterizer = vk::PipelineRasterizationStateCreateInfoBuilder::new()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .depth_clamp_enable(false);

    let multisampling = vk::PipelineMultisampleStateCreateInfoBuilder::new()
        .rasterization_samples(msaa_samples)
        .sample_shading_enable(false);

    let blend_settings = vk::PipelineColorBlendAttachmentStateBuilder::new()
        .color_write_mask(
            vk::ColorComponentFlags::R
                | vk::ColorComponentFlags::G
                | vk::ColorComponentFlags::B
                | vk::ColorComponentFlags::A,
        );
    
    let blend_settings = match blend {
        Blend::Opaque => blend_settings.blend_enable(false),
        Blend::Additive => blend_settings
            .blend_enable(true)
            .color_blend_op(vk::BlendOp::ADD)
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ONE)
            .alpha_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE),
    };

    let color_blend_attachments = [blend_settings];
    let color_blending = vk::PipelineColorBlendStateCreateInfoBuilder::new()
        .logic_op_enable(false)
        .attachments(&color_blend_attachments);

    let entry_point = CString::new("main")?;

    let shader_stages = [
        vk::PipelineShaderStageCreateInfoBuilder::new()
            .stage(vk::ShaderStageFlagBits::VERTEX)
            .module(vertex)
            .name(&entry_point),
        vk::PipelineShaderStageCreateInfoBuilder::new()
            .stage(vk::ShaderStageFlagBits::FRAGMENT)
            .module(fragment)
            .name(&entry_point),
    ];

    let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfoBuilder::new()
        .depth_test_enable(true)
        .depth_write_enable(match blend {
            Blend::Opaque => true,
            Blend::Additive => false,
        })
        .depth_compare_op(vk::CompareOp::LESS)
        .depth_bounds_test_enable(false)
        .stencil_test_enable(false);

    let create_info = vk::GraphicsPipelineCreateInfoBuilder::new()
        .stages(&shader_stages)
        .vertex_input_state(&vertex_input)
        .input_assembly_state(&input_assembly)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterizer)
        .multisample_state(&multisampling)
        .color_blend_state(&color_blending)
        .depth_stencil_state(&depth_stencil_state)
        .dynamic_state(&dynamic_state)
        .layout(pipeline_layout)
        .render_pass(render_pass)
        .subpass(0);

    let pipeline = unsafe {
        prelude
            .device
            .create_graphics_pipelines(None, &[create_info], None)
    }
    .result()?[0];

    unsafe {
        prelude.device.destroy_shader_module(Some(fragment), None);
        prelude.device.destroy_shader_module(Some(vertex), None);
    }

    Ok(pipeline)
}
