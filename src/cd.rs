use std::path::{Path,PathBuf};
use std::env;
use crate::output;
use crate::mystring;
use crate::error;
use crate::var;

const CD_PHYSICAL: i32 = 1;
const CD_PRINT: i32 = 2;

static mut CUR_DIR: Option<String> = None;
static mut PHYSDIR: Option<String> = None;

//TODO define this function elsewhere and properly
fn nextopt(opt: &str) -> i32 {
    0
}

fn cd_options() -> i32 {
    let mut flags = 0;

    let mut i:i32;
    let mut j:i32;

    j = 'L' as i32;
    while {i = nextopt("LP"); i} != 0 {
        if i != j {
            flags ^= CD_PHYSICAL;
            j = i
        }
    }

    flags
}

/*
 * Update curdir (the name of the current directory) in response to a 
 * cd command.
 */
fn updatepwd(dir: &str) -> &'static str {
    "temp"
}


/*
 * Find out what the current directory is. If we already know the current
 * directory, this routine returns immediately.
 */
#[inline]
fn getpwd() -> Result<PathBuf,std::io::Error>{
    match env::current_dir() {
        Ok(dir) => return Ok(dir),
        Err(e) => {
            error::sh_warnx("getcwd() failed %s", &e);
            return Err(e);
        },
    }
}

/*
 * Do Change Directory
 */
fn docd(dest: &str, flags: i32) -> i32 {
    0
}

pub fn cdcmd(argc: i32, argv: Vec<String>) -> i32 {

    0
}

pub fn pwdcmd(argc: i32, argv: Vec<String>) -> i32 {
    let flags: i32;
    let mut dir: String = unsafe {CUR_DIR.clone().unwrap()};

    flags = cd_options();
    if flags != 0 {
        if unsafe { PHYSDIR == None } {
            setpwd(Some(&dir), false);
            dir = unsafe { PHYSDIR.clone().unwrap() };
        }
    }
    output::out1fmt(mystring::SHELL_NEWLINE_FMT,&dir);
    0
}

pub fn setpwd(val: Option<&str>, setold:bool) {
    let oldcur: Option<String>;
    let mut dir: String;

    unsafe { 
        dir = CUR_DIR.clone().unwrap();
        oldcur = CUR_DIR.clone();
    };

    if setold {
        var::setvar("OLDPWD",&oldcur.as_ref().unwrap(), var::VEXPORT);
    }
    
    match unsafe {PHYSDIR.clone()} {
        Some(string) => {
            if string.as_str() == oldcur.as_ref().unwrap() {
                //free memory
            }
        },
        None => (),
    }
    if val == None || oldcur.as_ref().unwrap() == val.unwrap()  {
        let path = env::current_dir().expect("Unable to get curr dir");
        unsafe {
            PHYSDIR = Some(path.to_str().unwrap().to_string().clone());
        }
        if val == None {
            dir = path.to_str().unwrap().to_string();
        }
        else {
            dir = val.unwrap().to_string();
        }
        if oldcur != Some(dir.clone()) && oldcur != None {
            //free memory
        }
        unsafe {
            CUR_DIR = Some(dir.clone());
        }

        var::setvar("PWD", &dir, var::VEXPORT);
    }


}
