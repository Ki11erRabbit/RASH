use std::ops::BitAnd;
use std::ops::BitOr;

use lazy_static::lazy_static;
use std::io::BufReader;
use std::sync::Mutex;
use std::os::unix::io::FromRawFd;
use nix::unistd::close;

use crate::redir;

pub enum Input{
    InputPushFile = 1,
    InputNoFileOk = 2,
}

impl BitAnd<i32> for Input {
    type Output = i32;
    fn bitand(self, rhs: i32) -> i32 {
        match self { 
            Input::InputNoFileOk => 2 & rhs,
            Input::InputPushFile => 1 & rhs,
        }
    }
}

impl BitOr<Self> for Input {
    type Output = i32;
    fn bitor(self, rhs: Self) -> i32 {
        let left = match self { 
            Input::InputNoFileOk => 2,
            Input::InputPushFile => 1,
        };
        let right = match rhs { 
            Input::InputNoFileOk => 2,
            Input::InputPushFile => 1,
        };
        left | right
    }
}
impl BitOr<i32> for Input {
    type Output = i32;
    fn bitor(self, rhs: i32) -> i32 {
        match self { 
            Input::InputNoFileOk => 2 | rhs,
            Input::InputPushFile => 1 | rhs,
        }
    }
}


struct Parsefile {
    pub fd: i32,
    pub line_num: usize,
    pub num_left: usize,
    pub buf_pos: usize,
    pub buf: String,
}

impl Parsefile {
    
    pub fn new(fd: i32) -> Parsefile {
        Parsefile { fd: fd, line_num: 0 , num_left: 0, buf_pos: 0,buf: String::new() }
    }
    
}


static mut PARSEFILE_STACK: Mutex<Vec<Parsefile>> = Mutex::new(Vec::new());
/*
 * Set the input to take input from a file.  If push is set, push the
 * old input onto the stack first.
 */
pub fn set_input_file(file_name: &str,flags: i32) -> std::io::Result<i32> {
    
    let fd: i32;

    //block interrupts
    fd = redir::sh_open(file_name, Input::InputNoFileOk & flags)?;

    if fd < 10 {
        //save fd
    }
    set_input_fd(fd, true);
    //unblock interrupts
    return Ok(fd);
}

/*
 * Like setinputfile, but takes an open file descriptor.  Call this with
 * interrupts off.
 */
fn set_input_fd(fd: i32, push: bool) {
    if push {
        pushfile();
        unsafe {
            PARSEFILE_STACK.lock().unwrap().last_mut().unwrap().buf = "".to_string();
        }
    }
    let parse_file;
    unsafe {
        parse_file = PARSEFILE_STACK.lock().unwrap().last_mut().unwrap();
    }

    parse_file.fd = fd;
}


/*
 * Like setinputfile, but takes input from a string.
 */

fn set_input_string(string: &str) {
    //block interrupts

    pushfile();
    unsafe {
        let parse_file = PARSEFILE_STACK.lock().unwrap().last_mut().unwrap();
        parse_file.fd = -1;
        parse_file.line_num = 0;
        parse_file.num_left = string.len();
        parse_file.buf = string.to_string();
    }

    //unblock interrupts
}


/*
 * To handle the "." command, a stack of input files is used.  Pushfile
 * adds a new entry to the stack and popfile restores the previous level.
 */
fn pushfile() {
    
    let parse_file = Parsefile::new(-1);
    unsafe {
        PARSEFILE_STACK.lock().unwrap().push(parse_file);
    }

}

pub fn popfile() {
    let parse_file;
    unsafe {
        parse_file = PARSEFILE_STACK.lock().unwrap().pop();
    }

    //block interrupts
    let parse_file = parse_file.unwrap();
    if parse_file.fd >= 0 {
        close(parse_file.fd);
    }

    //unblock interrupts
}

fn unwind_files(stop_pos: usize) {
    let len = unsafe {
        PARSEFILE_STACK.lock().unwrap().len()
    };
    for i in (0..len).rev() {
        if i == stop_pos {
            break;
        }
        popfile();
    }
}

/*
 * Return to top level.
 */
fn pop_all_files() {
    unwind_files(0);
}
