use anyhow::{bail, Context as AnyhowContext, Result};
use idek::{prelude::*, IndexBuffer, MultiPlatformCamera};
use std::path::Path;

fn main() -> Result<()> {
    launch::<_, ImageApp>(Settings::default().vr_if_any_args())
}

struct ImageApp {
    verts: VertexBuffer,
    indices: IndexBuffer,
    texture: Texture,
    camera: MultiPlatformCamera,
}

impl App for ImageApp {
    fn init(ctx: &mut Context, platform: &mut Platform, _: ()) -> Result<Self> {
        let (width, rgb_data) = load_png_rgba("examples/seasons_greasons.png")?;
        let height = rgb_data.len() / (width * 4);

        let (vertices, indices) = image_quad(width as f32 / height as f32);

        Ok(Self {
            texture: ctx.texture(&rgb_data, width, false)?,
            verts: ctx.vertices(&vertices, false)?,
            indices: ctx.indices(&indices, false)?,
            camera: MultiPlatformCamera::new(platform),
        })
    }

    fn frame(&mut self, _ctx: &mut Context, _: &mut Platform) -> Result<Vec<DrawCmd>> {
        Ok(vec![DrawCmd::new(self.verts)
            .indices(self.indices)
            .texture(self.texture)])
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

/// Creates a quad with image sampling coordinates, with the given (width/height) aspect ratio
fn image_quad(aspect: f32) -> (Vec<Vertex>, Vec<u32>) {
    let vertices = vec![
        Vertex::new([-aspect, 0.0, -1.0], [0.0, 0.0, 0.0]),
        Vertex::new([-aspect, 0.0, 1.0], [0.0, 1.0, 0.0]),
        Vertex::new([aspect, 0.0, -1.0], [1.0, 0.0, 0.0]),
        Vertex::new([aspect, 0.0, 1.0], [1.0, 1.0, 0.0]),
    ];

    let indices = vec![0b00, 0b01, 0b10, 0b01, 0b11, 0b10];

    (vertices, indices)
}

/// Returns (width, RGBA data) for the given PNG path
fn load_png_rgba(path: impl AsRef<Path>) -> Result<(usize, Vec<u8>)> {
    let decoder = png::Decoder::new(std::fs::File::open(path)?);
    let mut reader = decoder.read_info().context("Creating reader")?;

    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).context("Reading frame")?;

    if info.bit_depth != png::BitDepth::Eight {
        bail!("Bit depth {:?} unsupported!", info.bit_depth);
    }

    buf.truncate(info.buffer_size());

    const DEFAULT_ALPHA: u8 = 0;

    let buf: Vec<u8> = match info.color_type {
        png::ColorType::Rgba => buf,
        png::ColorType::Rgb => buf
            .chunks_exact(3)
            .map(|px| [px[0], px[1], px[2], DEFAULT_ALPHA])
            .flatten()
            .collect(),
        png::ColorType::Grayscale => buf
            .iter()
            .map(|&px| [px, px, px, DEFAULT_ALPHA])
            .flatten()
            .collect(),
        png::ColorType::GrayscaleAlpha => buf
            .chunks_exact(2)
            .map(|px| [px[0], px[0], px[0], px[1]])
            .flatten()
            .collect(),
        other => bail!("Images with color type {:?} are unsupported", other),
    };

    Ok((info.width as usize, buf))
}
