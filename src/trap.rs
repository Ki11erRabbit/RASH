use std::process;

//todo flesh out
pub fn exit_shell() {

    process::exit(0);
}

pub static mut TRAP_COUNT: i32 = 0;

#[inline]
pub fn have_traps() -> i32 {
    unsafe {
        TRAP_COUNT
    }
}


/*
 * Called to execute a trap.  Perhaps we should avoid entering new trap
 * handlers while we are executing a trap handler.
 */

pub fn do_trap() {


}
