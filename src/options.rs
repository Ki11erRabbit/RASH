use lazy_static::lazy_static;

pub struct ShellParam {
    pub num_param: usize,
    pub param_list: Vec<String>,
    pub opt_index: usize,
    pub opt_off: isize,
}


pub static mut SHELLPARAM: ShellParam = ShellParam {
    num_param: 0,               // number of positionall parameters (without $0)
    param_list: Vec::new(),     // list of positional parameters
    opt_index: 0,               // next parameter ot be processed by getopts
    opt_off: 0,                 // used by getopts
};

pub static MINUSC: Option<String> = None;
pub static mut ARGLIST: Vec<&str> = Vec::new();
pub static mut ARGLIST_INDEX: usize = 0;
pub static mut OPTPTR: Option<String> = None;
pub static mut OPTION_ARG: Option<String> = None;
pub static mut ARGPOINTER: Vec<Option<String>> = Vec::new();
lazy_static!{

    pub static ref OPTNAMES: Vec<&'static str> = vec![
        "errexit",
        "noglob",
        "ignoreeof",
        "interactive",
        "monitor",
        "noexec",
        "stdin",
        "xtrace",
        "verbose",
        "vi",
        "emacs",
        "noclobber",
        "allexport",
        "notify",
        "nounset",
        "nolog",
        "debug",
    ];

    pub static ref OPTLETTERS: Vec<char> = vec![
        'e',
        'f',
        'I',
        'i',
        'm',
        'n',
        's',
        'x',
        'v',
        'V',
        'E',
        'C',
        'a',
        'b',
        'u',
        0 as char,
        0 as char,
    ];

}

pub static mut OPTLIST: [char;17] = [0 as char ;17];

#[macro_export]
macro_rules! eflag {
    () => {
        unsafe { crate::options::OPTLIST[0] }
    };
    ($val:expr) => {
        unsafe { crate::options::OPTLIST[0] = $val }
    };
}
#[macro_export]
macro_rules! fflag {
    () => {
        unsafe { crate::options::OPTLIST[1] }
    };
    ($val:expr) => {
        unsafe { crate::options::OPTLIST[1] = $val }
    };
}
#[macro_export]
macro_rules! Iflag {
    () => {
        unsafe { crate::options::OPTLIST[2] }
    };
    ($val:expr) => {
        unsafe { crate::options::OPTLIST[2] = $val }
    };
}
#[macro_export]
macro_rules! iflag {
    () => {
        unsafe { crate::options::OPTLIST[3] }
    };
    ($val:expr) => {
        unsafe { crate::options::OPTLIST[3] = $val }
    };
}
#[macro_export]
macro_rules! mflag {
    () => {
        unsafe { crate::options::OPTLIST[4] }
    };
    ($val:expr) => {
        unsafe { crate::options::OPTLIST[4] = $val }
    };
}
#[macro_export]
macro_rules! nflag {
    () => {
        unsafe { crate::options::OPTLIST[5] }
    };
    ($val:expr) => {
        unsafe { crate::options::OPTLIST[5] = $val }
    };
}
#[macro_export]
macro_rules! sflag {
    () => {
        unsafe { crate::options::OPTLIST[6] }
    };
    ($val:expr) => {
        unsafe { crate::options::OPTLIST[6] = $val }
    };
}
#[macro_export]
macro_rules! xflag {
    () => {
        unsafe { crate::options::OPTLIST[7] }
    };
    ($val:expr) => {
        unsafe { crate::options::OPTLIST[7] = $val }
    };
}
#[macro_export]
macro_rules! vflag {
    () => {
        unsafe { crate::options::OPTLIST[8] }
    };
    ($val:expr) => {
        unsafe { crate::options::OPTLIST[8] = $val }
    };
}
#[macro_export]
macro_rules! Vflag {
    () => {
        unsafe { crate::options::OPTLIST[9] }
    };
    ($val:expr) => {
        unsafe { crate::options::OPTLIST[9] = $val }
    };
}
#[macro_export]
macro_rules! Eflag {
    () => {
        unsafe { crate::options::OPTLIST[10] }
    };
    ($val:expr) => {
        unsafe { crate::options::OPTLIST[10] = $val }
    };
}
#[macro_export]
macro_rules! Cflag {
    () => {
        unsafe { crate::options::OPTLIST[11] }
    };
    ($val:expr) => {
        unsafe { crate::options::OPTLIST[11] = $val }
    };
}
#[macro_export]
macro_rules! aflag {
    () => {
        unsafe { crate::options::OPTLIST[12] }
    };
    ($val:expr) => {
        unsafe { crate::options::OPTLIST[12] = $val }
    };
}
#[macro_export]
macro_rules! bflag {
    () => {
        unsafe { crate::options::OPTLIST[13] }
    };
    ($val:expr) => {
        unsafe { crate::options::OPTLIST[13] = $val }
    };
}
#[macro_export]
macro_rules! uflag {
    () => {
        unsafe { crate::options::OPTLIST[14] }
    };
    ($val:expr) => {
        unsafe { crate::options::OPTLIST[14] = $val }
    };
}
#[macro_export]
macro_rules! nolog {
    () => {
        unsafe { crate::options::OPTLIST[15] }
    };
    ($val:expr) => {
        unsafe { crate::options::OPTLIST[15] = $val }
    };
}
#[macro_export]
macro_rules! debug {
    () => {
        unsafe { crate::options::OPTLIST[16] }
    };
    ($val:expr) => {
        unsafe { crate::options::OPTLIST[16] = $val }
    };
}


pub fn getoptsreset(value: &str) {

}

pub fn opts_changed() {

}

unsafe fn get_next_in_arglist(arg: &mut Option<String> ) -> Option<String> {
    if ARGLIST_INDEX > ARGLIST.len() {
        *arg = None;
        return None;
    }
    let arg = Some(ARGLIST[ARGLIST_INDEX].to_string());
    ARGLIST_INDEX += 1;
    arg
}

pub fn nextopt(optstring: &str) -> Option<char> {
    let mut p: Option<String>;
    let mut q: String;
    let c: char;
    
    unsafe {
        p = OPTPTR.clone();
    }
    if p == None || p == None {
        unsafe { 
            p = Some(ARGLIST[ARGLIST_INDEX].to_string()); 
        }
        if p == None || p.as_ref().unwrap().get(..1).unwrap() == "-" || p.as_ref().unwrap().len() == 1 {
            return None;
        }
        unsafe {
            ARGLIST_INDEX += 1;
        }
        if p.as_ref().unwrap().len() == 1 && p.as_ref().unwrap().get(..1).unwrap() == "-" {
            return None;
        }
    }
    
    c = p.as_ref().unwrap().chars().collect::<Vec<char>>()[1];

    q = optstring.to_string();
    while q.chars().collect::<Vec<char>>()[0] != c {
        if q.len() == 0 {
            eprintln!("Illegal option -{}",c);
        }
        if q.chars().collect::<Vec<char>>()[1] == ':' {
            let mut s = q.to_string();
            s.remove(0);
            s.remove(0);
            q = s;
        }
    }
    if q.chars().collect::<Vec<char>>()[1] == ':' {
        if p == None && unsafe { get_next_in_arglist(&mut p) == None } {
            eprintln!("No arg for -{}", c);
        }
        unsafe {
            OPTION_ARG = p;
            p = None;
        }
    }
    unsafe {
        OPTPTR = p;
    }
    Some(c)
}

pub fn process_args(argc: usize, argv: Vec<String>) -> usize {
    let mut login = 0;



    login
}

