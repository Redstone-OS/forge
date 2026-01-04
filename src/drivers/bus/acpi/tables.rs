//! # ACPI Tables Parser
//!
//! Responsável por localizar, mapear e validar as tabelas ACPI na memória física.
//! O ACPI (Advanced Configuration and Power Interface) fornece a descrição do
//! hardware que não pode ser descoberta dinamicamente (enumeração estática).

use core::ptr::NonNull;

/// Assinatura do RSDP (Root System Description Pointer)
const RSDP_SIGNATURE: &[u8; 8] = b"RSD PTR ";

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Rsdp {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct RsdpExtended {
    base: Rsdp,
    length: u32,
    xsdt_address: u64,
    extended_checksum: u8,
    reserved: [u8; 3],
}

/// Cabeçalho comum para todas as tabelas ACPI (SDT Header)
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct SdtHeader {
    pub signature: [u8; 4],
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub oem_table_id: [u8; 8],
    pub oem_revision: u32,
    pub creator_id: [u8; 4],
    pub creator_revision: u32,
}

impl SdtHeader {
    pub fn signature_as_str(&self) -> &str {
        core::str::from_utf8(&self.signature).unwrap_or("????")
    }

    /// Valida o checksum da tabela
    pub fn validate(&self) -> bool {
        let ptr = self as *const _ as *const u8;
        let mut sum: u8 = 0;
        for i in 0..self.length {
            sum = sum.wrapping_add(unsafe { *ptr.add(i as usize) });
        }
        sum == 0
    }
}

pub struct AcpiTables {
    pub rsdt: Option<NonNull<SdtHeader>>,
    pub xsdt: Option<NonNull<SdtHeader>>,
}

impl AcpiTables {
    pub const fn new() -> Self {
        Self {
            rsdt: None,
            xsdt: None,
        }
    }

    /// Tenta localizar a tabela raiz (XSDT ou RSDT)
    pub fn init(&mut self, rsdp_addr: u64) {
        // Mapear RSDP (Isso deve ser feito com cuidado em HHDM)
        let rsdp = unsafe { &*(rsdp_addr as *const Rsdp) };

        if rsdp.revision >= 2 {
            let rsdp_ext = unsafe { &*(rsdp_addr as *const RsdpExtended) };
            crate::kdebug!("(ACPI) Localizado XSDT em:", rsdp_ext.xsdt_address);
            self.xsdt = NonNull::new(rsdp_ext.xsdt_address as *mut SdtHeader);
        } else {
            crate::kdebug!("(ACPI) Localizado RSDT em:", rsdp.rsdt_address as u64);
            self.rsdt = NonNull::new(rsdp.rsdt_address as *mut SdtHeader);
        }
    }

    /// Busca uma tabela específica pela assinatura (ex: "APIC", "FACP")
    pub fn find_table(&self, signature: &[u8; 4]) -> Option<NonNull<SdtHeader>> {
        if let Some(xsdt_ptr) = self.xsdt {
            let xsdt = unsafe { xsdt_ptr.as_ref() };
            let entries = (xsdt.length - core::mem::size_of::<SdtHeader>() as u32) / 8;
            let ptr = unsafe {
                (xsdt_ptr.as_ptr() as *const u8).add(core::mem::size_of::<SdtHeader>())
                    as *const u64
            };

            for i in 0..entries {
                let table_ptr = unsafe { *ptr.add(i as usize) as *const SdtHeader };
                let table = unsafe { &*table_ptr };
                if &table.signature == signature {
                    return NonNull::new(table_ptr as *mut SdtHeader);
                }
            }
        } else if let Some(rsdt_ptr) = self.rsdt {
            let rsdt = unsafe { rsdt_ptr.as_ref() };
            let entries = (rsdt.length - core::mem::size_of::<SdtHeader>() as u32) / 4;
            let ptr = unsafe {
                (rsdt_ptr.as_ptr() as *const u8).add(core::mem::size_of::<SdtHeader>())
                    as *const u32
            };

            for i in 0..entries {
                let table_ptr = unsafe { *ptr.add(i as usize) as *const SdtHeader };
                let table = unsafe { &*table_ptr };
                if &table.signature == signature {
                    return NonNull::new(table_ptr as *mut SdtHeader);
                }
            }
        }
        None
    }
}
