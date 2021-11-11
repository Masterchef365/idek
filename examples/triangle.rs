use idek::prelude::*;

fn main() -> Result<()> {
    launch::<TriangleApp>(Settings::default())
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
}

impl App for TriangleApp {
    fn init(ctx: &mut Context) -> Result<Self> {
        let verts = ctx.vertices(&TRIANGLE_MESH, false)?;
        Ok(Self { verts })
    }

    fn frame(&mut self, _ctx: &mut Context) -> Result<Vec<DrawCmd>> {
        Ok(vec![DrawCmd::new(self.verts)])
    }
}
