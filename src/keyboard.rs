use std::collections::HashSet;

use glutin::event::{ElementState, VirtualKeyCode};

pub struct KeyboardState {
    state: HashSet<VirtualKeyCode>,
}

impl KeyboardState {
    /// Constructs a new KeyboardState with all the keys released.
    pub fn new() -> KeyboardState {
        KeyboardState {
            state: HashSet::new(),
        }
    }

    pub fn is_pressed(&self, key: VirtualKeyCode) -> bool {
        self.state.get(&key).is_some()
    }

    pub fn process_event(&mut self, key_state: ElementState, code: VirtualKeyCode) {
        match key_state {
            ElementState::Pressed => {
                self.state.insert(code);
            }
            ElementState::Released => {
                self.state.remove(&code);
            }
        }
    }
}
