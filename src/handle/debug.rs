use std::mem;
use std::ops::Range;

use syscall::data::{Map, Stat, TimeSpec};
use syscall::flag::*;
use syscall::number::*;

use super::process::Process;

// Copied from std
pub struct EscapeDefault {
    range: Range<usize>,
    data: [u8; 4],
}

pub fn escape_default(c: u8) -> EscapeDefault {
    let (data, len) = match c {
        b'\t' => ([b'\\', b't', 0, 0], 2),
        b'\r' => ([b'\\', b'r', 0, 0], 2),
        b'\n' => ([b'\\', b'n', 0, 0], 2),
        b'\\' => ([b'\\', b'\\', 0, 0], 2),
        b'\'' => ([b'\\', b'\'', 0, 0], 2),
        b'"' => ([b'\\', b'"', 0, 0], 2),
        b'\x20' ... b'\x7e' => ([c, 0, 0, 0], 1),
        _ => ([b'\\', b'x', hexify(c >> 4), hexify(c & 0xf)], 4),
    };

    return EscapeDefault { range: (0.. len), data: data };

    fn hexify(b: u8) -> u8 {
        match b {
            0 ... 9 => b'0' + b,
            _ => b'a' + b - 10,
        }
    }
}

impl Iterator for EscapeDefault {
    type Item = u8;
    fn next(&mut self) -> Option<u8> { self.range.next().map(|i| self.data[i]) }
    fn size_hint(&self) -> (usize, Option<usize>) { self.range.size_hint() }
}

struct ByteString(Vec<u8>);

impl ::std::fmt::Debug for ByteString {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "\"")?;
        for i in self.0.iter() {
            for ch in escape_default(*i) {
                write!(f, "{}", ch as char)?;
            }
        }
        write!(f, "\"")?;
        Ok(())
    }
}

pub unsafe fn format_call(p: &mut Process, a: usize, b: usize, c: usize, d: usize, e: usize, f: usize) -> String {
    match a {
        SYS_OPEN => format!(
            "open({:?}, {:#x})",
            p.read_type(b as *const u8, c).map(ByteString),
            d
        ),
        SYS_CHMOD => format!(
            "chmod({:?}, {:#o})",
            p.read_type(b as *const u8, c).map(ByteString),
            d
        ),
        SYS_RMDIR => format!(
            "rmdir({:?})",
            p.read_type(b as *const u8, c).map(ByteString)
        ),
        SYS_UNLINK => format!(
            "unlink({:?})",
            p.read_type(b as *const u8, c).map(ByteString)
        ),
        SYS_CLOSE => format!(
            "close({})", b
        ),
        SYS_DUP => format!(
            "dup({}, {:?})",
            b,
            p.read_type(c as *const u8, d).map(ByteString)
        ),
        SYS_DUP2 => format!(
            "dup2({}, {}, {:?})",
            b,
            c,
            p.read_type(d as *const u8, e).map(ByteString)
        ),
        SYS_READ => format!(
            "read({}, {:#x}, {})",
            b,
            c,
            d
        ),
        SYS_WRITE => format!(
            "write({}, {:#x}, {})",
            b,
            c,
            d
        ),
        SYS_LSEEK => format!(
            "lseek({}, {}, {} ({}))",
            b,
            c as isize,
            match d {
                SEEK_SET => "SEEK_SET",
                SEEK_CUR => "SEEK_CUR",
                SEEK_END => "SEEK_END",
                _ => "UNKNOWN"
            },
            d
        ),
        SYS_FCNTL => format!(
            "fcntl({}, {} ({}), {:#x})",
            b,
            match c {
                F_DUPFD => "F_DUPFD",
                F_GETFD => "F_GETFD",
                F_SETFD => "F_SETFD",
                F_SETFL => "F_SETFL",
                F_GETFL => "F_GETFL",
                _ => "UNKNOWN"
            },
            c,
            d
        ),
        SYS_FMAP => format!(
            "fmap({}, {:?})",
            b,
            p.read_type(
                c as *const Map,
                d/mem::size_of::<Map>()
            ),
        ),
        SYS_FUNMAP => format!(
            "funmap({:#x})",
            b
        ),
        SYS_FPATH => format!(
            "fpath({}, {:#x}, {})",
            b,
            c,
            d
        ),
        SYS_FSTAT => format!(
            "fstat({}, {:?})",
            b,
            p.read_type(
                c as *const Stat,
                d/mem::size_of::<Stat>()
            ),
        ),
        SYS_FSTATVFS => format!(
            "fstatvfs({}, {:#x}, {})",
            b,
            c,
            d
        ),
        SYS_FSYNC => format!(
            "fsync({})",
            b
        ),
        SYS_FTRUNCATE => format!(
            "ftruncate({}, {})",
            b,
            c
        ),

        SYS_BRK => format!(
            "brk({:#x})",
            b
        ),
        SYS_CHDIR => format!(
            "chdir({:?})",
            p.read_type(b as *const u8, c).map(ByteString)
        ),
        SYS_CLOCK_GETTIME => format!(
            "clock_gettime({}, {:?})",
            b,
            p.read_type(c as *mut TimeSpec, 1)
        ),
        SYS_CLONE => format!(
            "clone({})",
            b
        ),
        SYS_EXIT => format!(
            "exit({})",
            b
        ),
        //TODO: Cleanup, do not allocate
        SYS_FEXEC => format!(
            "fexec({}, {:?}, {:?})",
            b,
            p.read_type(
                c as *const [usize; 2],
                d
            ).map(|slice| {
                slice.iter().map(|a|
                    p.read_type(a[0] as *const u8, a[1]).ok()
                    .and_then(|s| String::from_utf8(s).ok())
                ).collect::<Vec<Option<String>>>()
            }),
            p.read_type(
                e as *const [usize; 2],
                f
            ).map(|slice| {
                slice.iter().map(|a|
                    p.read_type(a[0] as *const u8, a[1]).ok()
                    .and_then(|s| String::from_utf8(s).ok())
                ).collect::<Vec<Option<String>>>()
            })
        ),
        SYS_FUTEX => format!(
            "futex({:#x} [{:?}], {}, {}, {}, {})",
            b,
            p.read_type(b as *mut i32, 1).map(|uaddr| uaddr[0]),
            c,
            d,
            e,
            f
        ),
        SYS_GETCWD => format!(
            "getcwd({:#x}, {})",
            b,
            c
        ),
        SYS_GETEGID => format!("getegid()"),
        SYS_GETENS => format!("getens()"),
        SYS_GETEUID => format!("geteuid()"),
        SYS_GETGID => format!("getgid()"),
        SYS_GETNS => format!("getns()"),
        SYS_GETPID => format!("getpid()"),
        SYS_GETUID => format!("getuid()"),
        SYS_IOPL => format!(
            "iopl({})",
            b
        ),
        SYS_KILL => format!(
            "kill({}, {})",
            b,
            c
        ),
        SYS_SIGRETURN => format!("sigreturn()"),
        SYS_SIGACTION => format!(
            "sigaction({}, {:#x}, {:#x}, {:#x})",
            b,
            c,
            d,
            e
        ),
        SYS_SIGPROCMASK => format!(
            "sigprocmask({}, {:?}, {:?})",
            b,
            p.read_type(c as *const [u64; 2], 1),
            p.read_type(d as *const [u64; 2], 1)
        ),
        SYS_MKNS => format!(
            "mkns({:?})",
            p.read_type(b as *const [usize; 2], c)
        ),
        SYS_NANOSLEEP => format!(
            "nanosleep({:?}, ({}, {}))",
            p.read_type(b as *const TimeSpec, 1),
            c,
            d
        ),
        SYS_PHYSALLOC => format!(
            "physalloc({})",
            b
        ),
        SYS_PHYSFREE => format!(
            "physfree({:#x}, {})",
            b,
            c
        ),
        SYS_PHYSMAP => format!(
            "physmap({:#x}, {}, {:#x})",
            b,
            c,
            d
        ),
        SYS_PHYSUNMAP => format!(
            "physunmap({:#x})",
            b
        ),
        SYS_VIRTTOPHYS => format!(
            "virttophys({:#x})",
            b
        ),
        SYS_PIPE2 => format!(
            "pipe2({:?}, {})",
            p.read_type(b as *mut usize, 2),
            c
        ),
        SYS_SETREGID => format!(
            "setregid({}, {})",
            b,
            c
        ),
        SYS_SETRENS => format!(
            "setrens({}, {})",
            b,
            c
        ),
        SYS_SETREUID => format!(
            "setreuid({}, {})",
            b,
            c
        ),
        SYS_UMASK => format!(
            "umask({:#o}",
            b
        ),
        SYS_WAITPID => format!(
            "waitpid({}, {:#x}, {})",
            b,
            c,
            d
        ),
        SYS_YIELD => format!("yield()"),
        _ => format!(
            "UNKNOWN{} {:#x}({:#x}, {:#x}, {:#x}, {:#x}, {:#x})",
            a, a,
            b,
            c,
            d,
            e,
            f
        )
    }
}
