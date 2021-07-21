use std::collections::{HashMap, HashSet};

use glutin::event::{
    ElementState, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
};

pub struct KeyboardState {
    state: HashSet<VirtualKeyCode>,
    momentary_state: HashMap<VirtualKeyCode, ElementState>,
}

impl KeyboardState {
    pub fn new() -> Self {
        Self {
            state: HashSet::new(),
            momentary_state: HashMap::new(),
        }
    }

    pub fn is_pressed(&self, key: VirtualKeyCode) -> bool {
        self.state.get(&key).is_some()
    }

    pub fn was_pressed(&self, key: VirtualKeyCode) -> bool {
        self.momentary_state.get(&key) == Some(&ElementState::Pressed)
    }

    pub fn was_released(&self, key: VirtualKeyCode) -> bool {
        self.momentary_state.get(&key) == Some(&ElementState::Released)
    }

    pub fn process_event(&mut self, event: &WindowEvent) {
        match *event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(virtual_keycode),
                        ..
                    },
                ..
            } => {
                self.momentary_state.insert(virtual_keycode, state);
                match state {
                    ElementState::Pressed => {
                        self.state.insert(virtual_keycode);
                    }
                    ElementState::Released => {
                        self.state.remove(&virtual_keycode);
                    }
                }
            }
            _ => (),
        }
    }

    pub fn clear_momentary_state(&mut self) {
        self.momentary_state.clear();
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

pub struct MouseState {
    button_state: HashSet<MouseButton>,
    momentary_button_state: HashMap<MouseButton, ElementState>,
    pub position: Point,
    pub mouse_delta: Point,
    pub scroll_delta: f32,
}

impl MouseState {
    pub fn new() -> Self {
        Self {
            button_state: HashSet::new(),
            momentary_button_state: HashMap::new(),
            position: Default::default(),
            mouse_delta: Default::default(),
            scroll_delta: 0.0,
        }
    }

    pub fn is_pressed(&self, button: MouseButton) -> bool {
        self.button_state.get(&button).is_some()
    }

    pub fn was_pressed(&self, button: MouseButton) -> bool {
        self.momentary_button_state.get(&button) == Some(&ElementState::Pressed)
    }

    pub fn was_released(&self, button: MouseButton) -> bool {
        self.momentary_button_state.get(&button) == Some(&ElementState::Released)
    }

    pub fn process_event(&mut self, event: &WindowEvent) {
        match *event {
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_delta.x += position.x - self.position.x;
                self.mouse_delta.y += position.y - self.position.y;
                self.position = Point::new(position.x, position.y);
            }
            WindowEvent::MouseInput { button, state, .. } => {
                self.momentary_button_state.insert(button, state);
                match state {
                    ElementState::Pressed => {
                        self.button_state.insert(button);
                    }
                    ElementState::Released => {
                        self.button_state.remove(&button);
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(_, y) => {
                    self.scroll_delta += y;
                }
                _ => (),
            },
            _ => (),
        }
    }

    pub fn clear_momentary_state(&mut self) {
        self.momentary_button_state.clear();
        self.mouse_delta = Default::default();
        self.scroll_delta = 0.0;
    }
}
