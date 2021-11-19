use idek::{IndexBuffer, WinitArcBall, prelude::*, MultiPlatformCamera};

fn main() -> Result<()> {
    launch::<TriangleApp>(Settings::default().vr_if_any_args())
}

struct TriangleApp {
    verts: VertexBuffer,
    indices: IndexBuffer,
    camera: MultiPlatformCamera,
}

impl App for TriangleApp {
    fn init(ctx: &mut Context, platform: &mut Platform) -> Result<Self> {
        Ok(Self {
            verts: ctx.vertices(&QUAD_VERTS, false)?,
            indices: ctx.indices(&QUAD_INDICES, false)?,
            camera: MultiPlatformCamera::new(platform)
        })
    }

    fn frame(&mut self, ctx: &mut Context, _: &mut Platform) -> Result<Vec<DrawCmd>> {
        Ok(vec![DrawCmd::new(self.verts).indices(self.indices)])
    }

    fn event(&mut self, ctx: &mut Context, platform: &mut Platform, mut event: Event) -> Result<()> {
        if self.camera.handle_event(&mut event) {
            ctx.set_camera_prefix(self.camera.get_prefix(platform))
        }
        idek::close_when_asked(platform, &event);
        Ok(())
    }
}

const QUAD_VERTS: [Vertex; 4] = [
    Vertex {
        pos: [-1., -1., 0.],
        color: [0., 0., 0.],
    },
    Vertex {
        pos: [1., -1., 0.],
        color: [1., 0., 0.],
    },
    Vertex {
        pos: [1., 1., 0.],
        color: [1., 1., 0.],
    },
    Vertex {
        pos: [-1., 1., 0.],
        color: [0., 1., 0.],
    },
];

const QUAD_INDICES: [u32; 12] = [
    // Facing toward the camera
    3, 1, 0, 3, 2, 1,
    // Facing away
    0, 1, 3, 1, 2, 3,
];