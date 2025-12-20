//! Virtual Filesystem (VFS)
//!
//! Abstração de filesystem estilo Linux.
//!
//! # Arquitetura
//! - Inodes: Representam arquivos/diretórios
//! - Dentries: Entradas de diretório (cache)
//! - Files: Arquivos abertos
//! - Superblocks: Informações de filesystem
//! - Mount points: Pontos de montagem
//!
//! # Filesystems Suportados
//! - DevFS: Device files (/dev)
//! - ProcFS: Process info (/proc)
//! - SysFS: System info (/sys)
//! - TmpFS: Temporary files (RAM)
//! - FAT32: Compatibilidade
//! - TAR: InitRAMFS (read-only)
//!
//! # TODOs
//! - TODO(prioridade=alta, versão=v1.0): Implementar VFS completo
//! - TODO(prioridade=alta, versão=v1.0): Migrar código de scheme/
//! - TODO(prioridade=média, versão=v2.0): Adicionar Ext4
//! - TODO(prioridade=baixa, versão=v2.0): Adicionar ISO9660
//! - TODO(prioridade=baixa, versão=v2.0): Adicionar XFS
//! - TODO(prioridade=baixa, versão=v2.0): Adicionar F2FS
//! - TODO(prioridade=baixa, versão=v3.0): Adicionar Network FS
//! - TODO(prioridade=baixa, versão=v3.0): Adicionar NTFS

pub mod devfs;
pub mod fat32;
pub mod procfs;
pub mod sysfs;
pub mod tar;
pub mod tarfs;
pub mod tmpfs;
pub mod vfs;

#[cfg(test)]
pub mod tests;
