#![no_std]
#![no_main]

mod maps;

#[cfg(not(feature = "panic_led"))]
use panic_halt as _;
use pygamer::adc::Adc;
use pygamer::gpio::{OpenDrain, Output, Pa0, Pa23, Pb13, Pb14, Pb15, Pb5, PfC, PushPull};
use pygamer::pins::ButtonReader;
use pygamer::sercom::SPIMaster4;
use pygamer::{entry, hal, pac, Pins};

use hal::clock::GenericClockController;
//use hal::delay::Delay;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    text::Text,
};
use hal::pac::gclk::pchctrl::GEN_A::GCLK11;
use hal::prelude::*;
use hal::timer::TimerCounter;
use hal::watchdog::{Watchdog, WatchdogTimeout};
use pac::{CorePeripherals, Peripherals};
use st7735_lcd::ST7735;

use common::input::InputState;
use maps::E1M1;
use platform::input::{read_input, JoystickReaderWithAdc};
use simulation::{StateMachine, World};

/// A type alias for the Pygamer display
type PygamerDisplay = ST7735<
    SPIMaster4<
        hal::sercom::Sercom4Pad2<Pb14<PfC>>,
        hal::sercom::Sercom4Pad3<Pb15<PfC>>,
        hal::sercom::Sercom4Pad1<Pb13<PfC>>,
    >,
    Pb5<Output<PushPull>>,
    Pa0<Output<PushPull>>,
>;

/// Type alias for the Pygamer onboard LED
type PygamerOnboardRedLed = Pa23<Output<OpenDrain>>;

struct Resources<'a> {
    text_style: MonoTextStyle<'a, Rgb565>,
}

/// The set of outputs on the pygamer
struct Outputs<'a> {
    led: &'a mut PygamerOnboardRedLed,
    display: &'a mut PygamerDisplay,
}

/// The state of the game
struct GameState {
    state_machine: StateMachine,
}

fn simulate(game_state: &mut GameState, input: &InputState) {
    game_state.state_machine.tick(input);
    // TODO: Inject input here
    // Maybe factor out a common project so we can have input state, then the impl in the platform?
}

fn render(game_state: &GameState, outputs: &mut Outputs, resources: &Resources) {
    // TODO: Add double buffering
    // TODO: Figure out some vsync equivalent to avoid tearing/flickering

    outputs.display.clear(Rgb565::BLACK).unwrap();
    Text::new("Hello Rust!", Point::new(20, 30), resources.text_style)
        .draw(outputs.display)
        .unwrap();
}

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );
    let mut pins = Pins::new(peripherals.PORT).split();
    let mut delay = hal::delay::Delay::new(core.SYST, &mut clocks);

    let (mut display, mut backlight) = pins
        .display
        .init(
            &mut clocks,
            peripherals.SERCOM4,
            &mut peripherals.MCLK,
            peripherals.TC2,
            &mut delay,
            &mut pins.port,
        )
        .unwrap();

    let mut adc1 = Adc::adc1(peripherals.ADC1, &mut peripherals.MCLK, &mut clocks, GCLK11);

    let mut button_reader: ButtonReader = pins.buttons.init(&mut pins.port);
    let mut joystick_reader = JoystickReaderWithAdc {
        reader: &mut pins.joystick.init(&mut pins.port),
        adc: &mut adc1,
    };

    //let mut wdt = Watchdog::new(peripherals.WDT);
    //wdt.start(WatchdogTimeout::Cycles256 as u8);
    // TBD: What is the unit here? Looks like it should be milliseconds, but it's unclear...

    // Setup a 60hz timer to control the frame times
    let gclk1 = clocks.gclk1();
    let timer_clock = clocks.tc4_tc5(&gclk1).unwrap();
    let mut tcounter = TimerCounter::tc5_(&timer_clock, peripherals.TC5, &mut peripherals.MCLK);
    tcounter.start(60.hz());

    let mut game_state = GameState {
        state_machine: StateMachine::new(World::load(E1M1)),
    };

    let mut outputs = Outputs {
        led: &mut pins.led_pin.into_open_drain_output(&mut pins.port),
        display: &mut display,
    };

    let resources = Resources {
        text_style: MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE),
    };

    let mut last_input = InputState::new();
    // Main loop.
    loop {
        // Read input
        let input = read_input(&mut button_reader, &mut joystick_reader, &last_input);
        last_input = input;

        // Simulate
        simulate(&mut game_state, &input);

        // Draw
        render(&game_state, &mut outputs, &resources);

        // Sleep for remainder of frame
        // TODO: This probably doesn't work. Measure elapsed time!
        let _ = nb::block!(tcounter.wait());
    }
}
