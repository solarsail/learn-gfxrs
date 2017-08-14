#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate image;
extern crate cgmath;

use gfx::Device;
use gfx::format::Formatted;
use gfx::traits::FactoryExt;
use cgmath::{Deg, Matrix4, Point3, Vector3};
use glutin::GlContext;
use std::time::Instant;

//mod squares;
mod camera;

use camera::{Camera, Direction};

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

const BG: [f32; 4] = [0.2, 0.3, 0.3, 1.0];

gfx_defines!{
    vertex Vertex { // the vertex type which vertex shaders will receive
        pos: [f32; 3] = "a_Pos", // corresponding to names in glsl
        tex_coord: [f32; 2] = "a_TexCoord",
    }

    constant Transform { // will appear to shaders as a uniform struct
        transform: [[f32; 4];4] = "u_Transform",
    }

    pipeline pipe { // defines pipe::Data
        vbuf: gfx::VertexBuffer<Vertex> = (),
        transform: gfx::ConstantBuffer<Transform> = "Transform",
        texture: gfx::TextureSampler<<ColorFormat as Formatted>::View> = "u_Texture",
        out: gfx::RenderTarget<ColorFormat> = "Target0",
        out_depth: gfx::DepthTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

impl Vertex {
    fn new(p: [i8; 3], t: [i8; 2]) -> Vertex {
        Vertex {
            pos: [p[0] as f32, p[1] as f32, p[2] as f32],
            tex_coord: [t[0] as f32, t[1] as f32],
        }
    }
}





fn load_texture<F, R>(
    factory: &mut F,
    path: &str,
) -> gfx::handle::ShaderResourceView<R, <ColorFormat as Formatted>::View>
where
    F: gfx::Factory<R>,
    R: gfx::Resources,
{
    let img = image::open(path).unwrap().to_rgba();
    let (width, height) = img.dimensions();
    let kind = gfx::texture::Kind::D2(width as u16, height as u16, gfx::texture::AaMode::Single);
    let (_, view) = factory
        .create_texture_immutable_u8::<ColorFormat>(kind, &[&img])
        .unwrap();
    view
}


pub fn main() {
    let cube: &[Vertex] = &[
        // top (0, 0, 1)
        Vertex::new([-1, -1, 1], [0, 0]),
        Vertex::new([1, -1, 1], [1, 0]),
        Vertex::new([1, 1, 1], [1, 1]),
        Vertex::new([-1, 1, 1], [0, 1]),
        // bottom (0, 0, -1)
        Vertex::new([-1, 1, -1], [1, 0]),
        Vertex::new([1, 1, -1], [0, 0]),
        Vertex::new([1, -1, -1], [0, 1]),
        Vertex::new([-1, -1, -1], [1, 1]),
        // right (1, 0, 0)
        Vertex::new([1, -1, -1], [0, 0]),
        Vertex::new([1, 1, -1], [1, 0]),
        Vertex::new([1, 1, 1], [1, 1]),
        Vertex::new([1, -1, 1], [0, 1]),
        // left (-1, 0, 0)
        Vertex::new([-1, -1, 1], [1, 0]),
        Vertex::new([-1, 1, 1], [0, 0]),
        Vertex::new([-1, 1, -1], [0, 1]),
        Vertex::new([-1, -1, -1], [1, 1]),
        // front (0, 1, 0)
        Vertex::new([1, 1, -1], [1, 0]),
        Vertex::new([-1, 1, -1], [0, 0]),
        Vertex::new([-1, 1, 1], [0, 1]),
        Vertex::new([1, 1, 1], [1, 1]),
        // back (0, -1, 0)
        Vertex::new([1, -1, 1], [0, 0]),
        Vertex::new([-1, -1, 1], [1, 0]),
        Vertex::new([-1, -1, -1], [1, 1]),
        Vertex::new([1, -1, -1], [0, 1]),
    ];

    let indices: &[u16] = &[
         0,  1,  2,  2,  3,  0, // top
         4,  5,  6,  6,  7,  4, // bottom
         8,  9, 10, 10, 11,  8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];

    let mut events_loop = glutin::EventsLoop::new();
    let context = glutin::ContextBuilder::new();
    let builder = glutin::WindowBuilder::new()
        .with_title("Learn gfx-rs".to_string())
        .with_dimensions(1024, 768);

    let aspect_ratio = 1024.0 / 768.0;

    // gfx-rs init
    let (window, mut device, mut factory, render_target, depth_stencil) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder, context, &events_loop);

    let pso = factory
        .create_pipeline_simple(
            include_bytes!("shader/myshader_150.glslv"),
            include_bytes!("shader/myshader_150.glslf"),
            pipe::new(), // instantiates the pipe defined in `gfx_defines!`
        )
        .unwrap();

    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();


    let texture = load_texture(&mut factory, "textures/container.jpg");
    let sampler = factory.create_sampler_linear();

    let mut proj = cgmath::perspective(Deg(45.0), aspect_ratio, 0.1, 100.0);

    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(cube, indices);
    let transform_buffer = factory.create_constant_buffer(1);
    let mut data = pipe::Data {
        vbuf: vertex_buffer,
        transform: transform_buffer,
        texture: (texture, sampler),
        out: render_target,
        out_depth: depth_stencil,
    };

    let mut running = true;

    let mut camera = Camera::new(Point3::new(0.0, -4.0, 8.0), Point3::new(0.0, 0.0, 0.0));

    // glutin events loop
    let mut delta_time;
    let mut last_frame = Instant::now();
    while running {
        let current_frame = Instant::now();
        delta_time = current_frame.duration_since(last_frame);
        last_frame = current_frame;
        events_loop.poll_events(|event| {
            use glutin::WindowEvent::*;
            use glutin::VirtualKeyCode;
            use glutin::ElementState::*;
            match event {
                glutin::Event::WindowEvent { event, .. } => {
                    match event {
                        KeyboardInput {
                            input: glutin::KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape), ..
                            },
                            ..
                        } |
                        Closed => {
                            running = false; // cannot `break` in closure
                        }
                        KeyboardInput {
                            input: glutin::KeyboardInput {
                                state,
                                virtual_keycode: Some(vk),
                                ..
                            },
                            ..
                        } => {
                            match (state, vk) {
                                (Pressed, VirtualKeyCode::W) => {
                                    camera.prep_move(Direction::Up, true)
                                }
                                (Pressed, VirtualKeyCode::S) => {
                                    camera.prep_move(Direction::Down, true)
                                }
                                (Pressed, VirtualKeyCode::A) => {
                                    camera.prep_move(Direction::Left, true)
                                }
                                (Pressed, VirtualKeyCode::D) => {
                                    camera.prep_move(Direction::Right, true)
                                }
                                (Released, VirtualKeyCode::W) => {
                                    camera.prep_move(Direction::Up, false)
                                }
                                (Released, VirtualKeyCode::S) => {
                                    camera.prep_move(Direction::Down, false)
                                }
                                (Released, VirtualKeyCode::A) => {
                                    camera.prep_move(Direction::Left, false)
                                }
                                (Released, VirtualKeyCode::D) => {
                                    camera.prep_move(Direction::Right, false)
                                }
                                _ => {}
                            }
                        }
                        Resized(width, height) => {
                            proj = cgmath::perspective(
                                Deg(45.0),
                                width as f32 / height as f32,
                                0.1,
                                100.0,
                            );
                            gfx_window_glutin::update_views(
                                &window,
                                &mut data.out,
                                &mut data.out_depth,
                            );
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        });

        let speed = 3.0 * (delta_time.as_secs() as f32 + delta_time.subsec_nanos() as f32 / 1.0e9);
        camera.move_at(speed);

        let view = Matrix4::look_at(camera.pos(), camera.looking_at(), Vector3::unit_z());
        let transform = Transform { transform: (proj * view).into() };
        encoder.clear(&data.out, BG);
        encoder.clear_depth(&data.out_depth, 1.0);
        encoder
            .update_buffer(&data.transform, &[transform], 0)
            .unwrap();
        encoder.draw(&slice, &pso, &data);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
