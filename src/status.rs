use stm32l4xx_hal::qspi::{QspiMode, QspiReadCommand, QspiWriteCommand};

use crate::{FlashCommandError, FlashCommands, W25N01GV};

#[derive(Debug)]
pub enum ECCStatus {
    Successful,            // Data output is successful with no ECC correction
    CorrectedSuccessfully, // Data output is successful but had ECC correction for one or more pages
    SinglePageError,       // Data output had more errors than was fixable by ECC in a single page
    MultiPageError,        // Data output had more errors than was fixable by ECC in many pages
}

impl ECCStatus {
    pub fn from_bits(ecc_0: bool, ecc_1: bool) -> ECCStatus {
        if ecc_1 {
            if ecc_0 {
                ECCStatus::MultiPageError
            } else {
                ECCStatus::SinglePageError
            }
        } else {
            if ecc_0 {
                ECCStatus::CorrectedSuccessfully
            } else {
                ECCStatus::Successful
            }
        }
    }
}

#[derive(Debug)]
pub struct ProtectionRegister {
    pub srp0: bool,
    pub bp3: bool,
    pub bp2: bool,
    pub bp1: bool,
    pub bp0: bool,
    pub tb: bool,
    pub wpe: bool,
    pub srp1: bool,
}

#[derive(Debug)]
pub struct ConfigurationRegister {
    pub otp_l: bool,
    pub otp_e: bool,
    pub sr1_l: bool,
    pub ecc_e: bool,
    pub buf: bool,
}

#[derive(Debug)]
pub struct StatusRegister {
    /// Is true if the Bad Block Management Look-Up-Table (BBM LUT) has been completely filled
    pub bbm_lut_full: bool,
    /// The status of ECC for the last read operation(s)
    pub ecc_status: ECCStatus,
    /// Is true if a program/write operation failed for any reason
    pub write_failure: bool,
    /// Is true if an erase operation failed for any reason
    pub erase_failure: bool,
    /// Is true if device write is enabled
    pub write_enable_latch: bool,
    /// Is true if there's an operation in progress and the client must wait for the device to finish
    pub device_busy: bool,
}

impl ProtectionRegister {
    const SAR_ADDRESS: u8 = 0xA0;

    const SRP0_BIT: u8 = 0x80;
    const BP3_BIT: u8 = 0x40;
    const BP2_BIT: u8 = 0x20;
    const BP1_BIT: u8 = 0x10;
    const BP0_BIT: u8 = 0x08;
    const TB_BIT: u8 = 0x04;
    const WPE_BIT: u8 = 0x02;
    const SRP1_BIT: u8 = 0x01;

    fn to_u8(&self) -> u8 {
        return if self.srp0 {
            ProtectionRegister::SRP0_BIT
        } else {
            0
        } + if self.bp3 {
            ProtectionRegister::BP3_BIT
        } else {
            0
        } + if self.bp2 {
            ProtectionRegister::BP2_BIT
        } else {
            0
        } + if self.bp1 {
            ProtectionRegister::BP1_BIT
        } else {
            0
        } + if self.bp0 {
            ProtectionRegister::BP0_BIT
        } else {
            0
        } + if self.tb {
            ProtectionRegister::TB_BIT
        } else {
            0
        } + if self.wpe {
            ProtectionRegister::WPE_BIT
        } else {
            0
        } + if self.srp1 {
            ProtectionRegister::SRP1_BIT
        } else {
            0
        };
    }
}

impl ConfigurationRegister {
    const SAR_ADDRESS: u8 = 0xB0;

    const OTP_L_BIT: u8 = 0x80;
    const OTP_E_BIT: u8 = 0x40;
    const SR1_L_BIT: u8 = 0x20;
    const ECC_E_BIT: u8 = 0x10;
    const BUF_BIT: u8 = 0x08;

    fn to_u8(&self) -> u8 {
        return if self.otp_l {
            ConfigurationRegister::OTP_L_BIT
        } else {
            0
        } + if self.otp_e {
            ConfigurationRegister::OTP_E_BIT
        } else {
            0
        } + if self.sr1_l {
            ConfigurationRegister::SR1_L_BIT
        } else {
            0
        } + if self.ecc_e {
            ConfigurationRegister::ECC_E_BIT
        } else {
            0
        } + if self.buf {
            ConfigurationRegister::BUF_BIT
        } else {
            0
        };
    }
}

impl StatusRegister {
    const SAR_ADDRESS: u8 = 0xC0;

    const BBMLUT_FULL_BIT: u8 = 0x40;
    const ECC1_STATUS_BIT: u8 = 0x20;
    const ECC0_STATUS_BIT: u8 = 0x10;
    const PROGRAM_FAILURE_BIT: u8 = 0x08;
    const ERASE_FAILURE_BIT: u8 = 0x04;
    const WRITE_ENABLE_LATCH_BIT: u8 = 0x02;
    const BUSY_BIT: u8 = 0x01;
}

impl<CLK, NCS, IO0, IO1, IO2, IO3, MODE> W25N01GV<(CLK, NCS, IO0, IO1, IO2, IO3), MODE> {
    pub fn write_protection_register(
        &self,
        protection_register: ProtectionRegister,
    ) -> Result<(), FlashCommandError> {
        match self.check_busy() {
            Ok(busy) => {
                if busy {
                    return Err(FlashCommandError::DeviceBusy);
                }
            }
            Err(err) => return Err(err),
        }

        let bytes = [ProtectionRegister::SAR_ADDRESS, protection_register.to_u8()];

        let command = QspiWriteCommand {
            instruction: Some((
                FlashCommands::WriteStatusRegister as u8,
                QspiMode::SingleChannel,
            )),
            address: None,
            alternative_bytes: None,
            dummy_cycles: 0,
            data: Some((&bytes, QspiMode::SingleChannel)),
            double_data_rate: false,
        };

        if let Err(err) = self.qspi.write(command) {
            Err(FlashCommandError::from_qspi_error(err))
        } else {
            Ok(())
        }
    }

    pub fn read_protection_register(&self) -> Result<ProtectionRegister, FlashCommandError> {
        let mut reg_value = [0_u8; 1];
        let addr = [ProtectionRegister::SAR_ADDRESS];

        let command = QspiReadCommand {
            instruction: Some((
                FlashCommands::ReadStatusRegister as u8,
                QspiMode::SingleChannel,
            )),
            address: None,
            alternative_bytes: Some((&addr, QspiMode::SingleChannel)),
            dummy_cycles: 0,
            data_mode: QspiMode::SingleChannel,
            receive_length: 1,
            double_data_rate: false,
        };

        if let Err(err) = self.qspi.transfer(command, &mut reg_value) {
            return Err(FlashCommandError::from_qspi_error(err));
        }

        let reg_value = reg_value[0];

        let protection_register = ProtectionRegister {
            srp0: reg_value & ProtectionRegister::SRP0_BIT != 0,
            bp3: reg_value & ProtectionRegister::BP3_BIT != 0,
            bp2: reg_value & ProtectionRegister::BP2_BIT != 0,
            bp1: reg_value & ProtectionRegister::BP1_BIT != 0,
            bp0: reg_value & ProtectionRegister::BP0_BIT != 0,
            tb: reg_value & ProtectionRegister::TB_BIT != 0,
            wpe: reg_value & ProtectionRegister::WPE_BIT != 0,
            srp1: reg_value & ProtectionRegister::SRP1_BIT != 0,
        };

        Ok(protection_register)
    }

    pub fn write_configuration_register(
        &self,
        configuration_register: ConfigurationRegister,
    ) -> Result<(), FlashCommandError> {
        match self.check_busy() {
            Ok(busy) => {
                if busy {
                    return Err(FlashCommandError::DeviceBusy);
                }
            }
            Err(err) => return Err(err),
        }

        let bytes = [
            ConfigurationRegister::SAR_ADDRESS,
            configuration_register.to_u8(),
        ];

        let command = QspiWriteCommand {
            instruction: Some((
                FlashCommands::WriteStatusRegister as u8,
                QspiMode::SingleChannel,
            )),
            address: None,
            alternative_bytes: None,
            dummy_cycles: 0,
            data: Some((&bytes, QspiMode::SingleChannel)),
            double_data_rate: false,
        };

        if let Err(err) = self.qspi.write(command) {
            Err(FlashCommandError::from_qspi_error(err))
        } else {
            Ok(())
        }
    }

    pub fn read_configuration_register(&self) -> Result<ConfigurationRegister, FlashCommandError> {
        let mut reg_value = [0_u8; 1];
        let addr = [ConfigurationRegister::SAR_ADDRESS];

        let command = QspiReadCommand {
            instruction: Some((
                FlashCommands::ReadStatusRegister as u8,
                QspiMode::SingleChannel,
            )),
            address: None,
            alternative_bytes: Some((&addr, QspiMode::SingleChannel)),
            dummy_cycles: 0,
            data_mode: QspiMode::SingleChannel,
            receive_length: 1,
            double_data_rate: false,
        };

        if let Err(err) = self.qspi.transfer(command, &mut reg_value) {
            return Err(FlashCommandError::from_qspi_error(err));
        }

        let reg_value = reg_value[0];

        let configuration_register = ConfigurationRegister {
            otp_l: reg_value & ConfigurationRegister::OTP_L_BIT != 0,
            otp_e: reg_value & ConfigurationRegister::OTP_E_BIT != 0,
            sr1_l: reg_value & ConfigurationRegister::SR1_L_BIT != 0,
            ecc_e: reg_value & ConfigurationRegister::ECC_E_BIT != 0,
            buf: reg_value & ConfigurationRegister::BUF_BIT != 0,
        };

        Ok(configuration_register)
    }

    pub fn read_status_register(&self) -> Result<StatusRegister, FlashCommandError> {
        let mut reg_value = [0_u8; 1];
        let addr = [StatusRegister::SAR_ADDRESS];

        let command = QspiReadCommand {
            instruction: Some((
                FlashCommands::ReadStatusRegister as u8,
                QspiMode::SingleChannel,
            )),
            address: None,
            alternative_bytes: Some((&addr, QspiMode::SingleChannel)),
            dummy_cycles: 0,
            data_mode: QspiMode::SingleChannel,
            receive_length: 1,
            double_data_rate: false,
        };

        if let Err(err) = self.qspi.transfer(command, &mut reg_value) {
            return Err(FlashCommandError::from_qspi_error(err));
        }

        let reg_value = reg_value[0];

        let status_register = StatusRegister {
            bbm_lut_full: reg_value & StatusRegister::BBMLUT_FULL_BIT != 0,
            ecc_status: ECCStatus::from_bits(
                reg_value & StatusRegister::ECC0_STATUS_BIT != 0,
                reg_value & StatusRegister::ECC1_STATUS_BIT != 0,
            ),
            write_failure: reg_value & StatusRegister::PROGRAM_FAILURE_BIT != 0,
            erase_failure: reg_value & StatusRegister::ERASE_FAILURE_BIT != 0,
            write_enable_latch: reg_value & StatusRegister::WRITE_ENABLE_LATCH_BIT != 0,
            device_busy: reg_value & StatusRegister::BUSY_BIT != 0,
        };

        Ok(status_register)
    }
}
