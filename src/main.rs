mod show;
mod alias;
mod cd;
mod error;
mod eval;
mod exec;
mod expand;
mod histedit;
mod input;
mod jobs;
mod machdep;
mod mail;
mod makeinit;
mod makenodes;
mod makesyntax;
mod maketokens;
mod memalloc;
mod myhistoryedit;
mod mystring;
mod options;
mod output;
mod parser;
mod redir;
mod shell;
mod system;
mod trap;
mod var;

use nix::unistd::{Pid, getpid,getuid,geteuid,getgid,getegid};
use lazy_static::lazy_static;
use std::env;

lazy_static! {
    static ref ROOTPID: Pid = getpid();
}

static mut LOGIN: usize = 0;

// make it call other init functions
fn init() {

}

fn main() {
    let argv: Vec<String> = env::args().collect();

    let argc = argv.len();
    
    let mut state: i32 = 0;
    let shinit;

    init(); 
    unsafe {
        LOGIN = options::process_args(argc,argv);
    }
    
    if unsafe { LOGIN != 0 } {
        state = 1;
        read_profile("/etc/profile");
        state = 2;
        read_profile("$HOME/.profile");
    }
    state = 3;

    if getuid() == geteuid() && getgid() == getegid() && iflag!() as u32 != 0 {
        shinit = var::lookupvar("ENV");
        match shinit {
            None => (),
            Some(var) => read_profile(&var),
        }
    }

    state = 4;

    match options::MINUSC {
        None => (),
        Some(val) =>{
            let temp_val = if sflag!() as u32 != 0 { 0 } else { eval::EV_EXIT };
            eval::evalstring(options::MINUSC, temp_val);
        },
    }

    if sflag!() as u32 != 0 || options::MINUSC == None {
        cmdloop(1);

    }


    trap::exit_shell();
}




fn read_profile(path: &str) {

}




fn cmdloop(top: i32) -> i32 {

    0
}
