use std::collections::{BTreeMap,HashMap};
use std::env;
use std::{error::Error, fmt};
use crate::mail;
use crate::exec;
use crate::options::OPTLIST;
use crate::options;
use crate::myhistoryedit;
use crate::cd;
use lazy_static::lazy_static;
use std::sync::{Mutex, Arc};
use nix::unistd::geteuid;
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

#[derive(Debug)]
pub struct VarError {
    pub msg: String,
}

impl VarError {
    pub fn new(msg: &str) -> VarError {
        VarError { msg: msg.to_string() }
    }
}

impl Error for VarError {}


impl fmt::Display for VarError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg )
    }
}


#[derive(Clone)]
pub struct Var {
    pub flags: i32,         // flags are defined above
    pub text: String, // name=value
    pub func: Option<fn(&str)>,     // the function to be called when the variable gets set/unset 
}

impl Var {
    pub fn new(flags:i32, text: &str, func: Option<fn(&str)>) -> Var {
        Var {flags: flags, text: text.to_string(), func: func }
    }

    pub fn get_val_index(&self, index: usize) -> &str {
        self.text.get(index..).unwrap()
    }
    pub fn get_val(&self) -> Option<String> {
        let pos = match self.text.find("=") {
            None => return None,
            Some(pos) => pos + 1 ,
        };
        Some(self.text.get(pos..).unwrap().to_string())
    }

    pub fn get_key(&self) -> String {
        
        let pos = match self.text.find("=") {
            None => self.text.len(),
            Some(pos) => pos,
        };
        
        self.text.get(..pos).unwrap().to_string()
    }

    pub fn check_func(&self) -> bool {
        let func_set = match self.func {
            Some(_) => true,
            None => false,
        };
        let vnofunc_set = if self.flags & VNOFUNC == VNOFUNC {true} else {false};

        func_set && vnofunc_set
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
lazy_static! {
    static ref DEFOPTINDVAR: String = "OPTIND=1".to_string();
    static ref DEFPATHVAR: String = "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string();
    static ref DEFIFSVAR: String = "IFS= \t\n".to_string();
    static ref LINENOVAR: String = "LINENO=".to_string();
    pub static ref LOCALVARS: HashMap<String, LocalVar> = HashMap::new();
    pub static ref VARTAB: Mutex<BTreeMap<String, Arc<Mutex<Var>>>> = Mutex::new(BTreeMap::new());
    pub static ref VARINIT: Vec<Arc<Mutex<Var>>> = vec![
        Arc::new(Mutex::new(Var::new(VSTRFIXED|VTEXTFIXED|VUNSET,   	"ATTY",	    None ))),
        Arc::new(Mutex::new(Var::new( 	VSTRFIXED|VTEXTFIXED,	    	&DEFIFSVAR,	    None ))),
        Arc::new(Mutex::new(Var::new( 	VSTRFIXED|VTEXTFIXED|VUNSET,	"MAIL",	    Some(mail::changemail) ))),
        Arc::new(Mutex::new(Var::new( 	VSTRFIXED|VTEXTFIXED|VUNSET,	"MAILPATH",	Some(mail::changemail) ))),
        Arc::new(Mutex::new(Var::new( 	VSTRFIXED|VTEXTFIXED,		    &DEFOPTINDVAR,	    Some(exec::changepath) ))),
        Arc::new(Mutex::new(Var::new( 	VSTRFIXED|VTEXTFIXED,		    "PS1=$ ",	    None ))),
        Arc::new(Mutex::new(Var::new( 	VSTRFIXED|VTEXTFIXED,		    "PS2=> ",	    None ))),
        Arc::new(Mutex::new(Var::new( 	VSTRFIXED|VTEXTFIXED,		    "PS4=+ ",	    None ))),
        Arc::new(Mutex::new(Var::new( 	VSTRFIXED|VTEXTFIXED,		    &DEFOPTINDVAR,	Some(options::getoptsreset) ))),
        Arc::new(Mutex::new(Var::new( 	VSTRFIXED|VTEXTFIXED,		&LINENOVAR,	    None ))),
        Arc::new(Mutex::new(Var::new( 	VSTRFIXED|VTEXTFIXED|VUNSET,	"TERM",	    None ))),
        Arc::new(Mutex::new(Var::new( 	VSTRFIXED|VTEXTFIXED|VUNSET,	"HISTSIZE",	Some(myhistoryedit::sethistsize) ))),
];

    pub static ref VATTY: Arc<Mutex<Var>> = VARINIT[0].clone();
    pub static ref VIFS: Arc<Mutex<Var>> = VARINIT[1].clone();
    pub static ref VMAIL: Arc<Mutex<Var>> = VARINIT[2].clone();
    pub static ref VMPATH: Arc<Mutex<Var>> = VARINIT[3].clone();
    pub static ref VPATH: Arc<Mutex<Var>> = VARINIT[4].clone();
    pub static ref VPS1: Arc<Mutex<Var>> = VARINIT[5].clone();
    pub static ref VPS2: Arc<Mutex<Var>> = VARINIT[6].clone();
    pub static ref VPS4: Arc<Mutex<Var>> = VARINIT[7].clone();
    pub static ref VOPTIND: Arc<Mutex<Var>> = VARINIT[8].clone();
    pub static ref VLINENUM: Arc<Mutex<Var>> = VARINIT[9].clone();
    pub static ref VTERM: Arc<Mutex<Var>> = VARINIT[10].clone();
    pub static ref VHISTSIZE: Arc<Mutex<Var>> = VARINIT[11].clone();
}

/*
 * The following macros access the values of the above variables.
 * They have to skip over the name.  They return the null string
 * for unset variables.
 */

macro_rules! ifsval {
    () => {
        VIFS.lock().unwrap().get_val_index(4)
    };
}
macro_rules! ifsset {
    () => {
        (VIFS.lock().unwrap()flags & VUNSET) == 0
    };
}
macro_rules! mailval {
    () => {
        VMAIL.lock().unwrap().get_val_index(5)
    };
}
macro_rules! mpathval {
    () => {
        VMPATH.lock().unwrap().get_val_index(9)
    };
}
macro_rules! pathval {
    () => {
        VPATH.lock().unwrap().get_val_index(5)
    };
}
macro_rules! ps1val {
    () => {
        VPS1.lock().unwrap().get_val_index(4)
    };
}
macro_rules! ps2val {
    () => {
        VPS2.lock().unwrap().get_val_index(4)
    };
}
macro_rules! ps4val {
    () => {
        VPS4.lock().unwrap().get_val_index(4)
    };
}
macro_rules! optindval {
    () => {
        VOPTIND.lock().unwrap().get_val_index(7)
    };
}
macro_rules! linenumval {
    () => {
        VLINENUM.lock().unwrap().get_val_index(7)
    };
}
macro_rules! histsizevall {
    () => {
        VHISTSIZE.lock().unwrap().get_val_index(9)
    };
}
macro_rules! termval {
    () => {
        VTERM.lock().unwrap().get_val_index(5)
    };
}
macro_rules! attyset {
    () => {
        (vatty.lock().unwrap()flags & VUNSET) == 0
    };
}
macro_rules! mpathset {
    () => {
        (VMPATH.lock().unwrap()flags & VUNSET) == 0
    };
}

pub fn init() -> Result<(),VarError>{
    
    initvar();
    // add environment variables to vars
    for var in env::vars() {
        let temp_var = var.0 + "=" + &var.1;
        setvareq(&temp_var, VEXPORT|VTEXTFIXED)?;
    } 
    

    setvareq(&DEFIFSVAR, VTEXTFIXED)?; 
    setvareq(&DEFOPTINDVAR, VTEXTFIXED)?;

    let p = match env::var("PWD") {
        Ok(key) => Some(key),
        Err(_) => None,
    };

    cd::setpwd(p,false);
    return Ok(());
}


/*
 * This routine initializes the builtin variables.  It is called when the
 * shell is initialized.
 */
pub fn initvar() {
    
    for i in 0..VARINIT.len() {
        /*let pos = match VARINIT[i].lock().unwrap().text.find("=") {
            None => VARINIT[i].lock().unwrap().text.len(),
            Some(pos) => pos,
        };
        VARTAB.lock().unwrap().insert(VARINIT[i].lock().unwrap().text.get(0..pos).unwrap().to_string(), VARINIT[i].clone());*/
        VARTAB.lock().unwrap().insert(VARINIT[i].lock().unwrap().get_key(), VARINIT[i].clone());
    }

    if geteuid().is_root() {
        VPS1.lock().unwrap().text = "PS1=# ".to_string();
    }

}

pub fn setvar(name: &str,val: Option<&str>,flags: i32) -> Result<Option<Arc<Mutex<Var>>>,VarError> {
    let mut flags = flags;
    let name_end: usize = name.len();
    if name_end == 0 {
        panic!("{}: bad variable name", name_end);
    }
    let mut val_len = 0;
    if val == None {
        flags |= VUNSET;
    }
    else {
        val_len = val.unwrap().len();
    }
    let mut set_var: String = name.to_string() + "=";
    match val {
        Some(val) => set_var = set_var + val,
        None => (),
    }

    let var = setvareq(&set_var, flags | VNOSAVE)?;

    Ok(var)
}

/*
 * Same as setvar except that the variable and value are passed in
 * the first argument as name=value.  Since the first argument will
 * be actually stored in the table, it should not be a string that
 * will go away.
 * Called with interrupts off.
 */
fn setvareq(var_set: &str, flags: i32) -> Result<Option<Arc<Mutex<Var>>>,VarError> {

    let mut flags = flags;
    flags |= VEXPORT & (( (1 - super::aflag!())) - 1);

    let mut var: Option<Arc<Mutex<Var>>> = findvar(var_set);
    
    match &var {
        Some(var) => {
            if var.lock().unwrap().flags & VREADONLY == VREADONLY {
                if var.lock().unwrap().flags & VNOSAVE == VNOSAVE {
                    //free var_set
                }
                
                
                let msg = format!("{}: is read only",var.lock().unwrap().get_key());
                return Err(VarError::new(&msg));  
            } 
            if flags & VNOSET == VNOSET {
                return Ok(Some(var.clone()));
            }
            if var.lock().unwrap().check_func() {
                var.lock().unwrap().func = Some(tempfunc)
            }
            if var.lock().unwrap().flags & (VTEXTFIXED | VSTACK) == 0 {
                //put var.text on the heap
            }
            if var.lock().unwrap().flags & (VEXPORT|VREADONLY|VSTRFIXED|VUNSET) | (var.lock().unwrap().flags & VSTRFIXED) == VUNSET {
                //free var
                let var_name = var.lock().unwrap().get_key();
                VARTAB.lock().unwrap().remove(&var_name);
                if flags & (VTEXTFIXED|VSTACK|VNOSAVE) == VNOSAVE {
                    //free var_set
                }

                return Ok(None);
            }
            flags |= var.lock().unwrap().flags & !(VTEXTFIXED|VSTACK|VNOSAVE|VUNSET);
        },
        None => {
            if flags & VNOSET == VNOSET {
                return Ok(None);
            }
            if (flags & (VEXPORT|VREADONLY|VSTRFIXED|VUNSET)) == VUNSET {
                if flags & (VTEXTFIXED|VSTACK|VNOSAVE) == VNOSAVE {
                    //free var_set
                }

                return Ok(None); 
            }
            // not found
            var = Some(Arc::new(Mutex::new(Var::new(0, "",None))));
        },
    }
    if !(flags &(VTEXTFIXED|VSTACK|VNOSAVE) == (VTEXTFIXED|VSTACK|VNOSAVE)) {
        //put String on the stack
    }

    var.as_mut().unwrap().lock().unwrap().text = var_set.to_string();
    var.as_mut().unwrap().lock().unwrap().flags = flags;
    let ret_val = var;
    Ok(ret_val)
}


pub fn tempfunc(temp: &str) {

}

fn findvar(var_set: &str) -> Option<Arc<Mutex<Var>>> {
    let var;
    
    let pos = match var_set.find("=") {
        None => var_set.len(),
        Some(pos) => pos,
    };
    
    let vartable = VARTAB.lock().unwrap();

    var = vartable.get(var_set.get(..pos).unwrap());
    
    match var {
        None => return None,
        Some(var) => return Some(var.clone()),
    }
    
}

/*
 * Find the value of a variable.  Returns NULL if not set.
 */
fn lookupvar(name: &str) -> Option<String> {
    
    let result = findvar(name)?; 
    
    return result.lock().unwrap().get_val();

    
}


#[inline]
pub fn bltinlookup(name: &str) -> Option<String> {
    lookupvar(name)
}

