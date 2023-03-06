#![no_std]

//! This crate provides a ST7306 driver to connect to TFT displays.

pub mod instruction;

use crate::instruction::Instruction;

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::OutputPin;

// TODO: Make this configurable
const ADDR_WINDOW: ((u16, u16), (u16, u16)) = ((0x12, 0x2A), (0x00, 0xC7));

/// ST7306 driver to connect to TFT displays.
pub struct ST7306<SPI, DC, CS, RST, const COLS: usize, const ROWS: usize>
where
    SPI: spi::Write<u8>,
    DC: OutputPin,
    CS: OutputPin,
    RST: OutputPin,
{
    /// SPI
    pub spi: SPI,

    /// Data/command pin.
    pub dc: DC,

    /// Chip select pin
    pub cs: CS,

    /// Reset pin.
    pub rst: RST,

    /// Whether the colours are inverted (true) or not (false)
    inverted: bool,

    framebuffer: [[[u8; 3]; COLS]; ROWS],

    /// Global image offset
    dx: u16,
    dy: u16,
    width: u32,
    height: u32,
}

#[derive(Clone, Copy)]
pub enum Orientation {
    Portrait = 0x00,
    Landscape = 0x60,
    PortraitSwapped = 0xC0,
    LandscapeSwapped = 0xA0,
}

impl<SPI, DC, CS, RST, const COLS: usize, const ROWS: usize> ST7306<SPI, DC, CS, RST, COLS, ROWS>
where
    SPI: spi::Write<u8>,
    DC: OutputPin,
    CS: OutputPin,
    RST: OutputPin,
{
    /// Creates a new driver instance that uses hardware SPI.
    pub fn new(
        spi: SPI,
        dc: DC,
        cs: CS,
        rst: RST,
        inverted: bool,
        width: u32,
        height: u32,
    ) -> Self {
        let display = ST7306 {
            spi,
            dc,
            cs,
            rst,
            inverted,
            framebuffer: [[[0; 3]; COLS]; ROWS],
            dx: 0,
            dy: 0,
            width,
            height,
        };

        display
    }

    pub fn draw_pixels<I>(&mut self, pixels: I, flush: bool) -> Result<(), ()>
    where
        I: IntoIterator<Item = Pixel<Rgb565>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            // Only draw pixels that would be on screen
            if coord.x >= 0
                && coord.y >= 0
                && coord.x < self.width as i32
                && coord.y < self.height as i32
            {
                self.set_pixel(
                    coord.x as u16,
                    coord.y as u16,
                    RawU16::from(color).into_inner() as u8,
                )?;
            }
        }
        if flush {
            self.flush()?;
        }
        Ok(())
    }

    // TODO: Support partial screen updates
    //       Need to keep track of which cols and rows have changed.
    pub fn flush(&mut self) -> Result<(), ()> {
        let caset = (18, (42));
        let raset = (0, (199));
        self.write_command(Instruction::CASET, &[caset.0, caset.1])?;
        self.write_command(Instruction::RASET, &[raset.0, raset.1])?;

        self.write_command(Instruction::RAMWR, &[])?;
        self.start_data()?;

        for row in 0..ROWS {
            for col in 0..COLS {
                self.write_byte(self.framebuffer[row][col][0])?;
                self.write_byte(self.framebuffer[row][col][1])?;
                self.write_byte(self.framebuffer[row][col][2])?;
            }
        }
        Ok(())
    }

    //pub fn fill_contiguous_single_color(
    //    &mut self,
    //    area: &Rectangle,
    //    color: Rgb565,
    //) -> Result<(), ()> {
    //    // Clamp area to drawable part of the display target
    //    let drawable_area = area.intersection(&Rectangle::new(Point::zero(), self.size()));
    //    let brightness = col_to_bright(color);
    //    let colors =
    //        core::iter::repeat(brightness).take((area.size.width * area.size.height) as usize);
    //    //let colors = area.points()
    //    //            .filter(|pos| drawable_area.contains(*pos))
    //    //            .map(|_pos| brightness);

    //    if drawable_area.size != Size::zero() {
    //        let ex = (drawable_area.top_left.x + (drawable_area.size.width - 1) as i32) as u16;
    //        let ey = (drawable_area.top_left.y + (drawable_area.size.height - 1) as i32) as u16;
    //        self.set_pixels_buffered_u8(
    //            drawable_area.top_left.x as u16,
    //            drawable_area.top_left.y as u16,
    //            ex,
    //            ey,
    //            colors,
    //        )?;
    //    }

    //    Ok(())
    //}

    //pub fn read_did<DELAY>(&mut self, delay: &mut DELAY) -> Result<u8, ()>
    //where
    //    DELAY: DelayMs<u8>,
    //{
    //    self.dc.set_low().map_err(|_| ())?;
    //    self.spi.write(command as u8).map_err(|_| ())?;
    //    let res = self.spi.read().map_err(|_| ())?;
    //    Ok(res)
    //}

    /// Runs commands to initialize the display.
    pub fn init<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), ()>
    where
        DELAY: DelayMs<u8>,
    {
        // First do a hard reset because the controller might be in a bad state
        // if the voltage was unstable in the beginning.
        self.hard_reset(delay)?;
        self.write_command(Instruction::SWRESET, &[])?;
        delay.delay_ms(200);

        self.write_command(Instruction::NVMLOADCTRL, &[0x17, 0x02])?;
        self.write_command(Instruction::BSTEN, &[0x01])?;

        // Gate Voltage Control. VGH: 12V, VGL: -6V
        self.write_command(Instruction::GCTRL, &[0x08, 0x02])?;
        // VSHP Control: 4.02V
        self.write_command(Instruction::VSHPCTRL, &[0x0B, 0x0B, 0x0B, 0x0B])?;
        // VSLP Control: 0.8V
        self.write_command(Instruction::VSLPCTRL, &[0x23, 0x23, 0x23, 0x23])?;
        // VSHN Control: -3.28V
        self.write_command(Instruction::VSHNCTRL, &[0x27, 0x27, 0x27, 0x27])?;
        // VSLN Control: -0.06V
        self.write_command(Instruction::VSLNCTRL, &[0x35, 0x35, 0x35, 0x35])?;

        // Datasheet: 0x32, 0x03, 0x1F Reference code: not present
        //self.write_command(Instruction::GTCON, &[0x32, 0x03, 0x1F])?;

        // Datasheet: 0x26, 0xE9, Reference: 0xA6, 0xE9 (HPM: 32Hz)
        self.write_command(Instruction::OSCSET, &[0xA6, 0xE9])?;
        // Frame Rate Control: 32Hz in High Power Mode, 1Hz in Low Power Mode
        self.write_command(Instruction::FRCTRL, &[0x12])?;

        // HPM EQ Control
        self.write_command(
            Instruction::GTUPEQH,
            &[0xE5, 0xF6, 0x05, 0x46, 0x77, 0x77, 0x77, 0x77, 0x76, 0x45],
        )?;
        // LPM EQ Control
        self.write_command(
            Instruction::GTUPEQL,
            &[0x05, 0x46, 0x77, 0x77, 0x77, 0x77, 0x76, 0x45],
        )?;
        // Source EQ Enable
        self.write_command(Instruction::SOUEQ, &[0x13])?;

        // Gate Line Setting:
        // 0x64 (100) lines. Each line controls 2 pixels. 100*2 = 400px
        self.write_command(Instruction::GATESET, &[0x64])?;

        // Exit sleep mode
        self.write_command(Instruction::SLPOUT, &[])?;
        delay.delay_ms(255);

        // Ultra low power code (undocumented command)
        self.write_command(Instruction::LOWPOWER, &[0xC1, 0x4A, 0x26])?;

        // Source Voltage Select: VSHP1, VSLP1, VSHN1, VSLN1
        self.write_command(Instruction::VSHLSEL, &[0x00])?;

        // Memory Data Access Control. Default, nothing inverted
        //                 0      = MY (Page Address Order) Flips picture upside down
        //                  1     = MX (Column Address Order)
        //                   0    = MV (Page/Column Order)
        //                     1  = DO (Data Order)
        //                      0 = GS (Gate Scan Order)
        //                 010010
        // Make sure pixel 0,0 is in the top left
        let madctl: u8 = 0b01001000;
        self.write_command(Instruction::MADCTL, &[madctl])?;

        // Data Format: XDE=1, BPS=1 (3 bytes for 24 bits)
        self.write_command(Instruction::DTFORM, &[0x11])?;

        // Gamma Mode: Mono
        self.write_command(Instruction::GAMAMS, &[0x20])?;

        // Panel Setting
        //  01      = 1-Dot Inversion
        //  || 10   = Frame Interval
        //  || ||01 = One-Line Interface
        //  || ||||
        // 00101001 = 0x29
        self.write_command(Instruction::PNLSET, &[0x29])?;

        // Column and row settings.
        // Will be overridden by each pixel write
        // Columns 18-42 (S217-S516). 25 columns, one for 12 pixels => 300px
        self.write_command(Instruction::CASET, &[0x12, 0x2A])?;
        // Rows 0-199 (G1-G402). 200 rows, one for 2 pixels => 400px
        self.write_command(Instruction::RASET, &[0x00, 0xC7])?;

        // Enable auto power down
        self.write_command(Instruction::AUTOPWRCTRL, &[0xFF])?;

        // Tearing enable on
        self.write_command(Instruction::TEON, &[])?;

        // Go into low power mode
        self.write_command(Instruction::LPM, &[])?;

        // Invert screen colors
        if self.inverted {
            self.write_command(Instruction::INVON, &[])?;
        } else {
            self.write_command(Instruction::INVOFF, &[])?;
        }

        self.write_command(Instruction::DISPON, &[])?;

        Ok(())
    }

    pub fn on_off(&mut self, on: bool) -> Result<(), ()> {
        if on {
            self.write_command(Instruction::DISPON, &[])?;
        } else {
            self.write_command(Instruction::DISPOFF, &[])?;
        }
        Ok(())
    }

    pub fn sleep_in<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), ()>
    where
        DELAY: DelayMs<u8>,
    {
        // TODO: Detect if HPM or LPM. Because if we're in LPM, we first need to go into HPM
        if true {
            //mode == HPM {
            self.write_command(Instruction::SLPIN, &[])?;
            delay.delay_ms(100);
        } else {
            self.write_command(Instruction::HPM, &[])?;
            delay.delay_ms(200);
            self.sleep_in(delay)?;
        }
        Ok(())
    }

    pub fn sleep_out<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), ()>
    where
        DELAY: DelayMs<u8>,
    {
        self.write_command(Instruction::SLPOUT, &[])?;
        delay.delay_ms(100);
        Ok(())
    }

    pub fn switch_mode<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), ()>
    where
        DELAY: DelayMs<u8>,
    {
        panic!("TODO: Not implemented");
        self.write_command(Instruction::HPM, &[])?;
        self.write_command(Instruction::LPM, &[])?;
        delay.delay_ms(100);
        Ok(())
    }

    pub fn invert_screen(&mut self, inverted: bool) -> Result<(), ()> {
        if inverted {
            self.write_command(Instruction::INVON, &[])
        } else {
            self.write_command(Instruction::INVOFF, &[])
        }
    }

    pub fn hard_reset<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), ()>
    where
        DELAY: DelayMs<u8>,
    {
        self.rst.set_high().map_err(|_| ())?;
        delay.delay_ms(10);

        self.rst.set_low().map_err(|_| ())?;
        delay.delay_ms(10);

        self.rst.set_high().map_err(|_| ())
    }

    pub fn write_command(&mut self, command: Instruction, params: &[u8]) -> Result<(), ()> {
        self.cs.set_low().map_err(|_| ())?;
        self.dc.set_low().map_err(|_| ())?;
        self.spi.write(&[command as u8]).map_err(|_| ())?;
        if !params.is_empty() {
            self.start_data()?;
            self.write_data(params)?;
        }
        self.cs.set_high().map_err(|_| ())?;
        Ok(())
    }

    pub fn start_data(&mut self) -> Result<(), ()> {
        self.cs.set_low().map_err(|_| ())?;
        self.dc.set_high().map_err(|_| ())
    }

    pub fn write_data(&mut self, data: &[u8]) -> Result<(), ()> {
        data.iter().for_each(|d| {
            self.spi.write(&[*d as u8]);
        });
        Ok(())
    }

    pub fn write_byte(&mut self, value: u8) -> Result<(), ()> {
        self.write_data(&[value])
    }

    pub fn set_orientation(&mut self, orientation: &Orientation) -> Result<(), ()> {
        panic!("TODO: Not yet implemented");
        self.write_command(Instruction::MADCTL, &[*orientation as u8])?;
        Ok(())
    }

    /// Sets the global offset of the displayed image
    pub fn set_offset(&mut self, dx: u16, dy: u16) {
        self.dx = dx;
        self.dy = dy;
    }

    /// Sets the address window for the display.
    pub fn set_address_window(&mut self, sx: u16, sy: u16, ex: u16, ey: u16) -> Result<(), ()> {
        let ((x_lower, x_upper), (y_lower, y_upper)) = ADDR_WINDOW;
        // TODO: Check
        let x_lower = x_lower + (sx + self.dx) / 12;
        let x_upper = x_upper + (ex + self.dx) / 12;
        let y_lower = y_lower + (sy + self.dy) / 2;
        let y_upper = y_upper + (ey + self.dy) / 2;
        self.write_command(Instruction::CASET, &[x_lower as u8, x_upper as u8])?;
        self.write_command(Instruction::RASET, &[y_lower as u8, y_upper as u8])?;
        Ok(())
    }

    /// Sets a pixel color at the given coords.
    pub fn set_pixel(&mut self, x: u16, y: u16, color: u8) -> Result<(), ()> {
        let row: usize = (y as usize) / 2;
        let col: usize = (x as usize) / 12;
        let black = color < 1;

        let (byte, bitmask) = match (x % 12, y % 2) {
            (0, 0) => (0, 0x80),
            (0, 1) => (0, 0x40),
            (1, 0) => (0, 0x20),
            (1, 1) => (0, 0x10),
            (2, 0) => (0, 0x08),
            (2, 1) => (0, 0x04),
            (3, 0) => (0, 0x02),
            (3, 1) => (0, 0x01),

            (4, 0) => (1, 0x80),
            (4, 1) => (1, 0x40),
            (5, 0) => (1, 0x20),
            (5, 1) => (1, 0x10),
            (6, 0) => (1, 0x08),
            (6, 1) => (1, 0x04),
            (7, 0) => (1, 0x02),
            (7, 1) => (1, 0x01),

            (8, 0) => (2, 0x80),
            (8, 1) => (2, 0x40),
            (9, 0) => (2, 0x20),
            (9, 1) => (2, 0x10),
            (10, 0) => (2, 0x08),
            (10, 1) => (2, 0x04),
            (11, 0) => (2, 0x02),
            (11, 1) => (2, 0x01),
            _ => panic!("Impossible to reach"),
        };

        if black {
            self.framebuffer[row][col][byte] |= bitmask
        } else {
            self.framebuffer[row][col][byte] &= !bitmask;
        }
        Ok(())
    }

    /// Writes pixel colors sequentially into the current drawing window
    pub fn write_pixels<P: IntoIterator<Item = u8>>(&mut self, colors: P) -> Result<(), ()> {
        // TODO: Check if same
        // Only works if writing all pixels
        self.write_command(Instruction::RAMWR, &[])?;
        self.start_data()?;
        for color in colors {
            self.write_byte(color)?;
        }
        Ok(())
    }
}

#[cfg(feature = "graphics")]
extern crate embedded_graphics;
#[cfg(feature = "graphics")]
use self::embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::{
        raw::{RawData, RawU16},
        Gray2, Rgb565,
    },
    prelude::*,
    primitives::Rectangle,
};

fn col_to_bright(color: Rgb565) -> u8 {
    let brightness = ((color.r() as u16) + (color.g() as u16) + (color.b() as u16) / 3) as u8;
    brightness
}

#[cfg(feature = "graphics")]
// TODO: Remove color support from here
impl<SPI, DC, CS, RST, const COLS: usize, const ROWS: usize> DrawTarget
    for ST7306<SPI, DC, CS, RST, COLS, ROWS>
where
    SPI: spi::Write<u8>,
    DC: OutputPin,
    CS: OutputPin,
    RST: OutputPin,
{
    type Error = ();
    type Color = Rgb565;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.draw_pixels(pixels, true)
    }

    //fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    //where
    //    I: IntoIterator<Item = Self::Color>,
    //{
    //    // Clamp area to drawable part of the display target
    //    let drawable_area = area.intersection(&Rectangle::new(Point::zero(), self.size()));
    //    let colors = area
    //        .points()
    //        .zip(colors)
    //        .filter(|(pos, _color)| drawable_area.contains(*pos))
    //        .map(|(_pos, color)| col_to_bright(color));
    //    //let colors =
    //    //        area.points()
    //    //            .zip(colors)
    //    //            .filter(|(pos, _color)| drawable_area.contains(*pos))
    //    //            .map(|(_pos, color)| RawU16::from(color).into_inner());

    //    if drawable_area.size != Size::zero() {
    //        let ex = (drawable_area.top_left.x + (drawable_area.size.width - 1) as i32) as u16;
    //        let ey = (drawable_area.top_left.y + (drawable_area.size.height - 1) as i32) as u16;
    //        self.set_pixels_buffered_u8(
    //            drawable_area.top_left.x as u16,
    //            drawable_area.top_left.y as u16,
    //            ex,
    //            ey,
    //            colors,
    //        )?;
    //    }

    //    Ok(())
    //}

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        let brightness = ((color.r() as u16) + (color.g() as u16) + (color.b() as u16) / 3) as u8;
        let black = if brightness < 128 { 0xFF } else { 0x00 };

        for col in 0..COLS {
            for row in 0..ROWS {
                self.framebuffer[row][col][0] = black;
                self.framebuffer[row][col][1] = black;
                self.framebuffer[row][col][2] = black;
            }
        }
        self.flush()
    }
}

#[cfg(feature = "graphics")]
impl<SPI, DC, CS, RST, const COLS: usize, const ROWS: usize> OriginDimensions
    for ST7306<SPI, DC, CS, RST, COLS, ROWS>
where
    SPI: spi::Write<u8>,
    DC: OutputPin,
    CS: OutputPin,
    RST: OutputPin,
{
    fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
}
