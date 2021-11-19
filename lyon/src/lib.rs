use idek::prelude::Vertex;
use lyon::lyon_tessellation::{BuffersBuilder, FillVertex};
pub type VertexBuffers = lyon::tessellation::VertexBuffers<Vertex, u32>;
pub use lyon;

pub fn buffer_builder<'geom>(
    geometry: &'geom mut VertexBuffers,
    z: f32,
    color: [f32; 3],
) -> BuffersBuilder<'geom, Vertex, u32, impl Fn(FillVertex) -> Vertex> {
    BuffersBuilder::new(geometry, move |vertex: FillVertex| {
        let [x, y] = vertex.position().to_array();
        Vertex {
            pos: [x, y, z],
            color,
        }
    })
}