use memory_scanner_lib::{MemoryScanner, Pattern};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Memory Pattern Scanner ===\n");

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <PID> <pattern>", args[0]);
        eprintln!("Exemple: {} 1234 \"48 8B xx 48 89 xx\"", args[0]);
        std::process::exit(1);
    }

    let pid: u32 = args[1].parse()?;
    let pattern_str = &args[2];

    let pattern = Pattern::from_string(pattern_str)?;
    println!("Pattern: {}", pattern_str);
    println!("Attachement au PID: {}\n", pid);

    match MemoryScanner::attach(pid) {
        Ok(scanner) => {
            println!("Scan en cours...\n");
            let results = scanner.scan(&pattern, 100);

            println!("Trouvé {} résultat(s):", results.len());
            for (i, result) in results.iter().enumerate() {
                print!("  [{}] 0x{:016X} - ", i + 1, result.address);
                for byte in &result.matched_bytes {
                    print!("{:02X} ", byte);
                }
                println!();
            }
        }
        Err(e) => {
            eprintln!("❌ Erreur: {}", e);

            #[cfg(target_os = "windows")]
            eprintln!("\n💡 Lancez en tant qu'administrateur");

            #[cfg(target_os = "macos")]
            eprintln!("\n💡 Utilisez: sudo cargo run -- {} \"{}\"", pid, pattern_str);

            #[cfg(target_os = "linux")]
            {
                eprintln!("\n💡 Solutions possibles:");
                eprintln!("   - sudo cargo run -- {} \"{}\"", pid, pattern_str);
                eprintln!("   - echo 0 | sudo tee /proc/sys/kernel/yama/ptrace_scope");
            }
        }
    }

    Ok(())
}