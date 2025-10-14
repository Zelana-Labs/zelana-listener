use std::env;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};
use crossbeam_channel as xch;

#[cfg(unix)]
use nix::sys::signal::{kill, Signal};
#[cfg(unix)]
use nix::unistd::Pid;

fn main() -> io::Result<()> {
    let duration_ms: u64 = env::var("DURATION_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(15_000);
    
    let pre_tx_delay_ms: u64 = env::var("PRE_TX_DELAY_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5_000);

    println!("\n╔════════════════════════════════════════════════════════╗");
    println!("║         Solana Listener Benchmark Suite               ║");
    println!("╚════════════════════════════════════════════════════════╝");
    println!("\n⚙️  Configuration:");
    println!("   • Timeout: {} ms", duration_ms);
    println!("   • Pre-TX delay: {} ms", pre_tx_delay_ms);
    println!("   • Detection keyword: 'DETECTED' in stdout");
    println!();

    let project_root: PathBuf = env::current_dir()?.parent().unwrap().to_path_buf();
    let addr = "CSg4fcG4WqaVgTE33gzquXYGKAuZpikNWKQ4P4y71kke";

    let mut results = Vec::new();

    // TypeScript listeners
    results.push(run_listener(
        &project_root,
        "TypeScript (Helius HTTP)",
        "ts",
        &["npm", "run", "helius:http"],
        duration_ms,
        addr,
        pre_tx_delay_ms,
    )?);
    
    results.push(run_listener(
        &project_root,
        "TypeScript (Helius WSS)",
        "ts",
        &["npm", "run", "helius:wss"],
        duration_ms,
        addr,
        pre_tx_delay_ms,
    )?);
    
    results.push(run_listener(
        &project_root,
        "TypeScript (Native)",
        "ts",
        &["npm", "run", "native"],
        duration_ms,
        addr,
        pre_tx_delay_ms,
    )?);

    // Rust listeners
    results.push(run_listener(
        &project_root,
        "Rust (Helius)",
        "rust",
        &["target/debug/helius"],
        duration_ms,
        addr,
        pre_tx_delay_ms,
    )?);
    
    results.push(run_listener(
        &project_root,
        "Rust (Native)",
        "rust",
        &["target/debug/native"],
        duration_ms,
        addr,
        pre_tx_delay_ms,
    )?);

    // Print summary
    println!("\n╔════════════════════════════════════════════════════════╗");
    println!("║                    Summary Results                     ║");
    println!("╚════════════════════════════════════════════════════════╝\n");
    
    for (name, elapsed) in results {
        match elapsed {
            Some(ms) => println!("✓ {:<30} {:>12.6} ms", name, ms),
            None => println!("✗ {:<30} TIMEOUT", name),
        }
    }
    
    println!("\n✅ Benchmark complete!\n");
    Ok(())
}

fn run_listener(
    project_root: &Path,
    label: &str,
    dir: &str,
    cmd: &[&str],
    duration_ms: u64,
    listen_address: &str,
    pre_tx_delay_ms: u64,
) -> io::Result<(String, Option<f64>)> {
    println!("\n┌─────────────────────────────────────────────────────────");
    println!("│ Testing: {}", label);
    println!("└─────────────────────────────────────────────────────────");

    let saved_dir = env::current_dir()?;
    let target = project_root.join(dir);
    env::set_current_dir(&target)?;
    
    println!("  📁 Working directory: {}", env::current_dir()?.display());
    println!("  🚀 Command: {}", cmd.join(" "));
    println!("  ⏳ Starting listener...");

    let mut child = spawn_command(cmd)?;
    
    // Capture stdout to detect when transaction is found
    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let reader = BufReader::new(stdout);

    let (detected_tx, detected_rx) = xch::unbounded::<Instant>();
    let pr_clone = project_root.to_path_buf();
    let addr = listen_address.to_string();

    // Thread 1: Watch stdout for detection
    std::thread::spawn(move || {
        for line in reader.lines() {
            if let Ok(line) = line {
                println!("    {}", line); // Echo the line
                if line.contains("RECEIVED") {
                    let _ = detected_tx.send(Instant::now());
                    break;
                }
            }
        }
    });

    // Thread 2: Send transaction after delay
    let tx_start_time = std::sync::Arc::new(std::sync::Mutex::new(None));
    let tx_start_clone = tx_start_time.clone();
    
    std::thread::spawn(move || {
        sleep(Duration::from_millis(pre_tx_delay_ms));
        println!("  💸 Sending transaction...");
        let start = Instant::now();
        *tx_start_clone.lock().unwrap() = Some(start);
        let _ = send_tx_from_ts(&pr_clone, &addr);
    });

    // Wait for detection or timeout
    let result = match detected_rx.recv_timeout(Duration::from_millis(duration_ms)) {
        Ok(detected_at) => {
            // Calculate time from TX send to detection
            let tx_start = tx_start_time.lock().unwrap();
            if let Some(start) = *tx_start {
                let elapsed = detected_at.duration_since(start);
                println!("  ✓ Transaction detected in {:.6} ms", elapsed.as_secs_f64() * 1000.0);
                Some(elapsed.as_secs_f64() * 1000.0)
            } else {
                println!("  ⚠️  Detected before TX was sent (unexpected)");
                None
            }
        }
        Err(_) => {
            println!("  ✗ Timeout after {} ms", duration_ms);
            None
        }
    };

    println!("  🛑 Stopping listener...");
    easy_kill(&mut child);
    
    env::set_current_dir(saved_dir)?;

    Ok((label.to_string(), result))
}

fn send_tx_from_ts(project_root: &Path, _to_addr: &str) -> io::Result<()> {
    let ts_dir = project_root.join("ts");
    let status = Command::new("npm")
        .args(["run", "send"])
        .current_dir(&ts_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        eprintln!("  ⚠️  npm run send failed with status: {}", status);
    }
    Ok(())
}

fn spawn_command(cmd: &[&str]) -> io::Result<Child> {
    let (prog, args) = cmd.split_first().expect("empty command");
    
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        Command::new(prog)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped()) // Pipe stdout so we can read it
            .stderr(Stdio::inherit())
            .process_group(0)
            .spawn()
    }
    
    #[cfg(not(unix))]
    {
        Command::new(prog)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped()) // Pipe stdout so we can read it
            .stderr(Stdio::inherit())
            .spawn()
    }
}

fn easy_kill(child: &mut Child) {
    #[cfg(unix)]
    {
        let pid = child.id() as i32;
        let _ = kill(Pid::from_raw(-pid), Signal::SIGTERM);
        std::thread::sleep(std::time::Duration::from_millis(500));
        let _ = kill(Pid::from_raw(-pid), Signal::SIGKILL);
        let _ = child.wait();
    }
    
    #[cfg(not(unix))]
    {
        let _ = child.kill();
        let _ = child.wait();
    }
}