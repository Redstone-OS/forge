//! Tipos fundamentais do sistema

/// Process ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Pid(pub u32);

impl Pid {
    pub const KERNEL: Pid = Pid(0);
    pub const INIT: Pid = Pid(1);
    
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
    
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

/// Thread ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Tid(pub u32);

impl Tid {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
    
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

/// User ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Uid(pub u32);

impl Uid {
    pub const ROOT: Uid = Uid(0);
    
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Group ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Gid(pub u32);

impl Gid {
    pub const ROOT: Gid = Gid(0);
    
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Offset em arquivo
pub type FileOffset = i64;

/// Tamanho
pub type Size = usize;

/// Tempo
pub type Time = i64;
