#[cfg(target_os = "linux")]
pub mod linux {
    use super::{MemoryRegion, PatternError};
    use std::fs;
    use std::os::unix::fs::FileExt;

    pub struct PlatformScanner {
        pid: u32,
    }

    impl PlatformScanner {
        pub fn attach(pid: u32) -> Result<Self, PatternError> {
            let mem_path = format!("/proc/{}/mem", pid);
            if !std::path::Path::new(&mem_path).exists() {
                return Err(PatternError::ProcessError(
                    format!("Processus {} introuvable", pid)
                ));
            }

            Ok(PlatformScanner { pid })
        }

        pub fn get_memory_regions(&self) -> Vec<MemoryRegion> {
            let mut regions = Vec::new();
            let maps_path = format!("/proc/{}/maps", self.pid);

            if let Ok(content) = fs::read_to_string(maps_path) {
                for line in content.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() < 2 {
                        continue;
                    }

                    let addrs: Vec<&str> = parts[0].split('-').collect();
                    if addrs.len() != 2 {
                        continue;
                    }

                    let perms = parts[1];
                    // Seulement les régions lisibles
                    if !perms.starts_with('r') {
                        continue;
                    }

                    if let (Ok(start), Ok(end)) = (
                        usize::from_str_radix(addrs[0], 16),
                        usize::from_str_radix(addrs[1], 16),
                    ) {
                        regions.push(MemoryRegion {
                            start,
                            size: end - start,
                        });
                    }
                }
            }

            regions
        }

        pub fn read_memory(&self, region: &MemoryRegion) -> Option<Vec<u8>> {
            let mem_path = format!("/proc/{}/mem", self.pid);
            let file = fs::File::open(mem_path).ok()?;

            let mut buffer = vec![0u8; region.size];
            file.read_exact_at(&mut buffer, region.start as u64).ok()?;

            Some(buffer)
        }
    }
}