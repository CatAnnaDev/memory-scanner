#[cfg(target_os = "macos")]
pub mod macos {
    use crate::{MemoryRegion, PatternError};

    type MachPort = u32;
    type KernReturn = i32;
    type VmAddress = u64;
    type VmSize = u64;

    const KERN_SUCCESS: KernReturn = 0;
    const VM_PROT_READ: i32 = 0x01;

    #[repr(C)]
    struct VmRegionBasicInfo64 {
        protection: i32,
        max_protection: i32,
        inheritance: u32,
        shared: u32,
        reserved: u32,
        offset: u64,
        behavior: i32,
        user_wired_count: u16,
    }

    unsafe extern "C" {
        fn mach_task_self() -> MachPort;
        fn task_for_pid(target_tport: MachPort, pid: i32, task: *mut MachPort) -> KernReturn;
        fn mach_vm_region(
            target_task: MachPort,
            address: *mut VmAddress,
            size: *mut VmSize,
            flavor: i32,
            info: *mut VmRegionBasicInfo64,
            info_cnt: *mut u32,
            object_name: *mut MachPort,
        ) -> KernReturn;
        fn mach_vm_read_overwrite(
            target_task: MachPort,
            address: VmAddress,
            size: VmSize,
            data: VmAddress,
            outsize: *mut VmSize,
        ) -> KernReturn;
        fn mach_port_deallocate(task: MachPort, name: MachPort) -> KernReturn;
    }

    pub struct PlatformScanner {
        task: MachPort,
    }

    impl PlatformScanner {
        pub fn attach(pid: u32) -> Result<Self, PatternError> {
            unsafe {
                let mut task: MachPort = 0;
                let kr = task_for_pid(mach_task_self(), pid as i32, &mut task);

                if kr != KERN_SUCCESS {
                    eprintln!("DEBUG: task_for_pid returned: {}", kr);
                    eprintln!("DEBUG: mach_task_self: {}", mach_task_self());
                    eprintln!("DEBUG: pid: {}", pid);

                    return Err(PatternError::ProcessError(
                        format!("task_for_pid échoué: code {}", kr)
                    ));
                }

                Ok(PlatformScanner { task })
            }
        }

        pub fn get_memory_regions(&self) -> Vec<MemoryRegion> {
            let mut regions = Vec::new();
            let mut address: VmAddress = 0;
            let flavor = 9;

            unsafe {
                loop {
                    let mut size: VmSize = 0;
                    let mut info: VmRegionBasicInfo64 = std::mem::zeroed();
                    let mut count = (std::mem::size_of::<VmRegionBasicInfo64>() / 4) as u32;
                    let mut object_name: MachPort = 0;

                    let kr = mach_vm_region(
                        self.task,
                        &mut address,
                        &mut size,
                        flavor,
                        &mut info,
                        &mut count,
                        &mut object_name,
                    );

                    if kr != KERN_SUCCESS {
                        break;
                    }

                    if info.protection & VM_PROT_READ != 0 {
                        regions.push(MemoryRegion {
                            start: address as usize,
                            size: size as usize,
                        });
                    }

                    address += size;

                    if address == 0 {
                        break;
                    }
                }
            }

            regions
        }

        pub fn read_memory(&self, region: &MemoryRegion) -> Option<Vec<u8>> {
            let mut buffer = vec![0u8; region.size];
            let mut read_size: VmSize = 0;

            unsafe {
                let kr = mach_vm_read_overwrite(
                    self.task,
                    region.start as VmAddress,
                    region.size as VmSize,
                    buffer.as_mut_ptr() as VmAddress,
                    &mut read_size,
                );

                if kr == KERN_SUCCESS && read_size > 0 {
                    buffer.truncate(read_size as usize);
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
                if self.task != 0 {
                    mach_port_deallocate(mach_task_self(), self.task);
                }
            }
        }
    }
}