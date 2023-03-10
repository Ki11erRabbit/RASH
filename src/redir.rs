use std::fs::{File, OpenOptions};
use std::os::unix::io::AsRawFd;
use std::process;
use std::mem;
use std::io;
use std::os::unix::fs::OpenOptionsExt;
use crate::nodes::Node;


pub struct RedirTable {
    renamed: [i32;10],
}

pub static mut REDIR_LIST: Vec<Box<RedirTable>> = Vec::new();

pub const REDIR_PUSH:i32 = 0o1;     // save previous values of file descriptors
pub const REDIR_BACKQ: i32 = 0o2;   // save the command output in memory
pub const REDIR_SAVEFD2: i32 = 0o3; // set preverrout

pub fn sh_open(path_name: &str, mayfail: i32) -> std::io::Result<i32> {
    
    let fd;
    let error: i32; 
    //open file and return its file descriptor 
    let file;
    loop {
        
        file = OpenOptions::new()
            .read(true)
            .mode(0o666)
            .open(path_name);
        file = File::open(path_name);
        
        //make it continue only if the error is Interupted
        match &file {
            Err(e) => match e.kind() {
                io::ErrorKind::Interrupted => continue,
                _ => break,
            },
            Ok(_) => break,
        }
    }
    
    if mayfail != 0 {
        fd = file?.as_raw_fd().to_owned();
        mem::forget(file?);
        return Ok(fd);
        
    }
    else {
        match file {
            Err(e) => {
                eprintln!("cannot open {}: {}",path_name,e);
                process::exit(2);
            },
            Ok(file) => {
                fd = file.as_raw_fd().to_owned();
                mem::forget(file);

                return Ok(fd);

            },
        };
    }
    
}


pub fn push_redir(redir:Option<Box<Node>>) -> usize {

    unimplemented!()

}

pub fn pop_redir(drop: i32) {

}

pub fn unwind_redir(stop: usize) {
    loop {
        if stop == unsafe {REDIR_LIST.len() - 1 }{
            break;
        }
        pop_redir(0);
    }

}

/*
 * Process a list of redirection commands.  If the REDIR_PUSH flag is set,
 * old file descriptors are stashed away so that the redirection can be
 * undone by calling popredir.  If the REDIR_BACKQ flag is set, then the
 * standard output, and the standard error if it becomes a duplicate of
 * stdout, is saved in memory.
 */
pub fn redirect(redir: Option<Box<Node>>, flags: i32) {

}

pub fn redirect_safe(redir: Option<Box<Node>>, flags: i32) -> Result<i32,i32> {
    unimplemented!()
}
