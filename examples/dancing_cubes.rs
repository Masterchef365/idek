use idek::{IndexBuffer, MultiPlatformCamera, nalgebra::{Matrix4, Vector3}, prelude::*};

fn main() -> Result<()> {
    launch::<TriangleApp>(Settings::default().vr_if_any_args())
}

struct TriangleApp {
    verts: VertexBuffer,
    indices: IndexBuffer,
    camera: MultiPlatformCamera,
    original_verts: Vec<Vertex>,
}

impl App for TriangleApp {
    fn init(ctx: &mut Context, platform: &mut Platform) -> Result<Self> {
        let (vertices, indices) = rainbow_cube();
        Ok(Self {
            verts: ctx.vertices(&vertices, true)?,
            indices: ctx.indices(&indices, false)?,
            camera: MultiPlatformCamera::new(platform),
            original_verts: vertices,
        })
    }

    fn frame(&mut self, ctx: &mut Context, _: &mut Platform) -> Result<Vec<DrawCmd>> {
        let time = ctx.start_time().elapsed().as_secs_f32();
        let anim = time / 1000.;

        let mut draw_cmds = vec![];

        let mut mutated_verts = self.original_verts.clone();
        for (idx, vert) in mutated_verts.iter_mut().enumerate() {
            let time = time + idx as f32 / 10.;
            //vert.pos[0] += anim.cos() * 0.5;
            //vert.pos[1] += (anim * 12.).cos() * 0.5;
            //vert.pos[2] += (anim * 3.).cos() * 0.5;

            vert.pos[0] += time.cos() * 0.15;
            vert.pos[1] += (time * 12.).cos() * 0.15;
            vert.pos[2] += (time * 3.).cos() * 0.15;
        }
        ctx.update_vertices(self.verts, &mutated_verts)?;

        let cube = DrawCmd::new(self.verts).indices(self.indices);
        let n_cubes = 1000;
        for i in 0..n_cubes {
            let mut i = i as f32 / n_cubes as f32;
            i += anim;
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
            ctx.set_camera_prefix(self.camera.get_prefix(platform))
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
