use std::ffi::OsStr;
use std::process::{Command, ExitStatus, Output};
use std::io::{self, Error};

pub fn notify_process(process: &str, signal: i32) {
    let mut system = sysinfo::System::new();
    system.refresh_all();

    let pid = system
        .processes_by_exact_name(OsStr::new(process))
        .next()
        .map(|e| e.pid().as_u32() as i32);

    if let Some(pid) = pid {
        let signal_number = { libc::SIGRTMIN() + signal };
        let _ = unsafe { libc::kill(pid, signal_number) };
    } else {
        println!("Waybar not active")
    }
}
pub fn run_shell_command(command: &str) -> Result<ExitStatus, Error> {
    Command::new("/bin/sh")
        .arg("-c")
        .arg(command)
        .status()
}

