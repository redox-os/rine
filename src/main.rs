extern crate libc;
#[macro_use]
extern crate sc;
extern crate syscall;

use std::{ffi, mem, process, ptr};

// Constants
use libc::{
    //CLONE_FILES, CLONE_FS, CLONE_PARENT, CLONE_IO, CLONE_SIGHAND, CLONE_SYSVSEM, CLONE_VM,
    O_RDWR,
    PTRACE_O_EXITKILL, PTRACE_GETREGS, PTRACE_SETOPTIONS, PTRACE_SETREGS, PTRACE_TRACEME,
    SIGSTOP
};
const PTRACE_SYSEMU: c_uint = 31;
// Macros
use libc::{WEXITSTATUS, WIFEXITED};
// Types
use libc::{c_char, c_int, c_uint, c_void, pid_t, user_regs_struct};
// Functions
use libc::{fork, memalign, open, ptrace, raise, waitpid};

use self::handle::handle;
mod handle;

fn program() {
    let array = format!("something\n");
    if syscall::write(2, &array.as_bytes()).is_ok() {
        syscall::exit(0);
    } else {
        syscall::exit(1);
    }
}

extern "C" fn child() -> ! {
    unsafe {
        ptrace(PTRACE_TRACEME, 0, 0, 0);
        raise(SIGSTOP);
    };
    program();
    loop {}
}

unsafe fn parent(pid: pid_t) {
    ptrace(PTRACE_SETOPTIONS, pid, 0, PTRACE_O_EXITKILL);

    // Catch SIGSTOP
    waitpid(pid, ptr::null_mut(), 0);

    let path = format!("/proc/{}/mem", pid);
    let mem = {
        let path_c = ffi::CString::new(path.clone()).unwrap();
        open(path_c.as_ptr(), O_RDWR)
    };
    if mem < 0 {
        eprintln!("Failed to open {}", path);
    }

    loop {
        ptrace(PTRACE_SYSEMU, pid, 0, 0);

        let mut status = 0;
        waitpid(pid, &mut status, 0);
        if WIFEXITED(status) {
            println!("Process exited with status {}", status);
            process::exit(WEXITSTATUS(status));
        }

        let mut regs: user_regs_struct = mem::zeroed();
        ptrace(PTRACE_GETREGS, pid, 0, &mut regs);

        regs.rax = handle(
            pid,
            mem,
            regs.orig_rax as usize,
            regs.rbx as usize,
            regs.rcx as usize,
            regs.rdx as usize,
            regs.rsi as usize,
            regs.rdi as usize
        ) as u64;

        ptrace(PTRACE_SETREGS, pid, 0, &regs);
    }
}

fn main() {
    unsafe {
        /*
        let stack_size = 1024 * 1024;
        let child_stack = memalign(4096, stack_size);
        let pid = clone(
            child,
            child_stack.add(stack_size),
            CLONE_FILES | CLONE_FS | CLONE_PARENT | CLONE_IO | CLONE_SIGHAND | CLONE_SYSVSEM | CLONE_VM,
            ptr::null_mut()
        );
        */
        let pid = fork();
        if pid == 0 {
            child();
        } else if pid < 0 {
            panic!("failed to clone");
        } else {
            parent(pid);
        }
    }
}
