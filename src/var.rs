use std::collections::{BTreeMap,HashMap};
use std::env;
use std::{error::Error, fmt};
use crate::mail;
use crate::exec;
use crate::options::OPTLIST;
use crate::options;
use crate::myhistoryedit;
use crate::cd;
use super::trace;
use crate::show::{tracefn, TRACEFILE};
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

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.text )
    }
}

#[derive(Clone)]
pub struct LocalVar {
    var: Option<Arc<Mutex<Var>>>,           // the variable that was made local
    flags: i32,         // saved flags
    text: String, // saved text
}

impl LocalVar {

    pub fn new(flags:i32, text: &str, var: Option<Arc<Mutex<Var>>>) -> LocalVar {
        LocalVar {flags: flags, text: text.to_string(), var: var }
    }
}


static mut LINENUM: usize = 0;
lazy_static! {
    static ref DEFOPTINDVAR: String = "OPTIND=1".to_string();
    static ref DEFPATHVAR: String = "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string();
    static ref DEFIFSVAR: String = "IFS= \t\n".to_string();
    static ref LINENOVAR: String = "LINENO=".to_string();
    pub static ref LOCALVARS: HashMap<String, LocalVar> = HashMap::new();
    pub static ref VARTAB: Mutex<BTreeMap<String, Arc<Mutex<Var>>>> = Mutex::new(BTreeMap::new());
    pub static ref LOCALVAR_STACK: Mutex<Vec<Arc<Mutex<LocalVar>>>> = Mutex::new(Vec::new());
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

pub fn set_var_int(name: &str, val: isize, flags: i32) -> Result<isize,VarError> {
    setvar(name,Some(val.to_string()),flags)?;
    Ok(val)
}


pub fn setvar(name: &str,val: Option<String>,flags: i32) -> Result<Option<Arc<Mutex<Var>>>,VarError> {
    let mut flags = flags;
    let name_end: usize = name.len();
    if name_end == 0 {
        let msg = format!("{}: bad variable name", name_end);
        return Err(VarError::new(&msg));
    }
    let mut val_len = 0;
    if val == None {
        flags |= VUNSET;
    }
    else {
        val_len = val.unwrap().len();
    }
    let mut set_var: String = name.to_string();
    match val {
        Some(val) => set_var = set_var + "=" + &val,
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
    flags |= VEXPORT & (( (1 - super::aflag!() as i32)) - 1);

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

/*
 * POSIX requires that 'set' (but not export or readonly) output the
 * variables in lexicographic order - by the locale's collating order (sigh).
 * Maybe we could keep them in an ordered balanced binary tree
 * instead of hashed lists.
 * For now just roll 'em through qsort for printing...
 *
 * haha in rust I can just roll with a BTreeMap to solve this problem
 */
pub fn showvars() {
    
    for (key,var) in VARTAB.lock().unwrap().iter() {
        println!("{}",var.lock().unwrap());
    }

}

pub fn export_cmd(argc:i32, argv: Vec<String>) {
    
    let flag = if argv[0].chars().collect::<Vec<char>>()[0] == 'r' { VREADONLY } else { VEXPORT };
    
    let next_opt: i32 = match options::nextopt("p") {
        Some(opt) => opt as i32 - 'p' as i32,
        None => 0 - 'p' as i32,
    };
    let mut name;
    unsafe {
        name = &options::ARGPOINTER[0];
    }
    let mut pos = 0;
    let mut val;
    if next_opt != 0 && *name != None {
        loop {
            if name.as_ref().unwrap().contains("=") {
                val = match name.as_ref().unwrap().find('=') {
                    Some(pos) => Some(name.as_ref().unwrap().get(pos+1..).unwrap().to_string()),
                    None => None,
                };
                setvar(name.as_ref().unwrap(), val, flag);
            }
            else {
                let var = findvar(&name.as_ref().unwrap());
                match var {
                    Some(var) => var.lock().unwrap().flags |= flag,
                    None => {
                        setvar(&name.as_ref().unwrap(), val, flag);
                    },
                }
            }

            unsafe {
                if pos < options::ARGPOINTER.len() {
                    pos += 1;
                    name = &options::ARGPOINTER[pos];
                }
                else {
                    break;
                }
            }
        }
    }
}

/*
 * The "local" command.
 */

pub fn local_cmd(argc:i32, argv: Vec<String>) -> Result<(),VarError> {
    

    if LOCALVAR_STACK.lock().unwrap().len() == 0 {
        return Err(VarError::new("not in a function"));
    }
    
    let argv;
    unsafe {
        argv = &options::ARGPOINTER;
    }

    for arg in argv.iter() {
        make_local(&arg.unwrap(),0);
    }
    
    Ok(())
}

pub fn make_local(name: &str,flags:i32) -> Result<(),VarError>{
    
    let local_var: LocalVar;
    let mut var;
    
    if name.chars().collect::<Vec<char>>()[0] == '-' && name.len() == 1 {
        unsafe {
            local_var.text = options::OPTLIST.iter().collect::<String>();
            var = None;
        }
    }
    else {
        var = findvar(name);
        let equal = name.contains("=");
        match &mut var {
            None => {
                if equal {
                    var = setvareq(name, VSTRFIXED | flags)?;
                }
                else {
                    var = setvar(name,None, VSTRFIXED | flags)?;
                }
                local_var.flags = VUNSET;
            },
            Some(var) => {
                let temp_var = var.lock().unwrap();
                local_var.text = temp_var.text;
                local_var.flags = temp_var.flags;
                temp_var.flags |= VSTRFIXED|VTEXTFIXED;
                if equal {
                    setvareq(name, flags)?;
                }
            }
        }
    }

    local_var.var = Some(var.unwrap().clone());
    LOCALVAR_STACK.lock().unwrap().push(Arc::new(Mutex::new(local_var)));

    Ok(())
}


/*
 * Called after a function returns.
 * Interrupts must be off.
 */
pub fn pop_local_vars() {
    
    while LOCALVAR_STACK.lock().unwrap().len() != 0 {
        let local_var = LOCALVAR_STACK.lock().unwrap().pop();
        let var = local_var.unwrap().lock().unwrap().var;
        
        let output = match &var {
            None => "-".to_owned(),
            Some(val) => val.lock().unwrap().text,
        };

        trace!("poplocalvar{}\n",output);
        match &var {
            None => {

            },
            Some(var) => {
                if local_var.unwrap().lock().unwrap().flags == VUNSET {

                }
                else {
                    
                }

            },


        }
    }

}
