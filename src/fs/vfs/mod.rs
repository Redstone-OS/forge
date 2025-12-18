//! VFS - Virtual Filesystem
//!
//! Sistema de arquivos virtual que integra todos os filesystems.
//!
//! # Implementação Básica
//! Fornece interface unificada para DevFS, FAT32, ProcFS, SysFS e TmpFS.

#![allow(dead_code)]

pub mod file;
pub mod inode;
pub mod mount;

pub use file::*;
pub use inode::*;
pub use mount::*;

use crate::fs::{devfs, procfs, sysfs, tmpfs};

/// Tipo de filesystem
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesystemType {
    DevFS,
    ProcFS,
    SysFS,
    TmpFS,
    FAT32,
}

/// VFS - Virtual Filesystem
pub struct VFS {
    /// Mount points
    mounts: [MountPoint; 8],
    /// Número de mounts
    mount_count: usize,

    /// Filesystems
    pub devfs: devfs::DevFS,
    pub procfs: procfs::ProcFS,
    pub sysfs: sysfs::SysFS,
    pub tmpfs: tmpfs::TmpFS,
}

impl VFS {
    /// Cria uma nova instância do VFS
    pub fn new() -> Self {
        let mut vfs = Self {
            mounts: [MountPoint::empty(); 8],
            mount_count: 0,
            devfs: devfs::DevFS::new(),
            procfs: procfs::ProcFS::new(),
            sysfs: sysfs::SysFS::new(),
            tmpfs: tmpfs::TmpFS::default(),
        };

        // Montar filesystems padrão
        let _ = vfs.mount("/dev", FilesystemType::DevFS);
        let _ = vfs.mount("/proc", FilesystemType::ProcFS);
        let _ = vfs.mount("/sys", FilesystemType::SysFS);
        let _ = vfs.mount("/tmp", FilesystemType::TmpFS);

        vfs
    }

    /// Monta um filesystem
    pub fn mount(&mut self, path: &str, fs_type: FilesystemType) -> Result<(), &'static str> {
        if self.mount_count >= self.mounts.len() {
            return Err("Too many mounts");
        }

        self.mounts[self.mount_count] = MountPoint::new(path, fs_type)?;
        self.mount_count += 1;
        Ok(())
    }

    /// Resolve um path para filesystem
    fn resolve_path<'a>(&self, path: &'a str) -> Option<(FilesystemType, &'a str)> {
        for i in 0..self.mount_count {
            let mount = &self.mounts[i];
            if let Some(relative_path) = mount.strip_prefix(path) {
                return Some((mount.fs_type, relative_path));
            }
        }
        None
    }

    /// Abre um arquivo
    pub fn open(&self, path: &str) -> Result<VFSFile, &'static str> {
        let (fs_type, _relative_path) = self.resolve_path(path).ok_or("Path not found")?;

        // Por enquanto, retorna um file handle básico
        // TODO: Armazenar path de forma diferente (copiar para buffer estático ou usar índice)
        Ok(VFSFile {
            path: "", // Temporário - precisa refatorar para não usar &'static str
            fs_type,
            offset: 0,
        })
    }

    /// Lê um arquivo
    pub fn read(&self, file: &mut VFSFile, buf: &mut [u8]) -> Result<usize, &'static str> {
        match file.fs_type {
            FilesystemType::ProcFS => {
                // Extrair nome do arquivo de /proc/nome
                let name = file.path.strip_prefix("/proc/").unwrap_or(file.path);
                let content = self.procfs.read(name).ok_or("File not found")?;
                let bytes = content.as_bytes();

                let start = file.offset as usize;
                if start >= bytes.len() {
                    return Ok(0); // EOF
                }

                let end = (start + buf.len()).min(bytes.len());
                let len = end - start;
                buf[..len].copy_from_slice(&bytes[start..end]);
                file.offset += len as u64;
                Ok(len)
            }
            FilesystemType::SysFS => {
                let path = file.path.strip_prefix("/sys/").unwrap_or(file.path);
                let content = self.sysfs.read(path).ok_or("File not found")?;
                let bytes = content.as_bytes();

                let start = file.offset as usize;
                if start >= bytes.len() {
                    return Ok(0);
                }

                let end = (start + buf.len()).min(bytes.len());
                let len = end - start;
                buf[..len].copy_from_slice(&bytes[start..end]);
                file.offset += len as u64;
                Ok(len)
            }
            FilesystemType::TmpFS => {
                let name = file.path.strip_prefix("/tmp/").unwrap_or(file.path);
                let data = self.tmpfs.read_file(name)?;

                let start = file.offset as usize;
                if start >= data.len() {
                    return Ok(0);
                }

                let end = (start + buf.len()).min(data.len());
                let len = end - start;
                buf[..len].copy_from_slice(&data[start..end]);
                file.offset += len as u64;
                Ok(len)
            }
            _ => Err("Filesystem not implemented"),
        }
    }

    /// Escreve em um arquivo (apenas TmpFS)
    pub fn write(&mut self, file: &VFSFile, data: &[u8]) -> Result<usize, &'static str> {
        match file.fs_type {
            FilesystemType::TmpFS => {
                let name = file.path.strip_prefix("/tmp/").unwrap_or(file.path);
                self.tmpfs.create_file(name, data)?;
                Ok(data.len())
            }
            _ => Err("Filesystem is read-only"),
        }
    }
}

impl Default for VFS {
    fn default() -> Self {
        Self::new()
    }
}

// TODO(prioridade=alta, versão=v1.0): Implementar operações completas
// - readdir() - Listar diretórios
// - stat() - Informações de arquivo
// - mkdir() - Criar diretórios
// - unlink() - Remover arquivos

// TODO(prioridade=média, versão=v1.0): Implementar cache
// - Dentry cache
// - Inode cache
// - Page cache

// TODO(prioridade=baixa, versão=v2.0): Implementar permissões
// - Verificação de permissões
// - Ownership (uid/gid)
