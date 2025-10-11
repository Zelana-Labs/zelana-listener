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
        .unwrap_or(10000);

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
  // ✅ Use ? to extract the PathBuf from Result
    let mut current = std::env::current_dir()?;
    println!("Current dir before change: {:?}", current);

    // Go one directory up
    current.pop();

    // Then push the target directory (from argument)
    current.push(dir);

    let _ = std::env::set_current_dir(&current);

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

fn spawn_command(cmd: &[&str]) -> std::io::Result<std::process::Child> {
    let mut c = std::process::Command::new("cmd");
    c.args(&["/C"]).args(cmd); // use Windows shell
    c.stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit());
    c.spawn()
}


fn easy_kill(child: &mut std::process::Child) {
    let _ = child.kill(); // sends SIGKILL on Unix, TerminateProcess on Windows
    let _ = child.wait();
}

