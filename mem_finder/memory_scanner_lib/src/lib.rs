use std::error::Error;
use std::fmt;

// Modules spécifiques par plateforme
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows::PlatformScanner;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux::PlatformScanner;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use crate::macos::macos::PlatformScanner;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PatternByte {
    Exact(u8),
    Wildcard,
}

#[derive(Debug, Clone)]
pub struct Pattern {
    bytes: Vec<PatternByte>,
}

impl Pattern {
    pub fn from_string(pattern: &str) -> Result<Self, PatternError> {
        let mut bytes = Vec::new();

        for part in pattern.split_whitespace() {
            if part.eq_ignore_ascii_case("xx") || part == "?" {
                bytes.push(PatternByte::Wildcard);
            } else {
                let byte = u8::from_str_radix(part, 16)
                    .map_err(|_| PatternError::InvalidHex(part.to_string()))?;
                bytes.push(PatternByte::Exact(byte));
            }
        }

        if bytes.is_empty() {
            return Err(PatternError::EmptyPattern);
        }

        Ok(Pattern { bytes })
    }

    pub fn from_bytes(bytes: Vec<PatternByte>) -> Result<Self, PatternError> {
        if bytes.is_empty() {
            return Err(PatternError::EmptyPattern);
        }
        Ok(Pattern { bytes })
    }

    pub fn matches(&self, data: &[u8], offset: usize) -> bool {
        if offset + self.bytes.len() > data.len() {
            return false;
        }

        for (i, pattern_byte) in self.bytes.iter().enumerate() {
            match pattern_byte {
                PatternByte::Exact(expected) => {
                    if data[offset + i] != *expected {
                        return false;
                    }
                }
                PatternByte::Wildcard => continue,
            }
        }
        true
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub address: usize,
    pub matched_bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub start: usize,
    pub size: usize,
}

#[derive(Debug)]
pub enum PatternError {
    InvalidHex(String),
    EmptyPattern,
    ProcessError(String),
}

impl fmt::Display for PatternError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PatternError::InvalidHex(s) => write!(f, "Hex invalide: {}", s),
            PatternError::EmptyPattern => write!(f, "Pattern vide"),
            PatternError::ProcessError(s) => write!(f, "Erreur processus: {}", s),
        }
    }
}

impl Error for PatternError {}

pub struct MemoryScanner {
    scanner: PlatformScanner,
}

impl MemoryScanner {
    pub fn attach(pid: u32) -> Result<Self, PatternError> {
        Ok(MemoryScanner {
            scanner: PlatformScanner::attach(pid)?,
        })
    }

    pub fn scan(&self, pattern: &Pattern, max_results: usize) -> Vec<ScanResult> {
        let mut results = Vec::new();
        let regions = self.scanner.get_memory_regions();

        for region in regions {
            if results.len() >= max_results {
                break;
            }

            if let Some(buffer) = self.scanner.read_memory(&region) {
                for i in 0..buffer.len().saturating_sub(pattern.len()) {
                    if pattern.matches(&buffer, i) {
                        let addr = region.start + i;
                        let matched = buffer[i..i + pattern.len()].to_vec();

                        results.push(ScanResult {
                            address: addr,
                            matched_bytes: matched,
                        });

                        if results.len() >= max_results {
                            break;
                        }
                    }
                }
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_from_string() {
        let pattern = Pattern::from_string("22 55 xx 60").unwrap();
        assert_eq!(pattern.len(), 4);
    }

    #[test]
    fn test_pattern_matching() {
        let pattern = Pattern::from_string("22 55 xx 60").unwrap();
        let data = vec![0x22, 0x55, 0xFF, 0x60, 0x00];
        assert!(pattern.matches(&data, 0));
    }

    #[test]
    fn test_pattern_no_match() {
        let pattern = Pattern::from_string("22 55 xx 60").unwrap();
        let data = vec![0x22, 0x54, 0xFF, 0x60, 0x00];
        assert!(!pattern.matches(&data, 0));
    }

    #[test]
    fn test_wildcard_variants() {
        let p1 = Pattern::from_string("22 xx 60").unwrap();
        let p2 = Pattern::from_string("22 ? 60").unwrap();
        assert_eq!(p1.len(), p2.len());
    }
}