pub use winit::event::{ElementState, MouseButton};

pub enum BoardEvent {
    CursorMoved {
        position: (usize, usize),
    },
    MouseInput {
        state: ElementState,
        button: MouseButton,
    },
    CursorLeft,
}

#[derive(Debug, Default)]
pub struct MouseState {
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_cursor_in: bool,
    pos: (usize, usize),
    delta: (i16, i16),
}

impl MouseState {
    pub fn update_pos(&mut self, p: (usize, usize)) {
        self.delta = (
            p.0 as i16 - self.pos.0 as i16,
            p.1 as i16 - self.pos.1 as i16,
        );
        self.pos = p;
    }

    pub fn get_is_left_pressed(&self) -> bool {
        self.is_left_pressed
    }

    pub fn get_is_right_pressed(&self) -> bool {
        self.is_right_pressed
    }

    pub fn get_is_cursor_in(&self) -> bool {
        self.is_cursor_in
    }

    pub fn get_pos(&self) -> (usize, usize) {
        self.pos
    }

    pub fn get_delta(&self) -> (i16, i16) {
        self.delta
    }

    pub fn set_left_pressed(&mut self) {
        self.is_left_pressed = true;
    }

    pub fn set_left_released(&mut self) {
        self.is_left_pressed = false;
    }

    pub fn set_right_pressed(&mut self) {
        self.is_right_pressed = true;
    }

    pub fn set_right_released(&mut self) {
        self.is_right_pressed = false;
    }

    pub fn unset_cursor_in(&mut self) {
        self.is_cursor_in = false;
    }

    pub fn set_cursor_in(&mut self) {
        self.is_cursor_in = true;
    }
}
