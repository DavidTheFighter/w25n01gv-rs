//! Blinks an LED

#![deny(unsafe_code)]
#![no_std]
#![no_main]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate stm32l4xx_hal as hal;

use cortex_m_semihosting::hprintln;
use hal::{
    qspi::{AddressSize, Qspi, QspiConfig},
    rcc::{PllConfig, PllDivider, PllSource},
};

use crate::hal::prelude::*;
use crate::rt::entry;
use crate::rt::ExceptionFrame;
use w25n01gv_rs::{new_w25_n01_gv, PAGE_SIZE_WITH_ECC_BYTES};

use core::panic::PanicInfo;

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = hal::stm32::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let mut pwr = dp.PWR.constrain(&mut rcc.apb1r1);

    let clocks = rcc
        .cfgr
        .pll_source(PllSource::HSI16)
        .sysclk_with_pll(80.mhz(), PllConfig::new(2, 20, PllDivider::Div2))
        .pclk1(80.mhz())
        .pclk2(80.mhz())
        .freeze(&mut flash.acr, &mut pwr);

    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb2);

    let quadspi_clk = gpioa.pa3.into_af10(&mut gpioa.moder, &mut gpioa.afrl);
    let quadspi_ncs = gpioa.pa2.into_af10(&mut gpioa.moder, &mut gpioa.afrl);
    let quadspi_io0 = gpiob.pb1.into_af10(&mut gpiob.moder, &mut gpiob.afrl);
    let quadspi_io1 = gpiob.pb0.into_af10(&mut gpiob.moder, &mut gpiob.afrl);
    let quadspi_io2 = gpioa.pa7.into_af10(&mut gpioa.moder, &mut gpioa.afrl);
    let quadspi_io3 = gpioa.pa6.into_af10(&mut gpioa.moder, &mut gpioa.afrl);

    let mut quadspi = Qspi::new(
        dp.QUADSPI,
        (
            quadspi_clk,
            quadspi_ncs,
            quadspi_io0,
            quadspi_io1,
            quadspi_io2,
            quadspi_io3,
        ),
        &mut rcc.ahb3,
        QspiConfig::default()
            .flash_size(29)
            .address_size(AddressSize::Addr16Bit)
            .clock_prescaler(3),
    );

    let mut flash_chip = new_w25_n01_gv(quadspi);

    let id = flash_chip.get_jedec_id().unwrap();
    hprintln!("JEDEC Values: {}, {}, {}", id[0], id[1], id[2]).unwrap();

    flash_chip
        .set_write_protection(false, false, false, false, false)
        .unwrap();
    flash_chip.set_continuous_read_mode(false).unwrap();

    let flash_chip = flash_chip.into_write_mode().unwrap();
    let flash_chip = flash_chip.erase_128kb_block(0).unwrap();
    flash_chip.wait_while_busy();

    let buffer = [0, 1, 2, 3, 42];
    hprintln!(
        "Writing bytes: {} {} {} {} {}",
        buffer[0],
        buffer[1],
        buffer[2],
        buffer[3],
        buffer[4]
    )
    .unwrap();

    let flash_chip = flash_chip.into_write_mode().unwrap();
    flash_chip
        .single_load_to_data_buffer(&buffer, 0, true)
        .unwrap();

    let flash_chip = flash_chip.write_data_buffer_to_memory(0).unwrap();
    flash_chip.wait_while_busy();

    flash_chip.read_memory_to_data_buffer(0).unwrap();
    let mut buffer = [0_u8; PAGE_SIZE_WITH_ECC_BYTES];

    flash_chip.wait_while_busy();
    flash_chip.single_read_data_buffer(&mut buffer).unwrap();

    hprintln!(
        "Read back bytes: {} {} {} {} {}",
        buffer[0],
        buffer[1],
        buffer[2],
        buffer[3],
        buffer[4]
    )
    .unwrap();

    loop {
        continue;
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    hprintln!("{:?}", info).unwrap();
    loop {}
}
