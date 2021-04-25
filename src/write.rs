use core::marker::PhantomData;

use stm32l4xx_hal::qspi::{QspiMode, QspiWriteCommand};

use crate::{FlashCommandError, FlashCommands, ReadMode, WriteMode, W25N01GV};

impl<CLK, NCS, IO0, IO1, IO2, IO3> W25N01GV<(CLK, NCS, IO0, IO1, IO2, IO3), ReadMode> {
    pub fn into_write_mode(
        self,
    ) -> Result<W25N01GV<(CLK, NCS, IO0, IO1, IO2, IO3), WriteMode>, FlashCommandError> {
        match self.check_busy() {
            Ok(busy) => {
                if busy {
                    return Err(FlashCommandError::DeviceBusy);
                }
            }
            Err(err) => return Err(err),
        }

        let command = QspiWriteCommand {
            instruction: Some((FlashCommands::EnableWrite as u8, QspiMode::SingleChannel)),
            address: None,
            alternative_bytes: None,
            dummy_cycles: 0,
            data: None,
            double_data_rate: false,
        };

        if let Err(err) = self.qspi.write(command) {
            Err(FlashCommandError::from_qspi_error(err))
        } else {
            Ok(W25N01GV {
                _marker: PhantomData {},
                qspi: self.qspi,
            })
        }
    }
}

impl<CLK, NCS, IO0, IO1, IO2, IO3> W25N01GV<(CLK, NCS, IO0, IO1, IO2, IO3), WriteMode> {
    pub fn into_read_mode(
        self,
    ) -> Result<W25N01GV<(CLK, NCS, IO0, IO1, IO2, IO3), ReadMode>, FlashCommandError> {
        match self.check_busy() {
            Ok(busy) => {
                if busy {
                    return Err(FlashCommandError::DeviceBusy);
                }
            }
            Err(err) => return Err(err),
        }

        let command = QspiWriteCommand {
            instruction: Some((FlashCommands::DisableWrite as u8, QspiMode::SingleChannel)),
            address: None,
            alternative_bytes: None,
            dummy_cycles: 0,
            data: None,
            double_data_rate: false,
        };

        if let Err(err) = self.qspi.write(command) {
            Err(FlashCommandError::from_qspi_error(err))
        } else {
            Ok(W25N01GV {
                _marker: PhantomData {},
                qspi: self.qspi,
            })
        }
    }

    /// Erases a 128KB block within the block of the specified page. The W25N01GVxxIG/IT has 65,536
    /// pages of 2048 bytes each. Memory is erasable in groups of 64 pages (one group being a block).
    pub fn erase_128kb_block(
        self,
        page_address: u16,
    ) -> Result<W25N01GV<(CLK, NCS, IO0, IO1, IO2, IO3), ReadMode>, FlashCommandError> {
        match self.check_busy() {
            Ok(busy) => {
                if busy {
                    return Err(FlashCommandError::DeviceBusy);
                }
            }
            Err(err) => return Err(err),
        }

        let bytes = page_address.to_le_bytes();

        let command = QspiWriteCommand {
            instruction: Some((
                FlashCommands::Erase128KBBlock as u8,
                QspiMode::SingleChannel,
            )),
            address: None,
            alternative_bytes: None,
            dummy_cycles: 8,
            data: Some((&bytes, QspiMode::SingleChannel)),
            double_data_rate: false,
        };

        if let Err(err) = self.qspi.write(command) {
            Err(FlashCommandError::from_qspi_error(err))
        } else {
            if let Ok(register) = self.read_status_register() {
                if register.erase_failure {
                    return Err(FlashCommandError::WriteFailure);
                }
            }

            Ok(W25N01GV {
                _marker: PhantomData {},
                qspi: self.qspi,
            })
        }
    }

    pub fn quad_load_to_data_buffer(
        &self,
        bytes: &[u8],
        starting_address: u16,
        clear_unwritten_bytes: bool,
    ) -> Result<(), FlashCommandError> {
        match self.check_busy() {
            Ok(busy) => {
                if busy {
                    return Err(FlashCommandError::DeviceBusy);
                }
            }
            Err(err) => return Err(err),
        }

        let command = QspiWriteCommand {
            instruction: Some((
                if clear_unwritten_bytes {
                    FlashCommands::QuadLoadProgramData
                } else {
                    FlashCommands::QuadRandomLoadProgramData
                } as u8,
                QspiMode::SingleChannel,
            )),
            address: Some((starting_address as u32, QspiMode::SingleChannel)),
            alternative_bytes: None,
            dummy_cycles: 0,
            data: Some((bytes, QspiMode::QuadChannel)),
            double_data_rate: false,
        };

        if let Err(err) = self.qspi.write(command) {
            Err(FlashCommandError::from_qspi_error(err))
        } else {
            Ok(())
        }
    }

    pub fn single_load_to_data_buffer(
        &self,
        bytes: &[u8],
        starting_address: u16,
        clear_unwritten_bytes: bool,
    ) -> Result<(), FlashCommandError> {
        match self.check_busy() {
            Ok(busy) => {
                if busy {
                    return Err(FlashCommandError::DeviceBusy);
                }
            }
            Err(err) => return Err(err),
        }

        let command = QspiWriteCommand {
            instruction: Some((
                if clear_unwritten_bytes {
                    FlashCommands::LoadProgramData
                } else {
                    FlashCommands::RandomLoadProgramData
                } as u8,
                QspiMode::SingleChannel,
            )),
            address: Some((starting_address as u32, QspiMode::SingleChannel)),
            alternative_bytes: None,
            dummy_cycles: 0,
            data: Some((bytes, QspiMode::SingleChannel)),
            double_data_rate: false,
        };

        if let Err(err) = self.qspi.write(command) {
            Err(FlashCommandError::from_qspi_error(err))
        } else {
            Ok(())
        }
    }

    pub fn write_data_buffer_to_memory(
        self,
        page_address: u16,
    ) -> Result<W25N01GV<(CLK, NCS, IO0, IO1, IO2, IO3), ReadMode>, FlashCommandError> {
        match self.check_busy() {
            Ok(busy) => {
                if busy {
                    return Err(FlashCommandError::DeviceBusy);
                }
            }
            Err(err) => return Err(err),
        }

        let bytes = page_address.to_le_bytes();

        let command = QspiWriteCommand {
            instruction: Some((FlashCommands::ProgramExecute as u8, QspiMode::SingleChannel)),
            address: None,
            alternative_bytes: None,
            dummy_cycles: 8,
            data: Some((&bytes, QspiMode::SingleChannel)),
            double_data_rate: false,
        };

        if let Err(err) = self.qspi.write(command) {
            Err(FlashCommandError::from_qspi_error(err))
        } else {
            if let Ok(register) = self.read_status_register() {
                if register.erase_failure {
                    return Err(FlashCommandError::WriteFailure);
                }
            }

            Ok(W25N01GV {
                _marker: PhantomData {},
                qspi: self.qspi,
            })
        }
    }
}

impl<CLK, NCS, IO0, IO1, IO2, IO3, MODE> W25N01GV<(CLK, NCS, IO0, IO1, IO2, IO3), MODE> {
    pub fn set_write_protection(
        &self,
        tb: bool,
        bp3: bool,
        bp2: bool,
        bp1: bool,
        bp0: bool,
    ) -> Result<(), FlashCommandError> {
        match self.check_busy() {
            Ok(busy) => {
                if busy {
                    return Err(FlashCommandError::DeviceBusy);
                }
            }
            Err(err) => return Err(err),
        }

        match self.read_protection_register() {
            Ok(mut protection_register) => {
                protection_register.tb = tb;
                protection_register.bp3 = bp3;
                protection_register.bp2 = bp2;
                protection_register.bp1 = bp1;
                protection_register.bp0 = bp0;

                self.write_protection_register(protection_register)
            }
            Err(err) => Err(err),
        }
    }

    pub fn set_continuous_read_mode(&self, continuous_read: bool) -> Result<(), FlashCommandError> {
        match self.check_busy() {
            Ok(busy) => {
                if busy {
                    return Err(FlashCommandError::DeviceBusy);
                }
            }
            Err(err) => return Err(err),
        }

        match self.read_configuration_register() {
            Ok(mut configuration_register) => {
                configuration_register.buf = !continuous_read;

                self.write_configuration_register(configuration_register)
            }
            Err(err) => Err(err),
        }
    }
}
