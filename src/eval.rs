use crate::nodes::{Node, FuncNode};
use crate::trap;
use crate::init;
use crate::options;
use crate::exec;
use crate::output;
use crate::output::Output;
use crate::redir;
use crate::parser;
use crate::nodes;
use crate::jobs::Job;
use crate::jobs;
use crate::var;
use crate::input;
use crate::builtins;
use crate::builtins::BuiltInCmd;
use super::{Node_EOF,defpath,EXECCMD,COMMANDCMD,pathval};
use crate::expand;
use crate::expand::ArgList;
use crate::exec::{CmdEntry};
use std::cell::RefCell;
use std::rc::Rc;
use nix::unistd::{getpid,pipe,close,dup2,Pid};
use likely_stable::{unlikely,likely};
use super::{nflag, eflag,trace,iflag,mflag,xflag,ps4val,EVALCMD};
/* reasons for skipping commands (see comment on breakcmd routine) */
pub const SKIP_BREAK: i32 = 1 << 0;
pub const SKIP_CONT:i32 = 1 << 1;
pub const SKIP_FUNC:i32 = 1 << 2;
pub const SKIP_FUNC_DEF:i32 = 1 << 3;


pub struct BackCmd {        // result of eval_back_cmd
    pub fd: i32,            // file descriptor to read from
    pub buf: String,        // buffer
    pub job: Option<Rc<RefCell<Job>>>,  // job structure from command
}

const BUILTIN: builtins::BuiltInCmd = builtins::BuiltInCmd {name: "", builtin: None, flags: builtins::BUILTIN_REGULAR as u32 };

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
pub fn eval_cmd(argc: usize, argv: Vec<String>, flags: i32) -> Result<i32,i32> {
    
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
    let node: Option<Box<Node>>;
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

fn eval_tree_no_return(node: Option<Box<Node>>, flags: i32) {
    eval_tree(node, flags);
    std::process::abort();
}

/*
 * Evaluate a parse tree.  The value is left in the global variable
 * exitstatus.
 */
pub fn eval_tree(node: Option<Box<Node>>, flags: i32) -> Result<i32,i32> {
    let mut check_exit = 0;
    let eval_func: fn(Option<Box<Node>>, i32) -> Result<i32,i32>;
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

fn eval_loop(node: Option<Box<Node>>, flags: i32) -> Result<i32,i32> {
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

fn eval_for(node: Option<Box<Node>>, flags: i32) -> Result<i32,i32> {
    let arg_list: ArgList;
    let arg_ptr: Option<Box<Node>>; 
    let string_list: Vec<String>;
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

fn eval_case(node: Option<Box<Node>>, flags: i32) -> Result<i32,i32>  {

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
fn eval_subshell(node: Option<Box<Node>>, flags: i32) -> Result<i32,i32> {
    
    let background = node.unwrap().r#type == nodes::NBACKGND;
    let mut status = 0;
    unsafe {
        var::LINE_NUM = node.unwrap().ncase.line_num;
        if FUNC_LINE != 0 {
            var::LINE_NUM -= FUNC_LINE - 1;
        }
    }

    exp_redir(node.unwrap().nredir.redirect);
    //block interrupts
    if !background && flags & EV_EXIT != 0 && trap::have_traps() == 0 {
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

/*
 * Compute the names of the files in a redirection list.
 */
fn exp_redir(node: Option<Box<Node>>) {
    let mut redir;
    
    redir = node;
    while redir.is_some() {
        
        let file_name;
        match redir.unwrap().r#type {
            nodes::NFROMTO | nodes::NFROM | nodes::NTO | nodes::NCLOBBER | nodes:: NAPPEND => {
                expand::expand_arg(redir.unwrap().nfile.file_name, &mut file_name, expand::EXP_TILDE | expand::EXP_REDIR);
                redir.unwrap().nfile.expand_file_name = file_name.list[0];
            },
            nodes::NFROMFD | nodes::NTOFD => {
                if redir.unwrap().ndup.vname.is_some() {
                    expand::expand_arg(redir.unwrap().ndup.vname, &mut file_name, expand::EXP_TILDE | expand::EXP_REDIR);
                    parser::fix_redir(redir,&file_name.list[0],1);
                }
            }
        }

        redir = redir.unwrap().nfile.next;
    }
}

/*
 * Evaluate a pipeline.  All the processes in the pipeline are children
 * of the process creating the pipeline.  (This differs from some versions
 * of the shell, which make the last process in a pipeline the parent
 * of all the rest.)
 */
fn eval_pipe(node: Option<Box<Node>>, flags: i32) -> Result<i32,i32> {
    
    let mut pipe_len = 0;
    let mut pip = (-1,-1);
    let mut status = 0;

    trace!("evalpipe({}) called\n",node.unwrap());
    let mut list = node.unwrap().npipe.cmd_list;
    while list.is_some() { 
        pipe_len += 1;
        list = list.unwrap().next;
    }
    flags |= EV_EXIT;
    //block interrupts
    let job = jobs::make_job(node, pipe_len);
    let mut prev_fd = -1;

    list = node.unwrap().npipe.cmd_list;
    while list.is_some() {
        if list.unwrap().next.is_some() {
            let temp = pipe();
            if temp.is_err() {
                close(prev_fd); 
                panic!("Pipe call failed");
            }
        }
        if jobs::fork_shell(job, list.unwrap().node, node.unwrap().npipe.background) == Pid::from_raw(0) {
            //unblock interrupts
            if pip.1 >= 0 {
                close(pip.0);
            }
            if prev_fd > 0 {
                dup2(prev_fd,0);
                close(prev_fd);
            }
            if pip.1 > 1 {
                dup2(pip.1,1);
                close(pip.1);
            }
            eval_tree_no_return(list.unwrap().node, flags);
            //never returns
        }
        if prev_fd >= 0 {
            close(prev_fd);
        }
        prev_fd = pip.0;
        close(pip.1);

        list = list.unwrap().next;
    }
    if node.unwrap().npipe.background == 0 {
        status = jobs::wait_for_job(job);
        trace!("evalpipe: job done exit status {}\n",status);
    }
    //unblock interrupts
    return Ok(status)
} 

/*
 * Execute a command inside back quotes.  If it's a builtin command, we
 * want to save its output in a block obtained from malloc.  Otherwise
 * we fork off a subprocess and get the output of the command via a pipe.
 * Should be called with interrupts off.
 */
fn eval_back_cmd(node: Option<Box<Node>>, result: &mut BackCmd) -> Result<(),nix::errno::Errno>{
    let mut pip = (-1,-1);
    let mut job;

    *result = BackCmd {fd: -1, buf: String::new(), job: None};
    if node.is_none() {
        trace!("eval_back_cmd done: fd={} buf={} nleft= {} jp=",result.fd,result.buf,result.buf.len());
        return Err(nix::errno::Errno::UnknownErrno);
    }

    pip = pipe()?;

    job = jobs::make_job(node,1);
    if jobs::fork_shell(job,node, jobs::FORK_NOJOB) == Pid::from_raw(0) {
        //forcibly block interrupts
        close(pip.0);
        if pip.1 != 1 {
            dup2(pip.1,1);
            close(pip.1);
        }
        expand::ifs_free();
        eval_tree_no_return(node, EV_EXIT);
        // NOT REACHED
    }
    close(pip.1);
    result.fd = pip.0;
    result.job = Some(job);

    trace!("eval_back_cmd done: fd={} buf={} nleft= {} jp=",result.fd,result.buf,result.buf.len());
    
    return Ok(())
}

fn fill_arglist(arglist: ArgList, args: Vec<Box<Node>>) -> Vec<String> {
    let ret_args;
    for arg in args {
        expand::expand_arg(Some(arg), &mut ret_args, expand::EXP_FULL | expand::EXP_TILDE);
        
    }

    ret_args.list
}

fn parse_command_args(arg_list: ArgList, args: Vec<Box<Node>>, path: &mut Vec<String>) -> i32 {

    let strings = arg_list.list;
    let mut counter = 0;
    let mut sub_counter = 0;
    loop {
        strings = if unlikely(counter < strings.len()) {strings} else {{counter = 0; fill_arglist(arg_list, args)}};

        if strings.len() == 0 {
            return 0;
        }
        let text = strings[counter];

        if text.chars().collect::<Vec<char>>()[sub_counter] != '-' {
            break;
        }
        sub_counter += 1;
        
        if !text.len() > 2 {
            break;
        }
        if text.chars().collect::<Vec<char>>()[sub_counter] == '-' && text.len() == 2 {// this part is likely to have some problems
            if likely(strings.len() == counter - 1) && fill_arglist(arg_list, args).len() == 0 {
                return 0;
            }
            counter += 1;
            break;
        }
        sub_counter += 1;
        loop {
            
            match text.chars().collect::<Vec<char>>()[sub_counter] {
                'p' => path.push(defpath!()),
                _ => return 0,// run 'typecmd' for other options
            }

            
            
            sub_counter += 1;
            if sub_counter == text.len() {
                break;
            }
        }
    }
    arg_list.list = strings;
    return exec::DO_NOFUNC;
}

/*
 * Execute a simple command.
 */
fn eval_command(cmd: Option<Box<Node>>, flags: i32) -> Result<i32,i32> {
    
    unsafe {
        var::LINE_NUM = cmd.unwrap().ncase.line_num;
        if FUNC_LINE != 0 {
            var::LINE_NUM -= FUNC_LINE - 1;
        }
    }

    // First expand the arguments
    trace!("evalcommand({}, {}) called\n",cmd.unwrap(),flags);
    let file_stop; 
    unsafe {
        file_stop = input::PARSEFILE_STACK.lock().unwrap().len() -1;
        BACK_EXIT_STATUS = 0;
    }
    
    let mut cmd_entry = CmdEntry { cmd_type: exec::CMD_BUILTIN, param: exec::Param{cmd: *std::mem::ManuallyDrop::new(builtins::BUILTIN_CMD[0]) } };

    let mut cmd_flag;
    let exec_cmd = false;
    let special_builtin = -1;
    let mut vflags = 0;
    let mut vlocal = 0;
    let mut path;
    let arglist;
    let mut status = 0;

    let mut argc = 0;

    let args = cmd.unwrap().ncmd.args_vec;
    let osp = fill_arglist(arglist, args);
    if osp.len() > 0 {
        let mut pseudo_var_flag = 0;

        loop {
            exec::find_command(arglist.list[0], &mut cmd_entry, cmd_flag | DO_REGBLTIN, pathval!());

            vlocal += 1;

            if cmd_entry.cmd_type != exec::CMD_BUILTIN {
                break;
            }

            pseudo_var_flag = cmd_entry.param.cmd.flags & builtins::BUILTIN_ASSIGN as u32;
            if likely(special_builtin < 0) {
                special_builtin = cmd_entry.param.cmd.flags as i32 & builtins::BUILTIN_SPECIAL;

                vlocal = special_builtin as i32 ^ builtins::BUILTIN_SPECIAL;
            }
            exec_cmd = cmd_entry.param.cmd == *std::mem::ManuallyDrop::new(EXECCMD!());

            if likely(cmd_entry.param.cmd != *std::mem::ManuallyDrop::new(COMMANDCMD!())) {
                break
            }

            cmd_flag = parse_command_args(arglist, args, &mut path);
            if cmd_flag != 0 {
                break;
            }
        }

        for arg in args.iter() {
            let temp = if pseudo_var_flag != 0 && is_assignment(arg.narg.text) {expand::EXP_VARTILDE} else {expand::EXP_FULL | expand::EXP_TILDE};
            expand::expand_arg(Some(*arg),&mut arglist,temp);

        }

        for args in arglist.list.iter() {
            argc += 1;
        }

        if exec_cmd && argc > 1 {
            vflags = var::VEXPORT;
        }
    }
    let path = {
        if path.len() > 0 {
            path[0]
        }
        else {
            "".to_string()
        }
    };
    let local_var_stop = var::push_local_vars(vlocal != 0);
  

    let argv;
    for arg in arglist.list.iter() {
        trace!("eval_command arg: {}\n",arg);
        argv.push(arg);
    }

    let lastarg;
    if iflag!() as i32 != 0 && unsafe {FUNC_LINE == 0} && argc > 0 {
        lastarg = argv.last();
    } else {
        lastarg = None;
    }


    unsafe {
        output::PREV_ERR_OUT.fd = 2;
    }
    exp_redir(cmd.unwrap().ncmd.redirect);
    let redir_stop = redir::push_redir(cmd.unwrap().ncmd.redirect);
    status = redir::redirect_safe(cmd.unwrap().ncmd.redirect, redir::REDIR_PUSH|redir::REDIR_SAVEFD2)?;

    if unlikely(status != 0) {
        unsafe {
            EXIT_STATUS = status;
        }

        if special_builtin > 0 {
            eprintln!("Redirection Error");
        }

        if cmd.unwrap().ncmd.redirect.is_some() {
            redir::pop_redir(exec_cmd as i32);
        }
        var::unwind_local_vars(local_var_stop);
        input::unwind_files(file_stop);
        var::unwind_local_vars(local_var_stop);
        if lastarg.is_some() {
            
            var::setvar("_", &lastarg, 0);
        }

        return Err(status);
    }

    let mut args = cmd.unwrap().ncmd.assign;
    
    let varlist;
    while args.is_some() {
        expand::expand_arg(args, &mut varlist, expand::EXP_VARTILDE);
        
        let string = varlist.list.last().unwrap();
        if vlocal != 0 {
            var::make_local(string, var::VEXPORT);
        }
        else {
            var::setvareq(string, vflags);
        }   

        args = args.unwrap().narg.next;
    }

	/* Print the command if xflag is set. */
    if xflag!() as i32 != 0 && unsafe {init::INPS4} != 0 {
        let out: &mut Output = unsafe {&mut output::PREV_ERR_OUT};
        let sep = 0;
        
        unsafe {
            init::INPS4 = 1;
        }
        
        output::outstr(&parser::expand_str(ps4val!()),out);

        unsafe {
            init::INPS4 = 0;
        }

        sep = eprintlist(out,osp,sep);
        output::outcslow('\n');
    }

    let job;

	/* Execute the command. */
    match cmd_entry.cmd_type {
        exec::CMD_UNKNOWN => {
            status = 127;
            unsafe {
                EXIT_STATUS = status;
            }

            if special_builtin > 0 {
                eprintln!("Redirection Error");
            }

            if cmd.unwrap().ncmd.redirect.is_some() {
                redir::pop_redir(exec_cmd as i32);
            }
            var::unwind_local_vars(redir_stop);
            input::unwind_files(file_stop);
            var::unwind_local_vars(local_var_stop);
            if lastarg.is_some() {
                
                var::setvar("_", &lastarg, 0);
            }

            return Err(status);
        },
        _ => {
		    /* Fork off a child process if necessary. */
            'loop_once: loop {

                if !(flags & EV_EXIT != 0) || trap::have_traps() != 0 {
                    //block interrupts

                    job = jobs::vforkexec(cmd, argv, &path, cmd_entry.param.index);
                    break 'loop_once;
                }
                exec::shell_exec(argv, &path, cmd_entry.param.index);
                // NOT REACHED
            } 
        },
        exec::CMD_BUILTIN => {
            eval_builtin(cmd_entry.param.cmd, argc, argv, flags)?;
        },
        exec::CMD_FUNCTION => {
            eval_func(cmd_entry.param.func, argc.try_into().unwrap(), argv, flags)?;
        }
    }

    status = jobs::wait_for_job(job);

    //force interrupts on

    if cmd.unwrap().ncmd.redirect.is_some() {
        redir::pop_redir(exec_cmd as i32);
    }
    var::unwind_local_vars(redir_stop);
    input::unwind_files(file_stop);
    var::unwind_local_vars(local_var_stop);
    if lastarg.is_some() {
        
        var::setvar("_", &lastarg, 0);
    }

    return Ok(status);
}

fn eval_builtin(cmd: BuiltInCmd, argc:usize, argv: Vec<String>, flags: i32) -> Result<i32,i32> {
    let mut status = 0;
    let save_cmd_name = unsafe {COMMAND_NAME};

    if cmd == EVALCMD!() {
        status = eval_cmd(argc, argv, flags)?;
    }
    else {
        status = cmd.builtin.unwrap()(argc, argv);
    }
    output::flushall();
    if outerr(out1) {
        eprintln!("{}: I/O error", unsafe { COMMAND_NAME});
    }

    status |= outerr(out1);

    unsafe {
        EXIT_STATUS = status;
        output::freestdout();
        COMMAND_NAME = save_cmd_name;
    }
    Ok(status)
}

fn eval_func(func: std::mem::ManuallyDrop<FuncNode>, argc: usize, argv: Vec<String>, flags:i32) -> Result<i32,i32> {
    let mut status = 0;
    let save_cmd_name = unsafe {COMMAND_NAME};
    let save_func_line = unsafe {FUNC_LINE};
    let save_loop_nest = unsafe {LOOP_NEST};
    let save_param = unsafe {options::SHELLPARAM};

    //block interrupts
    func.count += 1;
    unsafe {
        FUNC_LINE = func.node.unwrap().ndefun.line_num;
        LOOP_NEST = 0;
    }
    //unblock interrupts
    unsafe {
        options::SHELLPARAM.num_param = argc - 1;
        options::SHELLPARAM.param_list = argv.into_iter().skip(1).collect(); 
        options::SHELLPARAM.opt_index = 1;
        options::SHELLPARAM.opt_off = -1;
    }
    eval_tree(func.node.unwrap().ndefun.body, flags & EV_TESTED);

    //block interrupts
    unsafe {
        LOOP_NEST = save_loop_nest;
        FUNC_LINE = save_func_line;
        nodes::free_parse_tree(func);
        options::free_param(&mut options::SHELLPARAM);
        options::SHELLPARAM = save_param;
    }
    //unblock interrupts

    unsafe {
        EVAL_SKIP &= !(SKIP_FUNC | SKIP_FUNC_DEF);
    }
    
    return Ok(status);
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

fn eprintlist(out: &mut Output, strings: Vec<String>, sep: i32) -> i32 {

    unimplemented!()
}
