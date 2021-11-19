use idek::prelude::*;
use std::time::Instant;

fn main() -> Result<()> {
    launch::<TriangleApp>(Settings::default().vr_if_any_args())
}

const TRIANGLE_MESH: [Vertex; 3] = [
    Vertex {
        pos: [0., 0.5, 0.],
        color: [1., 0., 0.],
    },
    Vertex {
        pos: [0.5, -0.5, 0.],
        color: [0., 0., 1.],
    },
    Vertex {
        pos: [-0.5, -0.5, 0.],
        color: [0., 1., 0.],
    },
];

struct TriangleApp {
    verts: VertexBuffer,
    time: Instant,
}

impl App for TriangleApp {
    fn init(ctx: &mut Context, _: &mut Platform) -> Result<Self> {
        let verts = ctx.vertices(&TRIANGLE_MESH, true)?;
        Ok(Self {
            verts,
            time: Instant::now(),
        })
    }

    fn frame(&mut self, ctx: &mut Context, _: &mut Platform) -> Result<Vec<DrawCmd>> {
        let mut mesh = TRIANGLE_MESH;
        let time = self.time.elapsed().as_secs_f32();
        mesh[0].pos = [time.cos() * 0.3, 0.5, 0.];
        ctx.update_vertices(self.verts, &mesh)?;
        Ok(vec![DrawCmd::new(self.verts)])
    }
}
