#![no_std]
#![no_main]

extern crate panic_halt;
extern crate metro_m4 as hal;

use embedded_graphics::image::Image16BPP;
use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::PixelColorU16;
use embedded_graphics::primitives::Rect;

use hal::spi_master;
use hal::prelude::*;
use hal::clock::GenericClockController;
use hal::{entry, Peripherals, CorePeripherals};
use st7735_lcd;
use st7735_lcd::Orientation;

#[entry]
fn main() -> ! {
    let core = CorePeripherals::take().unwrap();
    let mut peripherals = Peripherals::take().unwrap();
    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );

    let mut pins = hal::Pins::new(peripherals.PORT);

    let spi = spi_master(
        &mut clocks,
        16.mhz(),
        peripherals.SERCOM2,
        &mut peripherals.MCLK,
        pins.sck,
        pins.mosi,
        pins.miso,
        &mut pins.port
    );
    
    let dc = pins.d0.into_push_pull_output(&mut pins.port);
    let rst = pins.d1.into_push_pull_output(&mut pins.port);
    let mut delay = hal::delay::Delay::new(core.SYST, &mut clocks);

    let mut disp = st7735_lcd::ST7735::new(spi, dc, rst, false, true);
    disp.init(&mut delay).unwrap();
    disp.set_orientation(&Orientation::Landscape).unwrap();

    // There's a translation here because my particular lcd seems to be off a few pixels
    let black_backdrop: Rect<PixelColorU16> = Rect::new(Coord::new(0, 0), Coord::new(160, 80)).with_fill(Some(0x0000u16.into())).translate(Coord::new(1, 25));

    disp.draw(black_backdrop.into_iter());
    
    let ferris = Image16BPP::new(include_bytes!("./ferris.raw"), 86, 64).translate(Coord::new(40, 33));
    
    disp.draw(ferris.into_iter());

    loop {}
}

