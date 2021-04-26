use stm32l4xx_hal::qspi::{QspiMode, QspiReadCommand, QspiWriteCommand};

use crate::{
    FlashCommandError, FlashCommands, MAX_BBM_LUT_ENTIRES, PAGE_SIZE_WITH_ECC_BYTES, W25N01GV,
};

#[derive(Debug, Clone, Copy)]
pub enum ReadMethod {
    FastRead = 0x0B,
    DualFastRead = 0x3B,
    QuadFastRead = 0x6B,
    FastReadDualIO = 0xBB,
    FastReadQuadIO = 0xEB,
}

impl ReadMethod {
    fn dummy_cycles(&self) -> u8 {
        match self {
            ReadMethod::FastRead => 8,
            ReadMethod::DualFastRead => 8,
            ReadMethod::QuadFastRead => 8,
            ReadMethod::FastReadDualIO => 4,
            ReadMethod::FastReadQuadIO => 4,
        }
    }

    fn address_mode(&self) -> QspiMode {
        match self {
            ReadMethod::FastRead => QspiMode::SingleChannel,
            ReadMethod::DualFastRead => QspiMode::SingleChannel,
            ReadMethod::QuadFastRead => QspiMode::SingleChannel,
            ReadMethod::FastReadDualIO => QspiMode::DualChannel,
            ReadMethod::FastReadQuadIO => QspiMode::QuadChannel,
        }
    }

    fn data_mode(&self) -> QspiMode {
        match self {
            ReadMethod::FastRead => QspiMode::SingleChannel,
            ReadMethod::DualFastRead => QspiMode::DualChannel,
            ReadMethod::QuadFastRead => QspiMode::QuadChannel,
            ReadMethod::FastReadDualIO => QspiMode::DualChannel,
            ReadMethod::FastReadQuadIO => QspiMode::QuadChannel,
        }
    }
}

impl<CLK, NCS, IO0, IO1, IO2, IO3, MODE> W25N01GV<(CLK, NCS, IO0, IO1, IO2, IO3), MODE> {
    pub fn read_memory_to_data_buffer(&self, page_address: u16) -> Result<(), FlashCommandError> {
        match self.check_busy() {
            Ok(busy) => {
                if busy {
                    return Err(FlashCommandError::DeviceBusy);
                }
            }
            Err(err) => return Err(err),
        }

        let page_address = page_address.to_be_bytes();

        let command = QspiWriteCommand {
            instruction: Some((FlashCommands::PageDataRead as u8, QspiMode::SingleChannel)),
            address: None,
            alternative_bytes: None,
            dummy_cycles: 8,
            data: Some((&page_address, QspiMode::SingleChannel)),
            double_data_rate: false,
        };

        if let Err(err) = self.qspi.write(command) {
            match err {
                stm32l4xx_hal::qspi::QspiError::Busy => Err(FlashCommandError::QSPIBusy),
                stm32l4xx_hal::qspi::QspiError::Address => Err(FlashCommandError::QSPIAddress),
                stm32l4xx_hal::qspi::QspiError::Unknown => Err(FlashCommandError::QSPIUnknown),
            }
        } else {
            Ok(())
        }
    }

    pub fn read_data_buffer(
        &self,
        buffer: &mut [u8; PAGE_SIZE_WITH_ECC_BYTES],
        method: ReadMethod,
    ) -> Result<(), FlashCommandError> {
        match self.check_busy() {
            Ok(busy) => {
                if busy {
                    return Err(FlashCommandError::DeviceBusy);
                }
            }
            Err(err) => return Err(err),
        }

        let command = QspiReadCommand {
            instruction: Some((method as u8, QspiMode::SingleChannel)),
            address: Some((0, method.address_mode())),
            alternative_bytes: None,
            dummy_cycles: method.dummy_cycles(),
            data_mode: method.data_mode(),
            receive_length: PAGE_SIZE_WITH_ECC_BYTES as u32,
            double_data_rate: false,
        };

        if let Err(err) = self.qspi.transfer(command, buffer) {
            match err {
                stm32l4xx_hal::qspi::QspiError::Busy => Err(FlashCommandError::QSPIBusy),
                stm32l4xx_hal::qspi::QspiError::Address => Err(FlashCommandError::QSPIAddress),
                stm32l4xx_hal::qspi::QspiError::Unknown => Err(FlashCommandError::QSPIUnknown),
            }
        } else {
            Ok(())
        }
    }

    pub fn read_bbm_lookup_table(
        &self,
    ) -> Result<[Option<(u16, u16)>; MAX_BBM_LUT_ENTIRES], FlashCommandError> {
        match self.check_busy() {
            Ok(busy) => {
                if busy {
                    return Err(FlashCommandError::DeviceBusy);
                }
            }
            Err(err) => return Err(err),
        }

        let mut buffer = [0_u8; MAX_BBM_LUT_ENTIRES * 4];

        let command = QspiReadCommand {
            instruction: Some((FlashCommands::ReadBBM as u8, QspiMode::SingleChannel)),
            address: None,
            alternative_bytes: None,
            dummy_cycles: 8,
            data_mode: QspiMode::SingleChannel,
            receive_length: buffer.len() as u32,
            double_data_rate: false,
        };

        if let Err(err) = self.qspi.transfer(command, &mut buffer) {
            match err {
                stm32l4xx_hal::qspi::QspiError::Busy => Err(FlashCommandError::QSPIBusy),
                stm32l4xx_hal::qspi::QspiError::Address => Err(FlashCommandError::QSPIAddress),
                stm32l4xx_hal::qspi::QspiError::Unknown => Err(FlashCommandError::QSPIUnknown),
            }
        } else {
            let mut links = [None; MAX_BBM_LUT_ENTIRES];

            for link_index in 0..MAX_BBM_LUT_ENTIRES {
                let lba = buffer[link_index * 4] as u16 + (buffer[link_index * 4 + 1] as u16) << 8;
                let pba =
                    buffer[link_index * 4 + 2] as u16 + (buffer[link_index * 4 + 3] as u16) << 8;

                if lba != 0 || pba != 0 {
                    links[link_index] = Some((lba, pba));
                }
            }

            Ok(links)
        }
    }
}
