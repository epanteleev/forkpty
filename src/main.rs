use std::ffi::CString;
use std::fs::File;
use std::io::{Read, stdin, stdout, Write};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::process::Command;

use nix::{
    pty::forkpty,
    unistd::read,
    unistd::write,
};
use nix::poll::{poll, PollFd, PollFlags};
use nix::sys::termios::{InputFlags, LocalFlags, OutputFlags, SetArg, tcgetattr, tcsetattr, Termios};
use nix::unistd::{dup2, setsid};


fn exec_or_die(name: &str) {
    let name_cstr = CString::new(name).unwrap();
    match nix::unistd::execv(&name_cstr, &vec![name_cstr.clone()]) {
        Ok(_) => println!("Exit"),
        Err(err) => panic!("execv() failed: {}", err),
    }
}

fn main() {

    let pid = unsafe {
        forkpty(None, None).expect("TODO: panic message")
    };

    if pid.fork_result.is_parent() {

        let fd_master = PollFd::new(pid.master.clone(), PollFlags::POLLIN);
        let fd_stdin = PollFd::new(stdin().as_raw_fd(), PollFlags::POLLIN);

        let mut fds = vec![fd_master, fd_stdin];
        let mut buffer = [0; 128];

        loop {

             poll(&mut fds, 100)
                .expect("Cannot poll");

            for i in fds.iter() {
                let event = i.revents();
                if event.is_none() || !event.unwrap().contains(PollFlags::POLLIN) {
                    continue;
                }

                if i.as_raw_fd() == pid.master {

                    let size = read(pid.master, &mut buffer)
                        .expect("Couldn't read from pty");

                    let mut slice = &buffer[0..size];
                    write(1,&mut slice)
                        .expect("Couldn't write a buffer in stdout");

                }
                if i.as_raw_fd() == libc::STDIN_FILENO {

                    let size = read(libc::STDIN_FILENO, &mut buffer)
                        .expect("Couldn't read from stdin");

                    let mut slice = &buffer[0..size];

                    write(pid.master, &mut slice)
                        .expect("Couldn't write buffer to pty");
                }
            }
        }
    } else {
        exec_or_die("/bin/bash");
    }

}
