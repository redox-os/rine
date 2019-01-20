use libc;
use sc::nr;
use std::{mem, result, str};
use syscall::*;

mod debug;

use self::process::Process;
mod process;

fn convert_open(flags: u64) -> (u64, u64) {
    let rflags = flags as usize;
    let mut lflags = match rflags & O_ACCMODE {
        O_RDONLY => libc::O_RDONLY,
        O_WRONLY => libc::O_WRONLY,
        O_RDWR => libc::O_RDWR,
        _ => 0,
    };

    macro_rules! convert {
        ($name:ident) => (if rflags & syscall::flag::$name > 0 {
            lflags |= libc::$name;
        });
    }

    convert!(O_NONBLOCK);
    convert!(O_CREAT);
    convert!(O_EXCL);
    convert!(O_TRUNC);
    convert!(O_APPEND);
    convert!(O_NONBLOCK);
    convert!(O_DIRECTORY);
    convert!(O_NOFOLLOW);
    convert!(O_CLOEXEC);
    if rflags & syscall::flag::O_STAT > 0 {
        lflags |= libc::O_PATH;
    }

    (lflags as u64, flags & 0xFFFF)
}

fn convert_path(rpath: &[u8]) -> Vec<u8> {
    let mut lpath = if rpath.contains(&b':') {
        let mut parts = rpath.splitn(2, |b| b == &b':');
        let scheme = parts.next().unwrap();
        let path = parts.next().unwrap();
        match scheme {
            b"null" => b"/dev/null".to_vec(),
            b"rand" => b"/dev/urandom".to_vec(),
            b"zero" => b"/dev/zero".to_vec(),
            _ => {
                println!(
                    "TODO: {}:{}",
                    unsafe { str::from_utf8_unchecked(&scheme) },
                    unsafe { str::from_utf8_unchecked(&path) }
                );
                rpath.to_vec()
            }
        }
    } else {
        rpath.to_vec()
    };
    lpath.push(0);
    lpath
}

fn convert_stat(lstat: &[u8]) -> Stat {
    let stat = unsafe { &*(lstat.as_ptr() as *const libc::stat) };

    Stat {
        st_dev: stat.st_dev as _,
        st_ino: stat.st_ino as _,
        st_mode: stat.st_mode as _,
        st_nlink: stat.st_nlink as _,
        st_uid: stat.st_uid as _,
        st_gid: stat.st_gid as _,
        st_size: stat.st_size as _,
        st_blksize: stat.st_blksize as _,
        st_blocks: stat.st_blocks as _,
        st_mtime: stat.st_mtime as _,
        st_mtime_nsec: stat.st_mtime_nsec as _,
        st_atime: stat.st_atime as _,
        st_atime_nsec: stat.st_atime_nsec as _,
        st_ctime: stat.st_ctime as _,
        st_ctime_nsec: stat.st_ctime_nsec as _,
    }
}

const PAGE_SIZE: usize = 4096;

pub unsafe fn handle(pid: libc::pid_t) -> result::Result<(), i32> {
    // x86_64 syscall convention
    // rax, rdi, rsi, rdx, r10, r8, r9
    // Return value in rax
    // Clobbers rcx, r11

    let mut p = Process::new(pid);

    p.step()?;
    p.get();

    let (a, b, c, d, e, f) = p.args();

    let format_call = true;
    if format_call {
        debug!("{}", debug::format_call(
            &mut p,
            a as usize,
            b as usize,
            c as usize,
            d as usize,
            e as usize,
            f as usize
        ));
    } else {
        debug!("{} {:#x}({} {:#x}, {} {:#x}, {} {:#x}, {} {:#x}, {} {:#x})", a, a, b, b, c, c, d, d, e, e, f, f);
    }

    match a as usize {
        SYS_BRK => {
            p.set_nr(nr::BRK);
            p.set();
            p.step()?;
        },
        SYS_GETPID => {
            p.set_nr(nr::GETPID);
            p.set();
            p.step()?;
        },
        SYS_FSTAT => {
            // Save the current stack page
            let stack_addr = (p.regs.rsp as usize) & !(PAGE_SIZE - 1);
            let stack_page = p.pread(stack_addr, PAGE_SIZE).unwrap();

            // Set up the new arguments
            p.set_nr(nr::FSTAT);
            p.set_c(stack_addr as u64);
            p.set();

            // Call the system call
            p.step()?;

            // Read result
            let lstat = p.pread(stack_addr, mem::size_of::<libc::stat>()).unwrap();
            let rstat = convert_stat(&lstat);

            // Restore the stack page
            p.pwrite(stack_addr, &stack_page).unwrap();

            // Write result
            p.pwrite(c as usize, &rstat).unwrap();

            // Restore the old arguments
            p.set_c(c);
        },
        SYS_OPEN => {
            // Convert the path into a C string
            let rpath = p.pread(b as usize, c as usize).unwrap();
            let lpath = convert_path(&rpath);

            if lpath.len() > PAGE_SIZE {
                panic!("path larger than PAGE_SIZE {}", PAGE_SIZE);
            }

            // Save the current stack page
            let stack_addr = (p.regs.rsp as usize) & !(PAGE_SIZE - 1);
            let stack_page = p.pread(stack_addr, PAGE_SIZE).unwrap();

            // Write the path to the stack
            p.pwrite(stack_addr, &lpath).unwrap();

            // Convert the open flags
            let (oflag, mode) = convert_open(d);

            // Set up the new arguments
            p.set_nr(nr::OPEN);
            p.set_b(stack_addr as u64);
            p.set_c(oflag);
            p.set_d(mode);
            p.set();

            // Call the system call
            p.step()?;

            // Restore the stack page
            p.pwrite(stack_addr, &stack_page).unwrap();

            // Restore the old arguments
            p.set_b(b);
            p.set_c(c);
            p.set_d(d);
        },
        SYS_READ => {
            p.set_nr(nr::READ);
            p.set();
            p.step()?;
        },
        SYS_WRITE => {
            p.set_nr(nr::WRITE);
            p.set();
            p.step()?;
        },
        SYS_EXIT => {
            p.set_nr(nr::EXIT);
            p.set();
            p.step()?;
        },
        _ => {
            p.set_nr(!0);
            p.set();
            p.step()?;
        }
    }

    p.get();

    let res = p.result();
    if format_call {
        debug!("{} = {:?} {:x?}", debug::format_call(
            &mut p,
            a as usize,
            b as usize,
            c as usize,
            d as usize,
            e as usize,
            f as usize
        ), res, res);
    } else {
        debug!("{} {:#x}({} {:#x}, {} {:#x}, {} {:#x}, {} {:#x}, {} {:#x}) = {:?} {:x?}", a, a, b, b, c, c, d, d, e, e, f, f, res, res);
    }

    Ok(())
}
