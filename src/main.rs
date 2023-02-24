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
mod nodes;
mod options;
mod output;
mod parser;
mod redir;
mod shell;
mod system;
mod trap;
mod var;

use std::process;
use std::fs::metadata;
use nix::unistd::{Pid, getpid,getuid,geteuid,getgid,getegid};
use lazy_static::lazy_static;
use std::env;
use input::Input;


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



/*
 * Read /etc/profile or .profile.  Return on error.
 */

fn read_profile(name: &str) {
    let name = &parser::expand_str(name);
    match input::set_input_file(name, Input::InputPushFile | Input::InputNoFileOk) {
        Err(_) => return,
        Ok(_) => (),
    }

    cmdloop(0);
    input::popfile();
}

/*
 * Read a file containing shell functions.
 */
fn read_cmd_file(name: &str) {
    input::set_input_file(name, Input::InputPushFile | 0);
    cmdloop(0);
    input::popfile();
}

fn dot_cmd(argc: i32, argv: Vec<String>) -> i32 {
    let mut status = 0;
    
    options::nextopt("");
    let argv;
    unsafe {
        argv = options::ARGPOINTER;
    }

    match &argv[0] {
        Some(val) => {
            let fullname;

            fullname = find_dot_file(val);
        }
        None => (),
    }

    status
}

fn find_dot_file(base_name: &str) -> String {
    if base_name.find("/") != None {
        return base_name.to_string();
    }
    
    let path = vec![pathval!().to_string()];

    let mut len = exec::padvance(path, base_name);
    while len >= 0 {
        //fullname = stackblock();
        let fullname = base_name.to_string();
        let meta = metadata(fullname); 
        unsafe { 
            if (exec::PATHOPT == None || exec::PATHOPT.unwrap().chars().collect::<Vec<char>>()[0] == 'f') && matches!(meta,Ok(_)) && meta.unwrap().is_file() {
                return fullname;
            }
        }
        len = exec::padvance(path, base_name)
    }

    process::exit(2);
}

/*
 * Read and execute commands.  "Top" is nonzero for the top level command
 * loop; it turns on prompting if the shell is interactive.
 */
fn cmdloop(top: i32) -> i32 {
    let mut inter: i32;
    let mut status = 0;
    let mut numeof = 0;
    let mut n: i32; //TODO: make into a node
    
    trace!("cmdloop({}) called\n",top);

    loop {
        let skip: i32;

        if unsafe { jobs::JOBCTL } {
            jobs::show_jobs(output::ERROUT, jobs::SHOW_CHANGED);
        }
        inter = 0;
        if Iflag!() as i32 != 0 && top != 0 {
            inter += 1;
            mail::check_mail();
        }
        n = parser::parse_cmd(inter);

        if n == nodes::NEOF {
            if !top != 0 || numeof >= 50 {
                break;
            }
            if !jobs::stopped_jobs() != 0 {
                if !(Iflag!() as i32 != 0) {
                    if iflag!() as i32 != 0 {
                        output::out2c('\n');
                    }
                    break;
                }
                output::out2str("\nUse \"exit\" to leave shell.\n");
            }
            numeof += 1;
        }
        else {
            let i: i32;

            unsafe {
                jobs::JOB_WARNING = if jobs::JOB_WARNING == 2 {1} else {0};
            }
            numeof += 1;

            i = eval::eval_tree(n, 0);
            if n != 0 {
                status = i;
            }
        }
        unsafe { 
            skip = eval::EVAL_SKIP;
        }

        if skip != 9 {
            unsafe {
                eval::EVAL_SKIP &= !(skip_func!() | skip_func_def!());
                break;
            }
        }

    }

    status
}
