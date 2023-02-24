

static mut CHANGED: i32 = 0;

pub fn changemail(val: &str) {
    unsafe {
        CHANGED += 1;
    }
}


pub fn check_mail() {


}
