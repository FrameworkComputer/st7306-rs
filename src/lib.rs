#![no_std]

//! This crate provides a ST7306 driver to connect to TFT displays.

pub mod instruction;

use crate::instruction::Instruction;

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::OutputPin;

/// ST7735 driver to connect to TFT displays.
pub struct ST7735<SPI, DC, RST>
where
    SPI: spi::Write<u8>,
    DC: OutputPin,
    RST: OutputPin,
{
    /// SPI
    spi: SPI,

    /// Data/command pin.
    dc: DC,

    /// Reset pin.
    rst: RST,

    /// Whether the colours are inverted (true) or not (false)
    inverted: bool,

    /// Global image offset
    dx: u16,
    dy: u16,
    width: u32,
    height: u32,
}

/// Display orientation.
/// Bits:
/// 0: 0
/// 1: 0
/// 2: Gate Scan Order, 0==Refresh Top to Bottom
/// 3: Data Output Order, 0==Left To Right
/// 4: 0
/// 5: Page/Column Order, 0==Column Direction, 1==Page Direction
/// 6: Column Address Order, 0=Left to Right
/// 7: Page Address Order, 0==Top to Bottom
/// bin(0x00) = '0b00000000' Portrait
/// bin(0xC0) = '0b11000000' Portrait Swapped
///
/// bin(0x60) = '0b01100000' Landscape
/// bin(0xA0) = '0b10100000' Landscape Swapped

#[derive(Clone, Copy)]
pub enum Orientation {
    Portrait = 0x00,
    Landscape = 0x60,
    PortraitSwapped = 0xC0,
    LandscapeSwapped = 0xA0,
}

impl<SPI, DC, RST> ST7735<SPI, DC, RST>
where
    SPI: spi::Write<u8>,
    DC: OutputPin,
    RST: OutputPin,
{
    /// Creates a new driver instance that uses hardware SPI.
    pub fn new(
        spi: SPI,
        dc: DC,
        rst: RST,
        inverted: bool,
        width: u32,
        height: u32,
    ) -> Self {
        let display = ST7735 {
            spi,
            dc,
            rst,
            inverted,
            dx: 0,
            dy: 0,
            width,
            height,
        };

        display
    }
    /// Runs commands to initialize the display.
    pub fn init_st7306<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), ()>
    where
        DELAY: DelayMs<u8>,
    {
        self.hard_reset(delay)?;
        self.write_command(Instruction::SWRESET, &[])?;
        delay.delay_ms(200);
        self.write_command(Instruction::SLPOUT, &[])?;
        delay.delay_ms(200);

        self.write_command(Instruction::NVMLOADCTRL, &[0x17, 0x02])?;
        self.write_command(Instruction::BSTEN, &[0x01])?;
        self.write_command(Instruction::GCTRL, &[0x0E, 0x0A])?;
        self.write_command(Instruction::VSHPCTRL, &[0x41, 0x41, 0x41, 0x41])?;
        self.write_command(Instruction::VSLPCTRL, &[0x32, 0x32, 0x32, 0x32,])?;
        self.write_command(Instruction::VSHNCTRL, &[0x46, 0x46, 0x46, 0x46])?;
        self.write_command(Instruction::VSLNCTRL, &[0x46, 0x46, 0x46, 0x46])?;
        self.write_command(Instruction::FRCTRL, &[0x02])?;
        self.write_command(Instruction::GTUPEQH, &[0xE5, 0xF6, 0x05, 0x46, 0x77, 0x77, 0x77, 0x77, 0x76, 0x45])?;
        self.write_command(Instruction::GTUPEQL, &[0x05, 0x46, 0x77, 0x77, 0x77, 0x77, 0x76, 0x45])?;
        self.write_command(Instruction::GTCON, &[0x32, 0x03, 0x1F])?;
        self.write_command(Instruction::SOUEQ, &[0x13])?;
        self.write_command(Instruction::GATESET, &[0x78])?;
        self.write_command(Instruction::SLPOUT, &[])?;
        delay.delay_ms(120);
        self.write_command(Instruction::OSCSET, &[0xA6, 0xE9])?;
        self.write_command(Instruction::VSHLSEL, &[0x00])?;
        self.write_command(Instruction::MADCTL, &[0x48])?;
        self.write_command(Instruction::DTFORM, &[0x10])?;
        self.write_command(Instruction::GAMAMS, &[0x20])?;
        self.write_command(Instruction::PNLSET, &[0x00])?;
        self.write_command(Instruction::CASET, &[0x00, 0x3B])?;
        self.write_command(Instruction::RASET, &[0x00, 0xEF])?;
        //self.write_command(Instruction::TEON, &[0x00])?;
        self.write_command(Instruction::AUSOPWRCTRL, &[0xFF])?;
        self.write_command(Instruction::HPM, &[])?;

        //if self.inverted {
        //    self.write_command(Instruction::INVON, &[])?;
        //} else {
            self.write_command(Instruction::INVOFF, &[])?;
        //}

        self.write_command(Instruction::DISPON, &[])?;

        Ok(())
    }

    pub fn sleep_in<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), ()>
    where
        DELAY: DelayMs<u8>,
    {
        // TODO: Detect if HPM or LPM
        if true {//mode == HPM {
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
        // TODO: !!!
        self.write_command(Instruction::HPM, &[])?;
        self.write_command(Instruction::LPM, &[])?;
        delay.delay_ms(100);
        Ok(())
    }

    /// Runs commands to initialize the display.
    pub fn init<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), ()>
    where
        DELAY: DelayMs<u8>,
    {
        self.init_st7306(delay)?;
        //self.hard_reset(delay)?;
        //self.write_command(Instruction::SWRESET, &[])?;
        //delay.delay_ms(200);
        //self.write_command(Instruction::SLPOUT, &[])?;
        //delay.delay_ms(200);
        //self.write_command(Instruction::FRMCTR1, &[0x01, 0x2C, 0x2D])?;
        //self.write_command(Instruction::FRMCTR2, &[0x01, 0x2C, 0x2D])?;
        //self.write_command(Instruction::FRMCTR3, &[0x01, 0x2C, 0x2D, 0x01, 0x2C, 0x2D])?;
        //self.write_command(Instruction::INVCTR, &[0x07])?;
        //self.write_command(Instruction::PWCTR1, &[0xA2, 0x02, 0x84])?;
        //self.write_command(Instruction::PWCTR2, &[0xC5])?;
        //self.write_command(Instruction::PWCTR3, &[0x0A, 0x00])?;
        //self.write_command(Instruction::PWCTR4, &[0x8A, 0x2A])?;
        //self.write_command(Instruction::PWCTR5, &[0x8A, 0xEE])?;
        //self.write_command(Instruction::VMCTR1, &[0x0E])?;
        //if self.inverted {
        //    self.write_command(Instruction::INVON, &[])?;
        //} else {
        //    self.write_command(Instruction::INVOFF, &[])?;
        //}
        //if self.rgb {
        //    self.write_command(Instruction::MADCTL, &[0x00])?;
        //} else {
        //    self.write_command(Instruction::MADCTL, &[0x08])?;
        //}
        //self.write_command(Instruction::COLMOD, &[0x05])?;
        //self.write_command(Instruction::DISPON, &[])?;
        //delay.delay_ms(200);
        Ok(())
    }

    pub fn hard_reset<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), ()>
    where
        DELAY: DelayMs<u8>,
    {
        // TODO: Check if same
        self.rst.set_high().map_err(|_| ())?;
        delay.delay_ms(10);
        self.rst.set_low().map_err(|_| ())?;
        delay.delay_ms(10);
        self.rst.set_high().map_err(|_| ())
    }

    fn write_command(&mut self, command: Instruction, params: &[u8]) -> Result<(), ()> {
        // TODO: Check if same
        self.dc.set_low().map_err(|_| ())?;
        self.spi.write(&[command as u8]).map_err(|_| ())?;
        if !params.is_empty() {
            self.start_data()?;
            self.write_data(params)?;
        }
        Ok(())
    }

    fn start_data(&mut self) -> Result<(), ()> {
        self.dc.set_high().map_err(|_| ())
    }

    fn write_data(&mut self, data: &[u8]) -> Result<(), ()> {
        self.spi.write(data).map_err(|_| ())
    }

    /// Writes a data word to the display.
    fn write_word(&mut self, value: u16) -> Result<(), ()> {
        self.write_data(&value.to_be_bytes())
    }

    fn write_words_buffered(&mut self, words: impl IntoIterator<Item = u16>) -> Result<(), ()> {
        let mut buffer = [0; 32];
        let mut index = 0;
        for word in words {
            let as_bytes = word.to_be_bytes();
            buffer[index] = as_bytes[0];
            buffer[index + 1] = as_bytes[1];
            index += 2;
            if index >= buffer.len() {
                self.write_data(&buffer)?;
                index = 0;
            }
        }
        self.write_data(&buffer[0..index])
    }

    pub fn set_orientation(&mut self, orientation: &Orientation) -> Result<(), ()> {
        // TODO: Check if same
        self.write_command(Instruction::MADCTL, &[*orientation as u8 | 0x08])?;
        Ok(())
    }

    /// Sets the global offset of the displayed image
    pub fn set_offset(&mut self, dx: u16, dy: u16) {
        self.dx = dx;
        self.dy = dy;
    }

    /// Sets the address window for the display.
    pub fn set_address_window(&mut self, sx: u16, sy: u16, ex: u16, ey: u16) -> Result<(), ()> {
        // TODO: Check if same
        self.write_command(Instruction::CASET, &[])?;
        self.start_data()?;
        self.write_word(sx + self.dx)?;
        self.write_word(ex + self.dx)?;
        self.write_command(Instruction::RASET, &[])?;
        self.start_data()?;
        self.write_word(sy + self.dy)?;
        self.write_word(ey + self.dy)
    }

    /// Sets a pixel color at the given coords.
    pub fn set_pixel(&mut self, x: u16, y: u16, color: u16) -> Result<(), ()> {
        // TODO: Check if same
        self.set_address_window(x, y, x, y)?;
        self.write_command(Instruction::RAMWR, &[])?;
        self.start_data()?;
        self.write_word(color)
    }

    /// Writes pixel colors sequentially into the current drawing window
    pub fn write_pixels<P: IntoIterator<Item = u16>>(&mut self, colors: P) -> Result<(), ()> {
        // TODO: Check if same
        self.write_command(Instruction::RAMWR, &[])?;
        self.start_data()?;
        for color in colors {
            self.write_word(color)?;
        }
        Ok(())
    }
    pub fn write_pixels_buffered<P: IntoIterator<Item = u16>>(
        &mut self,
        colors: P,
    ) -> Result<(), ()> {
        self.write_command(Instruction::RAMWR, &[])?;
        self.start_data()?;
        self.write_words_buffered(colors)
    }

    /// Sets pixel colors at the given drawing window
    pub fn set_pixels<P: IntoIterator<Item = u16>>(
        &mut self,
        sx: u16,
        sy: u16,
        ex: u16,
        ey: u16,
        colors: P,
    ) -> Result<(), ()> {
        self.set_address_window(sx, sy, ex, ey)?;
        self.write_pixels(colors)
    }

    pub fn set_pixels_buffered<P: IntoIterator<Item = u16>>(
        &mut self,
        sx: u16,
        sy: u16,
        ex: u16,
        ey: u16,
        colors: P,
    ) -> Result<(), ()> {
        self.set_address_window(sx, sy, ex, ey)?;
        self.write_pixels_buffered(colors)
    }
}

#[cfg(feature = "graphics")]
extern crate embedded_graphics;
#[cfg(feature = "graphics")]
use self::embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::{
        raw::{RawData, RawU16},
        Rgb565,
    },
    prelude::*,
    primitives::Rectangle,
};

#[cfg(feature = "graphics")]
// TODO: Remove color support from here
impl<SPI, DC, RST> DrawTarget for ST7735<SPI, DC, RST>
where
    SPI: spi::Write<u8>,
    DC: OutputPin,
    RST: OutputPin,
{
    type Error = ();
    type Color = Rgb565;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
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
                    RawU16::from(color).into_inner(),
                )?;
            }
        }

        Ok(())
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        // Clamp area to drawable part of the display target
        let drawable_area = area.intersection(&Rectangle::new(Point::zero(), self.size()));

        if drawable_area.size != Size::zero() {
            self.set_pixels_buffered(
                drawable_area.top_left.x as u16,
                drawable_area.top_left.y as u16,
                (drawable_area.top_left.x + (drawable_area.size.width - 1) as i32) as u16,
                (drawable_area.top_left.y + (drawable_area.size.height - 1) as i32) as u16,
                area.points()
                    .zip(colors)
                    .filter(|(pos, _color)| drawable_area.contains(*pos))
                    .map(|(_pos, color)| RawU16::from(color).into_inner()),
            )?;
        }

        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.set_pixels_buffered(
            0,
            0,
            self.width as u16 - 1,
            self.height as u16 - 1,
            core::iter::repeat(RawU16::from(color).into_inner())
                .take((self.width * self.height) as usize),
        )
    }
}

#[cfg(feature = "graphics")]
impl<SPI, DC, RST> OriginDimensions for ST7735<SPI, DC, RST>
where
    SPI: spi::Write<u8>,
    DC: OutputPin,
    RST: OutputPin,
{
    fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
}
