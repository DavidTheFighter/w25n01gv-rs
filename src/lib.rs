#![no_std]
#![forbid(unsafe_code)]

extern crate embedded_hal as hal;
use core::marker::PhantomData;

use stm32l4xx_hal::qspi::{Qspi, QspiError, QspiMode, QspiReadCommand, QspiWriteCommand};

pub mod read;
pub mod status;
pub mod write;

pub use read::ReadMethod;
pub use write::WriteMethod;

pub const PAGE_SIZE_BYTES: usize = 2048;
pub const PAGE_SIZE_WITH_ECC_BYTES: usize = 2112;
pub const MAX_BBM_LUT_ENTIRES: usize = 20;
pub const PAGES_PER_BLOCK: usize = 64;

enum FlashCommands {
    DeviceReset = 0xFF,
    JEDECId = 0x9F,
    ReadStatusRegister = 0x05,
    WriteStatusRegister = 0x01,
    EnableWrite = 0x06,
    DisableWrite = 0x04,
    Erase128KBBlock = 0xD8,
    ReadBBM = 0xA5,
    ProgramExecute = 0x10,
    PageDataRead = 0x13,
}

#[derive(Debug)]
pub enum FlashCommandError {
    QSPIBusy,
    QSPIAddress,
    QSPIUnknown,
    DeviceBusy,
}

impl FlashCommandError {
    fn from_qspi_error(err: QspiError) -> FlashCommandError {
        match err {
            stm32l4xx_hal::qspi::QspiError::Busy => FlashCommandError::QSPIBusy,
            stm32l4xx_hal::qspi::QspiError::Address => FlashCommandError::QSPIAddress,
            stm32l4xx_hal::qspi::QspiError::Unknown => FlashCommandError::QSPIUnknown,
        }
    }
}

pub struct WriteMode;
pub struct ReadMode;

pub struct W25N01GV<PINS, MODE> {
    _marker: PhantomData<MODE>,
    qspi: Qspi<PINS>,
}

pub fn new_w25_n01_gv<CLK, NCS, IO0, IO1, IO2, IO3>(
    qspi: Qspi<(CLK, NCS, IO0, IO1, IO2, IO3)>,
) -> W25N01GV<(CLK, NCS, IO0, IO1, IO2, IO3), ReadMode> {
    W25N01GV {
        _marker: PhantomData {},
        qspi,
    }
}

impl<CLK, NCS, IO0, IO1, IO2, IO3, MODE> W25N01GV<(CLK, NCS, IO0, IO1, IO2, IO3), MODE> {
    pub fn reset_device(&self) -> Result<(), FlashCommandError> {
        match self.check_busy() {
            Ok(busy) => {
                if busy {
                    return Err(FlashCommandError::DeviceBusy);
                }
            }
            Err(err) => return Err(err),
        }

        let command = QspiWriteCommand {
            instruction: Some((FlashCommands::DeviceReset as u8, QspiMode::SingleChannel)),
            address: None,
            alternative_bytes: None,
            dummy_cycles: 0,
            data: None,
            double_data_rate: false,
        };

        if let Err(err) = self.qspi.write(command) {
            Err(FlashCommandError::from_qspi_error(err))
        } else {
            Ok(())
        }
    }

    pub fn get_jedec_id(&mut self) -> Result<[u8; 3], FlashCommandError> {
        match self.check_busy() {
            Ok(busy) => {
                if busy {
                    return Err(FlashCommandError::DeviceBusy);
                }
            }
            Err(err) => return Err(err),
        }

        let mut id = [0_u8; 3];

        let command = QspiReadCommand {
            instruction: Some((FlashCommands::JEDECId as u8, QspiMode::SingleChannel)),
            address: None,
            alternative_bytes: None,
            dummy_cycles: 8,
            data_mode: QspiMode::SingleChannel,
            receive_length: 3,
            double_data_rate: false,
        };

        if let Err(err) = self.qspi.transfer(command, &mut id) {
            Err(FlashCommandError::from_qspi_error(err))
        } else {
            Ok(id)
        }
    }

    pub fn wait_while_busy(&self) {
        loop {
            match self.check_busy() {
                Ok(busy) => {
                    if !busy {
                        break;
                    }
                }
                Err(_err) => break,
            }
        }
    }

    pub fn check_write_or_erase_failure(&self) -> Result<bool, FlashCommandError> {
        match self.read_status_register() {
            Ok(status_register) => {
                Ok(status_register.erase_failure || status_register.write_failure)
            }
            Err(err) => Err(err),
        }
    }

    /// An internal method every function that sends a command (minus a special few) uses to
    /// check if the flash device is busy. The flash device will silently reject commands while
    /// busy.
    fn check_busy(&self) -> Result<bool, FlashCommandError> {
        match self.read_status_register() {
            Ok(status_register) => Ok(status_register.device_busy),
            Err(err) => Err(err),
        }
    }
}
