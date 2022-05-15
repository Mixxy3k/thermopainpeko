#![no_std]
#![no_main]

// Importy
use core::f32;
use cortex_m::{interrupt::Mutex};
use core::{cell::RefCell, fmt::Write};
use embedded_hal::blocking::{delay::*};
use stm32f3xx_hal::{interrupt};
use panic_semihosting as _;
use cortex_m_rt::entry;
use dht11::{Dht11, Measurement};
use init::*;
use leds::*;
use lcd::*;
use usart_1::*;

// Zmienne dostępne w całym programie
static SERIAL: Mutex<RefCell<Option<SerialPort>>> = Mutex::new(RefCell::new(None));
static TB: Mutex<RefCell<Option<TemperatureBoundaries>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    // Wywołanie funkcji konfigurującej mikrokontroler
    let (usart, 
        led_array,
        mut i2c,
        mut delay,
        dht_pin) = init();
	
    // Utworzenie komponentów
	let mut dht = Dht11::new(dht_pin);
    let mut serial = SerialPort::new(usart);
	let mut leds = LedArray::new(led_array);
    let mut lcd = Lcd::new(&mut i2c)
        .address(0x3f)
        .cursor_on(false) 
        .rows(2)
        .init(&mut delay).unwrap();
    
	serial.enable_interrupt();
    cortex_m::interrupt::free(|cs| {
        SERIAL.borrow(cs).replace(Some(serial));
        TB.borrow(cs).replace(Some(TemperatureBoundaries::new(0.0, 25.0, 30.0, 35.0)));
    }); 
	
	lcd.clear(&mut delay).ok();
    loop {
        // Wykonanie pomiaru
		let measurement = dht.perform_measurement(&mut delay)
        .unwrap_or_else(|_| {
            Measurement { temperature: 2555, humidity: 2555 }
        });

        // Konwersja wartości z czujnika na dane
		let temp_f32 = (measurement.temperature as f32) / 10.0;
		let hum_f32 = (measurement.humidity as f32) / 10.0;

        // Jeśli pomiar jest prawidłowy na LCD pojawiają się wyniki z pomiaru
        // Jeśli pomiar jest nieprawidłowy na LCD pojawiają się informacje o błędzie
        if temp_f32 != 255.5 {
            lcd.send_temp(&mut delay, temp_f32, hum_f32);
        } else {
            lcd.clear(&mut delay).ok();
            lcd.write_str(&mut delay, "Connect DHT11!").ok();
        }

        // Wysłanie wartości z pomiaru do komputera oraz zapalenie diod LED
		cortex_m::interrupt::free(|cs| {
			if let Some(ref mut serial) = SERIAL.borrow(cs).borrow_mut().as_mut() {
                serial.enable_interrupt();
				uprintln!(serial, "Temp: {}, Hum: {}", temp_f32, hum_f32);
			}
            if let Some(ref mut tb) = TB.borrow(cs).borrow_mut().as_mut() {
                leds.set_from_tb(&mut delay, tb, temp_f32).ok();
            }
		}); 

        // https://www.mouser.com/datasheet/2/758/DHT11-Technical-Data-Sheet-Translated-Version-1143054.pdf
        // Strona 8: Note: Sampling period at intervals should be no less than 1 second.
        // Ponieważ polecenia powyżej trwają ~250ms mikrokontroler zostaje uspany na 800ms,
		delay.delay_ms(800u16);
    }
}

#[interrupt]
fn USART1_EXTI25() {
    cortex_m::interrupt::free(|cs| {
        // Jesli istnieje komponent SerialPort nalezy go odblokować
        if let Some(ref mut serial) = SERIAL.borrow(cs).borrow_mut().as_mut() {
            // Jeśli są dane do odczytu
            if serial.ReciveDataRegisterNotEmpty() {
                // Reset flagi informującej o odczycie
                serial.resetReciveDataRegisterNotEmpty();
                // Odczytanie kodu polecenia
                let command = serial.read(); 

                // Jeśli polecenie 's' to program czeka na wpisanie nowych wartości granic temperatury
                if command == 's' {
                    let val = serial.block_readln();
                    uprintln!(serial, "Got {}: {}", command, val);
                    let val: &str = &val[..];
                    if let Some(ref mut tb) = TB.borrow(cs).borrow_mut().as_mut() {
                        tb.set_from_string(val).ok();
                        uprintln!(serial, "Temp values changed: Low: {}, Optimal: {},  High: {}, Critical: {}", tb.cold, tb.optimal, tb.hot, tb.critical);
                    }
                }

                // Jeśli polecenie 'g' to program wysyła wartości granic temperatury
                else if command == 'g' {
                    if let Some(ref mut tb) = TB.borrow(cs).borrow_mut().as_mut() {
                        uprintln!(serial, "Cold: {}, Optimal: {},  High: {}, Critical: {}", tb.cold, tb.optimal, tb.hot, tb.critical);
                    }
                }
                // Transmisja zakończona
                serial.setTransmitionComplete();
            }
            // Jeśli transmisja jest zakończona to flagi odpowiedzialne za przerwanie są czyszczone
            if serial.TransmitionComplete() {
                serial.clear_interrupt();
            }
            return;
        }
    });
    return;
}