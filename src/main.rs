mod keyboard;
mod shader;

use std::{
    ffi::{c_void, CString},
    mem,
    time::Instant,
};

use cgmath::{
    ortho, point2, point3, vec2, vec3, vec4, Deg, Matrix2, Matrix4, Point2, Rad, Vector3,
};
use gl::types::*;
use glutin::event::ElementState;
use keyboard::KeyboardState;
use shader::Shader;

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
    }

    let image = image::open("assets/textures/belt-1.png").unwrap().flipv();
    let image = image.as_rgba8().unwrap();
    let mut tex = 0;

    unsafe {
        // load texture
        gl::CreateTextures(gl::TEXTURE_2D, 1, &mut tex);
        gl::TextureParameteri(tex, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TextureParameteri(tex, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TextureParameteri(tex, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TextureParameteri(tex, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

        gl::TextureStorage2D(
            tex,
            1,
            gl::RGBA8,
            image.width() as i32,
            image.height() as i32,
        );
        gl::TextureSubImage2D(
            tex,
            0,
            0,
            0,
            image.width() as i32,
            image.height() as i32,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            image.as_raw().as_ptr() as *const c_void,
        );
    }

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
            4 * mem::size_of::<GLfloat>() as GLsizei,
        );

        gl::EnableVertexArrayAttrib(quad_vao, 0);
        gl::EnableVertexArrayAttrib(quad_vao, 1);

        gl::VertexArrayAttribFormat(quad_vao, 0, 2, gl::FLOAT, gl::FALSE, 0);
        gl::VertexArrayAttribFormat(
            quad_vao,
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            2 * mem::size_of::<GLfloat>() as GLuint,
        );

        gl::VertexArrayAttribBinding(quad_vao, 0, 0);
        gl::VertexArrayAttribBinding(quad_vao, 1, 0);

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
    camera.position.x = 300.0;
    camera.position.y = 300.0;

    let mut keyboard_state = KeyboardState::new();

    let mut grid = Grid::new();

    let mut mouse_pos = vec2(0.0, 0.0);
    let mut mouse_left = false;
    let mut mouse_right = false;

    let mut zoom = 2.0;

    let mut debug_grid = true;
    let mut show_fps = false;

    let mut last_update_time = Instant::now();

    let mut current_belt = Belt::new();

    let mut is_placing = true;

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

                if keyboard_state.was_pressed(VirtualKeyCode::T) {
                    current_belt.output = match current_belt.turn() {
                        Turn::Right => current_belt.output.rotate_clockwise().rotate_clockwise(),
                        _ => current_belt.output.rotate_clockwise(),
                    };
                }
            }

            if keyboard_state.was_pressed(VirtualKeyCode::Space) {
                is_placing = !is_placing;
            }

            println!("{}", zoom);
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

            let window_size = gl_window.window().inner_size();

            let mut mouse_grid_pos = camera.position * zoom;
            mouse_grid_pos.x += mouse_pos.x - window_size.width as f32 / 2.0;
            mouse_grid_pos.y += window_size.height as f32 / 2.0 - mouse_pos.y;

            let mouse_grid_x = (mouse_grid_pos.x / 32.0 / zoom).floor() as i32;
            let mouse_grid_y = (mouse_grid_pos.y / 32.0 / zoom).floor() as i32;

            let mouse_in_grid =
                mouse_grid_x >= 0 && mouse_grid_x < 128 && mouse_grid_y >= 0 && mouse_grid_y < 128;
            if is_placing && mouse_in_grid {
                if mouse_left {
                    grid.place_belt(mouse_grid_x as isize, mouse_grid_y as isize, current_belt);
                }

                if mouse_right {
                    grid.clear_tile(mouse_grid_x as usize, mouse_grid_y as usize);
                }
            }

            let start = Instant::now();
            unsafe {
                gl::ClearColor(0.3, 0.3, 0.6, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);

                gl::BindTextureUnit(0, tex);

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
                gl::BindVertexArray(quad_vao);

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
                    gl::BindVertexArray(line_vao);
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
    tiles: [[Option<Belt>; 128]; 128],
}

impl Grid {
    fn new() -> Self {
        Self {
            tiles: [[None; 128]; 128],
        }
    }

    fn place_belt(&mut self, x: isize, y: isize, belt: Belt) {
        let belt = self.calculate_belt_position(x, y, belt);
        // Adjust input of belt in front
        // - -
        //   |
        // front belt direction west/east
        // current belt direction north
        if let (Some(mut front_belt), (front_belt_x, front_belt_y)) =
            self.belt_in_front_of(x, y, belt)
        {
            if (front_belt.input == belt.output.rotate_clockwise()
                || front_belt.input == belt.output.rotate_anti_clockwise())
                && self
                    .belt_behind(front_belt_x, front_belt_y, front_belt)
                    .0
                    .is_none()
            {
                front_belt.input = belt.output.flip();
                self.set_belt_in_front_of(x, y, belt, front_belt);
            } else if front_belt.output == belt.output.rotate_clockwise()
                || front_belt.output == belt.output.rotate_anti_clockwise()
            {
                if front_belt.input == belt.output {
                    front_belt.input = front_belt.output.flip();
                    self.set_belt_in_front_of(x, y, belt, front_belt);
                }
            } else if front_belt.output == belt.output && front_belt.input != belt.output.flip() {
                front_belt.input = front_belt.output.flip();
                self.set_belt_in_front_of(x, y, belt, front_belt);
            }
        }

        self.set_belt(x, y, belt);
    }

    fn calculate_belt_position(&self, x: isize, y: isize, mut belt: Belt) -> Belt {
        let (belt_behind, _) = self.belt_behind(x, y, belt);
        if let Some(belt_behind) = belt_behind {
            if belt_behind.output == belt.input.flip() {
                return belt;
            }
        }

        let (left_belt, a) = self.belt_left_of(x, y, belt);
        let (right_belt, b) = self.belt_right_of(x, y, belt);
        if left_belt.is_some() && right_belt.is_some() {
            let left_belt = left_belt.unwrap();
            let right_belt = right_belt.unwrap();

            let left_belt_facing_into = left_belt.output == belt.output.rotate_clockwise();
            let right_belt_facing_into = right_belt.output == belt.output.rotate_anti_clockwise();

            if left_belt_facing_into && !right_belt_facing_into {
                belt.input = left_belt.output.flip();
            } else if !left_belt_facing_into && right_belt_facing_into {
                belt.input = right_belt.output.flip();
            }
        } else if let Some(left_belt) = left_belt {
            let left_belt_facing_into = left_belt.output == belt.output.rotate_clockwise();

            if left_belt_facing_into {
                belt.input = left_belt.output.flip();
            }
        } else if let Some(right_belt) = right_belt {
            let right_belt_facing_into = right_belt.output == belt.output.rotate_anti_clockwise();

            if right_belt_facing_into {
                belt.input = right_belt.output.flip();
            }
        }

        belt
    }

    fn clear_tile(&mut self, x: usize, y: usize) {
        self.tiles[y][x] = None;
    }

    fn get_belt(&self, x: isize, y: isize) -> Option<Belt> {
        if x >= 0 && x < self.tiles[0].len() as isize && y >= 0 && y < self.tiles.len() as isize {
            self.tiles[y as usize][x as usize]
        } else {
            None
        }
    }

    fn set_belt(&mut self, x: isize, y: isize, belt: Belt) {
        if x >= 0 && x < self.tiles[0].len() as isize && y >= 0 && y < self.tiles.len() as isize {
            self.tiles[y as usize][x as usize] = Some(belt);
        }
    }

    fn left_pos(x: isize, y: isize, belt: Belt) -> (isize, isize) {
        match belt.output {
            Direction::West => (x, y - 1),
            Direction::North => (x - 1, y),
            Direction::East => (x, y + 1),
            Direction::South => (x + 1, y),
        }
    }

    fn right_pos(x: isize, y: isize, belt: Belt) -> (isize, isize) {
        match belt.output {
            Direction::West => (x, y + 1),
            Direction::North => (x + 1, y),
            Direction::East => (x, y - 1),
            Direction::South => (x - 1, y),
        }
    }

    fn front_pos(x: isize, y: isize, belt: Belt) -> (isize, isize) {
        match belt.output {
            Direction::West => (x - 1, y),
            Direction::North => (x, y + 1),
            Direction::East => (x + 1, y),
            Direction::South => (x, y - 1),
        }
    }

    fn behind_pos(x: isize, y: isize, belt: Belt) -> (isize, isize) {
        match belt.input {
            Direction::West => (x - 1, y),
            Direction::North => (x, y + 1),
            Direction::East => (x + 1, y),
            Direction::South => (x, y - 1),
        }
    }

    fn belt_left_of(&self, x: isize, y: isize, belt: Belt) -> (Option<Belt>, (isize, isize)) {
        let (x, y) = Self::left_pos(x, y, belt);
        (self.get_belt(x, y), (x, y))
    }

    fn belt_right_of(&self, x: isize, y: isize, belt: Belt) -> (Option<Belt>, (isize, isize)) {
        let (x, y) = Self::right_pos(x, y, belt);
        (self.get_belt(x, y), (x, y))
    }

    fn belt_in_front_of(&self, x: isize, y: isize, belt: Belt) -> (Option<Belt>, (isize, isize)) {
        let (x, y) = Self::front_pos(x, y, belt);
        (self.get_belt(x, y), (x, y))
    }

    fn set_belt_in_front_of(&mut self, x: isize, y: isize, belt: Belt, new_belt: Belt) {
        let (x, y) = Self::front_pos(x, y, belt);
        self.set_belt(x, y, new_belt);
    }

    fn belt_behind(&self, x: isize, y: isize, belt: Belt) -> (Option<Belt>, (isize, isize)) {
        let (x, y) = Self::behind_pos(x, y, belt);
        (self.get_belt(x, y), (x, y))
    }
}

#[derive(Clone, Copy, Debug)]
struct Belt {
    input: Direction,
    output: Direction,
}

impl Belt {
    fn new() -> Self {
        Self {
            input: Direction::West,
            output: Direction::East,
        }
    }
}

impl Belt {
    fn turn(&self) -> Turn {
        let dir = self.input.rotate_clockwise();
        if dir == self.output {
            return Turn::Left;
        }

        let dir = dir.rotate_clockwise();
        if dir == self.output {
            return Turn::Forward;
        }

        let dir = dir.rotate_clockwise();
        if dir == self.output {
            return Turn::Right;
        }

        panic!();
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Direction {
    West,
    North,
    East,
    South,
}

impl Direction {
    fn rotate_clockwise(&self) -> Self {
        match *self {
            Self::West => Self::North,
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
        }
    }

    fn rotate_anti_clockwise(&self) -> Self {
        match *self {
            Self::West => Self::South,
            Self::North => Self::West,
            Self::East => Self::North,
            Self::South => Self::East,
        }
    }

    fn flip(&self) -> Self {
        match *self {
            Self::West => Self::East,
            Self::North => Self::South,
            Self::East => Self::West,
            Self::South => Self::North,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Turn {
    Left,
    Forward,
    Right,
}
