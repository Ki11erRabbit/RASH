use crate::nodes::{Node, FuncNode};
use crate::nodes;
use crate::trap;
use crate::options;
use crate::exec;
use crate::redir;
use nix::unistd::getpid;
use super::{nflag, eflag,trace,iflag,mflag};
/* reasons for skipping commands (see comment on breakcmd routine) */
#[macro_export]
macro_rules! skip_break {
    () => {
        (1 << 0)
    };
}	
#[macro_export]
macro_rules! skip_cont {
    () => {
        (1 << 1)
    };
}	
#[macro_export]
macro_rules! skip_func {
    () => {
        (1 << 2)
    };
}	
#[macro_export]
macro_rules! skip_func_def {
    () => {
        (1 << 3)
    };
}	

/* flags in argument to evaltree */
pub const EV_EXIT: i32 = 0o1;                            // exit after evaluating tree
pub const EV_TESTED: i32 = 0o2;                          // exit status is checked; ignore -e flag

pub static mut EVAL_SKIP: i32 = 0;                       // set if we are skipping commands
pub static mut EXIT_STATUS: i32 = 0;                     // exit status of last command
static mut FUNC_LINE: i32 = 0;                           // starting line number of current function, or 0 if not in a function
pub static mut COMMAND_NAME: String = String::new();     // currently executing command
pub static mut BACK_EXIT_STATUS: i32 = 0;                // exit status of backquoted command
pub static mut SAVE_STATUS: i32 = 0;                     // exit status of last command outside traps


/*
 * The eval commmand.
 */
pub fn eval_cmd(argc: i32, argv: Vec<String>, flags: i32) -> i32 {

    0
}

/*
 * Execute a command or commands contained in a string.
 */
pub fn evalstring(string: Option<String>, flags: i32) -> i32 {
    
    0
}

macro_rules! eval_tree_error {
    ($flags:tt,$check_exit:tt, $status:tt) => {    
        let flag_invert = !$flags;
        if eflag!() as i32 != 0 && (flag_invert & $check_exit) != 0 && $status != 0 {
            panic!("{}", "invalid syntax");
        }
        
        if $flags & EV_EXIT != 0 {
            panic!("invalid syntax");
        }
        unsafe {
            return Err(EXIT_STATUS);
        }
    };
}


/*
 * Evaluate a parse tree.  The value is left in the global variable
 * exitstatus.
 */
pub fn eval_tree(node: Box<Option<Node>>, flags: i32) -> Result<i32,i32> {
    let mut check_exit = 0;
    let eval_func: fn(Box<Option<Node>>, i32) -> Result<i32,i32>;
    let is_or: i32;
    let mut status = 0;

    if nflag!() as i32 != 0 {
        trap::do_trap();
        eval_tree_error!(flags,check_exit,status);
    }

    if node.is_none() {
        trap::do_trap();
        trace!("evaltree(None) called\n");
        eval_tree_error!(flags,check_exit,status); 
    }
    
    trap::do_trap();

    trace!("pid {}, eval_tree({}: {}, {}) called\n",getpid(), node.unwrap(), node.unwrap().r#type, flags);
    
    match node.unwrap().r#type {
        _ => eprintln!("Node type = {}", node.unwrap()),
        nodes::NNOT => {
            status = eval_tree(node.unwrap().nnot.com, flags)?;
            if unsafe {!EVAL_SKIP} != 0 {
                status = !status;
            }
        }
        nodes::NREDIR => {
            let lineno = node.unwrap().nredir.line_num;
            let errlinno = lineno;
            if unsafe { FUNC_LINE != 0 } {
                lineno -= unsafe {FUNC_LINE} -1;
                exp_redir(node.unwrap().nredir.redirect);
                redir::push_redir(node.unwrap().nredir.redirect);
                status = redirect_safe(node.unwrap().nredir.redirect, REDIR_PUSH)?;
                if status != 0 {
                    check_exit = EV_TESTED;
                }
                else {
                    status = eval_tree(node.unwrap().nredir.node, flags & EV_TESTED)?;
                }
                if node.unwrap().nredir != 0 {
                    pop_redir(0);
                }
            }
        },
        nodes::NCMD => {
            eval_func = eval_command;
            check_exit = EV_TESTED;
            status = eval_func(node,flags)?;
        },
        nodes::NFOR => {
            eval_func = eval_for;
            status = eval_func(node,flags)?;
        }
        nodes::NWHILE | nodes::NUNTIL => {
            eval_func = eval_loop;
            status = eval_func(node,flags)?;
        }
        nodes::NSUBSHELL | nodes::NBACKGND => {
            eval_func = eval_sub_shell;
            check_exit = EV_TESTED;
            status = eval_func(node,flags)?;
        },
        nodes::NPIPE => {
            eval_func = eval_pipe;
            check_exit = EV_TESTED;
            status = eval_func(node,flags)?;
        },
        nodes::NCASE => {
            eval_func = eval_case;
            status = eval_func(node,flags)?;
        },
        nodes::NAND | nodes::NOR | nodes::NSEMI => {
            'run_once: loop {
                is_or = node.unwrap().r#type - NAND;
                status = eval_tree(node.unwrap().nbinary.ch1, flags |((is_or >> 1)) & EV_TESTED)?;
                
                let not_status = !status;
                if not_status == is_or || EVAL_SKIP {
                    break 'run_once;
                }
                
                node = node.unwrap().nbinary.ch2;

                eval_func = eval_tree;

                status = eval_func(node, flags)?;

                break;
            } 
        },
        nodes::NIF => {
            'run_once: loop {
                status = eval_tree(node.unwrap().nif.test, EV_TESTED)?;
                if unsafe {EVAL_SKIP} != 0 {
                    break 'run_once;
                }
                if status == 0 {
                    node = node.unwrap().nif.if_part;
                    eval_func = eval_tree;
                    status = eval_func(node,flags)?;
                    break 'run_once;
                }
                else if node.unwrap().nif.else_part.is_some() {
                    node = node.unwrap().nif.else_part;
                    eval_func = eval_tree;
                    status = eval_func(node,flags)?;
                    break 'run_once;
                }
                
                status = 0;
                break;
            }
        },
        nodes::NDEFUN => {
            defun(node);
        },
    }
    unsafe {
        EXIT_STATUS = status;
    }

    trap::do_trap();

    let flag_invert = !flags;
    if eflag!() as i32 != 0 && (flag_invert & check_exit) != 0 && status != 0 {
        panic!("{}", "invalid syntax");
    }
    
    if flags & EV_EXIT != 0 {
        panic!("invalid syntax");
    }

    unsafe {
        Ok(EXIT_STATUS)
    }
}

fn skip_loop() -> bool {
    unimplemented!()
}

fn eval_loop(node: Box<Option<Node>>, flags: i32) -> Result<i32,i32> {
    unimplemented!()
}

fn eval_for(node: Box<Option<Node>>, flags: i32) -> Result<i32,i32> {
    unimplemented!()
}

fn eval_case(node: Box<Option<Node>>, flags: i32) -> Result<i32,i32>  {
    unimplemented!()
}

fn eval_subshell(node: Box<Option<Node>>, flags: i32) -> Result<i32,i32> {
    unimplemented!()
}

fn exp_redir(node: Box<Option<Node>>) {
    unimplemented!()
}

fn eval_pipe(node: Box<Option<Node>>, flags: i32) -> Result<i32,i32> {
    unimplemented!()
} 

fn eval_back_cmd(node: Box<Option<Node>>, result: BackCmd) {
    unimplemented!()
}

fn eval_command(node: Box<Option<Node>>, flags: i32) -> Result<i32,i32> {
    unimplemented!()
}

fn eval_builtin(cmd: BuiltInCmd, argc:i32, argv: Vec<String>, flags: i32) -> Result<i32,i32> {
    unimplemented!()
}

fn eval_func(func: FuncNode, argc:i32, argv: Vec<String>, flags:i32) -> Result<i32,i32> {
    unimplemented!()
}

fn break_cmd(argc:i32, argv: Vec<String>) {

}

fn return_cmd(argc:i32, argv: Vec<String>) -> Result<i32,i32> {
    unimplemented!()
}

fn false_cmd(argc:i32,argv: Vec<String>) -> bool {
    false
}

fn true_cmd(argc:i32,argv: Vec<String>) -> bool {
    true
}

fn exec_cmd(argc:i32, argv: Vec<String>) -> Result<i32,i32> {
    let mut argv = argv;
    if argc > 1 {
        iflag!(0 as char);
        mflag!(0 as char);
        options::opts_changed();
        argv.remove(0);
        exec::shell_exec(argv,pathval!(), 0);
    }

    return Ok(0)
}
