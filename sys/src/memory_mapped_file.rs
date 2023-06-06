#![allow(dead_code)]

use super::unique_resource::{ResourceDeleter, UniqueResource};
use std::path::Path;

#[cfg(unix)]
mod unix {
    use libc::{c_int, c_void, close, munmap};

    #[derive(Default)]
    pub struct UnixFileDescriptorDeleter {}

    impl super::ResourceDeleter<c_int> for UnixFileDescriptorDeleter {
        fn null() -> c_int {
            -1
        }

        fn destroy(&mut self, fd: c_int) {
            unsafe {
                close(fd);
            }
        }
    }

    pub type UniqueOSFileDescriptor = super::UniqueResource<c_int, UnixFileDescriptorDeleter>;

    #[derive(Default)]
    pub struct MappedMemoryHandleDeleter(pub usize);

    impl super::ResourceDeleter<*mut c_void> for MappedMemoryHandleDeleter {
        fn null() -> *mut c_void {
            std::ptr::null_mut()
        }

        fn destroy(&mut self, ptr: *mut c_void) {
            unsafe {
                munmap(ptr, self.0);
            }
        }
    }

    pub type UniqueMappedMemoryHandle =
        super::UniqueResource<*mut c_void, MappedMemoryHandleDeleter>;
} // mod unix

#[cfg(windows)]
mod win32 {
    use std::{os::windows::prelude::*, ptr::null_mut};
    use winapi::{
        shared::minwindef::LPVOID,
        um::{
            handleapi::{CloseHandle, INVALID_HANDLE_VALUE},
            memoryapi::UnmapViewOfFile,
            winnt::HANDLE,
        },
    };

    pub fn win_str(s: &str) -> Vec<u16> {
        std::ffi::OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    pub fn path_to_win_str<P: AsRef<std::path::Path>>(p: P) -> Vec<u16> {
        p.as_ref()
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    #[derive(Default)]
    pub struct WindowsFileDeleter {}

    impl super::ResourceDeleter<HANDLE> for WindowsFileDeleter {
        fn null() -> HANDLE {
            INVALID_HANDLE_VALUE
        }

        fn destroy(&mut self, fd: HANDLE) {
            unsafe {
                CloseHandle(fd);
            }
        }
    }

    pub type UniqueOSFileDescriptor = super::UniqueResource<HANDLE, WindowsFileDeleter>;

    #[derive(Default)]
    pub struct FileMappingDeleter {}

    impl super::ResourceDeleter<HANDLE> for FileMappingDeleter {
        fn null() -> HANDLE {
            null_mut()
        }

        fn destroy(&mut self, file_mapping: HANDLE) {
            unsafe {
                CloseHandle(file_mapping);
            }
        }
    }

    pub type UniqueFileMapping = super::UniqueResource<HANDLE, FileMappingDeleter>;

    #[derive(Default)]
    pub struct MappedMemoryHandleDeleter {}

    impl super::ResourceDeleter<LPVOID> for MappedMemoryHandleDeleter {
        fn null() -> LPVOID {
            null_mut()
        }

        fn destroy(&mut self, res: LPVOID) {
            unsafe {
                UnmapViewOfFile(res);
            }
        }
    }

    pub type UniqueMappedMemoryHandle = super::UniqueResource<LPVOID, MappedMemoryHandleDeleter>;
} // mod win32

#[cfg(unix)]
use unix::{UniqueMappedMemoryHandle, UniqueOSFileDescriptor};

#[cfg(windows)]
use win32::{UniqueMappedMemoryHandle, UniqueOSFileDescriptor};

/// A file mapped into the memory of the process. Contents can be accessed as
/// a byte slice. Read-only access.
pub struct MemoryMappedFile {
    /// starting address where file was mapped in memory
    memory: UniqueMappedMemoryHandle,
    /// handle to the file
    file_handle: UniqueOSFileDescriptor,
    /// length in bytes of the mapping
    bytes: usize,
}

impl MemoryMappedFile {
    /// Construct by mapping the specified file into memory
    #[cfg(unix)]
    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<MemoryMappedFile> {
        use libc::{mmap, open, MAP_PRIVATE, O_RDONLY, PROT_READ};
        use std::{
            ffi::CString,
            io::{Error, ErrorKind},
            ptr::null_mut,
        };

        let metadata = std::fs::metadata(&path)?;

        path.as_ref()
            .to_str()
            .ok_or(Error::new(
                ErrorKind::InvalidData,
                "failed to convert path to string",
            ))
            .and_then(|str_path| {
                //
                // convert path to C-string
                CString::new(str_path.as_bytes())
                    .map_err(|_| {
                        Error::new(ErrorKind::InvalidData, "failed to convert path to C-string")
                    })
                    .and_then(|cstr_path| {
                        //
                        // open file
                        UniqueOSFileDescriptor::new(unsafe {
                            open(cstr_path.as_c_str().as_ptr(), O_RDONLY)
                        })
                        .ok_or(Error::last_os_error())
                        .and_then(|ufd| {
                            //
                            // map into memory
                            UniqueMappedMemoryHandle::new_with_deleter(
                                unsafe {
                                    mmap(
                                        null_mut(),
                                        metadata.len() as usize,
                                        PROT_READ,
                                        MAP_PRIVATE,
                                        *ufd,
                                        0,
                                    )
                                },
                                unix::MappedMemoryHandleDeleter(metadata.len() as usize),
                            )
                            .ok_or(Error::last_os_error())
                            .and_then(|ummap| {
                                Ok(MemoryMappedFile {
                                    memory: ummap,
                                    file_handle: ufd,
                                    bytes: metadata.len() as usize,
                                })
                            })
                        })
                    })
            })
    }

    /// Construct by mapping the specified file into memory
    #[cfg(windows)]
    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<MemoryMappedFile> {
        use std::{io::Error, ptr::null_mut};
        use winapi::um::{
            fileapi::{CreateFileW, OPEN_EXISTING},
            memoryapi::{CreateFileMappingW, MapViewOfFile, FILE_MAP_READ},
            winnt::{FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, GENERIC_READ, PAGE_READONLY},
        };

        let win_path = win32::path_to_win_str(&path);
        let metadata = std::fs::metadata(&path)?;

        UniqueOSFileDescriptor::new(unsafe {
            CreateFileW(
                win_path.as_ptr(),
                GENERIC_READ,
                FILE_SHARE_READ,
                null_mut(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                null_mut(),
            )
        })
        .ok_or(Error::last_os_error())
        .and_then(|file_handle| {
            // Use the file handle to create a file mapping object. This gets
            // destroyed once we leave the closure, but it is not  a problem
            // since the mapping of the file stays valid untill all mapped views
            // are closed.
            win32::UniqueFileMapping::new(unsafe {
                CreateFileMappingW(*file_handle, null_mut(), PAGE_READONLY, 0, 0, null_mut())
            })
            .ok_or(Error::last_os_error())
            .and_then(|file_mapping| {
                // finally we can map the view of file in the process memory
                UniqueMappedMemoryHandle::new(unsafe {
                    MapViewOfFile(*file_mapping, FILE_MAP_READ, 0, 0, 0)
                })
                .ok_or(Error::last_os_error())
                .and_then(|memory| {
                    Ok(MemoryMappedFile {
                        memory,
                        file_handle,
                        bytes: metadata.len() as usize,
                    })
                })
            })
        })
    }

    /// Returns the length in bytes of the file that was mapped in memory.
    pub fn len(&self) -> usize {
        self.bytes
    }

    /// Returns a slice spanning the contents of the file that was mapped in
    /// memory
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(*self.memory as *const u8, self.bytes) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{ffi::CStr, fs::File, io::prelude::*, os::raw::c_char, path::Path};

    #[test]
    fn test_memory_mapped_file() {
        let mmfile = MemoryMappedFile::new(Path::new("non-existing-test-file.txt"));
        assert!(mmfile.is_err());

        let txt = b"A memory mapped file\0";
        {
            let mut f = File::create("test.txt").unwrap();
            f.write_all(txt).unwrap();
        }

        let mmfile = MemoryMappedFile::new(Path::new("test.txt"));
        assert!(!mmfile.is_err());
        let mmfile = mmfile.unwrap();

        unsafe {
            let m = CStr::from_ptr(mmfile.as_slice().as_ptr() as *const c_char);
            let org = CStr::from_bytes_with_nul(b"A memory mapped file\0").unwrap();
            assert_eq!(m, org);
        }
    }
}
