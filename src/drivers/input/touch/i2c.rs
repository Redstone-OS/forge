//! # Controlador I2C Intel PCH (Designware)
//!
//! Driver para o controlador I2C encontrado em chipsets Intel (Designware IP).
//!
//! ## Nota
//!
//! Este módulo precisa de suporte ACPI para descobrir o endereço base MMIO
//! do controlador I2C. Por enquanto, aceita o endereço como parâmetro.

#![allow(dead_code)]

use crate::mm::VirtAddr;

/// Registradores do Designware I2C Controller
mod regs {
    /// Control Register
    pub const IC_CON: u64 = 0x00;
    /// Target Address Register
    pub const IC_TAR: u64 = 0x04;
    /// Slave Address Register
    pub const IC_SAR: u64 = 0x08;
    /// Master High Speed Code
    pub const IC_HS_MADDR: u64 = 0x0C;
    /// Data Buffer and Command Register
    pub const IC_DATA_CMD: u64 = 0x10;
    /// Standard Speed SCL High Count
    pub const IC_SS_SCL_HCNT: u64 = 0x14;
    /// Standard Speed SCL Low Count
    pub const IC_SS_SCL_LCNT: u64 = 0x18;
    /// Fast Mode SCL High Count
    pub const IC_FS_SCL_HCNT: u64 = 0x1C;
    /// Fast Mode SCL Low Count
    pub const IC_FS_SCL_LCNT: u64 = 0x20;
    /// High Speed SCL High Count
    pub const IC_HS_SCL_HCNT: u64 = 0x24;
    /// High Speed SCL Low Count
    pub const IC_HS_SCL_LCNT: u64 = 0x28;
    /// Interrupt Status
    pub const IC_INTR_STAT: u64 = 0x2C;
    /// Interrupt Mask
    pub const IC_INTR_MASK: u64 = 0x30;
    /// Raw Interrupt Status
    pub const IC_RAW_INTR_STAT: u64 = 0x34;
    /// Receive FIFO Threshold
    pub const IC_RX_TL: u64 = 0x38;
    /// Transmit FIFO Threshold
    pub const IC_TX_TL: u64 = 0x3C;
    /// Clear Combined Interrupt
    pub const IC_CLR_INTR: u64 = 0x40;
    /// Clear RX_UNDER Interrupt
    pub const IC_CLR_RX_UNDER: u64 = 0x44;
    /// Clear RX_OVER Interrupt
    pub const IC_CLR_RX_OVER: u64 = 0x48;
    /// Clear TX_OVER Interrupt
    pub const IC_CLR_TX_OVER: u64 = 0x4C;
    /// Clear RD_REQ Interrupt
    pub const IC_CLR_RD_REQ: u64 = 0x50;
    /// Clear TX_ABRT Interrupt
    pub const IC_CLR_TX_ABRT: u64 = 0x54;
    /// Clear RX_DONE Interrupt
    pub const IC_CLR_RX_DONE: u64 = 0x58;
    /// Clear Activity Interrupt
    pub const IC_CLR_ACTIVITY: u64 = 0x5C;
    /// Clear STOP_DET Interrupt
    pub const IC_CLR_STOP_DET: u64 = 0x60;
    /// Clear START_DET Interrupt
    pub const IC_CLR_START_DET: u64 = 0x64;
    /// Clear GEN_CALL Interrupt
    pub const IC_CLR_GEN_CALL: u64 = 0x68;
    /// Enable Register
    pub const IC_ENABLE: u64 = 0x6C;
    /// Status Register
    pub const IC_STATUS: u64 = 0x70;
    /// TX FIFO Level
    pub const IC_TXFLR: u64 = 0x74;
    /// RX FIFO Level
    pub const IC_RXFLR: u64 = 0x78;
    /// SDA Hold
    pub const IC_SDA_HOLD: u64 = 0x7C;
    /// TX Abort Source
    pub const IC_TX_ABRT_SOURCE: u64 = 0x80;
    /// Enable Status Register
    pub const IC_ENABLE_STATUS: u64 = 0x9C;
    /// Component Parameter 1
    pub const IC_COMP_PARAM_1: u64 = 0xF4;
    /// Component Version
    pub const IC_COMP_VERSION: u64 = 0xF8;
    /// Component Type
    pub const IC_COMP_TYPE: u64 = 0xFC;
}

/// Bits do IC_CON
mod con {
    pub const MASTER_MODE: u32 = 1 << 0;
    pub const SPEED_STANDARD: u32 = 1 << 1;
    pub const SPEED_FAST: u32 = 2 << 1;
    pub const SPEED_HIGH: u32 = 3 << 1;
    pub const IC_10BITADDR_SLAVE: u32 = 1 << 3;
    pub const IC_10BITADDR_MASTER: u32 = 1 << 4;
    pub const IC_RESTART_EN: u32 = 1 << 5;
    pub const IC_SLAVE_DISABLE: u32 = 1 << 6;
    pub const STOP_DET_IFADDRESSED: u32 = 1 << 7;
    pub const TX_EMPTY_CTRL: u32 = 1 << 8;
    pub const RX_FIFO_FULL_HLD_CTRL: u32 = 1 << 9;
}

/// Bits do IC_STATUS
mod status {
    pub const ACTIVITY: u32 = 1 << 0;
    pub const TFNF: u32 = 1 << 1; // TX FIFO not full
    pub const TFE: u32 = 1 << 2; // TX FIFO empty
    pub const RFNE: u32 = 1 << 3; // RX FIFO not empty
    pub const RFF: u32 = 1 << 4; // RX FIFO full
    pub const MST_ACTIVITY: u32 = 1 << 5;
    pub const SLV_ACTIVITY: u32 = 1 << 6;
}

/// Bits do IC_DATA_CMD
mod data_cmd {
    pub const CMD_READ: u32 = 1 << 8;
    pub const CMD_STOP: u32 = 1 << 9;
    pub const CMD_RESTART: u32 = 1 << 10;
}

/// Controlador I2C Designware
pub struct I2cController {
    /// Base MMIO
    base: VirtAddr,
    /// Se está inicializado
    initialized: bool,
}

impl I2cController {
    /// Cria um novo controlador (não inicializado)
    pub const fn new(base: VirtAddr) -> Self {
        Self {
            base,
            initialized: false,
        }
    }

    /// Cria de um endereço MMIO raw
    pub fn from_addr(addr: u64) -> Self {
        Self::new(VirtAddr::new(addr))
    }

    /// Lê um registrador de 32 bits
    #[inline]
    unsafe fn read_reg(&self, offset: u64) -> u32 {
        let addr = self.base.as_u64() + offset;
        core::ptr::read_volatile(addr as *const u32)
    }

    /// Escreve em um registrador de 32 bits
    #[inline]
    unsafe fn write_reg(&self, offset: u64, value: u32) {
        let addr = self.base.as_u64() + offset;
        core::ptr::write_volatile(addr as *mut u32, value);
    }

    /// Desabilita o controlador (necessário para configuração)
    fn disable(&self) {
        unsafe {
            self.write_reg(regs::IC_ENABLE, 0);
            // Aguardar desabilitar
            for _ in 0..1000 {
                if (self.read_reg(regs::IC_ENABLE_STATUS) & 1) == 0 {
                    break;
                }
                core::hint::spin_loop();
            }
        }
    }

    /// Habilita o controlador
    fn enable(&self) {
        unsafe {
            self.write_reg(regs::IC_ENABLE, 1);
        }
    }

    /// Inicializa o controlador em modo master, fast mode (400kHz)
    pub fn init(&mut self) -> bool {
        crate::kinfo!("(I2C) Inicializando controlador...");

        // Verificar se é um Designware I2C
        let comp_type = unsafe { self.read_reg(regs::IC_COMP_TYPE) };
        if comp_type != 0x44570140 {
            // "DW\x01@"
            crate::kwarn!("(I2C) Tipo de componente inválido:", comp_type as u64);
            // Pode não ser fatal, alguns chipsets têm valores diferentes
        }

        // Desabilitar antes de configurar
        self.disable();

        unsafe {
            // Configurar como master, fast mode, 7-bit addr, restart enabled
            let config =
                con::MASTER_MODE | con::SPEED_FAST | con::IC_RESTART_EN | con::IC_SLAVE_DISABLE;
            self.write_reg(regs::IC_CON, config);

            // Configurar timing para 400kHz (valores típicos)
            // Esses valores dependem do clock do controlador
            self.write_reg(regs::IC_FS_SCL_HCNT, 0x3C); // High count
            self.write_reg(regs::IC_FS_SCL_LCNT, 0x82); // Low count

            // Configurar RX/TX FIFO thresholds
            self.write_reg(regs::IC_RX_TL, 0);
            self.write_reg(regs::IC_TX_TL, 0);

            // Desabilitar todas as interrupções
            self.write_reg(regs::IC_INTR_MASK, 0);
        }

        // Habilitar
        self.enable();

        self.initialized = true;
        crate::kinfo!("(I2C) Controlador inicializado");
        true
    }

    /// Define o endereço do target (slave)
    pub fn set_target(&self, addr: u8) {
        self.disable();
        unsafe {
            self.write_reg(regs::IC_TAR, addr as u32);
        }
        self.enable();
    }

    /// Escreve bytes para o dispositivo
    pub fn write(&self, data: &[u8]) -> Result<(), I2cError> {
        if !self.initialized {
            return Err(I2cError::NotInitialized);
        }

        for (i, &byte) in data.iter().enumerate() {
            // Aguardar TX FIFO não cheio
            if !self.wait_tx_not_full() {
                return Err(I2cError::Timeout);
            }

            let mut cmd = byte as u32;
            // STOP no último byte
            if i == data.len() - 1 {
                cmd |= data_cmd::CMD_STOP;
            }

            unsafe {
                self.write_reg(regs::IC_DATA_CMD, cmd);
            }
        }

        // Aguardar TX completar
        self.wait_tx_empty()?;

        // Verificar erros
        let abort = unsafe { self.read_reg(regs::IC_TX_ABRT_SOURCE) };
        if abort != 0 {
            // Limpar abort
            unsafe {
                self.read_reg(regs::IC_CLR_TX_ABRT);
            }
            return Err(I2cError::Nack);
        }

        Ok(())
    }

    /// Lê bytes do dispositivo
    pub fn read(&self, buf: &mut [u8]) -> Result<(), I2cError> {
        if !self.initialized {
            return Err(I2cError::NotInitialized);
        }

        for i in 0..buf.len() {
            // Aguardar TX FIFO não cheio
            if !self.wait_tx_not_full() {
                return Err(I2cError::Timeout);
            }

            let mut cmd = data_cmd::CMD_READ;
            // STOP no último byte
            if i == buf.len() - 1 {
                cmd |= data_cmd::CMD_STOP;
            }

            unsafe {
                self.write_reg(regs::IC_DATA_CMD, cmd);
            }

            // Aguardar RX FIFO ter dados
            if !self.wait_rx_not_empty() {
                return Err(I2cError::Timeout);
            }

            buf[i] = unsafe { self.read_reg(regs::IC_DATA_CMD) as u8 };
        }

        Ok(())
    }

    /// Write-then-read (operação comum de I2C)
    pub fn write_read(&self, write_data: &[u8], read_buf: &mut [u8]) -> Result<(), I2cError> {
        if !self.initialized {
            return Err(I2cError::NotInitialized);
        }

        // Escrever dados sem STOP
        for &byte in write_data {
            if !self.wait_tx_not_full() {
                return Err(I2cError::Timeout);
            }
            unsafe {
                self.write_reg(regs::IC_DATA_CMD, byte as u32);
            }
        }

        // Ler com RESTART
        for i in 0..read_buf.len() {
            if !self.wait_tx_not_full() {
                return Err(I2cError::Timeout);
            }

            let mut cmd = data_cmd::CMD_READ;
            if i == 0 {
                cmd |= data_cmd::CMD_RESTART;
            }
            if i == read_buf.len() - 1 {
                cmd |= data_cmd::CMD_STOP;
            }

            unsafe {
                self.write_reg(regs::IC_DATA_CMD, cmd);
            }

            if !self.wait_rx_not_empty() {
                return Err(I2cError::Timeout);
            }

            read_buf[i] = unsafe { self.read_reg(regs::IC_DATA_CMD) as u8 };
        }

        Ok(())
    }

    fn wait_tx_not_full(&self) -> bool {
        for _ in 0..10000 {
            if unsafe { self.read_reg(regs::IC_STATUS) } & status::TFNF != 0 {
                return true;
            }
            core::hint::spin_loop();
        }
        false
    }

    fn wait_tx_empty(&self) -> Result<(), I2cError> {
        for _ in 0..100000 {
            if unsafe { self.read_reg(regs::IC_STATUS) } & status::TFE != 0 {
                return Ok(());
            }
            core::hint::spin_loop();
        }
        Err(I2cError::Timeout)
    }

    fn wait_rx_not_empty(&self) -> bool {
        for _ in 0..100000 {
            if unsafe { self.read_reg(regs::IC_STATUS) } & status::RFNE != 0 {
                return true;
            }
            core::hint::spin_loop();
        }
        false
    }
}

/// Erros do I2C
#[derive(Debug, Clone, Copy)]
pub enum I2cError {
    /// Controlador não inicializado
    NotInitialized,
    /// Timeout na operação
    Timeout,
    /// NACK recebido
    Nack,
    /// Erro de barramento
    BusError,
}
