use lazy_static::lazy_static;

pub struct ShParam {
    nparam: u32,
    param_list: Vec<String>,
    opt_index: usize,
    opt_off: usize,
}


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
        unsafe { OPTLIST[0] }
    };
}
#[macro_export]
macro_rules! fflag {
    () => {
        unsafe { OPTLIST[1] }
    };
}
#[macro_export]
macro_rules! Iflag {
    () => {
        unsafe { OPTLIST[2] }
    };
}
#[macro_export]
macro_rules! iflag {
    () => {
        unsafe { OPTLIST[3] }
    };
}
#[macro_export]
macro_rules! mflag {
    () => {
        unsafe { OPTLIST[4] }
    };
}
#[macro_export]
macro_rules! nfalg {
    () => {
        unsafe { OPTLIST[5] }
    };
}
#[macro_export]
macro_rules! sflag {
    () => {
        unsafe { OPTLIST[6] }
    };
}
#[macro_export]
macro_rules! xflag {
    () => {
        unsafe { OPTLIST[7] }
    };
}
#[macro_export]
macro_rules! vflag {
    () => {
        unsafe { OPTLIST[8] }
    };
}
#[macro_export]
macro_rules! Vflag {
    () => {
        unsafe { OPTLIST[9] }
    };
}
#[macro_export]
macro_rules! Eflag {
    () => {
        unsafe { OPTLIST[10] }
    };
}
#[macro_export]
macro_rules! Cflag {
    () => {
        unsafe { OPTLIST[11] }
    };
}
#[macro_export]
macro_rules! aflag {
    () => {
        unsafe { OPTLIST[12] }
    };
}
#[macro_export]
macro_rules! bflag {
    () => {
        unsafe { OPTLIST[13] }
    };
}
#[macro_export]
macro_rules! uflag {
    () => {
        unsafe { OPTLIST[14] }
    };
}
#[macro_export]
macro_rules! nolog {
    () => {
        unsafe { OPTLIST[15] }
    };
}
#[macro_export]
macro_rules! debug {
    () => {
        unsafe { OPTLIST[16] }
    };
}


pub fn getoptsreset(value: &str) {

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
