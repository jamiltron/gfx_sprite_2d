extern crate cgmath;
#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate image;

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::Depth;

use gfx::{Device, tex};
use gfx::traits::FactoryExt;
use std::io::Cursor;

gfx_defines!{
    // this is the form of our vertex buffer objects
    vertex Vertex {
        position: [f32; 2] = "position",
        tex_coord: [f32; 2] = "texCoord",
    }

    // represents the form of data we pass to and from the gpu
    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        model: gfx::Global<[[f32; 4]; 4]> = "model",
        projection: gfx::Global<[[f32; 4]; 4]> = "projection",
        our_texture: gfx::TextureSampler<[f32; 4]> = "ourTexture",
        out: gfx::RenderTarget<ColorFormat> = "color",
    }
}

const CLEAR_COLOR: [f32; 4] = [0.1, 0.1, 0.1, 1.0];

// each texture quad's position and what point on the texture to tie it to
const TEX_QUAD: [Vertex; 4] = [Vertex {
                                   position: [0.5, 0.5],
                                   tex_coord: [1.0, 1.0],
                               },
                               Vertex {
                                   position: [0.5, -0.5],
                                   tex_coord: [1.0, 0.0],
                               },
                               Vertex {
                                   position: [-0.5, -0.5],
                                   tex_coord: [0.0, 0.0],
                               },
                               Vertex {
                                   position: [-0.5, 0.5],
                                   tex_coord: [0.0, 1.0],
                               }];

// the indices used by the element buffer
const TEX_INDICES: [u16; 6] = [0, 3, 1, 1, 3, 2];

const NEAR_PLANE: f32 = -1.0;
const FAR_PLANE: f32 = 10.0;

const WINDOW_WIDTH: u32 = 640;
const WINDOW_HEIGHT: u32 = 480;
const WINDOW_TITLE: &'static str = "GFX-SPRITE-2D";

const SPRITE_X: f32 = WINDOW_WIDTH as f32 / 2.0;
const SPRITE_Y: f32 = WINDOW_HEIGHT as f32 / 2.0;
const SPRITE_WIDTH: f32 = 64.0;
const SPRITE_HEIGHT: f32 = 64.0;

// stolen from: https://github.com/gfx-rs/gfx/tree/master/examples/blend
fn load_texture<R, F>(factory: &mut F,
                      data: &[u8])
                      -> Result<gfx::handle::ShaderResourceView<R, [f32; 4]>, String>
    where R: gfx::Resources,
          F: gfx::Factory<R>
{
    let img = image::load(Cursor::new(data), image::PNG).unwrap().to_rgba();
    let (width, height) = img.dimensions();
    let kind = tex::Kind::D2(width as tex::Size, height as tex::Size, tex::AaMode::Single);
    let (_, view) = factory.create_texture_const_u8::<gfx::format::Rgba8>(kind, &[&img])
        .unwrap();
    Ok(view)
}

fn main() {
    // get ready to build our application with the desired settings
    let builder = glutin::WindowBuilder::new()
        .with_title(WINDOW_TITLE)
        .with_dimensions(WINDOW_WIDTH, WINDOW_HEIGHT)
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 3)))
        .with_gl_profile(glutin::GlProfile::Core)
        .with_vsync();

    // initialize everything provided by glutin
    let (window, mut device, mut factory, main_color, _) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);

    // build up our pipeline by providing the sources to our vertex and fragment shaders
    let pso = factory.create_pipeline_simple(include_bytes!("../res/shaders/sprite.vs"),
                                include_bytes!("../res/shaders/sprite.fs"),
                                pipe::new())
        .unwrap();

    // build the actual vertex buffer object, and the slice of data used for rendering
    let (vertex_buffer, slice) =
        factory.create_vertex_buffer_with_slice(&TEX_QUAD, &TEX_INDICES[..]);

    // load our png as a texture
    let smile_texture = load_texture(&mut factory, &include_bytes!("../res/images/smile.png")[..])
        .unwrap();
    let sampler = factory.create_sampler_linear();

    // create a orthographic projection covering the whole window
    let projection = cgmath::ortho(0.0,
                                   WINDOW_WIDTH as f32,
                                   0.0,
                                   WINDOW_HEIGHT as f32,
                                   NEAR_PLANE,
                                   FAR_PLANE);

    // translate our texture to the middle of the screen
    let translation =
        cgmath::Matrix4::from_translation(cgmath::Vector3::new(SPRITE_X, SPRITE_Y, 0.0));
    // scale our texture to the size we want
    let scale = cgmath::Matrix4::from_nonuniform_scale(SPRITE_WIDTH, SPRITE_HEIGHT, 1.0);

    // create the data we will be using with the gpu
    let data = pipe::Data {
        model: (translation * scale).into(),
        our_texture: (smile_texture, sampler),
        out: main_color,
        projection: projection.into(),
        vbuf: vertex_buffer,
    };

    'main: loop {
        // grab an encoder to pass commands to the gpu to
        let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

        // grab our events, close the program if the window closes or escape is pressed
        for event in window.poll_events() {
            match event {
                glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) => {
                    break 'main
                }
                glutin::Event::Closed => break 'main,
                _ => {}
            }
        }

        // render everything
        encoder.clear(&data.out, CLEAR_COLOR);
        encoder.draw(&slice, &pso, &data);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
