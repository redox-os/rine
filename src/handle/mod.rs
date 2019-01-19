use std::{process, ptr, slice};
use libc;
use libc::{c_char, c_int, c_void, off_t, pid_t};
use syscall::*;

fn exit(status: usize) -> Result<usize> {
    process::exit(status as i32);
}

fn write(fd: usize, buffer: &[u8]) -> Result<usize> {
    Error::demux(unsafe {
        syscall!(WRITE, fd, buffer.as_ptr(), buffer.len())
    })
}

struct Mapping {
    address: usize,
    length: usize,
    page_address: usize,
    page_length: usize,
}

impl Mapping {
    unsafe fn new(fd: c_int, address: usize, length: usize, write: bool) -> Result<Mapping> {
        let prot = if write {
            libc::PROT_READ | libc::PROT_WRITE
        } else {
            libc::PROT_READ
        };

        let flags = libc::MAP_SHARED;

        let page_size = 4096;
        // Align address to page size
        let page_address = address & !(page_size - 1);
        // Calculate offset from page aligned address to address
        let page_offset = address - page_address;
        // Calculate length to the nearest page
        let page_length = ((length + page_offset + page_size - 1)/page_size) * page_size;

        let mapping = libc::mmap(ptr::null_mut(), page_length, prot, flags, fd, page_address as off_t);
        if mapping == libc::MAP_FAILED {
            libc::perror(b"mmap\0".as_ptr() as *const c_char);
            Err(Error::new(EINVAL))
        } else {
            Ok(Mapping {
                address: mapping as usize + page_offset,
                length: length,
                page_address: mapping as usize,
                page_length: page_length
            })
        }
    }

    unsafe fn as_slice(&self) -> &[u8] {
        slice::from_raw_parts(self.address as *const u8, self.length)
    }
}

impl Drop for Mapping {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.page_address as *mut c_void, self.page_length);
        }
    }
}

unsafe fn process_read(pid: pid_t, address: usize, length: usize) -> Vec<u8> {
    let mut buffer = vec![0; length];

    let local_iov = libc::iovec {
        iov_base: buffer.as_mut_ptr() as *mut libc::c_void,
        iov_len: buffer.len(),
    };

    let remote_iov = libc::iovec {
        iov_base: address as *mut libc::c_void,
        iov_len: buffer.len(),
    };

    libc::process_vm_readv(
        pid,
        &local_iov as *const _,
        1,
        &remote_iov as *const _,
        1,
        0
    );

    buffer
}

unsafe fn process_write(pid: pid_t, address: usize, buffer: &[u8]) {
    let local_iov = libc::iovec {
        iov_base: buffer.as_ptr() as *mut libc::c_void,
        iov_len: buffer.len(),
    };

    let remote_iov = libc::iovec {
        iov_base: address as *mut libc::c_void,
        iov_len: buffer.len(),
    };

    libc::process_vm_writev(
        pid,
        &local_iov as *const _,
        1,
        &remote_iov as *const _,
        1,
        0
    );
}

pub unsafe fn handle(pid: pid_t, mem: c_int, a: usize, b: usize, c: usize, d: usize, e: usize, f: usize) -> usize {
    let inner = || -> Result<usize> {
        match a {
            SYS_WRITE => {
                println!("write({}, {:#x}, {})", b, c, d);
                //let mapping = Mapping::new(mem, c, d, false)?;
                //println!("Map {:#x}:{}", mapping.address, mapping.length);
                write(b, &process_read(pid, c, d))
            },
            SYS_EXIT => {
                println!("exit({})", b);
                exit(b)
            },
            _ => {
                println!("Unknown: {:#x}", a);
                Err(Error::new(ENOSYS))
            }
        }
    };

    let res = inner();

    println!("= {:?}", res);
    Error::mux(res)
}
