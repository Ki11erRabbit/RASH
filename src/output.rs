

pub struct Output {
    buf: Option<String>,
    fd: i32,
    flags: i32,
}

pub const OUTPUT: Output = Output {buf: None, fd: 1, flags: 0};
pub const ERROUT: Output = Output {buf: None, fd: 2, flags: 0};




#[macro_export]
macro_rules! outfmt {
    ($fmt:literal, $arg:tt) => {
        doformat!(out1,$fmt,$arg); 
    };
    ($format:literal, $arg:tt,$($args:tt),*) => {
        doformat!(out1,$fmt,$arg,$($args),*); 
    };
    ($file:tt, $format:literal, $arg:tt) => {
        doformat!($file,$fmt,$arg); 
    };
    ($file:tt, $format:literal, $arg:tt,$($args:tt),*) => {
        doformat!($file,$fmt,$arg,$($args),*); 
    };
}


#[macro_export]
macro_rules! doformat {
    ($dest:tt, $f:tt,$arg:tt) => {

    };
    ($dest:tt, $f:tt,$arg:tt, $($args:tt),*) => {
        
    };
}

//TODO turn into variadic macro
pub fn out1fmt(format: &str, values: &str) {

}

pub fn flushall() {

}

//TODO turn into macro that calls the correct function
#[inline]
pub fn out2c(c: char) {

}
pub fn out2str(string: &str) {

}
