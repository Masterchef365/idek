use crate::*;

/// A draw command, flexibly represents a number of different drawing
#[derive(Copy, Clone)]
pub struct DrawCmd {
    pub vertices: VertexBuffer,
    //pub instances: Option<InstanceBuffer>,
    pub indices: Option<IndexBuffer>,
    //pub texture: Option<Texture>,
    pub shader: Option<Shader>,
    pub transform: Option<Transform>,
    pub limit: Option<u32>,
}

impl DrawCmd {
    pub fn new(vertices: VertexBuffer) -> Self {
        Self {
            vertices,
            //instances: None,
            indices: None,
            //texture: None,
            shader: None,
            transform: None,
            limit: None,
        }
    }

    /*pub fn instances(mut self, instances: InstanceBuffer) -> Self {
        self.instances = Some(instances);
        self
    }*/

    pub fn indices(mut self, indices: IndexBuffer) -> Self {
        self.indices = Some(indices);
        self
    }

    /*pub fn texture(mut self, texture: Texture) -> Self {
        self.texture = Some(texture);
        self
    }*/

    pub fn shader(mut self, shader: Shader) -> Self {
        self.shader = Some(shader);
        self
    }

    pub fn transform(mut self, transform: Transform) -> Self {
        self.transform = Some(transform);
        self
    }

    /// If vertices have been defined:              Limit vertex drawing to this number
    /// If indices and vertices have been defined:  Limit indexes used to this number
    /// If neither vertices nor indices:            Draw this many vertices
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}
