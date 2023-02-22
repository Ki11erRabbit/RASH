use std::fs::{File, OpenOptions};
use std::io::{BufWriter,Write};

pub static mut TRACEFILE: Option<File> = None;

#[macro_export]
macro_rules! trace {
    ($fmt:literal, $($arg:tt),*) =>  {
        let string: String;
        {
        string = format!($fmt, $($arg),*);
    }   
        let tracefile;
        unsafe {
        tracefile = TRACEFILE.take().unwrap();}
        tracefn(&tracefile,&string);
        unsafe {
        TRACEFILE = Some(tracefile);}
    };
}

pub fn tracefn(tracefile: &File, string:&str) {
    let mut file = BufWriter::new(tracefile);
    file.write_all(string.as_bytes()).expect("Unable to write data");
}
