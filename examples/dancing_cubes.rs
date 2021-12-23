use idek::{
    nalgebra::{Matrix4, Vector3},
    prelude::*,
    IndexBuffer, MultiPlatformCamera,
};

fn main() -> Result<()> {
    launch::<_, TriangleApp>(Settings::default().vr_if_any_args())
}

struct TriangleApp {
    verts: VertexBuffer,
    indices: IndexBuffer,
    camera: MultiPlatformCamera,
}

impl App for TriangleApp {
    fn init(ctx: &mut Context, platform: &mut Platform, _: ()) -> Result<Self> {
        let (vertices, indices) = rainbow_cube();
        Ok(Self {
            verts: ctx.vertices(&vertices, false)?,
            indices: ctx.indices(&indices, false)?,
            camera: MultiPlatformCamera::new(platform),
        })
    }

    fn frame(&mut self, ctx: &mut Context, _: &mut Platform) -> Result<Vec<DrawCmd>> {
        let mut draw_cmds = vec![];

        let cube = DrawCmd::new(self.verts).indices(self.indices);
        let n_cubes = 1000;
        for i in 0..n_cubes {
            let mut i = i as f32 / n_cubes as f32;
            i += ctx.start_time().elapsed().as_secs_f32() / 1000.;
            i *= std::f32::consts::TAU;

            let sz = 20.;
            let x = i.cos() * sz;
            let y = (i * 12. + 94.234).cos() * sz;
            let z = (i * 4. + 9.234).cos() * sz;
            let transform = Matrix4::new_translation(&Vector3::new(x, y, z));
            draw_cmds.push(cube.transform(*transform.as_ref()));
        }

        Ok(draw_cmds)
    }

    fn event(
        &mut self,
        ctx: &mut Context,
        platform: &mut Platform,
        mut event: Event,
    ) -> Result<()> {
        if self.camera.handle_event(&mut event) {
            ctx.set_camera_prefix(self.camera.get_prefix())
        }
        idek::close_when_asked(platform, &event);
        Ok(())
    }
}

fn rainbow_cube() -> (Vec<Vertex>, Vec<u32>) {
    let vertices = vec![
        Vertex::new([-1.0, -1.0, -1.0], [0.0, 1.0, 1.0]),
        Vertex::new([1.0, -1.0, -1.0], [1.0, 0.0, 1.0]),
        Vertex::new([1.0, 1.0, -1.0], [1.0, 1.0, 0.0]),
        Vertex::new([-1.0, 1.0, -1.0], [0.0, 1.0, 1.0]),
        Vertex::new([-1.0, -1.0, 1.0], [1.0, 0.0, 1.0]),
        Vertex::new([1.0, -1.0, 1.0], [1.0, 1.0, 0.0]),
        Vertex::new([1.0, 1.0, 1.0], [0.0, 1.0, 1.0]),
        Vertex::new([-1.0, 1.0, 1.0], [1.0, 0.0, 1.0]),
    ];

    let indices = vec![
        3, 1, 0, 2, 1, 3, 2, 5, 1, 6, 5, 2, 6, 4, 5, 7, 4, 6, 7, 0, 4, 3, 0, 7, 7, 2, 3, 6, 2, 7,
        0, 5, 4, 1, 5, 0,
    ];

    (vertices, indices)
}
