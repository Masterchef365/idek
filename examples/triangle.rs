use idek::prelude::*;
//use idek::{launch, Vertex, VertexBuffer, Context, DrawCmd, App, Result};

fn main() -> Result<()> {
    launch::<TriangleApp>(Settings::default())
}

const TRIANGLE_MESH: [Vertex; 3] = [
    Vertex {
        pos: [0., 0.5],
        color: [1., 0., 0.],
    },
    Vertex {
        pos: [-0.5, -0.5],
        color: [0., 1., 0.],
    },
    Vertex {
        pos: [0.5, -0.5],
        color: [0., 0., 1.],
    },
];

struct TriangleApp {
    verts: VertexBuffer,
}

impl App for TriangleApp {
    fn init(ctx: Context) -> Result<()> {
        let verts = ctx.vertices(&TRIANGLE_MESH, false)?;
        Ok(Self { verts })
    }

    fn frame(&mut self, ctx: Context) -> Result<Vec<DrawCmd>> {
        Ok(vec![DrawCmd::new(self.verts)])
    }

    fn event(&mut self, event: Event) -> Result<()> {
        // Update internal state
    }
}
