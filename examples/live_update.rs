use idek::{prelude::*, IndexBuffer, MultiPlatformCamera};

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
        let (vertices, indices) = pattern(ctx.start_time().elapsed().as_secs_f32());
        Ok(Self {
            verts: ctx.vertices(&vertices, true)?,
            indices: ctx.indices(&indices, false)?,
            camera: MultiPlatformCamera::new(platform),
        })
    }

    fn frame(&mut self, ctx: &mut Context, _: &mut Platform) -> Result<Vec<DrawCmd>> {
        let (vertices, _) = pattern(ctx.start_time().elapsed().as_secs_f32());
        ctx.update_vertices(self.verts, &vertices)?;
        Ok(vec![DrawCmd::new(self.verts).indices(self.indices)])
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

/// Time-varying pattern
fn pattern(time: f32) -> (Vec<Vertex>, Vec<u32>) {
    let width = 100;
    (
        pattern_internal(
            time / 10.,
            width,
            1.0,
            0.1,
            &[(-0.3, 0.2), (0.6, 0.8), (-0.1, -0.3)],
            18.,
        ),
        dense_grid_tri_indices(width),
    )
}

/// Euclidean distance
fn dist((ax, ay): (f32, f32), (bx, by): (f32, f32)) -> f32 {
    let (dx, dy) = (ax - bx, ay - by);
    (dx * dx + dy * dy).sqrt()
}

/// Vertices for the pattern
fn pattern_internal(
    time: f32,
    width: u32,
    scale: f32,
    amp: f32,
    sources: &[(f32, f32)],
    freq: f32,
) -> Vec<Vertex> {
    let grid_to_world = |i| (i as f32 / width as f32) * 2. - 1.;

    let mut vertices = vec![];

    for x in 0..width {
        for z in 0..width {
            let pos @ (x, z) = (grid_to_world(x), grid_to_world(z));

            let y = sources
                .iter()
                .map(|&src| ((dist(src, pos) + time) * freq).cos() * amp)
                .sum::<f32>();

            vertices.push(Vertex {
                pos: [x * scale, y * scale, z * scale],
                color: [1. - y, y, 1.],
            })
        }
    }

    vertices
}

/// Indices of internal vertices
fn dense_grid_edge_indices(width: u32) -> impl Iterator<Item = u32> {
    (0..width - 1)
        .map(move |x| (0..width - 1).map(move |y| (x, y)))
        .flatten()
        .map(move |(x, y)| x + y * width)
}

/// Indices of triangles for a dense grid mesh
fn dense_grid_tri_indices(width: u32) -> Vec<u32> {
    let mut indices = Vec::new();
    for base in dense_grid_edge_indices(width) {
        indices.push(base);
        indices.push(base + 1);
        indices.push(base + width);
        indices.push(base + 1);
        indices.push(base + width + 1);
        indices.push(base + width);
    }
    indices
}
