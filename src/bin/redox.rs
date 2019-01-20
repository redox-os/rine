extern crate syscall;

fn main() {
    let pid = syscall::getpid();
    let array = format!("PID: {:?}\n", pid);
    if syscall::write(2, &array.as_bytes()).is_ok() {
        syscall::exit(0);
    } else {
        syscall::exit(1);
    }
}
