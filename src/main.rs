#![no_std]
#![no_main]

use core::convert::Infallible;

#[cfg(not(feature = "panic_led"))]
use panic_halt as _;
use pygamer::pins::Display;
use pygamer::pwm::Pwm2;
use pygamer::sercom::SPIMaster4;
use pygamer::{entry, hal, pac, Pins};
use pygamer::gpio::{OpenDrain, Output, Pa23, PfC, PushPull, Pa0, Pb5, Pb13, Pb14, Pb15};

use hal::clock::GenericClockController;
//use hal::delay::Delay;
use hal::timer::TimerCounter;
use hal::prelude::*;
use hal::watchdog::{Watchdog, WatchdogTimeout};
use pac::{CorePeripherals, Peripherals};
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
use embedded_graphics::{image::Image, pixelcolor::Rgb565};
use st7735_lcd::ST7735;



#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct GameState {
    led_state: bool
}

type PygamerDisplay = ST7735<
    SPIMaster4<
        hal::sercom::Sercom4Pad2<Pb14<PfC>>,
        hal::sercom::Sercom4Pad3<Pb15<PfC>>,
        hal::sercom::Sercom4Pad1<Pb13<PfC>>,
    >,
    Pb5<Output<PushPull>>,
    Pa0<Output<PushPull>>,
>;

type PygamerBacklight = Pwm2<pygamer::gpio::v2::PA01>;

type PygamerOnboardRedLed = Pa23<Output<OpenDrain>>;

struct Outputs<'a> {
    led: PygamerOnboardRedLed,
    display: &'a mut PygamerDisplay,
    backlight: &'a mut PygamerBacklight
}

fn simulate(game_state: &mut GameState) {
    game_state.led_state = !game_state.led_state
}

fn render(game_state: &GameState, outputs: &mut Outputs) {
    (match game_state.led_state {
        true => outputs.led.set_high(),
        false => outputs.led.set_low()
    }).unwrap();

    (match game_state.led_state {
        true => {
            Rectangle::with_corners(Point::new(0, 0), Point::new(160, 128))
            .into_styled(
                PrimitiveStyleBuilder::new()
                    .fill_color(Rgb565::BLACK)
                    .build(),
            )
            .draw(outputs.display)
        },
        false => {
            Rectangle::with_corners(Point::new(0, 0), Point::new(160, 128))
            .into_styled(
                PrimitiveStyleBuilder::new()
                    .fill_color(Rgb565::WHITE)
                    .build(),
            )
            .draw(outputs.display)
        }
    }).unwrap();
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


    let mut wdt = Watchdog::new(peripherals.WDT);
    wdt.start(WatchdogTimeout::Cycles256 as u8);
    // TBD: What is the unit here? Looks like it should be milliseconds, but it's unclear...


    // Setup a 60hz timer to control the frame times
    // let gclk1 = clocks.gclk1();
    // let timer_clock = clocks.tc2_tc3(&gclk1).unwrap();
    // let mut tcounter = TimerCounter::tc3_(&timer_clock, peripherals.TC3, &mut peripherals.MCLK);
    // tcounter.start(60.hz());


    let mut game_state = GameState {
        led_state: true
    };

    let mut outputs = Outputs {
        led: pins.led_pin.into_open_drain_output(&mut pins.port),
        display: &mut display,
        backlight: &mut backlight
    };


    // Main loop.
    loop {
        // Simulate
        simulate(&mut game_state);

        // Draw
        render(&game_state, &mut outputs);

        // Sleep for remainder of frame
        // TODO: Actually sleep for rest of frame
        // TODO: Measure elapsed times
        delay.delay_ms(20u8);
        wdt.feed();
    }
}
