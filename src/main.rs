mod camera;
mod grid;
mod input;
mod renderer;

use std::{ffi::CString, time::Instant};

use cgmath::{vec3, vec4, Deg, Matrix2, Matrix4, Rad};
use gl::types::*;
use input::{KeyboardState, MouseState};
use renderer::{shader::Shader, texture::Texture, vertex_buffer::VertexBuffer};

use crate::{
    camera::Camera,
    grid::{Belt, Direction, Grid, Turn},
    renderer::{debug::DebugCallback, vertex_array::VertexArray, VertexBufferElement},
};

// Vertex data
static QUAD_DATA: [GLfloat; 24] = [
    0.5, 0.5, 1.0, 1.0, // top right
    -0.5, 0.5, 0.0, 1.0, // top left
    -0.5, -0.5, 0.0, 0.0, // bottom left
    -0.5, -0.5, 0.0, 0.0, // bottom left
    0.5, -0.5, 1.0, 0.0, // bottom right
    0.5, 0.5, 1.0, 1.0, // top right
];

static LINE_DATA: [GLfloat; 4] = [
    0.0, 0.0, // origo
    1.0, 0.0,
];

fn main() {
    let event_loop = glutin::event_loop::EventLoop::new();
    let window = glutin::window::WindowBuilder::new();
    let gl_window = glutin::ContextBuilder::new()
        .build_windowed(window, &event_loop)
        .unwrap();

    // It is essential to make the context current before calling `gl::load_with`.
    let gl_window = unsafe { gl_window.make_current() }.unwrap();

    // Load the OpenGL function pointers
    gl::load_with(|symbol| gl_window.get_proc_address(symbol));

    let base_shader = Shader::from_file("base.vert", "base.frag");
    let tex_shader = Shader::from_file("texture.vert", "texture.frag");

    unsafe {
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        // gl::Enable(gl::DEBUG_OUTPUT);
        // gl::DebugMessageCallback(Some(debug_callback), ptr::null());
    }

    let _debug_callback = unsafe {
        DebugCallback::new(|message| {
            println!("{:?}", message);
        })
    };

    let tex = unsafe { Texture::from_path("assets/textures/belt-1.png", true) };
    println!("{:?}", tex);
    let quad_va = unsafe {
        let vb = VertexBuffer::new(
            &QUAD_DATA,
            vec![
                VertexBufferElement::floats(2),
                VertexBufferElement::floats(2),
            ],
        );
        VertexArray::new(&[vb])
    };

    let line_va = unsafe {
        let vb = VertexBuffer::new(&LINE_DATA, vec![VertexBufferElement::floats(2)]);
        VertexArray::new(&[vb])
    };

    let mut camera = Camera::new();
    camera.position.x = 300.0;
    camera.position.y = 300.0;

    let mut keyboard_state = KeyboardState::new();
    let mut mouse_state = MouseState::new();

    let mut grid = Grid::new();

    let mut zoom = 2.0;

    let mut debug_grid = true;
    let mut show_fps = false;

    let mut last_update_time = Instant::now();

    let mut current_belt = Belt::new();

    let mut is_placing = true;

    event_loop.run(move |event, _, control_flow| {
        use glutin::event::{Event, MouseButton, VirtualKeyCode, WindowEvent};
        use glutin::event_loop::ControlFlow;
        *control_flow = ControlFlow::Poll;

        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => {
                keyboard_state.process_event(&event);
                mouse_state.process_event(&event);
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(size) => unsafe {
                        gl::Viewport(0, 0, size.width as i32, size.height as i32);
                    },
                    _ => (),
                }
            }
            _ => (),
        }

        let now = Instant::now();
        let ms_since_last_update = (now - last_update_time).as_nanos() as f64 / 1_000_000.0;
        if ms_since_last_update > 16.666 {
            last_update_time = now;

            if keyboard_state.was_pressed(VirtualKeyCode::G) {
                debug_grid = !debug_grid;
            }

            if keyboard_state.was_pressed(VirtualKeyCode::F) {
                show_fps = !show_fps;
            }

            if is_placing {
                if keyboard_state.was_pressed(VirtualKeyCode::R) {
                    current_belt.input = current_belt.input.rotate_clockwise();
                    current_belt.output = current_belt.output.rotate_clockwise();
                }
            }

            if keyboard_state.was_pressed(VirtualKeyCode::Space) {
                is_placing = !is_placing;
            }

            if keyboard_state.is_pressed(VirtualKeyCode::W) {
                camera.move_vertical(10.0 / zoom);
            } else if keyboard_state.is_pressed(VirtualKeyCode::S) {
                camera.move_vertical(-10.0 / zoom);
            }

            if keyboard_state.is_pressed(VirtualKeyCode::D) {
                camera.move_horizontal(10.0 / zoom);
            } else if keyboard_state.is_pressed(VirtualKeyCode::A) {
                camera.move_horizontal(-10.0 / zoom);
            }

            if mouse_state.scroll_delta < 0.0 {
                zoom /= 1.0 + -mouse_state.scroll_delta / 10.0;
            } else {
                zoom *= 1.0 + mouse_state.scroll_delta / 10.0;
            }

            let window_size = gl_window.window().inner_size();

            let mut mouse_grid_pos = camera.position * zoom;
            mouse_grid_pos.x += mouse_state.position.x as f32 - window_size.width as f32 / 2.0;
            mouse_grid_pos.y += window_size.height as f32 / 2.0 - mouse_state.position.y as f32;

            let mouse_grid_x = (mouse_grid_pos.x / 32.0 / zoom).floor() as i32;
            let mouse_grid_y = (mouse_grid_pos.y / 32.0 / zoom).floor() as i32;

            let mouse_in_grid =
                mouse_grid_x >= 0 && mouse_grid_x < 128 && mouse_grid_y >= 0 && mouse_grid_y < 128;
            if is_placing && mouse_in_grid {
                if mouse_state.is_pressed(MouseButton::Left) {
                    grid.place_belt(mouse_grid_x as isize, mouse_grid_y as isize, current_belt);
                }

                if mouse_state.is_pressed(MouseButton::Right) {
                    grid.clear_tile(mouse_grid_x as usize, mouse_grid_y as usize);
                }
            }

            let start = Instant::now();
            unsafe {
                gl::ClearColor(0.3, 0.3, 0.6, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);

                tex.bind_to_unit(0);

                tex_shader.enable();
                tex_shader.set_int(&CString::new("atlas_size").unwrap(), 2);
                tex_shader.set_mat4(&CString::new("view").unwrap(), camera.view_matrix());
                tex_shader.set_mat4(
                    &CString::new("projection").unwrap(),
                    camera.projection_matrix(
                        window_size.width as f32,
                        window_size.height as f32,
                        zoom,
                    ),
                );
                quad_va.bind();

                for (y, row) in grid.tiles.iter().enumerate() {
                    for (x, tile) in row.iter().enumerate() {
                        if let Some(belt) = tile {
                            let model_scale = Matrix4::from_nonuniform_scale(32.0, 32.0, 0.0);
                            let model_trans = Matrix4::from_translation(cgmath::vec3(
                                16.0 + 32.0 * x as f32,
                                16.0 + 32.0 * y as f32,
                                0.0,
                            ));
                            let model = model_trans * model_scale;

                            let tex_angle = match belt.input {
                                Direction::West => 90.0,
                                Direction::North => 180.0,
                                Direction::East => 270.0,
                                Direction::South => 0.0,
                            };
                            let tex_rot = Matrix2::from_angle(Deg(tex_angle));

                            let atlas_index = match belt.turn() {
                                Turn::Left => 0,
                                Turn::Forward => 2,
                                Turn::Right => 3,
                            };

                            tex_shader.set_mat4(&CString::new("model").unwrap(), model);
                            tex_shader.set_vec4(
                                &CString::new("color").unwrap(),
                                vec4(1.0, 1.0, 1.0, 1.0),
                            );
                            tex_shader.set_mat2(&CString::new("tex_rot").unwrap(), tex_rot);
                            tex_shader.set_int(&CString::new("atlas_index").unwrap(), atlas_index);

                            gl::DrawArrays(gl::TRIANGLES, 0, 6);
                        }
                    }
                }

                if is_placing && mouse_in_grid {
                    let current_belt = grid.calculate_belt_position(
                        mouse_grid_x as isize,
                        mouse_grid_y as isize,
                        current_belt,
                    );
                    let model_scale = Matrix4::from_nonuniform_scale(32.0, 32.0, 0.0);
                    let model_trans = Matrix4::from_translation(cgmath::vec3(
                        16.0 + 32.0 * mouse_grid_x as f32,
                        16.0 + 32.0 * mouse_grid_y as f32,
                        0.0,
                    ));
                    let model = model_trans * model_scale;

                    let tex_angle = match current_belt.input {
                        Direction::West => 90.0,
                        Direction::North => 180.0,
                        Direction::East => 270.0,
                        Direction::South => 0.0,
                    };
                    let tex_rot = Matrix2::from_angle(Deg(tex_angle));

                    let atlas_index = match current_belt.turn() {
                        Turn::Left => 0,
                        Turn::Forward => 2,
                        Turn::Right => 3,
                    };

                    tex_shader.set_vec4(&CString::new("color").unwrap(), vec4(1.0, 1.0, 1.0, 0.4));
                    tex_shader.set_mat4(&CString::new("model").unwrap(), model);
                    tex_shader.set_mat2(&CString::new("tex_rot").unwrap(), tex_rot);
                    tex_shader.set_int(&CString::new("atlas_index").unwrap(), atlas_index);

                    gl::DrawArrays(gl::TRIANGLES, 0, 6);
                }

                if debug_grid {
                    line_va.bind();
                    base_shader.enable();
                    base_shader.set_mat4(&CString::new("view").unwrap(), camera.view_matrix());
                    base_shader.set_mat4(
                        &CString::new("projection").unwrap(),
                        camera.projection_matrix(
                            window_size.width as f32,
                            window_size.height as f32,
                            zoom,
                        ),
                    );
                    base_shader.set_vec3(&CString::new("color").unwrap(), vec3(0.0, 0.0, 0.0));
                    for y in 0..=128 {
                        let model_scale = Matrix4::from_nonuniform_scale(32.0 * 128.0, 0.0, 0.0);
                        let model_trans =
                            Matrix4::from_translation(cgmath::vec3(0.0, y as f32 * 32.0, 0.0));
                        let model = model_trans * model_scale;
                        base_shader.set_mat4(&CString::new("model").unwrap(), model);
                        gl::DrawArrays(gl::LINES, 0, 2);
                    }
                    for x in 0..=128 {
                        let rect_rot = std::f32::consts::FRAC_PI_2;
                        let model_rot = Matrix4::from_angle_z(Rad(rect_rot));
                        let model_scale = Matrix4::from_nonuniform_scale(32.0 * 128.0, 0.0, 0.0);
                        let model_trans =
                            Matrix4::from_translation(cgmath::vec3(x as f32 * 32.0, 0.0, 0.0));
                        let model = model_trans * model_rot * model_scale;
                        base_shader.set_mat4(&CString::new("model").unwrap(), model);
                        gl::DrawArrays(gl::LINES, 0, 2);
                    }
                }
            }
            let dur = Instant::now() - start;
            let ms = dur.as_nanos() as f64 / 1_000_000.0;
            if show_fps {
                println!("Render time: {}ms", ms);
            }
            gl_window.swap_buffers().unwrap();
            keyboard_state.clear_momentary_state();
            mouse_state.clear_momentary_state();
        }
    });
}
