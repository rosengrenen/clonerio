use std::collections::{HashMap, HashSet};

use glutin::event::{ElementState, VirtualKeyCode};

pub struct KeyboardState {
    state: HashSet<VirtualKeyCode>,
    momentary_state: HashMap<VirtualKeyCode, ElementState>,
}

impl KeyboardState {
    /// Constructs a new KeyboardState with all the keys released.
    pub fn new() -> KeyboardState {
        KeyboardState {
            state: HashSet::new(),
            momentary_state: HashMap::new(),
        }
    }

    pub fn is_pressed(&self, key: VirtualKeyCode) -> bool {
        self.state.get(&key).is_some()
    }

    pub fn is_released(&self, key: VirtualKeyCode) -> bool {
        !self.is_pressed(key)
    }

    pub fn was_pressed(&self, key: VirtualKeyCode) -> bool {
        self.momentary_state.get(&key) == Some(&ElementState::Pressed)
    }

    pub fn was_released(&self, key: VirtualKeyCode) -> bool {
        self.momentary_state.get(&key) == Some(&ElementState::Released)
    }

    pub fn process_event(&mut self, key_state: ElementState, code: VirtualKeyCode) {
        self.momentary_state.insert(code, key_state);
        match key_state {
            ElementState::Pressed => {
                self.state.insert(code);
            }
            ElementState::Released => {
                self.state.remove(&code);
            }
        }
    }

    pub fn clear_momentary_state(&mut self) {
        self.momentary_state.clear();
    }
}
