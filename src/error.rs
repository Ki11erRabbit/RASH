use super::trace;
use super::exvwarning;
use nix::unistd::getpid;
use std::io::Write;


#[macro_export]
macro_rules! sh_warnx {
    ($($arg:tt)*) => {
        "temp"
    };
}

#[macro_export]
macro_rules! sh_err {
    ($fmt:tt, $arg:tt) => {
        //exvwarning!($fmt,$arg)
    };
    ($fmt:tt, $arg:tt ,$($args:tt),*) => {
        //exvwarning!($fmt,$arg, $($args),*)
    };
}

#[macro_export]
macro_rules! exverror {
    ($msg:literal, $($arg:tt),*) => {
        #[cfg(debug_assertions)]
        if $msg.len() > 0 {
            trace!("exverror({} \")", $cond);
            tracev!($msg,$($arg),*);
            trace!("\") pid={}\n", getpid());
        }
        else {
            trace!("exverror({}, NULL) pid ={}\n",$cond, getpid());
        }
        //exvwarning!(-1,$msg,$($arg),*);

        flushall();
        
        panic!("Error {}",$cond);
    };
}

#[macro_export]
macro_rules! exvwarning {
    ($msg:literal, $arg:tt) => {
        let fmt: &str;
        let errs: Output;
        let name: &str;
        name = "sh"; //change to see if shell or script
        let fmt: &str = if COMMAND_NAME == None {
            "{}: {}: "
        }
        else {
            "{}: {}: {}: "
        };
        outfmt!(errs,fmt,name,COMMAND_NAME);//make it take output struct errs,fmt,name,errlinno, COMMAND_NAME
        doformat!(errs, msg, $arg);
    };
    ($msg:literal, $arg:tt, $($args:tt),*) => {
        let fmt: &str;
        let errs: Output;
        let name: &str;
        name = "sh"; //change to see if shell or script
        if COMMAND_NAME == None {
            fmt = "{}: {}: ";
        }
        else {
            fmt = "{}: {}: {}: ";
        }
        outformat(errs,fmt,name,COMMAND_NAME);//make it take output struct errs,fmt,name,errlinno, COMMAND_NAME
        doformat!(errs, msg, $arg,$($args),*);
    };
}
