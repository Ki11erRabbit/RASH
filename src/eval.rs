
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

pub static EV_EXIT: i32 = 0o1;

pub static mut EVAL_SKIP: i32 = 0;


pub fn evalstring(string: Option<String>, flags: i32) -> i32 {

    0
}

pub fn eval_tree(n: i32 /* change to node */, flags: i32) -> i32 {

    0
}
