/// The state of a single button
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ButtonState {
    /// If the button is currently down
    pub down: bool,
    /// If the button has just been pressed
    pub pressed: bool,
    /// If the button has just been released
    pub released: bool,
}

impl ButtonState {
    fn new() -> ButtonState {
        ButtonState {
            down: false,
            pressed: false,
            released: false,
        }
    }

    pub fn up() -> ButtonState {
        ButtonState {
            down: false,
            pressed: false,
            released: true,
        }
    }

    pub fn down() -> ButtonState {
        ButtonState {
            down: true,
            pressed: true,
            released: false,
        }
    }
}

/// The state of all input devices
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct InputState {
    /// Joystick sensor reading in range [-2048, 2047]
    pub js_x: i16,
    pub js_y: i16,

    /// Joystick 'flicks'
    pub js_up: ButtonState,
    pub js_down: ButtonState,
    pub js_left: ButtonState,
    pub js_right: ButtonState,

    /// Button state
    pub btn_a: ButtonState,
    pub btn_b: ButtonState,
    pub btn_start: ButtonState,
    pub btn_select: ButtonState,
}

impl InputState {
    pub fn new() -> InputState {
        InputState {
            js_x: 0,
            js_y: 0,
            js_up: ButtonState::new(),
            js_down: ButtonState::new(),
            js_left: ButtonState::new(),
            js_right: ButtonState::new(),
            btn_a: ButtonState::new(),
            btn_b: ButtonState::new(),
            btn_start: ButtonState::new(),
            btn_select: ButtonState::new(),
        }
    }
}
