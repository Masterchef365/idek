use idek::{prelude::*, IndexBuffer};
use std::fs;

fn main() -> Result<()> {
    launch::<_, TriangleApp>(Settings::default().vr_if_any_args())
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
    3, 1, 0, 3, 2, 1, // Facing away
    0, 1, 3, 1, 2, 3,
];

struct TriangleApp {
    verts: VertexBuffer,
    indices: IndexBuffer,
    shader: Shader,
}

impl App for TriangleApp {
    fn init(ctx: &mut Context, _: &mut Platform, _: ()) -> Result<Self> {
        let verts = ctx.vertices(&QUAD_VERTS, false)?;
        let indices = ctx.indices(&QUAD_INDICES, false)?;

        let custom_frag = fs::read("examples/custom.frag.spv")?;
        let shader = ctx.shader(DEFAULT_VERTEX_SHADER, &custom_frag, Primitive::Triangles)?;

        Ok(Self {
            verts,
            indices,
            shader,
        })
    }

    fn frame(&mut self, _ctx: &mut Context, _: &mut Platform) -> Result<Vec<DrawCmd>> {
        Ok(vec![DrawCmd::new(self.verts)
            .indices(self.indices)
            .shader(self.shader)])
    }
}
