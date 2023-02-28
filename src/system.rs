use nix::sys::signal::Signal;






pub fn string_signal(sig: Signal) -> String {

    return format!("Signal {}", sig);
}
