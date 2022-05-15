#![crate_type = "dylib"]
#![no_std]

use cortex_m::prelude::_embedded_hal_blocking_delay_DelayMs;
use stm32f3xx_hal::delay::Delay;
use embedded_hal::digital::v2::{OutputPin};


/// Struct zawierający możlwe błędy
#[derive(Debug)]
pub enum Error<E> {
    ArrayOutOfBounds,
    Gpio(E),
}

/// Struct zawierający tablicę poziomów grnicznych temperatur
pub struct TemperatureBoundaries {
    pub cold: f32,
    pub optimal: f32,
    pub hot: f32,
    pub critical: f32,
}

impl TemperatureBoundaries {
    pub fn new(cold: f32, optimal: f32, hot: f32, critical: f32) -> Self {
        Self {
            cold,
            optimal,
            hot,
            critical,
        }
    }
    pub fn set_cold(&mut self, cold: f32) {
        self.cold = cold;
    }
    pub fn set_optimal(&mut self, optimal: f32) {
        self.optimal = optimal;
    }
    pub fn set_hot(&mut self, hot: f32) {
        self.hot = hot;
    }
    pub fn set_critical(&mut self, critical: f32) {
        self.critical = critical;
    }

    pub fn set_from_string(&mut self, string: &str) -> Result<(), Error<&str>> {
        let mut iter = string.split(',');
        let cold = iter.next();
        let optimal = iter.next();
        let hot = iter.next();
        let critical = iter.next();
        if cold.is_none() || optimal.is_none() || hot.is_none() || critical.is_none() {
            return Err(Error::ArrayOutOfBounds);
        }
        self.cold = cold.unwrap().parse::<f32>().map_err(|_| Error::Gpio("Błąd parsowania"))?;
        self.optimal = optimal.unwrap().parse::<f32>().map_err(|_| Error::Gpio("Błąd parsowania"))?;
        self.hot = hot.unwrap().parse::<f32>().map_err(|_| Error::Gpio("Błąd parsowania"))?;
        self.critical = critical.unwrap().parse::<f32>().map_err(|_| Error::Gpio("Błąd parsowania"))?;
        Ok(())
    }
}

/// Kompoment opakowujący tablice ledów i dodający do niej funkcjonalność
/// # Examples
/// ```
/// use leds::*
/// 
/// // Utworzenie kompomentu
/// let mut leds = LedArray::new(leds_array);
/// 
/// // Włączenie LED 0
/// leds.set(0, true).ok(); 
/// // Wyłączenie LED 0
/// leds.set(0, false).ok();
/// 
/// // Wyłączenie wszystkich LED
/// leds.set_all(false);
/// ```
/// Animacja kółka <br/>
/// <b> Uwaga: </b> Funkcja ta blokuje wykonywanie pozostałego kodu w tle!
/// ```
/// leds.circle_animation(delay);
/// ```
pub struct LedArray<GPIO> {
    pub leds: [GPIO; 8],
}

impl <GPIO, E> LedArray<GPIO>
where
    GPIO: OutputPin<Error = E>
{
    /// # Examples
    /// ```
    /// // Tworzenie kompomentu LedArray
    /// let mut leds = = LedArray::new(leds_array);
    /// ```
    pub fn new(leds: [GPIO; 8]) -> Self {
        LedArray {
            leds,
        }
    }

    /// Funkcja służąca do manipolacji pojedyńczą diodą LED
    /// # Przykład
    /// ```
    /// // Włączenie LED 0
    /// leds.set(0, true).ok(); 
    /// // Wyłączenie LED 0
    /// leds.set(0, false).ok();
    /// ```
    pub fn set(&mut self, led_index: u8, state: bool) -> Result<(), Error<E>> {
        if led_index > 7 {
            return Err(Error::ArrayOutOfBounds);
        }

        if state {
            self.leds[led_index as usize].set_high().map_err(Error::Gpio)?;
        } else {
            self.leds[led_index as usize].set_low().map_err(Error::Gpio)?;
        }
        Ok(())
    }

    /// Funkcja służąca do manipulacji wszystkimi diodami LED
    /// # Examples
    /// ```
    /// // Włączenie wszyrkich LED
    /// leds.set_all(true).ok(); 
    /// // Wyłączenie wszystkich LED
    /// leds.set_all(false).ok();
    /// ```
    pub fn set_all(&mut self, state: bool) -> Result<(), E> {
        if state {
            for i in 0..8 {
                self.leds[i].set_high()?;
            }
        } else {
            for i in 0..8 {
                self.leds[i].set_low()?;
            }
        }
        Ok(())
    }

    /// Funkcja służąca do utworzenia animacji kręcącego kółka <br/>
    /// <b> Uwaga: </b> Funkcja ta blokuje wykonywanie pozostałego kodu w tle!
    /// # Examples
    /// ```
    /// leds.circle_animation(delay);
    /// ```
    pub fn circle_animation(&mut self, delay: &mut Delay) -> Result<(), E> 
    {
        for i in 0..8 {
            self.leds[i].set_high()?;
            delay.delay_ms(25_u16);
        }
        //delay.delay_ms(50_u16);
        for i in 0..8 {
            self.leds[i].set_low()?;
            delay.delay_ms(25_u16);
        }
        Ok(())
    }

    ///Funkcja służąca do usatwienia LEDów zgodnie z wartością temperatury
    pub fn set_from_tb(&mut self, delay: &mut Delay, tb: &TemperatureBoundaries, temperature: f32) -> Result<(), E> {
        self.set_all(false)?;
        if temperature >= tb.critical { // Critical: Led spining
            self.circle_animation(delay)?;
        }
        else if temperature >= tb.hot { // High: Red led on
            self.set(4, true).ok();
        }
        else if temperature >= tb.optimal { // Normal: Green led on
            self.set(2, true).ok();
        }
        else if temperature < tb.cold { // Low: Blue led on
            self.set(3, true).ok();
        }
        Ok(())
    }
}