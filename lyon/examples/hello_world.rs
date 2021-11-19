use idek::{prelude::*, IndexBuffer};
use idek_lyon::buffer_builder;
use lyon::math::point;
use lyon::path::Path;
use lyon::tessellation::*;


fn main() -> Result<()> {
    launch::<TriangleApp>(Settings::default().vr_if_any_args())
}

struct TriangleApp {
    verts: VertexBuffer,
    indices: IndexBuffer,
}

impl App for TriangleApp {
    fn init(ctx: &mut Context, _platform: &mut Platform) -> Result<Self> {
        // Build a Path.
        let mut builder = Path::builder();
        builder.begin(point(-0.5, 0.5));
        builder.line_to(point(0.5, 0.5));
        builder.line_to(point(0., -1.));
        builder.close();
        let path = builder.build();

        let mut geometry = VertexBuffers::new();
        let mut tessellator = FillTessellator::new();
        
        // Compute the tessellation.
        tessellator.tessellate_path(
            &path,
            &FillOptions::default(),
            &mut buffer_builder(&mut geometry, 0., [0., 1., 0.]),
        ).expect("Tesselation failed");

        Ok(Self {
            verts: ctx.vertices(&geometry.vertices, false)?,
            indices: ctx.indices(&geometry.indices, false)?,
        })
    }

    fn frame(&mut self, _ctx: &mut Context, _: &mut Platform) -> Result<Vec<DrawCmd>> {
        Ok(vec![DrawCmd::new(self.verts).indices(self.indices)])
    }
}