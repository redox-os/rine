use libc;
use std::mem;
use syscall;

pub struct Process{
    pid: libc::pid_t,
    pub regs: libc::user_regs_struct
}

impl Process {
    pub unsafe fn new(pid: libc::pid_t) -> Process {
        Process {
            pid,
            regs: mem::zeroed()
        }
    }

    pub unsafe fn step(&mut self) -> Result<(), i32> {
        loop {
            if libc::ptrace(libc::PTRACE_SYSCALL, self.pid, 0, 0) < 0 {
                libc::perror(b"PTRACE_SYSCALL\0".as_ptr() as *const _);
                return Err(1);
            }

            let mut status = 0;
            if libc::waitpid(self.pid, &mut status, 0) < 0 {
                libc::perror(b"waitpid\0".as_ptr() as *const _);
                return Err(1);
            }

            trace!("waitpid {:#x}", status);
            if libc::WIFSTOPPED(status) && libc::WSTOPSIG(status) == (0x80 | libc::SIGTRAP) {
                trace!("  SYSCALL");
                return Ok(());
            } else if libc::WIFSTOPPED(status) {
                let signal = libc::WSTOPSIG(status);
                trace!("  STOPPED {}", signal);
            } else if libc::WIFSIGNALED(status) {
                let signal = libc::WTERMSIG(status);
                trace!("  SIGNALED {}", signal);
            } else if libc::WIFEXITED(status) {
                let exit_status = libc::WEXITSTATUS(status);
                trace!("  EXIT {}", exit_status);
                return Err(exit_status);
            }
        }
    }

    pub unsafe fn get(&mut self) {
        if libc::ptrace(libc::PTRACE_GETREGS, self.pid, 0, &mut self.regs) < 0 {
            libc::perror(b"PTRACE_GETREGS\0".as_ptr() as *const _);
        }
    }

    pub unsafe fn set(&mut self) {
        if libc::ptrace(libc::PTRACE_SETREGS, self.pid, 0, &self.regs) < 0 {
            libc::perror(b"PTRACE_SETREGS\0".as_ptr() as *const _);
        }
    }

    pub fn args(&self) -> (u64, u64, u64, u64, u64, u64) {
        (
            self.regs.orig_rax,
            self.regs.rdi,
            self.regs.rsi,
            self.regs.rdx,
            self.regs.r10,
            self.regs.r8
        )
    }

    pub fn set_nr(&mut self, nr: usize) {
        self.set_a(nr as u64);
    }

    pub fn set_a(&mut self, value: u64) {
        self.regs.orig_rax = value;
    }

    pub fn set_b(&mut self, value: u64) {
        self.regs.rdi = value;
    }

    pub fn set_c(&mut self, value: u64) {
        self.regs.rsi = value;
    }

    pub fn set_d(&mut self, value: u64) {
        self.regs.rdx = value;
    }

    pub fn set_e(&mut self, value: u64) {
        self.regs.r10 = value;
    }

    pub fn set_f(&mut self, value: u64) {
        self.regs.r8 = value;
    }

    pub fn result(&self) -> syscall::Result<usize> {
        syscall::Error::demux(self.regs.rax as usize)
    }

    pub unsafe fn read_type<T: Clone + Default>(&mut self, address: *const T, length: usize) -> syscall::Result<Vec<T>> {
        let mut buffer = vec![T::default(); length];

        let local_iov = libc::iovec {
            iov_base: buffer.as_mut_ptr() as *mut libc::c_void,
            iov_len: buffer.len(),
        };

        let remote_iov = libc::iovec {
            iov_base: address as *mut libc::c_void,
            iov_len: buffer.len(),
        };

        libc::process_vm_readv(
            self.pid,
            &local_iov as *const _,
            1,
            &remote_iov as *const _,
            1,
            0
        );

        Ok(buffer)
    }

    pub unsafe fn pread(&mut self, address: usize, length: usize) -> syscall::Result<Vec<u8>> {
        self.read_type(address as *const u8, length)
    }

    pub unsafe fn pwrite(&mut self, address: usize, buffer: &[u8]) -> syscall::Result<()> {
        let local_iov = libc::iovec {
            iov_base: buffer.as_ptr() as *mut libc::c_void,
            iov_len: buffer.len(),
        };

        let remote_iov = libc::iovec {
            iov_base: address as *mut libc::c_void,
            iov_len: buffer.len(),
        };

        libc::process_vm_writev(
            self.pid,
            &local_iov as *const _,
            1,
            &remote_iov as *const _,
            1,
            0
        );

        Ok(())
    }
}
