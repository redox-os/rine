use libc;
use sc::nr;
use std::result;
use syscall::*;

mod debug;

use self::process::Process;
mod process;

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
