use idek::prelude::*;

fn main() -> Result<()> {
    launch::<_, TriangleApp>(Settings::default().args([
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
    ]))
}

struct TriangleApp {
    verts: VertexBuffer,
}

impl App<[Vertex; 3]> for TriangleApp {
    fn init(ctx: &mut Context, _: &mut Platform, vertices: [Vertex; 3]) -> Result<Self> {
        let verts = ctx.vertices(&vertices, false)?;
        Ok(Self { verts })
    }

    fn frame(&mut self, _ctx: &mut Context, _: &mut Platform) -> Result<Vec<DrawCmd>> {
        Ok(vec![DrawCmd::new(self.verts)])
    }
}
