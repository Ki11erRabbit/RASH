use crate::nodes::{Node,FuncNode};
use crate::builtins::BuiltInCmd;

pub union Param {
    pub index: i32,
    pub cmd: BuiltInCmd,
    pub func: std::mem::ManuallyDrop<FuncNode>,
}

pub struct CmdEntry {
    pub cmd_type: i32,
    pub param: Param, 
}


pub const CMD_UNKNOWN: i32 = -1;    // no entry in table for command
pub const CMD_NORMAL: i32 = 0;      // command is an executable progam
pub const CMD_FUNCTION: i32 = 1;   // command is a shell function
pub const CMD_BUILTIN: i32 = 2;     // command is a shell builtin


/* action to find_command() */
pub const DO_ERR: i32 =	    	0x01;	/* prints errors */
pub const DO_ABS: i32 =		    0x02;	/* checks absolute paths */
pub const DO_NOFUNC: i32 =	    0x04;	/* don't return shell functions, for command */
pub const DO_ALTPATH: i32 =	    0x08;	/* using alternate path */
pub const DO_REGBLTIN: i32 =	0x10;	/* regular built-ins and functions only */

pub static mut PATHOPT: Option<String> = None;

pub fn hashcd() {


}

pub fn unset_func(name: Option<String>) {

}

pub fn changepath(newval: &str) {

}


/*
 * Do a path search.  The variable path (passed by reference) should be
 * set to the start of the path before the first call; padvance will update
 * this value as it proceeds.  Successive calls to padvance will return
 * the possible path expansions in sequence.  If an option (indicated by
 * a percent sign) appears in the path entry then the global variable
 * pathopt will be set to point to it; otherwise pathopt will be set to
 * NULL.
 *
 * If magic is 0 then pathopt recognition will be disabled.  If magic is
 * 1 we shall recognise %builtin/%func.  Otherwise we shall accept any
 * pathopt.
 */
pub fn padvance_magic(path: Vec<String>, name: &str, magic: i32) -> i32 {
     
    0
}

#[inline]
pub fn padvance(path: Vec<String>, name: &str) -> i32 {
    padvance_magic(path, name, 1)
}



pub fn shell_exec(argv: Vec<String>, path: &str, idx: i32) {

}

/*
 * Define a shell function.
 */
pub fn def_func(func: Box<Option<Node>>) {


}
