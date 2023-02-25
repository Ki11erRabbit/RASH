



pub fn fork_reset() {
    pop_all_files();
    if PARSE_FILE.fd > 0 {
        close(PARSE_FILE.fd);
        PARSE_FILE.fd = 0;
    }

    //block interrupts
    //todo finish fleshing out
    unimplemented!()

    //unblock interrupts
}
