

pub struct Output {
    nextc: String,
    end: String,
    buf: String,
    fd: u32,
    flags: i32,
}





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
