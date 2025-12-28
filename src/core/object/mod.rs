//! Sistema de objetos do kernel

pub mod dispatcher;
pub mod handle;
pub mod kobject;
pub mod refcount;
pub mod rights;

pub use handle::Handle;
pub use kobject::KernelObject;
pub use refcount::RefCount;
pub use rights::Rights;
