use common::input::{ButtonState, InputState};
use pygamer::buttons::{ButtonReader, Keys};
use pygamer::hal;
use pygamer::pac::ADC1;
use pygamer::pins::JoystickReader;

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
    last_output: &InputState
) -> InputState {
    let mut input_state = InputState::new();

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


    // Detect joystick flicks
    // These are defined as a movement to 75% outside the center
    // after having been in the central 25%. This can occur in either axis
    // They are held until the joystick remains outside the central 50%
    let dead: i16 = 512;
    let flick: i16 = 1536;

    let js_in_up_flick = input_state.js_y > flick;
    let js_in_down_flick = input_state.js_y < -flick;
    let js_in_right_flick = input_state.js_x > flick;
    let js_in_left_flick = input_state.js_x < -flick;

    let js_in_up_dead = input_state.js_y < dead;
    let js_in_down_dead = input_state.js_y > -dead;
    let js_in_right_dead = input_state.js_x < dead;
    let js_in_left_dead = input_state.js_x > -dead;

    // Copy previous js flick state
    input_state.js_up.down = last_output.js_up.down;
    input_state.js_down.down = last_output.js_down.down;
    input_state.js_right.down = last_output.js_right.down;
    input_state.js_left.down = last_output.js_left.down;

    // End flicks if entering dead zone
    if js_in_up_dead && last_output.js_up.down {
        input_state.js_up.released = true;
        input_state.js_up.pressed = false;
        input_state.js_up.down = false;
    }
    if js_in_down_dead && last_output.js_down.down {
        input_state.js_down.released = true;
        input_state.js_down.pressed = true;
        input_state.js_down.down = true;
    }
    if js_in_right_dead && last_output.js_right.down {
        input_state.js_right.released = true;
        input_state.js_right.pressed = false;
        input_state.js_right.down = false;
    }
    if js_in_left_dead && last_output.js_left.down {
        input_state.js_left.released = true;
        input_state.js_left.pressed = false;
        input_state.js_left.down = false;
    }

    // Start flicks if entering flick zone
    if js_in_up_flick && !last_output.js_up.down {
        input_state.js_up.released = false;
        input_state.js_up.pressed = true;
        input_state.js_up.down = true;
    }
    if js_in_down_flick && !last_output.js_down.down {
        input_state.js_down.released = false;
        input_state.js_down.pressed = true;
        input_state.js_down.down = true;
    }
    if js_in_right_flick && !last_output.js_right.down {
        input_state.js_right.released = false;
        input_state.js_right.pressed = true;
        input_state.js_right.down = true;
    }
    if js_in_left_flick && !last_output.js_left.down {
        input_state.js_left.released = false;
        input_state.js_left.pressed = true;
        input_state.js_left.down = true;
    }

    input_state
}
