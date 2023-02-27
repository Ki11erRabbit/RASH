use crate::output::Output;
use crate::nodes::Node;
use crate::options;
use crate::redir;
use super::mflag;
use nix::unistd::{Pid,close};
use nix::errno::errno;
use nix::sys::signal::{kill,SIGTTIN,SIGTTOU,SIGTSTP};
use std::cell::RefCell;
use std::rc::Rc;

enum JobStates {
    Running,
    Stopped,
    Done,
}

/*
 * A job structure contains information about a job.  A job is either a
 * single process or a set of processes contained in a pipeline.  In the
 * latter case, pidlist will be non-NULL, and will point to a -1 terminated
 * array of pids.
 */
#[derive(PartialEq, Eq, Debug)]
pub struct ProcessStatus {
    pid: Pid,       // process id
    status: i32,    // last process status from wait()
    pub cmd: String,    // text of command being run
}

pub struct Job {
    pub ps0: ProcessStatus,         // status of process 
    ps: Option<ProcessStatus>,  // status or processes when more than one
    stop_status: i32,           // status of a stopped job
    num_procs: u16,             // number of processes
    pub state: JobStates,
    sigint: bool,               // job was killed by SIGINT
    jobctl: bool,               // job running under job control
    waited: bool,               // true if this entry has been waited for
    used: bool,                 // true if this entry is in used
    changed: bool,              // true if status has changed
}

/* mode flags for showjob(s) */
pub const SHOW_PGID: i32 = 0x01;        // only show pgid - for jobs -p
pub const SHOW_PID: i32 = 0x04;         // include process pid
pub const SHOW_CHANGED: i32 = 0x08;     // only jobs whose state has changed


/* Mode argument to forkshell.  Don't change FORK_FG or FORK_BG. */
pub const FORK_FG: i32 = 0;
pub const FORK_BG: i32 = 0;
pub const FORK_NOJOB: i32 = 2;

pub static mut JOBCTL: bool = false;    // true if doing job control
pub static mut JOB_WARNING: i32 = 0;

pub static mut BACKGROUND_PID: Pid = Pid::from_raw(0); // pid of last background process
pub static mut VFORKED: bool = false;                  // Set if we are in the vforked child

static mut INITIAL_PROCESS_GROUP: Pid = Pid::from_raw(0); // process group of shell at invocation
static mut TTYFD: i32 = -1;                               // file descriptor of controlling terminal

static mut JOBTABLE: Vec<Rc<RefCell<Job>>> = Vec::new();

static mut CUR_JOB: Option<Rc<RefCell<Job>>> = None;


fn set_curjob(job: Rc<RefCell<Job>>, mode: i32) {// TODO: come back after implementing other things
    let temp_job;
    unsafe {
        temp_job = CUR_JOB.take().unwrap();

        let pos = JOBTABLE.iter().position(|x| x.borrow().ps0 == temp_job.borrow().ps0).unwrap();
        JOBTABLE.remove(pos);
    }

    match mode {
        _ => std::process::abort(),
        CUR_DELETE => (), // job being deleted
        CUR_RUNNING => {
            unsafe {
                CUR_JOB = Some(job.clone());
                JOBTABLE.push(job);
            }
        },

    } 
}

/*
 * Turn job control on and off.
 *
 * Note:  This code assumes that the third arg to ioctl is a character
 * pointer, which is true on Berkeley systems but not System V.  Since
 * System V doesn't have job control yet, this isn't a problem now.
 *
 * Called with interrupts off.
 */

fn set_jobctl(on: bool) {
    let mut fd: i32;
    let mut pgroup: i32;

    if on == unsafe { JOBCTL } || ROOTSHELL == 0 {
        return;
    }

    if on {
        let old_fd;
        old_fd = fd = redir::sh_open("/dev/tty",1).unwrap();
        if fd < 0 {
            fd += 3;
            while !nix::unistd::isatty(fd).unwrap() {
                fd -= 1;
                if fd < 0 {
                    eprintln!("can't access tty; job control turned off");
                    mflag!(0 as char);
                    on = false;
                    close(fd);
                    fd = -1
                }
            }
        }

        fd = savefd(fd,old_fd);
        loop { // while we are in the background
            if {pgroup = tc_get_process_group(fd); pgroup } < 0 {
                eprintln!("can't access tty; job control turned off");
                mflag!(0 as char);
                on = false;
                close(fd);
                fd = -1
            }
            if pgroup == get_process_group() {
                break;
            }
            kill_process_group(0, SIGTTIN);
        }
        unsafe {
            INITIAL_PROCESS_GROUP = Pid::from_raw(pgroup);
        }

        set_signal(SIGTSTP);
        set_signal(SIGTTOU);
        set_signal(SIGTTIN);
        pgroup = super::ROOTPID.as_raw();
        setpgid(0, pgroup);
        xtc_set_process_group(fd, pgroup);
    }
    else {
        // turing job control off
        fd = unsafe { TTYFD };
        pgroup = unsafe { INITIAL_PROCESS_GROUP.as_raw() };
        xtc_set_process_group(fd, pgroup);
        setpgid(0, pgroup);
        set_signal(SIGTSTP);
        set_signal(SIGTTOU);
        set_signal(SIGTTIN);

        close(fd);
        fd = -1;
    }
    unsafe {
        TTYFD = fd;
        JOBCTL = on;
    }
}

pub fn killcmd(argc: usize, argv: Vec<String>) -> Result<i32,i32> {
    let sig_num;
    let list;
    let pid: Pid;
    let job;
    let mut pos = 1;
    //TODO: move this somewhere else
    let signal_names = vec!["HUP", "INT", "QUIT", "ILL", "TRAP", "ABRT", "IOT", "BUS", "FPE", "KILL", "USR1", "SEGV", "USR2", "PIPE", "ALRM", "TERM", "STKFLT", "CHLD", "CONT", "STOP", "TSTP", "TTIN", "TTOU", "URG", "XCPU", "XFSZ", "VTALRM", "PROF", "WINCH", "IO", "PWR", "SYS"];

    if argc <= 1 {
        eprintln!("Usage: kill [-s sigspec | -n signum | -sigspec] pid | jobspec ... or kill -l [sigspec]");
        return Err(1);
    }

    if argv[pos].chars().nth(0) == Some('-') {
        sig_num = decode_signal(argv[pos].get(1..).unwrap(), 1);
        if sig_num < 0 {
            let chr;
            while {chr = options::nextopt("ls:"); chr} != None {
                match chr {
                    _ => std::process::abort(),
                    Some('l') => list = 0,
                    Some('s') => {
                        sig_num = decode_signal(options::OPTION_ARG.unwrap(),1);
                        if sig_num < 0 {
                            eprintln!("invalid signal number or name: {}", unsafe {options::OPTION_ARG.unwrap()});
                        }
                        break;
                    },
                }
            }
            argv = unsafe { options::ARGPOINTER.iter().map(|x| x.unwrap()).collect() };
        }
        else {
            pos += 1;
        }
    }

    if list == 0 && sig_num < 0 {
        sig_num = 15; //SIGTERM.as_raw();
    }

    if (sig_num < 0 || argv.get(pos).is_none()) ^ (list != 0)  {//TODO: check to see that this condition is correct
        eprintln!("Usage: kill [-s sigspec | -n signum | -sigspec] pid | jobspec ... or kill -l [sigspec]");
        return Err(1);
    }
    if list != 0 {
        let out = out1;

        if argv.get(pos).is_none() {
            outstr("0\n", &out);
            for i in 1..32 {//NSIG
                outfmt(&mut out, mystring::STRING_NEWLINE_FMT, signal_names[i]);
            }
            return Ok(0);
        }
        sig_num = argv[pos].parse::<i32>().unwrap();
        if sig_num > 128 {
            sig_num -= 128;
        }
        if 0 < sig_num && sig_num < 32 {
            outfmt(&mut out, mystring::STRING_NEWLINE_FMT, signal_names[sig_num]);
        }
        else {
            eprintln!("invalid signal number or exit status: {}", argv[pos]);
            return Err(1);
        }
        return Ok(0);
    }

    let i = 0;
    loop {
        if argv[pos].chars().nth(0) == Some('%') {
            job = get_job(argv[pos],0);
            pid = -job.ps.unwrap().pid;
        }
        else {
            pid = Pid::from_raw(if argv[pos].chars().nth(0) == Some('-') { - argv[pos].get(1..).unwrap().parse::<i32>().unwrap() } else { argv[pos].parse::<i32>().unwrap() });
        }
        let result = kill(pid, sig_num);
        if result.is_err() {
            eprintln!("kill: {}", result.err().unwrap());
            i = 1;
        }
        pos += 1;
        if pos == argv.len() {
            break;
        }
    }

    if i != 0 {
        return Err(i);
    }
    Ok(i)

}


/*
 * Return a new job structure.
 * Called with interrupts off.
 */
pub fn make_job(node: Option<Box<Node>>, num_procs:i32) -> Rc<RefCell<Job>> {
    unimplemented!()
}
/*
 * Print a list of jobs.  If "change" is nonzero, only print jobs whose
 * statuses have changed since the last call to showjobs.
 */
pub fn show_jobs(out: Output, mode: i32) {


}

pub fn stopped_jobs() -> i32 {
    
    0
}

pub fn fork_shell(job: Rc<RefCell<Job>>, node: Option<Box<Node>>, mode: i32) -> Pid {
    unimplemented!()
}

pub fn vforkexec(node: Option<Box<Node>>, argv: Vec<String>, path: &str, idx: i32) -> Rc<RefCell<Job>> {
    unimplemented!()
}
/*
 * Wait for job to finish.
 *
 * Under job control we have the problem that while a child process is
 * running interrupts generated by the user are sent to the child but not
 * to the shell.  This means that an infinite loop started by an inter-
 * active user may be hard to kill.  With job control turned off, an
 * interactive user may place an interactive program inside a loop.  If
 * the interactive program catches interrupts, the user doesn't want
 * these interrupts to also abort the loop.  The approach we take here
 * is to have the shell ignore interrupt signals while waiting for a
 * forground process to terminate, and then send itself an interrupt
 * signal if the child process was terminated by an interrupt signal.
 * Unfortunately, some programs want to do a bit of cleanup and then
 * exit on interrupt; unless these processes terminate themselves by
 * sending a signal to themselves (instead of calling exit) they will
 * confuse this approach.
 *
 * Called with interrupts off.
 */

pub fn wait_for_job(job: Rc<RefCell<Job>>) -> i32 {
    unimplemented!()
}
