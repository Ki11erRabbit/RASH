use std::collections::HashMap;
use std::env;
use crate::mail;
use crate::exec;
use crate::options;
use crate::myhistoryedit;
use crate::cd;
use lazy_static::lazy_static;
use std::sync::Mutex;
/* flags */
pub const VEXPORT:i32    = 0x01; // variable is exported
pub const VREADONLY:i32  = 0x02; // variable cannot be modified
pub const VSTRFIXED:i32  = 0x04; // variable struct is statically allocated
pub const VTEXTFIXED:i32 = 0x08; // text is statically allocated
pub const VSTACK:i32     = 0x10; // text is allocated on the stack
pub const VUNSET:i32     = 0x20; // the variable is not set
pub const VNOFUNC:i32    = 0x40; // don't call the callback function
pub const VNOSET:i32     = 0x80; // do not set variable - just readonly test
pub const VNOSAVE:i32    = 0x100;// when text is on the heap before setvareq

#[derive(Clone)]
pub struct Var {
    flags: i32,         // flags are defined above
    pub text: String, // name=value
    func: Option<fn(&str)>,     // the function to be called when the variable gets set/unset 
}

impl Var {
    pub fn new(flags:i32, text: &str, func: Option<fn(&str)>) -> Var {
        Var {flags: flags, text: text.to_string(), func: func }
    }

    pub fn get_val(&self, index: usize) -> &str {
        self.text.get(index..).unwrap()
    }
}

#[derive(Clone)]
pub struct LocalVar {
    var: Var,           // the variable that was made local
    flags: i32,         // saved flags
    text: &'static str, // saved text
}

struct LocalVarList {

}

static mut LINENUM: usize = 0;
const DEFOPTINDVAR: &str = "OPTIND=1";
const DEFPATHVAR: &str = "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";
const DEFIFSVAR: &str = "IFS= \t\n";
const LINENOVAR: &str = "LINENO=";
lazy_static! {
    pub static ref LOCALVARS: HashMap<String, LocalVar> = HashMap::new();
    pub static ref VARTAB: Mutex<HashMap<String, Box<Var>>> = Mutex::new(HashMap::new());
    pub static ref VARINIT: Vec<Box<Var>> = vec![
	Box::new(Var::new(VSTRFIXED|VTEXTFIXED|VUNSET,   	"ATTY",	    None )),
	Box::new(Var::new( 	VSTRFIXED|VTEXTFIXED,		    DEFIFSVAR,	    None )),
	Box::new(Var::new( 	VSTRFIXED|VTEXTFIXED|VUNSET,	"MAIL",	    Some(mail::changemail) )),
	Box::new(Var::new( 	VSTRFIXED|VTEXTFIXED|VUNSET,	"MAILPATH",	Some(mail::changemail) )),
	Box::new(Var::new( 	VSTRFIXED|VTEXTFIXED,		    DEFOPTINDVAR,	    Some(exec::changepath) )),
	Box::new(Var::new( 	VSTRFIXED|VTEXTFIXED,		    "PS1=$ ",	    None )),
	Box::new(Var::new( 	VSTRFIXED|VTEXTFIXED,		    "PS2=> ",	    None )),
	Box::new(Var::new( 	VSTRFIXED|VTEXTFIXED,		    "PS4=+ ",	    None )),
	Box::new(Var::new( 	VSTRFIXED|VTEXTFIXED,		    DEFOPTINDVAR,	Some(options::getoptsreset) )),
	Box::new(Var::new( 	VSTRFIXED|VTEXTFIXED,		    LINENOVAR,	    None )),
	Box::new(Var::new( 	VSTRFIXED|VTEXTFIXED|VUNSET,	"TERM",	    None )),
	Box::new(Var::new( 	VSTRFIXED|VTEXTFIXED|VUNSET,	"HISTSIZE",	Some(myhistoryedit::sethistsize) )),
];

    pub static ref VATTY: Box<Var> = VARINIT[0].clone();
    pub static ref VIFS: Box<Var> = VARINIT[1].clone();
    pub static ref VMAIL: Box<Var> = VARINIT[2].clone();
    pub static ref VMPATH: Box<Var> = VARINIT[3].clone();
    pub static ref VPATH: Box<Var> = VARINIT[4].clone();
    pub static ref VPS1: Box<Var> = VARINIT[5].clone();
    pub static ref VPS2: Box<Var> = VARINIT[6].clone();
    pub static ref VPS4: Box<Var> = VARINIT[7].clone();
    pub static ref VOPTIND: Box<Var> = VARINIT[8].clone();
    pub static ref VLINENUM: Box<Var> = VARINIT[9].clone();
    pub static ref VTERM: Box<Var> = VARINIT[10].clone();
    pub static ref VHISTSIZE: Box<Var> = VARINIT[11].clone();
}

/*
 * The following macros access the values of the above variables.
 * They have to skip over the name.  They return the null string
 * for unset variables.
 */

macro_rules! ifsval {
    () => {
        VIFS.get(4)
    };
}
macro_rules! ifsset {
    () => {
        (VIFS.flags & VUNSET) == 0
    };
}
macro_rules! mailval {
    () => {
        VMAIL.get(5)
    };
}
macro_rules! mpathval {
    () => {
        VMPATH.get(9)
    };
}
macro_rules! pathval {
    () => {
        VPATH.get(5)
    };
}
macro_rules! ps1val {
    () => {
        VPS1.get(4)
    };
}
macro_rules! ps2val {
    () => {
        VPS2.get(4)
    };
}
macro_rules! ps4val {
    () => {
        VPS4.get(4)
    };
}
macro_rules! optindval {
    () => {
        VOPTIND.get(7)
    };
}
macro_rules! linenumval {
    () => {
        VLINENUM.get(7)
    };
}
macro_rules! histsizevall {
    () => {
        VHISTSIZE.get(9)
    };
}
macro_rules! termval {
    () => {
        VTERM.get(5)
    };
}
macro_rules! attyset {
    () => {
        (vatty.flags & VUNSET) == 0
    };
}
macro_rules! mpathset {
    () => {
        (VMPATH.flags & VUNSET) == 0
    };
}

pub fn init() {
    
    initvar();
    
    let p = match env::var("PWD") {
        Ok(key) => Some(key),
        Err(_) => None,
    };

    cd::setpwd(p,false)
}


/*
 * This routine initializes the builtin variables.  It is called when the
 * shell is initialized.
 */
pub fn initvar() {
    
    for i in 0..VARINIT.len() {
        let pos = match VARINIT[i].text.find("=") {
            None => VARINIT[i].text.len(),
            Some(pos) => pos,
        };
        VARTAB.lock().unwrap().insert(VARINIT[i].text.get(0..pos).unwrap().to_string(), VARINIT[i].clone());
    }



}

pub fn tempfunc(temp: &str) {

}

pub fn setvar(name: &str, val: &str, flags: i32) -> Box<Var> {
    
    Box::new(Var::new(VEXPORT, "temp",Some(tempfunc)))
}

fn lookupvar(name: &str) -> Option<&str> {
    None
}


#[inline]
pub fn bltinlookup(name: &str) -> Option<&str> {
    lookupvar(name)
}

