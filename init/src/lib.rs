#![crate_type = "dylib"]
#![no_std]
#![allow(non_snake_case)]

use hal::delay::Delay;
// Import potrzebnych komponentów z bilbioteki HAL
use stm32f3xx_hal::{
    self as hal,
    prelude::*,
    serial::Serial,
    pac::{self, USART1, usart1, NVIC, I2C1},
    gpio::{Output, PushPull, Gpiob, Gpioc, Gpioe, Ux, Pin, U, Alternate, OpenDrain},
    interrupt,
    i2c::I2c,
};


/// Funkcja ta inicjuje komponenty wymagane do działania termometru.
/// # Examples
/// ```
/// // Załączenie bilioteki
/// use init::*;
/// // Inicjalizacja komponentów
/// let (usart, leds_array) = init();
/// ```
pub fn init() -> (&'static mut usart1::RegisterBlock, 
                [Pin<Gpioe, Ux, Output<PushPull>>; 8],
                I2c<I2C1, (Pin<Gpiob, U<6>, Alternate<OpenDrain, 4>>, Pin<Gpiob, U<7>, Alternate<OpenDrain, 4>>)>,
                Delay,
                Pin<Gpioc, U<1>, Output<OpenDrain>>) 
{
    // Inicjalizacja komponentów
    let dp = pac::Peripherals::take().unwrap();
    let mut cp = cortex_m::peripheral::Peripherals::take().unwrap();
    cp.DWT.enable_cycle_counter();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    // Ustawienie zegara
    let clocks = rcc
            .cfgr
            .use_hse(8.MHz())
            .sysclk(48.MHz())
            .freeze(&mut flash.acr);

    let delay = stm32f3xx_hal::delay::Delay::new(cp.SYST, clocks);
    // Aktywacja nasłuchiwania przerwania USART1_EXTI25 (odbiór danych)
    unsafe {
        NVIC::unmask(interrupt::USART1_EXTI25);
    }    
    // Przypisanie wyjśc GPIOX do zmiennych
    let mut gpioc = dp.GPIOC.split(&mut rcc.ahb);
    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);

    // Przypisanie pinów rx/tx wraz z ustawieniem trybu
    let tx = gpioc.pc4.into_af_push_pull(&mut gpioc.moder, &mut gpioc.otyper, &mut gpioc.afrl);
    let rx = gpioc.pc5.into_af_push_pull(&mut gpioc.moder, &mut gpioc.otyper, &mut gpioc.afrl);

    // Utworzenie komponentu USART1 z boundrate = 9600
    Serial::new(dp.USART1, (tx, rx), 115_200.Bd(), clocks, &mut rcc.apb2);

    // Tworzenie tablicy pinów LED
    let leds = [
        gpioe
            .pe9
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper)
            .downgrade(),
        gpioe
            .pe10
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper)
            .downgrade(),
        gpioe
            .pe11
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper)
            .downgrade(),
        gpioe
            .pe12
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper)
            .downgrade(),
        gpioe
            .pe13
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper)
            .downgrade(),
        gpioe
            .pe14
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper)
            .downgrade(),
        gpioe
            .pe15
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper)
            .downgrade(),
        gpioe
            .pe8
            .into_push_pull_output(&mut gpioe.moder, &mut &mut gpioe.otyper)
            .downgrade(),
    ];
    // Utworzenie pinów służących do komunikacji w interfejsie I2C
    let mut scl =
        gpiob
            .pb6
            .into_af_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
    let mut sda =
        gpiob
            .pb7
            .into_af_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);

    scl.internal_pull_up(&mut gpiob.pupdr, true);
    sda.internal_pull_up(&mut gpiob.pupdr, true);

    let i2c = hal::i2c::I2c::new(
        dp.I2C1,
        (scl, sda),
        100.kHz().try_into().unwrap(),
        clocks,
        &mut rcc.apb1,
    );

    // Utworzenie pinu komunikującego się z DHT11
    let dht_pin = gpioc.pc1.into_open_drain_output(&mut gpioc.moder,&mut gpioc.otyper);

    // Zwrócenie wskaznika do USART, tablicy LED
    unsafe {
        (
            &mut *(USART1::ptr() as *mut _),
            leds,
            i2c,
            delay,
            dht_pin,
        )
    }
}