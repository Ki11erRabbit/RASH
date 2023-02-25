use crate::nodes::Node;
/*
 * expandarg() flags
 */
pub const EXP_FULL: i32 = 0x1;  	/* perform word splitting & file globbing */
pub const EXP_TILDE: i32 = 0x2;	    /* do normal tilde expansion */
pub const EXP_VARTILDE: i32 = 0x4;	/* expand tildes in an assignment */
pub const EXP_REDIR: i32 = 0x8;	    /* file glob for a redirection (1 match only) */
pub const EXP_CASE: i32 = 0x10;	    /* keeps quotes around for CASE pattern */
pub const EXP_VARTILDE2: i32 = 0x40;/* expand tildes after colons only */
pub const EXP_WORD: i32 = 0x80;	    /* expand word in parameter expansion */
pub const EXP_QUOTED: i32 = 0x100;	/* expand word in double quotes */
pub const EXP_KEEPNUL: i32 = 0x200;	/* do not skip NUL characters */
pub const EXP_DISCARD: i32 = 0x400;	/* discard result of expansion */


pub struct ArgList {
    pub list: Vec<String>,
}




/*
 * Perform variable substitution and command substitution on an argument,
 * placing the resulting list of arguments in arglist.  If EXP_FULL is true,
 * perform splitting and file name expansion.  When arglist is NULL, perform
 * here document expansion.
 */
pub fn expand_arg(arg: Box<Option<Node>>, arg_list: &mut ArgList, flag: i32) {
    unimplemented!()
}

/*
 * See if a pattern matches in a case statement.
 */
pub fn case_match(pattern: Box<Option<Node>>, val: &str) -> i32 {
    unimplemented!()
}

/*
 * Record the fact that we have to scan this region of the
 * string for IFS characters.
 */
pub fn record_region(start: i32,end: i32,null_only: i32) {
    unimplemented!()
}

pub fn remove_record_region(end_off: i32) {

}

/*
 * Break the argument string into pieces based upon IFS and add the
 * strings to the argument list.  The regions of the string to be
 * searched for IFS characters have been stored by recordregion.
 * If maxargs is non-negative, at most maxargs arguments will be created, by
 * joining together the last arguments.
 */
pub fn ifs_break_up(string: &str,max_args: i32, arg_list: &ArgList) {
    unimplemented!()
}

pub fn ifs_free() {
    unimplemented!()
}

