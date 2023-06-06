#[macro_use]
mod unique_resource;

pub use unique_resource::{ResourceDeleter, UniqueResource};

mod scope_guard;
pub use scope_guard::ScopeGuard;

mod memory_mapped_file;
pub use self::memory_mapped_file::MemoryMappedFile;
