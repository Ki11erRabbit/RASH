use std::path::{Path,PathBuf};
use std::env;
use std::fs::{File,metadata};
use std::io::{BufWriter,Write};
use crate::output;
use crate::mystring;
use crate::error;
use crate::var;
use super::{trace,sh_warnx,sh_err,exvwarning};
use crate::show::{TRACEFILE,tracefn};
use crate::exec;
use crate::options;

const CD_PHYSICAL: i32 = 1;
const CD_PRINT: i32 = 2;

static mut CUR_DIR: Option<String> = None;
static mut PHYSDIR: Option<String> = None;


fn cd_options() -> i32 {
    let mut flags = 0;

    let mut i: Option<char>;
    let mut j: char;

    j = 'L';
    while {i = options::nextopt("LP"); i} != None {
        if i != Some(j) {
            flags ^= CD_PHYSICAL;
            j = i.unwrap();
        }
    }

    flags
}

/*
 * Update curdir (the name of the current directory) in response to a 
 * cd command.
 */
fn updatepwd(dir: &str) -> Option<String> {
    
    let mut new: String = String::new();
    let mut p: Option<String>;
    let mut cdcomppath: String;
    
    cdcomppath = dir.to_string();
    if dir.get(..1).unwrap() != "/" {
        if unsafe {CUR_DIR == None} {
            return None;
        }
        new = unsafe { CUR_DIR.clone().unwrap() };
    } 
    new.reserve(dir.bytes().len() + 2);
    if dir.get(..1).unwrap() != "/" {
        if new.get(..1).unwrap() != "/" {
            new = "/".to_owned() + &new;
        }
    }
    
    let pos = match cdcomppath.find("/") {
        None => cdcomppath.len(),
        Some(pos) => pos,
    };

    p = Some(cdcomppath.drain(..pos).collect());
    
    while p != None {
        match p.as_ref().unwrap().as_str() {
            "." => {
                if p.as_ref().unwrap().len() == 1 && p.as_ref().unwrap().get(1..2).unwrap() == "."  {
                    break;
                }
                else if p.unwrap().len() == 0 {
                    break;
                }
            },
            _ => {
                new = new + &p.unwrap();
                new = new + "/";
            }

        }
        let pos = match cdcomppath.find("/") {
            None => cdcomppath.len(),
            Some(pos) => pos,
        };

        p = Some(cdcomppath.drain(..pos).collect());
        

    }

    new.truncate(new.len()-1);
    
    Some(new)
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
            eprintln!("getcwd() failed {}", &e);
            return Err(e);
        },
    }
}

/*
 * Actually do the chdir.  We also call hashcd to let the routines in exec.c
 * know that the current directory has changed.
 */
fn docd(dest: &str, flags: i32) -> Result<i32,std::io::Error>{
    let mut dir: String = String::new();
    let mut dest = dest;
    trace!("docd(\"{}\", {})", dest, flags);

    if (flags & CD_PHYSICAL) != 1 {
        dir = updatepwd(dest).unwrap();
        if dir != "" {
            dest = &dir;
        }
        
    }
    match env::set_current_dir(Path::new(dest)) {
        Ok(_) => {
            setpwd(Some(dir),true);
            exec::hashcd()
        },
        Err(e) => return Err(e),
    }

    Ok(0)
}

pub fn cdcmd(argc: i32, argv: Vec<String>) -> Option<i32> {

    let mut dest: Option<String> = None;
    let mut path: String;
    let mut p: &str;
    let mut c: Option<char> = None;

    let mut flags: i32;
    let mut len: i32;

    flags = cd_options();
    unsafe {
        if options::ARGLIST.len() > 0 {
            dest = Some(options::ARGLIST[0].to_string());
        }
    }
    if dest == None {
        dest = var::bltinlookup(mystring::HOMESTR);
    }
    else if dest.as_ref().unwrap().len() == 1 && dest.as_ref().unwrap().contains('-') {
        dest = var::bltinlookup("OLDPWD");
        flags |= CD_PRINT;
    }
    let mut goto_step6: bool = false;
    'out: loop {
        'step6: loop {
            if dest.as_ref() != None {
                dest = None;
            }
            if dest.as_ref().unwrap().get(..1).unwrap() == "/" {
                goto_step6 = true;
                break 'step6;
            }

            if dest.as_ref().unwrap().get(..1).unwrap() == "." {
                c = Some(dest.as_ref().unwrap().chars().collect::<Vec<char>>()[1]);
            
                'dotdot: loop {
                    match c { 
                        None => {
                            goto_step6 = true;
                            break 'step6;
                        },
                        Some(char) => match char {
                            '/'=> {
                                goto_step6 = true;
                                break 'step6;
                            },
                            '.'=> {
                                c = Some(dest.as_ref().unwrap().chars().collect::<Vec<char>>()[2]);
                                if c == Some('.') {
                                    break 'dotdot;
                                }
                            },
                            _ => return None,

                        }
                    }
                }
            }
            if dest == None {
                dest = Some(".".to_string());
            }
            break;
        }
        'docd: loop {
            if !goto_step6 {
                path = var::bltinlookup("CDPATH")?;
                while {let q = (p = &path, (len = exec::padvance_magic(vec![&path], dest.as_ref().unwrap(), 0))); len} >= 0 {
                    c = Some(p.chars().collect::<Vec<char>>()[0]);
                    
                    //if stat64(p, &statb) >= 0 && S_ISDIR(statb.st_mode) 
                    let md = metadata(p).expect("Invalid file path metadata");
                    if md.is_dir() {
                        if c != None && c.unwrap() != ':' {
                            flags |= CD_PRINT;
                        }

                        match docd(p, flags) {
                            Ok(_) => break 'out,
                            Err(_) => (),
                        }
                        sh_err!("can't cd to {}", dest);
                        return None;
                    }

                    
                }

            }
            else {
                goto_step6 = false;
                p = dest.as_ref().unwrap();
                match docd(p, flags) {
                    Ok(_) => break 'out,
                    Err(_) => {
                        eprintln!("can't cd to {}",dest.unwrap());
                        return None;
                    },
                }
            }

            break;
        }

        break;
    }

    if flags & CD_PRINT == CD_PRINT {
        //TODO output things
        //output::out1fmt(mystring::SHELL_NEWLINE_FMT,CUR_DIR);
    }


    Some(0)
}

pub fn pwdcmd(argc: i32, argv: Vec<String>) -> i32 {
    let flags: i32;
    let mut dir: String = unsafe {CUR_DIR.clone().unwrap()};

    flags = cd_options();
    if flags != 0 {
        if unsafe { PHYSDIR == None } {
            setpwd(Some(dir), false);
            dir = unsafe { PHYSDIR.clone().unwrap() };
        }
    }
    output::out1fmt(mystring::SHELL_NEWLINE_FMT,&dir);
    0
}

pub fn setpwd(val: Option<String>, setold:bool) {
    let mut oldcur: Option<String>;
    let mut dir: String;

    unsafe { 
        dir = CUR_DIR.clone().unwrap();
        oldcur = CUR_DIR.clone();
    };

    if setold {
        var::setvar("OLDPWD",Some(&oldcur.as_ref().unwrap()), var::VEXPORT);
    }
    
    match unsafe {PHYSDIR.take()} {
        Some(physdir) => {
            if physdir.as_str() == oldcur.as_ref().unwrap() {
                //free memory PHYSDIR
                unsafe {
                    PHYSDIR = None;
                }
            }
            else {
                unsafe {
                    PHYSDIR = Some(physdir);
                }
            }
        },
        None => (),
    }
    if val == None || oldcur.as_ref().unwrap() == val.as_ref().unwrap().as_str()  {
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
            //free memory oldcur
            oldcur = None;
        }
        unsafe {
            CUR_DIR = Some(dir.clone());
        }

        var::setvar("PWD", Some(&dir), var::VEXPORT);
    }


}
