#[cfg(target_os = "windows")]
pub mod windows {
    use super::{MemoryRegion, PatternError};
    use winapi::um::memoryapi::{ReadProcessMemory, VirtualQueryEx};
    use winapi::um::winnt::{MEMORY_BASIC_INFORMATION, MEM_COMMIT, PAGE_GUARD, PAGE_NOACCESS};
    use winapi::shared::minwindef::LPVOID;
    use winapi::um::processthreadsapi::OpenProcess;
    use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
    use winapi::um::handleapi::CloseHandle;
    use std::ptr;

    pub struct PlatformScanner {
        process_handle: *mut std::ffi::c_void,
    }

    impl PlatformScanner {
        pub fn attach(pid: u32) -> Result<Self, PatternError> {
            unsafe {
                let handle = OpenProcess(
                    PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                    0,
                    pid,
                );

                if handle.is_null() {
                    return Err(PatternError::ProcessError(
                        "Impossible d'ouvrir le processus (admin requis?)".to_string()
                    ));
                }

                Ok(PlatformScanner {
                    process_handle: handle,
                })
            }
        }

        pub fn get_memory_regions(&self) -> Vec<MemoryRegion> {
            let mut regions = Vec::new();
            let mut address: usize = 0;

            unsafe {
                loop {
                    let mut mbi: MEMORY_BASIC_INFORMATION = std::mem::zeroed();
                    let result = VirtualQueryEx(
                        self.process_handle,
                        address as LPVOID,
                        &mut mbi,
                        std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                    );

                    if result == 0 {
                        break;
                    }

                    if mbi.State == MEM_COMMIT
                        && mbi.Protect & PAGE_GUARD == 0
                        && mbi.Protect & PAGE_NOACCESS == 0
                    {
                        regions.push(MemoryRegion {
                            start: mbi.BaseAddress as usize,
                            size: mbi.RegionSize,
                        });
                    }

                    address = (mbi.BaseAddress as usize) + mbi.RegionSize;

                    if address == 0 {
                        break;
                    }
                }
            }

            regions
        }

        pub fn read_memory(&self, region: &MemoryRegion) -> Option<Vec<u8>> {
            let mut buffer = vec![0u8; region.size];
            let mut bytes_read = 0;

            unsafe {
                let success = ReadProcessMemory(
                    self.process_handle,
                    region.start as LPVOID,
                    buffer.as_mut_ptr() as LPVOID,
                    region.size,
                    &mut bytes_read,
                );

                if success != 0 && bytes_read > 0 {
                    buffer.truncate(bytes_read);
                    Some(buffer)
                } else {
                    None
                }
            }
        }
    }

    impl Drop for PlatformScanner {
        fn drop(&mut self) {
            unsafe {
                if !self.process_handle.is_null() {
                    CloseHandle(self.process_handle);
                }
            }
        }
    }
}