use std::mem;
use libc;
use libc::{PTRACE_GETREGS, PTRACE_SETREGS, PTRACE_SYSCALL};
use libc::{pid_t, user_regs_struct};
use libc::ptrace;
use sc;
use syscall::*;

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

unsafe fn ptrace_syscall(pid: pid_t) -> Option<i32> {
    loop {
        if ptrace(PTRACE_SYSCALL, pid, 0, 0) < 0 {
            libc::perror(b"PTRACE_SYSCALL\0".as_ptr() as *const _);
            return Some(1);
        }

        let mut status = 0;
        if libc::waitpid(pid, &mut status, 0) < 0 {
            libc::perror(b"waitpid\0".as_ptr() as *const _);
            return Some(1);
        }
        trace!("waitpid {:#x}", status);
        if libc::WIFSTOPPED(status) && libc::WSTOPSIG(status) == (0x80 | libc::SIGTRAP) {
            trace!("  SYSCALL");
            return None;
        } else if libc::WIFSTOPPED(status) {
            let signal = libc::WSTOPSIG(status);
            trace!("  STOPPED {}", signal);
        } else if libc::WIFSIGNALED(status) {
            let signal = libc::WTERMSIG(status);
            trace!("  SIGNALED {}", signal);
        } else if libc::WIFEXITED(status) {
            let exit_status = libc::WEXITSTATUS(status);
            trace!("  EXIT {}", exit_status);
            return Some(exit_status);
        }
    }
}

pub unsafe fn handle(pid: pid_t) -> Option<i32> {
    // Redox convention
    // rax, rbx, rcx, rdx, rsi, rdi
    // Return value in rax
    // No clobbers

    // Linux convention
    // rax, rdi, rsi, rdx, r10, r8, r9
    // Return value in rax
    // Clobbers rcx, r11

    macro_rules! step {
        () => (if let Some(status) = ptrace_syscall(pid) {
            return Some(status);
        });
    }

    let mut regs: user_regs_struct = mem::zeroed();
    macro_rules! get {
        () => (if ptrace(PTRACE_GETREGS, pid, 0, &mut regs) < 0 {
            libc::perror(b"PTRACE_GETREGS\0".as_ptr() as *const _);
        });
    }
    macro_rules! set {
        () => (if ptrace(PTRACE_SETREGS, pid, 0, &regs) < 0 {
            libc::perror(b"PTRACE_SETREGS\0".as_ptr() as *const _);
        });
    }

    step!();
    get!();

    let (a, b, c, d, e, f) = (
        regs.orig_rax,
        regs.rdi,
        regs.rsi,
        regs.rdx,
        regs.r10,
        regs.r8
    );

    debug!("{:#x}({:#x}, {:#x}, {:#x}, {:#x}, {:#x})", a, b, c, d, e, f);

    match a as usize {
        SYS_GETPID => {
            regs.orig_rax = sc::nr::GETPID as u64;
            set!();
            step!();
        },
        SYS_WRITE => {
            regs.orig_rax = sc::nr::WRITE as u64;
            set!();
            step!();
        },
        SYS_EXIT => {
            regs.orig_rax = sc::nr::EXIT as u64;
            set!();
            step!();
        },
        _ => {
            step!();
        }
    }

    get!();

    let res = Error::demux(regs.rax as usize);
    debug!("{:#x}({:#x}, {:#x}, {:#x}, {:#x}, {:#x}) = {:?}", a, b, c, d, e, f, res);

    None
}
