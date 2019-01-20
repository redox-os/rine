extern crate env_logger;
extern crate libc;
#[macro_use]
extern crate log;
extern crate sc;
extern crate syscall;

use std::process;

// Constants
use libc::{
    PTRACE_O_EXITKILL, PTRACE_O_TRACESYSGOOD, PTRACE_SETOPTIONS, PTRACE_TRACEME,
    SIGSTOP
};
use libc::pid_t;
use libc::{fork, ptrace, raise};

use self::handle::handle;
mod handle;

fn program() {
    let pid = syscall::getpid();
    let array = format!("PID: {:?}\n", pid);
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
    env_logger::init();

    loop {
        let mut status = 0;
        libc::waitpid(pid, &mut status, 0);
        trace!("waitpid {:#x}", status);
        if libc::WIFSTOPPED(status) && libc::WSTOPSIG(status) == SIGSTOP {
            trace!("  SIGSTOP");
            break;
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
