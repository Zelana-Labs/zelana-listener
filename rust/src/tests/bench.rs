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
        .unwrap_or(30_000);

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
    println!("   • Detection keyword: 'RECEIVED' in stdout");
    println!();

    // Assumes this binary sits inside a workspace subdir; adjust if needed.
    // If not in a workspace, you can set PROJECT_ROOT manually via env.
    let project_root: PathBuf = if let Ok(root) = env::var("PROJECT_ROOT") {
        PathBuf::from(root)
    } else {
        // current_dir() is typically <repo>/bench (or similar)
        // parent() hops back to <repo>
        env::current_dir()?.parent().unwrap_or_else(|| Path::new(".")).to_path_buf()
    };

    // Address your listeners should react to (your programs should use/print "RECEIVED" on detection)
    let addr = env::var("LISTEN_ADDRESS")
        .unwrap_or_else(|_| "CSg4fcG4WqaVgTE33gzquXYGKAuZpikNWKQ4P4y71kke".to_string());

    let mut results = Vec::new();

    // TypeScript listeners
    results.push(run_listener(
        &project_root,
        "TypeScript (Helius HTTP)",
        "ts",
        &["npm", "run", "helius:http"],
        duration_ms,
        &addr,
        pre_tx_delay_ms,
    )?);

    results.push(run_listener(
        &project_root,
        "TypeScript (Helius WSS)",
        "ts",
        &["npm", "run", "helius:wss"],
        duration_ms,
        &addr,
        pre_tx_delay_ms,
    )?);

    results.push(run_listener(
        &project_root,
        "TypeScript (Native)",
        "ts",
        &["npm", "run", "native"],
        duration_ms,
        &addr,
        pre_tx_delay_ms,
    )?);

    // Rust listeners
    results.push(run_listener(
        &project_root,
        "Rust (Helius)",
        "rust",
        &["target/debug/helius"],
        duration_ms,
        &addr,
        pre_tx_delay_ms,
    )?);

    results.push(run_listener(
        &project_root,
        "Rust (Native)",
        "rust",
        &["target/debug/native"],
        duration_ms,
        &addr,
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

    // Channels for async timestamps (capacity=1 to keep only the first event)
    let (detected_tx, detected_rx) = xch::bounded::<Instant>(1);
    let (sent_tx, sent_rx) = xch::bounded::<Instant>(1);

    // Thread 1: Watch stdout for detection
    std::thread::spawn(move || {
        for line in reader.lines() {
            if let Ok(line) = line {
                println!("    {}", line); // Echo the line from child
                if line.contains("RECEIVED") {
                    // Record the instant we saw the detection
                    let _ = detected_tx.send(Instant::now());
                    break;
                }
            } else {
                break; // EOF or error
            }
        }
    });

    // Thread 2: Send transaction after delay
    let pr_clone = project_root.to_path_buf();
    let addr = listen_address.to_string();
    
    std::thread::spawn(move || {
        sleep(Duration::from_millis(pre_tx_delay_ms));
        println!("  💸 Sending transaction...");
        // Start the timer right before invoking the sender (best approximation)
        let tx_start = Instant::now();
        let _ = send_tx_from_ts(&pr_clone, &addr);
        let _ = sent_tx.send(tx_start);
    });

    // Wait until we have BOTH timestamps (or timeout)
    let deadline = Instant::now() + Duration::from_millis(duration_ms);
    let mut sent_at: Option<Instant> = None;
    let mut detected_at: Option<Instant> = None;

    while Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }

        xch::select! {
            recv(sent_rx) -> v => if let Ok(t) = v { sent_at = Some(t); },
            recv(detected_rx) -> v => if let Ok(t) = v { detected_at = Some(t); },
            default(Duration::from_millis(10)) => { /* poll */ }
        }

        if let (Some(s), Some(d)) = (sent_at, detected_at) {
            let elapsed = d.saturating_duration_since(s);
            println!("  ✓ Transaction detected in {:.6} ms", elapsed.as_secs_f64() * 1000.0);
            println!("  🛑 Stopping listener...");
            easy_kill(&mut child);
            env::set_current_dir(saved_dir)?;
            return Ok((label.to_string(), Some(elapsed.as_secs_f64() * 1000.0)));
        }
    }

    // Timeout path
    println!("  ✗ Timeout after {} ms", duration_ms);
    println!("  🛑 Stopping listener...");
    easy_kill(&mut child);
    env::set_current_dir(saved_dir)?;
    Ok((label.to_string(), None))
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
        // Kill the whole process group we spawned
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
