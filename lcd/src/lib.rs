#![crate_type = "dylib"]
#![no_std]

use cortex_m::prelude::_embedded_hal_blocking_delay_DelayMs;
use embedded_hal::blocking::i2c::Write;
use stm32f3xx_hal::delay::Delay;
use write_to::write_to;

pub struct Lcd<'a, I>
where
    I: Write
{
    i2c: &'a mut I,
    address: u8,
    rows: u8,
    backlight_state: Backlight,
    cursor_on: bool,
    cursor_blink: bool,
}

pub enum DisplayControl
{
    Off = 0x00,
    CursorBlink = 0x01,
    CursosOn = 0x02,
    DisplayOn = 0x04,
}

#[derive(Copy, Clone)]
pub enum Backlight {
    Off = 0x00,
    On = 0x08,
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Mode {
    Cmd = 0x00,
    Data = 0x01,
    DisplayControl = 0x08,
    FunctionSet = 0x20,
}

enum Commands {
    Clear = 0x01,
    ReturnHome = 0x02,
    ShiftCursor = 16 | 4
}

enum BitMode {
    Bit4 = 0x0 << 4,
    Bit8 = 0x1 << 4,
}

impl<'a, I> Lcd<'a, I>
where
    I: Write
{
    pub fn new(i2c: &'a mut I) -> Self {
        Self {
            i2c,
            backlight_state: Backlight::On,
            address: 0,
            rows: 0,
            cursor_blink: false,
            cursor_on: false,
        }
    }

    pub fn send_temp(&mut self, delay: &mut Delay, temp: f32, hum: f32)  {
        let mut buf = [0u8; 16];
        let temp: &str = write_to::show(
            &mut buf,
            format_args!("Temp: {}C  ", temp),
        ).unwrap();
        let mut buf2 = [0u8; 16];
        let humidity: &str = write_to::show(
            &mut buf2,
            format_args!("Humidity: {}%", hum),
        ).unwrap();
        self.return_home(delay).ok();
        self.write_str(delay, temp).ok();
        self.write_str(delay, "   ").ok(); //clear the rest of the line
        self.set_cursor(delay, 1, 0).ok();
        self.write_str(delay, humidity).ok();
        self.set_cursor(delay, 0, 0).ok();
    }

    pub fn rows(mut self, rows: u8) -> Self {
        self.rows = rows;
        self
    }

    pub fn address(mut self, address: u8) -> Self {
        self.address = address;
        self
    }

    pub fn cursor_on(mut self, on: bool) -> Self {
        self.cursor_on = on;
        self
    }

    pub fn write4bits(&mut self, delay: &mut Delay, data: u8) -> Result<(), <I as Write>::Error> {
        self.i2c.write(
            self.address,
            &[data | DisplayControl::DisplayOn as u8 | self.backlight_state as u8],
        )?;
        delay.delay_ms(1_u8);
        self.i2c.write(
            self.address,
            &[DisplayControl::Off as u8 | self.backlight_state as u8],
        )?;
        delay.delay_ms(1_u8);
        Ok(())
    }

    pub fn init(mut self, delay: &mut Delay) -> Result<Self, <I as Write>::Error>{
        delay.delay_ms(80_u8);

        // Init with 8 bit mode
        let mode_8bit = Mode::FunctionSet as u8 | BitMode::Bit8 as u8;
        self.write4bits(delay, mode_8bit)?;
        delay.delay_ms(1_u8);
        self.write4bits(delay, mode_8bit)?;
        delay.delay_ms(1_u8);
        self.write4bits(delay, mode_8bit)?;
        delay.delay_ms(1_u8);

        // Switch to 4 bit mode
        let mode_4bit = Mode::FunctionSet as u8 | BitMode::Bit4 as u8;
        self.write4bits(delay, mode_4bit)?;

        // Function set command
        let lines = if self.rows == 0 { 0x00 } else { 0x08 };
        self.command(delay,
            Mode::FunctionSet as u8 |
            // 5x8 display: 0x00, 5x10: 0x4
            lines, // Two line display
        )?;

        let display_ctrl = if self.cursor_on {
            DisplayControl::DisplayOn as u8 | DisplayControl::CursosOn as u8
        } else {
            DisplayControl::DisplayOn as u8
        };
        let display_ctrl = if self.cursor_blink {
            display_ctrl | DisplayControl::CursorBlink as u8
        } else {
            display_ctrl
        };
        self.command(delay, Mode::DisplayControl as u8 | display_ctrl)?;
        self.command(delay, Mode::Cmd as u8 | Commands::Clear as u8)?; // Clear Display

        // Entry right: shifting cursor moves to right
        self.command(delay, 0x04)?;
        self.backlight(self.backlight_state)?;
        Ok(self)
    }
    fn send(&mut self, delay: &mut Delay, data: u8, mode: Mode) -> Result<(), <I as Write>::Error> {
        let high_bits: u8 = data & 0xf0;
        let low_bits: u8 = (data << 4) & 0xf0;
        self.write4bits(delay, high_bits | mode as u8)?;
        self.write4bits(delay, low_bits | mode as u8)?;
        Ok(())
    }

    fn command(&mut self, delay: &mut Delay, data: u8) -> Result<(), <I as Write>::Error> {
        self.send(delay, data, Mode::Cmd)
    }

    pub fn backlight(&mut self, backlight: Backlight) -> Result<(), <I as Write>::Error> {
        self.backlight_state = backlight;
        self.i2c.write(
            self.address,
            &[DisplayControl::DisplayOn as u8 | backlight as u8],
        )
    }

    /// Write string to display.
    pub fn write_str(&mut self, delay: &mut Delay, data: &str) -> Result<(), <I as Write>::Error> {
        for c in data.chars() {
            self.send(delay, c as u8, Mode::Data)?;
        }
        Ok(())
    }

    pub fn write_f32(&mut self, delay: &mut Delay, data: f32) -> Result<(), <I as Write>::Error> {
        let mut data = data;
        let mut digits = 0;
        while data > 0.0 {
            data /= 10.0;
            digits += 1;
        }
        let mut i = 0;
        while i < digits {
            let digit = (data % 10.0) as u8 + 48;
            self.send(delay, digit, Mode::Data)?;
            data /= 10.0;
            i += 1;
        }
        Ok(())
    }

    /// Clear the display
    pub fn clear(&mut self, delay: &mut Delay) -> Result<(), <I as Write>::Error> {
        self.command(delay, Commands::Clear as u8)?;
        Ok(())
    }

    /// Return cursor to upper left corner, i.e. (0,0).
    pub fn return_home(&mut self, delay: &mut Delay) -> Result<(), <I as Write>::Error> {
        self.command(delay, Commands::ReturnHome as u8)?;
        Ok(())
    }

    /// Set the cursor to (rows, col). Coordinates are zero-based.
    pub fn set_cursor(&mut self, delay: &mut Delay, row: u8, col: u8) -> Result<(), <I as Write>::Error> {
        self.return_home(delay)?;
        let shift: u8 = row * 40 + col;
        for _i in 0..shift {
            self.command(delay, Commands::ShiftCursor as u8)?;
        }
        Ok(())
    }
}