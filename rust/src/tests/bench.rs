use std::process::{Command, Child};
use std::thread::sleep;
use std::time::Duration;
use std::env;
use std::io;

fn main() -> io::Result<()> {
    // Duration in milliseconds (default 30000 ms = 30 seconds)
    let duration_ms: u64 = env::var("DURATION_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30000);

    println!("Starting listeners sequentially…");

    // --- TypeScript ---
    run_listener("ts_helius_http", "ts", &["npm", "run", "helius:http"], duration_ms)?;
    run_listener("ts_helius_wss", "ts", &["npm", "run", "helius:wss"], duration_ms)?;
    run_listener("ts_native", "ts", &["npm", "run", "native"], duration_ms)?;

    // --- Rust ---
    run_listener("rust_helius", "rust", &["cargo", "run", "--bin", "helius"], duration_ms)?;
    run_listener("rust_native", "rust", &["cargo", "run", "--bin", "native"], duration_ms)?;

    println!("\nAll done ✅");
    Ok(())
}

fn run_listener(label: &str, dir: &str, cmd: &[&str], duration_ms: u64) -> io::Result<()> {
    println!("\n=== {label} ===");
    println!("→ entering: {dir}");
    std::env::set_current_dir(dir)?;

    println!("→ current path: {:?}", std::env::current_dir()?);
    println!("→ starting: {:?}", cmd.join(" "));

    // Spawn listener
    let mut child = spawn_command(cmd)?;

    println!("→ running for {} ms...", duration_ms);
    sleep(Duration::from_millis(duration_ms));

    println!("→ stopping: {label}");
    easy_kill(&mut child);

    println!("→ returning to root");
    std::env::set_current_dir("..")?;

    Ok(())
}

fn spawn_command(cmd: &[&str]) -> io::Result<Child> {
    let mut command = Command::new(cmd[0]);
    if cmd.len() > 1 {
        command.args(&cmd[1..]);
    }
    command.spawn()
}

fn easy_kill(child: &mut Child) {
    if let Some(id) = child.id() {
        let _ = Command::new("kill")
            .arg("-TERM")
            .arg(id.to_string())
            .status();
    }
    let _ = child.wait();
}
