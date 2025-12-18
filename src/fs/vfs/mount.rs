//! Mount Points - Pontos de montagem de filesystems

use super::FilesystemType;

/// Mount Point
#[derive(Clone, Copy)]
pub struct MountPoint {
    /// Path de montagem
    path: [u8; 32],
    /// Tamanho do path
    path_len: usize,
    /// Tipo de filesystem
    pub fs_type: FilesystemType,
}

impl MountPoint {
    /// Cria um mount point vazio
    pub const fn empty() -> Self {
        Self {
            path: [0; 32],
            path_len: 0,
            fs_type: FilesystemType::DevFS,
        }
    }

    /// Cria um novo mount point
    pub fn new(path: &str, fs_type: FilesystemType) -> Result<Self, &'static str> {
        if path.len() > 32 {
            return Err("Path too long");
        }

        let mut mount = Self::empty();
        mount.path[..path.len()].copy_from_slice(path.as_bytes());
        mount.path_len = path.len();
        mount.fs_type = fs_type;

        Ok(mount)
    }

    /// Verifica se o path começa com este mount point
    pub fn strip_prefix<'a>(&self, path: &'a str) -> Option<&'a str> {
        let mount_path = core::str::from_utf8(&self.path[..self.path_len]).ok()?;

        if path.starts_with(mount_path) {
            // Retorna o path relativo
            let relative = &path[mount_path.len()..];
            Some(relative.trim_start_matches('/'))
        } else {
            None
        }
    }

    /// Retorna o path de montagem
    pub fn path(&self) -> Result<&str, &'static str> {
        core::str::from_utf8(&self.path[..self.path_len]).map_err(|_| "Invalid UTF-8")
    }
}
