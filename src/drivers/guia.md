# Guia de Implementação: `drivers/`

> **ATENÇÃO**: Este guia deve ser seguido LITERALMENTE.

---

## 1. PROPÓSITO

Drivers de hardware e modelo de dispositivos. Conecta hardware ao kernel.

---

## 2. ESTRUTURA

```
drivers/
├── mod.rs              ✅ JÁ EXISTE
├── base/
│   ├── mod.rs
│   ├── device.rs       → Abstração de dispositivo
│   ├── driver.rs       → Trait Driver
│   ├── bus.rs          → Barramentos
│   └── class.rs        → Classes de dispositivo
├── serial/
│   └── mod.rs          → UART/Serial
├── timer/
│   ├── mod.rs
│   ├── pit.rs          → Programmable Interval Timer
│   ├── hpet.rs         → High Precision Event Timer
│   └── tsc.rs          → Timestamp Counter
├── video/
│   ├── mod.rs
│   ├── framebuffer.rs  → Framebuffer linear
│   ├── font.rs         → Fonte bitmap
│   └── font_data.rs    → Dados da fonte
├── pci/
│   ├── mod.rs
│   ├── pci.rs          → Enumeração PCI
│   └── config.rs       → Config space
├── block/
│   ├── mod.rs
│   ├── ahci.rs         → SATA controller
│   ├── nvme.rs         → NVMe SSD
│   └── ramdisk.rs      → RAM disk
├── input/
│   ├── mod.rs
│   ├── keyboard.rs     → PS/2 keyboard
│   └── mouse.rs        → PS/2 mouse
├── net/
│   ├── mod.rs
│   └── virtio_net.rs   → VirtIO network
├── irq/
│   ├── mod.rs
│   └── apic.rs         → APIC controller
└── pic.rs              → Legacy 8259 PIC
```

---

## 3. REGRAS

### ❌ NUNCA:
- Acessar IO sem verificar existência do dispositivo
- Usar polling infinito (sempre timeout)
- Alocar memória em interrupt handlers

### ✅ SEMPRE:
- Implementar trait Driver
- Registrar dispositivo no bus
- Usar DMA com IOMMU quando disponível

---

## 4. IMPLEMENTAÇÕES

### 4.1 `base/driver.rs`

```rust
//! Trait base para drivers

/// Tipo de dispositivo
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceType {
    Block,
    Char,
    Network,
    Input,
    Display,
    Timer,
    Bus,
    Unknown,
}

/// Erro de driver
#[derive(Debug, Clone, Copy)]
pub enum DriverError {
    NotSupported,
    NotFound,
    InitFailed,
    BusError,
    Timeout,
    IoError,
}

/// Trait que todo driver deve implementar
pub trait Driver: Send + Sync {
    /// Nome do driver
    fn name(&self) -> &'static str;
    
    /// Tipo de dispositivo
    fn device_type(&self) -> DeviceType;
    
    /// Chamado quando dispositivo é detectado
    fn probe(&self, dev: &mut Device) -> Result<(), DriverError>;
    
    /// Chamado quando dispositivo é removido
    fn remove(&self, dev: &mut Device) -> Result<(), DriverError>;
    
    /// Chamado durante suspend
    fn suspend(&self, _dev: &mut Device) -> Result<(), DriverError> {
        Ok(())
    }
    
    /// Chamado durante resume
    fn resume(&self, _dev: &mut Device) -> Result<(), DriverError> {
        Ok(())
    }
}
```

### 4.2 `base/device.rs`

```rust
//! Abstração de dispositivo

use super::driver::DeviceType;

/// ID de dispositivo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceId(pub u64);

/// Representação de um dispositivo
pub struct Device {
    pub id: DeviceId,
    pub name: [u8; 32],
    pub device_type: DeviceType,
    pub bus_type: BusType,
    pub driver: Option<&'static dyn super::driver::Driver>,
}

/// Tipo de barramento
#[derive(Debug, Clone, Copy)]
pub enum BusType {
    Platform,   // Dispositivos integrados
    Pci,
    Usb,
    Acpi,
}

impl Device {
    pub fn new(id: DeviceId, name: &str) -> Self {
        let mut name_buf = [0u8; 32];
        let len = name.len().min(31);
        name_buf[..len].copy_from_slice(&name.as_bytes()[..len]);
        
        Self {
            id,
            name: name_buf,
            device_type: DeviceType::Unknown,
            bus_type: BusType::Platform,
            driver: None,
        }
    }
}
```

### 4.3 `serial/mod.rs`

```rust
//! Driver de porta serial (UART 16550)

use crate::arch::x86_64::ports::{inb, outb};
use crate::sync::Spinlock;

/// Porta COM1
const COM1_PORT: u16 = 0x3F8;

/// Estado da serial
static SERIAL: Spinlock<SerialPort> = Spinlock::new(SerialPort::new(COM1_PORT));

struct SerialPort {
    port: u16,
    initialized: bool,
}

impl SerialPort {
    const fn new(port: u16) -> Self {
        Self { port, initialized: false }
    }
    
    fn init(&mut self) {
        if self.initialized {
            return;
        }
        
        // Desabilitar interrupções
        outb(self.port + 1, 0x00);
        // Habilitar DLAB (set baud rate)
        outb(self.port + 3, 0x80);
        // Divisor low byte (115200 baud)
        outb(self.port + 0, 0x03);
        // Divisor high byte
        outb(self.port + 1, 0x00);
        // 8 bits, no parity, 1 stop bit
        outb(self.port + 3, 0x03);
        // Enable FIFO
        outb(self.port + 2, 0xC7);
        // IRQs enabled, RTS/DSR set
        outb(self.port + 4, 0x0B);
        
        self.initialized = true;
    }
    
    fn is_transmit_empty(&self) -> bool {
        (inb(self.port + 5) & 0x20) != 0
    }
    
    fn write_byte(&self, byte: u8) {
        // Esperar FIFO estar pronto
        while !self.is_transmit_empty() {
            core::hint::spin_loop();
        }
        outb(self.port, byte);
    }
}

/// Inicializa serial
pub fn init() {
    SERIAL.lock().init();
}

/// Escreve byte
pub fn write_byte(byte: u8) {
    SERIAL.lock().write_byte(byte);
}

/// Escreve string
pub fn write_str(s: &str) {
    let serial = SERIAL.lock();
    for byte in s.bytes() {
        serial.write_byte(byte);
    }
}

/// Escreve número hexadecimal
pub fn write_hex(value: u64) {
    let serial = SERIAL.lock();
    
    // Escrever dígitos
    for i in (0..16).rev() {
        let digit = ((value >> (i * 4)) & 0xF) as u8;
        let c = if digit < 10 {
            b'0' + digit
        } else {
            b'A' + digit - 10
        };
        serial.write_byte(c);
    }
}
```

### 4.4 `timer/pit.rs`

```rust
//! Programmable Interval Timer (8254)

use crate::arch::x86_64::ports::{outb, inb};

/// Frequência base do PIT (Hz)
const PIT_FREQUENCY: u32 = 1193182;

/// Portas do PIT
const PIT_CHANNEL0: u16 = 0x40;
const PIT_COMMAND: u16 = 0x43;

/// Inicializa PIT para frequência específica
pub fn init(frequency_hz: u32) {
    let divisor = PIT_FREQUENCY / frequency_hz;
    
    // Channel 0, lobyte/hibyte, mode 3 (square wave)
    outb(PIT_COMMAND, 0x36);
    
    // Divisor
    outb(PIT_CHANNEL0, (divisor & 0xFF) as u8);
    outb(PIT_CHANNEL0, ((divisor >> 8) & 0xFF) as u8);
    
    crate::kinfo!("(PIT) Inicializado com freq=", frequency_hz as u64);
}
```

### 4.5 `video/framebuffer.rs`

```rust
//! Framebuffer linear

use crate::mm::VirtAddr;
use crate::sync::Spinlock;

/// Informações do framebuffer
pub struct FramebufferInfo {
    pub addr: VirtAddr,
    pub width: u32,
    pub height: u32,
    pub stride: u32,  // bytes por linha
    pub bpp: u32,     // bits por pixel
}

static FRAMEBUFFER: Spinlock<Option<FramebufferInfo>> = Spinlock::new(None);

/// Inicializa com informações do bootloader
pub fn init(info: FramebufferInfo) {
    *FRAMEBUFFER.lock() = Some(info);
    crate::kinfo!("(FB) Inicializado:", info.width as u64);
}

/// Escreve pixel
pub fn put_pixel(x: u32, y: u32, color: u32) {
    let fb = FRAMEBUFFER.lock();
    if let Some(ref info) = *fb {
        if x >= info.width || y >= info.height {
            return;
        }
        
        let offset = (y * info.stride + x * (info.bpp / 8)) as usize;
        let ptr = info.addr.offset(offset as u64).as_mut_ptr::<u32>();
        
        // SAFETY: offset foi validado
        unsafe { *ptr = color; }
    }
}

/// Preenche retângulo
pub fn fill_rect(x: u32, y: u32, w: u32, h: u32, color: u32) {
    for dy in 0..h {
        for dx in 0..w {
            put_pixel(x + dx, y + dy, color);
        }
    }
}

/// Limpa tela
pub fn clear(color: u32) {
    let fb = FRAMEBUFFER.lock();
    if let Some(ref info) = *fb {
        fill_rect(0, 0, info.width, info.height, color);
    }
}
```

---

## 5. ORDEM DE IMPLEMENTAÇÃO

1. `base/driver.rs` - Trait Driver
2. `base/device.rs` - Device abstraction
3. `serial/mod.rs` - Logging (PRIMEIRO!)
4. `timer/pit.rs` - Timer básico
5. `video/framebuffer.rs` - Display
6. `pci/pci.rs` - Bus enumeration
7. `block/` - Storage drivers

---

## 6. DEPENDÊNCIAS

Pode importar de:
- `crate::arch` (ports, Cpu)
- `crate::sync`
- `crate::mm`
- `crate::klib`

---

## 7. CHECKLIST

- [ ] Driver trait implementado
- [ ] Serial funciona antes de MM
- [ ] PIT configurado
- [ ] Framebuffer suporta bootloader info
