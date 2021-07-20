mod keyboard;
mod shader;

use std::{ffi::CString, mem, time::Instant};

use cgmath::{ortho, point2, point3, vec2, vec3, Matrix4, Point2, Rad, Vector3};
use gl::types::*;
use glutin::event::ElementState;
use keyboard::KeyboardState;
use shader::Shader;

// Vertex data
static QUAD_DATA: [GLfloat; 12] = [
    0.5, 0.5, // top right
    -0.5, 0.5, // top left
    -0.5, -0.5, // bottom left
    -0.5, -0.5, // bottom left
    0.5, -0.5, // bottom right
    0.5, 0.5, // top right
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

    let shader = Shader::from_file("base.vert", "base.frag");

    let mut quad_vao = 0;
    let mut line_vao = 0;

    unsafe {
        // quad
        let mut vbo = 0;
        gl::CreateBuffers(1, &mut vbo);
        gl::NamedBufferStorage(
            vbo,
            (QUAD_DATA.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            mem::transmute(&QUAD_DATA[0]),
            gl::DYNAMIC_STORAGE_BIT,
        );

        gl::CreateVertexArrays(1, &mut quad_vao);

        gl::VertexArrayVertexBuffer(
            quad_vao,
            0,
            vbo,
            0,
            2 * mem::size_of::<GLfloat>() as GLsizei,
        );

        gl::EnableVertexArrayAttrib(quad_vao, 0);
        gl::VertexArrayAttribFormat(quad_vao, 0, 2, gl::FLOAT, gl::FALSE, 0);
        gl::VertexArrayAttribBinding(quad_vao, 0, 0);

        // line
        let mut vbo = 0;
        gl::CreateBuffers(1, &mut vbo);
        gl::NamedBufferStorage(
            vbo,
            (LINE_DATA.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            mem::transmute(&LINE_DATA[0]),
            gl::DYNAMIC_STORAGE_BIT,
        );

        gl::CreateVertexArrays(1, &mut line_vao);

        gl::VertexArrayVertexBuffer(
            line_vao,
            0,
            vbo,
            0,
            2 * mem::size_of::<GLfloat>() as GLsizei,
        );

        gl::EnableVertexArrayAttrib(line_vao, 0);
        gl::VertexArrayAttribFormat(line_vao, 0, 2, gl::FLOAT, gl::FALSE, 0);
        gl::VertexArrayAttribBinding(line_vao, 0, 0);
    }

    let mut camera = Camera::new();

    let mut keyboard_state = KeyboardState::new();

    let mut grid = Grid::new();

    let mut mouse_pos = vec2(0.0, 0.0);
    let mut mouse_left = false;
    let mut mouse_right = false;

    let mut zoom = 2.0;

    let mut debug_grid = false;
    let mut show_fps = false;

    let mut last_update_time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        use glutin::event::{
            DeviceEvent, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode,
            WindowEvent,
        };
        use glutin::event_loop::ControlFlow;
        *control_flow = ControlFlow::Poll;

        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(size) => unsafe {
                    gl::Viewport(0, 0, size.width as i32, size.height as i32);
                },
                WindowEvent::CursorMoved { position, .. } => {
                    mouse_pos.x = position.x as f32;
                    mouse_pos.y = position.y as f32;
                }
                WindowEvent::MouseInput { button, state, .. } => match button {
                    MouseButton::Left => {
                        if state == ElementState::Pressed {
                            mouse_left = true;
                        } else {
                            mouse_left = false;
                        }
                    }
                    MouseButton::Right => {
                        if state == ElementState::Pressed {
                            mouse_right = true;
                        } else {
                            mouse_right = false;
                        }
                    }
                    _ => (),
                },
                _ => (),
            },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::Key(KeyboardInput {
                    state,
                    virtual_keycode: Some(virtual_keycode),
                    ..
                }) => {
                    keyboard_state.process_event(state, virtual_keycode);
                    if virtual_keycode == VirtualKeyCode::G && state == ElementState::Pressed {
                        debug_grid = !debug_grid;
                    }

                    if virtual_keycode == VirtualKeyCode::F && state == ElementState::Pressed {
                        show_fps = !show_fps;
                    }
                }
                DeviceEvent::MouseWheel { delta } => match delta {
                    MouseScrollDelta::LineDelta(_, y) => {
                        if y < 0.0 {
                            zoom *= 1.0 + -y / 100.0;
                        } else {
                            zoom /= 1.0 + y / 100.0;
                        }
                    }
                    _ => (),
                },
                _ => (),
            },
            _ => (),
        }

        let now = Instant::now();
        let ms_since_last_update = (now - last_update_time).as_nanos() as f64 / 1_000_000.0;
        if ms_since_last_update > 16.666 {
            last_update_time = now;

            if keyboard_state.is_pressed(VirtualKeyCode::W) {
                camera.move_vertical(4.0);
            } else if keyboard_state.is_pressed(VirtualKeyCode::S) {
                camera.move_vertical(-4.0);
            }

            if keyboard_state.is_pressed(VirtualKeyCode::D) {
                camera.move_horizontal(4.0);
            } else if keyboard_state.is_pressed(VirtualKeyCode::A) {
                camera.move_horizontal(-4.0);
            }

            let window_size = gl_window.window().inner_size();

            let mut mouse_grid_pos = camera.position * zoom;
            mouse_grid_pos.x += mouse_pos.x - window_size.width as f32 / 2.0;
            mouse_grid_pos.y += window_size.height as f32 / 2.0 - mouse_pos.y;

            let mouse_grid_x = (mouse_grid_pos.x / 32.0 / zoom).floor() as i32;
            let mouse_grid_y = (mouse_grid_pos.y / 32.0 / zoom).floor() as i32;

            let mouse_in_grid =
                mouse_grid_x >= 0 && mouse_grid_x < 128 && mouse_grid_y >= 0 && mouse_grid_y < 128;
            if mouse_in_grid {
                if mouse_left {
                    grid.tiles[mouse_grid_y as usize][mouse_grid_x as usize] = 1;
                }
                if mouse_right {
                    grid.tiles[mouse_grid_y as usize][mouse_grid_x as usize] = 0;
                }
            }

            let start = Instant::now();
            unsafe {
                gl::ClearColor(0.3, 0.3, 0.3, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
                shader.enable();
                shader.set_mat4(&CString::new("view").unwrap(), camera.view_matrix());
                shader.set_mat4(
                    &CString::new("projection").unwrap(),
                    camera.projection_matrix(
                        window_size.width as f32,
                        window_size.height as f32,
                        zoom,
                    ),
                );
                gl::BindVertexArray(quad_vao);

                if mouse_in_grid && grid.tiles[mouse_grid_y as usize][mouse_grid_x as usize] == 0 {
                    shader.set_vec3(&CString::new("color").unwrap(), vec3(0.3, 0.3, 1.0));
                    let model_scale = Matrix4::from_nonuniform_scale(32.0, 32.0, 0.0);
                    let model_trans = Matrix4::from_translation(cgmath::vec3(
                        16.0 + 32.0 * mouse_grid_x as f32,
                        16.0 + 32.0 * mouse_grid_y as f32,
                        0.0,
                    ));
                    let model = model_trans * model_scale;
                    shader.set_mat4(&CString::new("model").unwrap(), model);
                    gl::DrawArrays(gl::TRIANGLES, 0, 6);
                }

                for (y, row) in grid.tiles.iter().enumerate() {
                    for (x, tile) in row.iter().enumerate() {
                        if *tile > 0 {
                            let model_scale = Matrix4::from_nonuniform_scale(32.0, 32.0, 0.0);
                            let model_trans = Matrix4::from_translation(cgmath::vec3(
                                16.0 + 32.0 * x as f32,
                                16.0 + 32.0 * y as f32,
                                0.0,
                            ));
                            let model = model_trans * model_scale;
                            shader.set_mat4(&CString::new("model").unwrap(), model);
                            shader.set_vec3(&CString::new("color").unwrap(), vec3(0.0, 0.0, 1.0));
                            gl::DrawArrays(gl::TRIANGLES, 0, 6);
                        }
                    }
                }

                if debug_grid {
                    gl::BindVertexArray(line_vao);
                    shader.set_vec3(&CString::new("color").unwrap(), vec3(0.0, 0.0, 0.0));
                    for y in 0..=128 {
                        let model_scale = Matrix4::from_nonuniform_scale(32.0 * 128.0, 0.0, 0.0);
                        let model_trans =
                            Matrix4::from_translation(cgmath::vec3(0.0, y as f32 * 32.0, 0.0));
                        let model = model_trans * model_scale;
                        shader.set_mat4(&CString::new("model").unwrap(), model);
                        gl::DrawArrays(gl::LINES, 0, 2);
                    }
                    for x in 0..=128 {
                        let rect_rot = std::f32::consts::FRAC_PI_2;
                        let model_rot = Matrix4::from_angle_z(Rad(rect_rot));
                        let model_scale = Matrix4::from_nonuniform_scale(32.0 * 128.0, 0.0, 0.0);
                        let model_trans =
                            Matrix4::from_translation(cgmath::vec3(x as f32 * 32.0, 0.0, 0.0));
                        let model = model_trans * model_rot * model_scale;
                        shader.set_mat4(&CString::new("model").unwrap(), model);
                        gl::DrawArrays(gl::LINES, 0, 2);
                    }
                }
            }
            let dur = Instant::now() - start;
            let ms = dur.as_nanos() as f64 / 1_000_000.0;
            if show_fps {
                println!("FPS: {}\tRender time: {}ms", 1000.0 / ms, ms);
            }
            gl_window.swap_buffers().unwrap();
        }
    });
}

struct Camera {
    position: Point2<GLfloat>,
}

static CAMERA_DIRECTION: Vector3<GLfloat> = vec3(0.0, 0.0, -1.0);
static CAMERA_UP: Vector3<GLfloat> = vec3(0.0, 1.0, 0.0);

impl Camera {
    fn new() -> Self {
        Self {
            position: point2(0.0, 0.0),
        }
    }

    fn move_horizontal(&mut self, amount: f32) {
        self.position.x += amount;
    }

    fn move_vertical(&mut self, amount: f32) {
        self.position.y += amount;
    }

    fn view_matrix(&self) -> Matrix4<GLfloat> {
        Matrix4::look_to_rh(
            point3(self.position.x, self.position.y, 1.0),
            CAMERA_DIRECTION,
            CAMERA_UP,
        )
    }

    fn projection_matrix(&self, width: GLfloat, height: GLfloat, zoom: f32) -> Matrix4<GLfloat> {
        ortho(
            -width / zoom / 2.0,
            width / zoom / 2.0,
            -height / zoom / 2.0,
            height / zoom / 2.0,
            1.0,
            -1.0,
        )
    }
}

struct Grid {
    tiles: [[u32; 128]; 128],
}

impl Grid {
    fn new() -> Self {
        Self {
            tiles: [[0; 128]; 128],
        }
    }
}
