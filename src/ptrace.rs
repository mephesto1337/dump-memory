use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};

use crate::memory::Region;
use crate::{Error, Result};

#[derive(Debug)]
pub struct Ptrace {
    pid: u32,
    mem: Option<File>,
}

extern "C" {
    fn ptrace(req: i32, pid: u32, addr: usize, data: usize) -> usize;
    fn __errno_location() -> *mut i32;
}

fn ptrace_errno() -> Result<()> {
    let e = unsafe { *__errno_location() };
    if e == 0 {
        Ok(())
    } else {
        Err(Error::Ptrace(io::Error::from_raw_os_error(e)))
    }
}

fn ptrace_wrapper(req: i32, pid: u32, addr: usize, data: usize) -> Result<usize> {
    let ret = unsafe { ptrace(req, pid, addr, data) };

    if ret == usize::MAX {
        ptrace_errno()?;
    }
    Ok(ret)
}

const PTRACE_ATTACH: i32 = 16;
const PTRACE_DETACH: i32 = 17;

impl Ptrace {
    pub fn new(pid: u32) -> Result<Self> {
        ptrace_wrapper(PTRACE_ATTACH, pid, 0, 0)?;
        Ok(Self { pid, mem: None })
    }

    fn open_mem(&mut self) -> Result<&mut File> {
        if self.mem.is_none() {
            let mem = File::open(format!("/proc/{}/mem", self.pid))?;
            self.mem = Some(mem);
        }
        Ok(self.mem.as_mut().unwrap())
    }

    pub fn dump(&mut self, region: &Region, buffer: &mut Vec<u8>) -> Result<()> {
        let mem = self.open_mem()?;
        mem.seek(SeekFrom::Start(
            region
                .start
                .try_into()
                .expect("Cannot fit a usize into a u64"),
        ))?;
        let size = region.size();
        buffer.reserve(size);
        // SAFETY: if the content of the buffer is not properly initialized the `read_exact`, then
        // an error is returned and the buffer is discarded.
        let old_len = buffer.len();
        unsafe { buffer.set_len(old_len + size) };
        match mem.read_exact(&mut buffer[old_len..]) {
            Ok(_) => Ok(()),
            Err(e) => {
                // SAFETY: old_len was fine
                unsafe { buffer.set_len(old_len) };
                Err(e.into())
            }
        }
    }
}

impl Drop for Ptrace {
    fn drop(&mut self) {
        if let Err(e) = ptrace_wrapper(PTRACE_DETACH, self.pid, 0, 0) {
            eprintln!("Could not detach from process {}: {}", self.pid, e);
        }
    }
}
