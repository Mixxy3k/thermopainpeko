#![crate_type = "dylib"]
#![no_std]
#![allow(non_snake_case)]

pub use stm32f3xx_hal::pac::usart1;
use heapless::String;
use core;

/// Komponent opakowujący interfejs USART i dodający do niego funkcjonalność
/// # Examples
/// ```
/// use core::fmt::Write;
/// use usart_1::*;
/// use init::*;
/// 
/// // Pobranie wskaźnika do USART
/// let (usart, _) = init();
/// // Utworzenie kompomentu
/// let mut serial = SerialPort::new(usart);
/// 
/// // Wysłanie wiadomości "2+2=4"
/// uprintln!(serial, "2+2 = {}", 2+2);
/// ```
pub struct SerialPort {
    pub usart1: &'static mut usart1::RegisterBlock,
}

/// Implementacja interfejsu Write dla komponentu SerialPort
impl core::fmt::Write for SerialPort {
    // nadpisanie funkcji write_str
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        // pętla po wszystkich znakach w stringu
        for c in s.bytes() {
            // oczekiwanie na moment kidy rejestr TXE zostanie wyłączony
            while self.usart1.isr.read().txe().bit_is_clear() {}
            // wysłanie znaku
            self.usart1.tdr.write(|w| w.tdr().bits(c as u16));
        }
        Ok(())
    }
}


impl SerialPort {
    /// # Examples
    /// ```
    /// use usart_1::*;
    /// use init::*;
    /// 
    /// Pobieranie wskaźnika do USART
    /// let (usart, _) = init();
    /// // Utworzenie kompomentu
    /// let mut serial = SerialPort::new(usart);
    /// ```
    pub fn new(usart1: &'static mut usart1::RegisterBlock) -> Self {
        SerialPort { usart1 }
    }
    
    /// Funkcja odczytująca znak z bufora RX
    /// # Examples
    /// ```
    /// let c = serial.read();
    /// ```
    pub fn read(&mut self) -> char {
        char::from(self.usart1.rdr.read().rdr().bits() as u8)
    }

    /// Funkcja odczutująca 10 znaków z bufora RX
    /// <b> Uwaga! </b> Funkcja ta blockuje wątek aż do momentu odebrania 10 znaków lub odebrania znaaku ';'
    /// # Examples
    /// ```
    /// let command = serial.block_readln();
    /// ```
    pub fn block_readln(&mut self) -> String<18> {
        let mut buf: String<18> = String::new();
        loop {
            // oczekiwanie na moment rejestr RXNE zostanie wyłączony
            //set timeout
            while self.usart1.isr.read().rxne().bit_is_clear() {}
            // pobranie znaku
            let c = self.read();
            // sprawdzenie czy znak jest konca wiadomości, jeśli nie to dodaję znak do bufora
            if c == ';' {
                break;
            }
            buf.push(c).ok();
        }
        buf
    }

    /// Funkcja aktywująca perzerwanie USART1_EXTI25
    pub fn enable_interrupt(&mut self) {
        self.usart1.cr1.modify(|_, w| w.rxneie().set_bit());
    }

    /// Funkcja dezaktywująca perzerwanie USART1_EXTI25
    pub fn clear_interrupt(&mut self) {
        self.usart1.cr1.modify(|_, w| w.rxneie().clear_bit());
        self.usart1.cr1.modify(|_, w| w.tcie().clear_bit());
    }

    /// Funkcja sprawdzająca czy zostały wysłane dane
    pub fn ReciveDataRegisterNotEmpty(&mut self) -> bool {
        self.usart1.cr1.read().rxneie().bit_is_set()
    }

    /// Funkcja resetująca rejest sprawdzający czy zostały wysłane dane
    pub fn resetReciveDataRegisterNotEmpty(&mut self) {
        self.usart1.cr1.modify(|_, w| w.rxneie().clear_bit());
    }

    /// Funkcja włączająca flagę zakonczenia transmisji
    pub fn setTransmitionComplete(&mut self) {
        self.usart1.cr1.modify(|_, w| w.tcie().set_bit());
    }

    /// Funkcja sprawdzająca stan flagi zakończenia transmisji
    pub fn TransmitionComplete(&mut self) -> bool {
        self.usart1.cr1.read().tcie().bit_is_set()
    }
}

/// Makro do wysłania wiadomości do USART
#[macro_export]
macro_rules! uprint {
    ($serial:expr, $($arg:tt)*) => {
        $serial.write_fmt(format_args!($($arg)*)).ok()
    };
}

/// Makro do wysłania wiadomości do USART z nową linijką i konkatenacją
/// # Examples
/// ```
/// // Biblioteka wymaga do konatenacji
/// use core::fmt::Write;
/// // Przyukład użycia makra
/// uprintln!(serial, "2+2 = {}", 2+2);
/// ```
#[macro_export]
macro_rules! uprintln {
    ($serial:expr, $fmt:expr) => {
        usart_1::uprint!($serial, concat!($fmt, "\n"))
    };
    // gdy podano argumenty
    ($serial:expr, $fmt:expr, $($arg:tt)*) => {
        usart_1::uprint!($serial, concat!($fmt, "\n"), $($arg)*)
    };
}