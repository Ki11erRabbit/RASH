use crate::output::Output;
use nix::unistd::Pid;

enum JobStates {
    Running,
    Stopped,
    Done,
}

struct ProcessStatus {
    pid: Pid,       // process id
    status: i32,    // last process status from wait()
    cmd: String,    // text of command being run
}

/*
 * A job structure contains information about a job.  A job is either a
 * single process or a set of processes contained in a pipeline.  In the
 * latter case, pidlist will be non-NULL, and will point to a -1 terminated
 * array of pids.
 */
struct Job {
    ps0: ProcessStatus,         // status of process 
    ps: Option<ProcessStatus>,  // status or processes when more than one
    stop_status: i32,           // status of a stopped job
    num_procs: u16,             // number of processes
    state: u8,
    sigint: bool,               // job was killed by SIGINT
    jobctl: bool,               // job running under job control
    waited: bool,               // true if this entry has been waited for
    used: bool,                 // true if this entry is in used
    changed: bool,              // true if status has changed
}

pub const SHOW_PGID: i32 = 0x01;        // only show pgid - for jobs -p
pub const SHOW_PID: i32 = 0x04;         // include process pid
pub const SHOW_CHANGED: i32 = 0x08;     // only jobs whose state has changed

pub static mut JOBCTL: bool = false;    // true if doing job control
pub static mut JOB_WARNING: i32 = 0;


/*
 * Print a list of jobs.  If "change" is nonzero, only print jobs whose
 * statuses have changed since the last call to showjobs.
 */
pub fn show_jobs(out: Output, mode: i32) {


}

pub fn stopped_jobs() -> i32 {
    
    0
}
