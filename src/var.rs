
/* flags */
pub const VEXPORT:i32    = 0x01; // variable is exported
pub const VREADONLY:i32  = 0x02; // variable cannot be modified
pub const VSTRFIXED:i32  = 0x04; // variable struct is statically allocated
pub const VTEXTFIXED:i32 = 0x08; // text is statically allocated
pub const VSTACK:i32     = 0x10; // text is allocated on the stack
pub const VUNSET:i32     = 0x20; // the variable is not set
pub const VNOFUNC:i32    = 0x40; // don't call the callback function
pub const VNOSET:i32     = 0x80; // do not set variable - just readonly test
pub const VNOSAVE:i32    = 0x100;// when text is on the heap before setvareq


pub struct Var {
    flags: i32,         // flags are defined above
    text: &'static str, // name=value
    func: fn(&str),     // the function to be called when the variable gets set/unset 
}

pub struct LocalVar {
    var: Var,           // the variable that was made local
    flags: i32,         // saved flags
    text: &'static str, // saved text
}

pub fn tempfunc(temp: &str) {

}

pub fn setvar(name: &str, val: &str, flags: i32) -> Box<Var> {
    
    Box::new(Var{ flags: VEXPORT, text: "temp",func: tempfunc})
}
