use stm32l4xx_hal::qspi::{QspiMode, QspiReadCommand, QspiWriteCommand};

use crate::{
    FlashCommandError, FlashCommands, MAX_BBM_LUT_ENTIRES, PAGE_SIZE_WITH_ECC_BYTES, W25N01GV,
};

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

        let page_address = page_address.to_le_bytes();

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

    pub fn single_read_data_buffer(
        &self,
        buffer: &mut [u8; PAGE_SIZE_WITH_ECC_BYTES],
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
            instruction: Some((FlashCommands::FastRead as u8, QspiMode::SingleChannel)),
            address: Some((0, QspiMode::SingleChannel)),
            alternative_bytes: None,
            dummy_cycles: 8,
            data_mode: QspiMode::SingleChannel,
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

    pub fn quad_read_data_buffer(
        &self,
        buffer: &mut [u8; PAGE_SIZE_WITH_ECC_BYTES],
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
            instruction: Some((FlashCommands::QuadFastRead as u8, QspiMode::SingleChannel)),
            address: Some((0, QspiMode::SingleChannel)),
            alternative_bytes: None,
            dummy_cycles: 8,
            data_mode: QspiMode::QuadChannel,
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
