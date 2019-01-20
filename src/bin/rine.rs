extern crate env_logger;
extern crate libc;
#[macro_use]
extern crate log;
extern crate sc;
extern crate syscall;

use std::{env, ffi, process, ptr};

// Constants
use libc::{
    PTRACE_O_EXITKILL, PTRACE_O_TRACESYSGOOD, PTRACE_SETOPTIONS, PTRACE_TRACEME,
    SIGSTOP
};
use libc::{c_char, pid_t};
use libc::{execv, fork, ptrace};

use self::handle::handle;
mod handle;

unsafe fn child(path: *const c_char, argv: *const *const c_char) -> ! {
    ptrace(PTRACE_TRACEME, 0, 0, 0);

    if execv(path, argv) < 0 {
        libc::perror(b"execv\0".as_ptr() as *const _);
    }

    process::exit(1);
}

unsafe fn parent(pid: pid_t) {
    env_logger::init();

    loop {
        let mut status = 0;
        if libc::waitpid(pid, &mut status, 0) < 0 {
            libc::perror(b"waitpid\0".as_ptr() as *const _);
            process::exit(1);
        }
        trace!("waitpid {:#x}", status);
        if libc::WIFSTOPPED(status) && libc::WSTOPSIG(status) == (0x80 | libc::SIGTRAP) {
            trace!("  SYSCALL");
        } else if libc::WIFSTOPPED(status) {
            let signal = libc::WSTOPSIG(status);
            trace!("  STOPPED {}", signal);
            break;
        } else if libc::WIFSIGNALED(status) {
            let signal = libc::WTERMSIG(status);
            trace!("  SIGNALED {}", signal);
        } else if libc::WIFEXITED(status) {
            let exit_status = libc::WEXITSTATUS(status);
            trace!("  EXIT {}", exit_status);
            process::exit(exit_status);
        }
    }

    ptrace(PTRACE_SETOPTIONS, pid, 0, PTRACE_O_EXITKILL | PTRACE_O_TRACESYSGOOD);

    loop {
        if let Some(status) = handle(pid) {
            println!("Process exited with status {}", status);
            process::exit(status);
        }
    }
}

fn main() {
    let mut args = Vec::new();
    for arg in env::args().skip(1) {
        args.push(ffi::CString::new(arg).unwrap());
    }

    if args.is_empty() {
        eprintln!("rine [command]");
        process::exit(1);
    }

    let mut arg_ptrs = Vec::new();
    for arg in args.iter() {
        arg_ptrs.push(arg.as_ptr())
    }
    arg_ptrs.push(ptr::null());

    unsafe {
        let pid = fork();
        if pid == 0 {
            child(arg_ptrs[0], arg_ptrs.as_ptr());
        } else if pid < 0 {
            panic!("failed to clone");
        } else {
            parent(pid);
        }
    }
}
