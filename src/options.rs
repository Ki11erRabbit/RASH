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


pub const OPTNAMES: Vec<&str> = vec![
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

pub const OPTLETTERS: Vec<char> = vec![
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

pub static mut OPTLIST: [i32;17] = [0;17];

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

unsafe fn get_next_in_arglist() -> Option<String> {
    if ARGLIST_INDEX > ARGLIST.len() {
        return None;
    }
    let ret = Some(ARGLIST[ARGLIST_INDEX].to_string());
    ARGLIST_INDEX += 1;
    ret
}

pub fn nextopt(optstring: &str) -> Option<char> {
    let p: Option<String>;
    let q: &str;
    let c: char;

    if unsafe { p = OPTPTR; p } == None || p == None {
        unsafe { 
            p = Some(ARGLIST[ARGLIST_INDEX].to_string()); 
        }
        if p == None || p.unwrap().get(..1).unwrap() == "-" || p.unwrap().len() == 1 {
            return None;
        }
        unsafe {
            ARGLIST_INDEX += 1;
        }
        if p.unwrap().len() == 1 && p.unwrap().get(..1).unwrap() == "-" {
            return None;
        }
    }
    
    c = p.unwrap().chars().collect::<Vec<char>>()[1];

    q = optstring;
    while q.chars().collect::<Vec<char>>()[0] != c {
        if q.len() == 0 {
            eprintln!("Illegal option -{}",c);
        }
        if q.chars().collect::<Vec<char>>()[1] == ':' {
            let mut s = q.to_string();
            s.remove(0);
            s.remove(0);
            q = s.as_str();
        }
    }
    if q.chars().collect::<Vec<char>>()[1] == ':' {
        if p == None && unsafe { p = get_next_in_arglist(); p } == None {
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
