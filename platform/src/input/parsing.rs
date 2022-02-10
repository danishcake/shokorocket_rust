use pygamer::buttons::{ButtonReader, Keys};
use pygamer::hal;
use pygamer::pac::ADC1;
use pygamer::pins::JoystickReader;

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

    fn up() -> ButtonState {
        ButtonState {
            down: false,
            pressed: false,
            released: true,
        }
    }

    fn down() -> ButtonState {
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
    /// Button state
    pub btn_a: ButtonState,
    pub btn_b: ButtonState,
    pub btn_start: ButtonState,
    pub btn_select: ButtonState,
}

/// A joystick reader and the associated adc
pub struct JoystickReaderWithAdc<'a> {
    pub reader: &'a mut JoystickReader,
    pub adc: &'a mut hal::adc::Adc<ADC1>,
}

/// Returns the instantaneous input state
/// Annoyingly, the button reader class only exposes change events
/// despite having the instantanous state. We have to reconstruct the current state
pub fn read_input(
    button_reader: &mut ButtonReader,
    joystick: &mut JoystickReaderWithAdc,
) -> InputState {
    let mut input_state: InputState = InputState {
        js_x: 0,
        js_y: 0,
        btn_a: ButtonState::new(),
        btn_b: ButtonState::new(),
        btn_start: ButtonState::new(),
        btn_select: ButtonState::new(),
    };

    // Grab the change events, which also updates the instantaneous state in button_reader.last
    for ev in button_reader.events() {
        match ev {
            Keys::AUp => input_state.btn_a = ButtonState::up(),
            Keys::ADown => input_state.btn_a = ButtonState::down(),
            Keys::BUp => input_state.btn_b = ButtonState::up(),
            Keys::BDown => input_state.btn_b = ButtonState::down(),
            Keys::StartUp => input_state.btn_start = ButtonState::up(),
            Keys::StartDown => input_state.btn_start = ButtonState::down(),
            Keys::SelectUp => input_state.btn_select = ButtonState::up(),
            Keys::SelectDown => input_state.btn_select = ButtonState::down(),
        }
    }

    // Now update the instantaneous state for buttons that didn't change this frame
    input_state.btn_select.down = button_reader.last & 0x01 != 0;
    input_state.btn_start.down = button_reader.last & 0x02 != 0;
    input_state.btn_a.down = button_reader.last & 0x04 != 0;
    input_state.btn_b.down = button_reader.last & 0x08 != 0;

    // Read the joystick state
    let (x, y) = joystick.reader.read(&mut joystick.adc);

    input_state.js_x = (x as i16) - 2048i16;
    input_state.js_y = (y as i16) - 2048i16;

    input_state
}
