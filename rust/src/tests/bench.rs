use std::thread::sleep;
use std::time::Duration;
use std::env;
use std::io;
use std::process::{Command, Child, Stdio};

fn main() -> io::Result<()> {
    // Duration in milliseconds (default 10000 ms = 10 seconds)
    let duration_ms: u64 = env::var("DURATION_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10_000);

    println!("Starting listeners sequentially…");

    // --- TypeScript ---
    run_listener("ts_helius_http", "ts", &["npm", "run", "helius:http"], duration_ms)?;
    run_listener("ts_helius_wss",  "ts", &["npm", "run", "helius:wss"],  duration_ms)?;
    run_listener("ts_native",      "ts", &["npm", "run", "native"],      duration_ms)?;

    // --- Rust (run compiled binaries directly) ---
    run_listener("rust_helius", "rust", &["target/debug/helius"], duration_ms)?;
    run_listener("rust_native", "rust", &["target/debug/native"], duration_ms)?;

    println!("\nAll done ✅");
    Ok(())
}

fn run_listener(label: &str, dir: &str, cmd: &[&str], duration_ms: u64) -> io::Result<()> {
    println!("\n=== {label} ===");
    // save where we started (this will be .../zelana-listener/rust)
    let saved_dir = std::env::current_dir()?;
    println!("Current dir before change: {:?}", saved_dir);

    // go to <parent-of-saved>/<dir>  => project_root/<dir>
    let mut target = saved_dir.clone();
    target.pop();          // parent of rust/  => project root
    target.push(dir);      // project_root/ts or project_root/rust
    std::env::set_current_dir(&target)?;

    println!("→ current path: {:?}", std::env::current_dir()?);
    println!("→ starting: {}", cmd.join(" "));

    let mut child = spawn_command(cmd)?;

    println!("→ running for {} ms…", duration_ms);
    std::thread::sleep(std::time::Duration::from_millis(duration_ms));

    println!("→ stopping: {label}");
    easy_kill(&mut child);

    // restore exactly where we started this call (rust/)
    println!("→ returning to {:?}", saved_dir);
    std::env::set_current_dir(saved_dir)?;

    Ok(())
}


// Linux-only: run program directly (no Windows cmd)
fn spawn_command(cmd: &[&str]) -> io::Result<Child> {
    let (prog, args) = cmd.split_first().expect("empty command");
    let mut c = Command::new(prog);
    c.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    c.spawn()
}

fn easy_kill(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}
