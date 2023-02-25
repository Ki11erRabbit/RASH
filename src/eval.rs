use crate::nodes::{Node, FuncNode};
use crate::trap;
use crate::options;
use crate::exec;
use crate::redir;
use crate::parser;
use crate::nodes;
use crate::jobs::Job;
use crate::var;
use crate::jobs;
use super::Node_EOF;
use crate::expand;
use crate::expand::ArgList;
use nix::unistd::getpid;
use likely_stable::likely;
use super::{nflag, eflag,trace,iflag,mflag};
/* reasons for skipping commands (see comment on breakcmd routine) */
pub const SKIP_BREAK: i32 = 1 << 0;
pub const SKIP_CONT:i32 = 1 << 1;
pub const SKIP_FUNC:i32 = 1 << 2;
pub const SKIP_FUNC_DEF:i32 = 1 << 3;


pub struct BackCmd {        // result of eval_back_cmd
    pub fd: i32,            // file descriptor to read from
    pub buf: String,        // buffer
    pub job: Box<Option<Job>>,  // job structure from command
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

static mut SKIP_COUNT: i32 = 0;                          // number of levels to skip
static mut LOOP_NEST: i32 = 0;                           // current loop nesting level

/*
 * The eval commmand.
 */
pub fn eval_cmd(argc: i32, argv: Vec<String>, flags: i32) -> Result<i32,i32> {
    
    let string = String::new(); 
    if argv.len() > 1 {
        for i in 1..argv.len() {
            string = string + &argv[i] + " ";
        }
        string.pop();

        return eval_string(Some(string), flags & EV_TESTED);

    }
    Ok(0)
}

/*
 * Execute a command or commands contained in a string.
 */
pub fn eval_string(string: Option<String>, flags: i32) -> Result<i32,i32> {
    let node: Box<Option<Node>>;
    let mut status: i32 = 0;

    while {node = parser::parse_cmd(0); node != Node_EOF!()} {
        let i;

        let not_result = !(if parser::eof() {0} else {EV_EXIT});
        i = eval_tree(node, flags & not_result)?;

        if node.is_some() {
            status = i;
        }

        if unsafe { EVAL_SKIP != 0} {
            break;
        }
    }

    Ok(status)
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

fn eval_tree_no_return(node: Box<Option<Node>>, flags: i32) {
    eval_tree(node, flags);
    std::process::abort();
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
            unsafe {
                var::LINE_NUM = node.unwrap().nredir.line_num;
                let errlinno = var::LINE_NUM;
            }
            if unsafe { FUNC_LINE != 0 } {
                unsafe{
                    var::LINE_NUM -= FUNC_LINE -1;
                }
                exp_redir(node.unwrap().nredir.redirect);
                redir::push_redir(node.unwrap().nredir.redirect);
                status = redir::redirect_safe(node.unwrap().nredir.redirect, redir::REDIR_PUSH)?;
                if status != 0 {
                    check_exit = EV_TESTED;
                }
                else {
                    status = eval_tree(node.unwrap().nredir.node, flags & EV_TESTED)?;
                }
                if node.unwrap().nredir.redirect.is_some() {
                    redir::pop_redir(0);
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
            eval_func = eval_subshell;
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
                is_or = node.unwrap().r#type - nodes::NAND;
                status = eval_tree(node.unwrap().nbinary.ch1, flags |((is_or >> 1)) & EV_TESTED)?;
                
                let not_status = !status;
                if not_status == is_or || unsafe { EVAL_SKIP != 0 } {
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
            exec::def_func(node);
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

fn skip_loop() -> i32 {

    let skip = unsafe {EVAL_SKIP};

    match skip {
        0 => (),
        SKIP_BREAK | SKIP_CONT => {
            'loop_once: loop {
                unsafe {
                
                    if likely( { SKIP_COUNT -= 1; SKIP_COUNT <= 0}) {
                           EVAL_SKIP = 0;
                           break 'loop_once;
                    }

                    skip = SKIP_BREAK;
                    break;
                }
            }
        }
    }
    skip
}

fn eval_loop(node: Box<Option<Node>>, flags: i32) -> Result<i32,i32> {
    let mut skip;
    let mut status = 0;
    
    unsafe {
        LOOP_NEST += 1;
    }

    flags &= EV_TESTED;

    loop {

        let i = eval_tree(node.unwrap().nbinary.ch1, EV_TESTED)?;
        skip = skip_loop();
        if skip == SKIP_FUNC {
            status = i;
        }
        if skip != 0 {
            continue;
        }
        if node.unwrap().r#type != nodes::NWHILE {
            i = !i;
        }
        if i != 0 {
            break;
        }

        status = match eval_tree(node.unwrap().nbinary.ch2, flags) {
            Ok(stat) => stat,
            Err(e) => {
                unsafe {
                    LOOP_NEST -= 1;
                }
                
                return Err(e);
            }
        };
        skip = skip_loop();
        
        let not_skip_cont = ! SKIP_CONT;
        let nand_skip = !(skip & not_skip_cont);

        if nand_skip != 0 {
            break;
        }
    }
    unsafe {
        LOOP_NEST -= 1;
    }
    
    return Ok(status);
}

fn eval_for(node: Box<Option<Node>>, flags: i32) -> Result<i32,i32> {
    let arg_list: ArgList;
    let arg_ptr: Box<Option<Node>>; 
    let string_list: StringList;
    let mut status = 0;

    if unsafe {FUNC_LINE != 0} {
        unsafe {
            var::LINE_NUM = node.unwrap().nredir.line_num;

            if FUNC_LINE != 0 {
                var::LINE_NUM -= FUNC_LINE - 1;
            }
        }
    }
        
    arg_ptr = node.unwrap().nfor.args;
    while arg_ptr.is_some() {
        expand::expand_arg(arg_ptr,&mut arg_list,expand::EXP_FULL | expand::EXP_TILDE);

        arg_ptr = arg_ptr.unwrap().narg.next;
    }

    unsafe{
        LOOP_NEST += 1;
    }
    flags &= EV_TESTED;

    for arg in arg_list.list.iter_mut() {
        var::setvar(&node.unwrap().nfor.var, &Some(arg.to_string()), flags);
        status = match eval_tree(node.unwrap().nfor.body, flags) {
            Ok(stat) => status,
            Err(e) => {
                unsafe{
                    LOOP_NEST -= 1;
                }
                
                return Err(e);
            }
        };

        if skip_loop() & !SKIP_CONT != 0 {
            break;
        }
    }

    unsafe{
        LOOP_NEST -= 1;
    }
    return Ok(status)    
}

fn eval_case(node: Box<Option<Node>>, flags: i32) -> Result<i32,i32>  {

    let mut arg_list;
    let mut status = 0;

    unsafe {
        var::LINE_NUM = node.unwrap().ncase.line_num;
        if FUNC_LINE != 0 {
            var::LINE_NUM -= FUNC_LINE - 1;
        }
    }
    expand::expand_arg(node.unwrap().ncase.expr, &mut arg_list, expand::EXP_TILDE);
    let mut case = node.unwrap().ncase.cases;
    'out: while case.is_some() && unsafe {EVAL_SKIP == 0} {
        let mut pattern = case.unwrap().nclist.pattern;
        while pattern.is_some() {
            if expand::case_match(pattern, &arg_list.list[0]) != 0 {
				/* Ensure body is non-empty as otherwise
				 * EV_EXIT may prevent us from setting the
				 * exit status.
				 */
                if unsafe {EVAL_SKIP == 0} && case.unwrap().nclist.body.is_some() {
                    status = eval_tree(case.unwrap().nclist.body,flags)?;
                }
                break 'out;
            }
            pattern = pattern.unwrap().narg.next;
        }
        case = case.unwrap().nclist.next;
    }
    Ok(status)
}

/*
 * Kick off a subshell to evaluate a tree.
 */
fn eval_subshell(node: Box<Option<Node>>, flags: i32) -> Result<i32,i32> {
    
    let background = node.unwrap().r#type == nodes::NBACKGND;
    let mut status = 0;
    unsafe {
        var::LINE_NUM = node.unwrap().ncase.line_num;
        if FUNC_LINE != 0 {
            var::LINE_NUM -= FUNC_LINE - 1;
        }
    }

    redir::exp_redir(node.unwrap().nredir.redirect);
    //block interrupts
    if !background && flags & EV_EXIT != 0 && !trap::have_traps() {
        crate::init::fork_reset();
        //unblock interrupts
        redir::redirect(node.unwrap().nredir.redirect, 0);
        eval_tree_no_return(node.unwrap().nredir.node, flags);
    }
    
    let job = jobs::make_job(node,1);
    
    if {let background = if background {1} else {0}; jobs::fork_shell(job,node,background).as_raw() == 0 } {
        flags |= EV_EXIT;
        if background {
            flags &= !EV_TESTED;
        }

        //unblock interrupts
        redir::redirect(node.unwrap().nredir.redirect, 0);
        eval_tree_no_return(node.unwrap().nredir.node, flags);
    }

    if !background {
        status = jobs::wait_for_job(job);
    }
    //unblock interrupts

    return Ok(status)
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
