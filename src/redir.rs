use std::fs::{File, OpenOptions};
use std::os::unix::io::AsRawFd;
use std::process;
use std::mem;
use std::io;
use std::os::unix::fs::OpenOptionsExt;



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
