use std::env;

mod error;
mod memory;
mod ptrace;

pub use error::{Error, Result};

use memory::Memory;
use ptrace::Ptrace;

fn get_program_name(pid: u32) -> Result<String> {
    let mut invocation = std::fs::read_to_string(format!("/proc/{}/cmdline", pid))?;

    if let Some(nullbyte) = invocation.find('\0') {
        invocation.truncate(nullbyte);
    }

    Ok(invocation)
}

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let pid: u32 = args
        .next()
        .expect("Usage: dump-memory PID [OUTPUT_DIR]")
        .parse()?;

    let output_dir = std::path::PathBuf::from(if let Some(dir) = args.next() {
        dir
    } else {
        let invocation = get_program_name(pid)?;
        format!("{}-{}", invocation, pid)
    });

    std::fs::create_dir_all(&output_dir)?;
    let mut process = Ptrace::new(pid)?;
    let memory = Memory::from_pid(pid)?;

    let mut buffer = Vec::new();
    for region in memory.iter() {
        buffer.clear();
        if let Err(e) = process.dump(region, &mut buffer) {
            eprintln!(
                "Could not dump region {:x}-{:x} {} ({}): {}",
                region.start,
                region.end,
                region.perms,
                region.path().unwrap_or("no file"),
                e
            );
            continue;
        }
        let outfile = format!("{}", region);
        std::fs::write(output_dir.join(outfile), &buffer[..])?;
        println!(
            "Dumped region {:x}-{:x} {} ({})",
            region.start,
            region.end,
            region.perms,
            region.path().unwrap_or("no file")
        );
    }

    Ok(())
}
