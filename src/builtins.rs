use lazy_static::lazy_static;

#[derive(Clone,Copy)]
pub struct BuiltInCmd {
    pub name: &'static str,
    pub builtin: Option<fn(usize,Vec<String>) -> i32>,
    pub flags: u32,
}

impl PartialEq for BuiltInCmd {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

pub const BUILTIN_CMD: Vec<BuiltInCmd> = vec![
    BuiltInCmd {name: ".", builtin: Some(super::dot_cmd), flags: 3},
    BuiltInCmd {name: ":", builtin: Some(truecmd), flags: 3},
    BuiltInCmd {name: "[", builtin: Some(testcmd), flags: 0},
    BuiltInCmd {name: "alias", builtin: Some(aliascmd), flags: 6},
    BuiltInCmd {name: "bg", builtin: Some(bgcmd), flags: 2},
    BuiltInCmd {name: "break", builtin: Some(breakcmd), flags: 3},
    BuiltInCmd {name: "cd", builtin: Some(cdcmd), flags: 2},
    BuiltInCmd {name: "chdir", builtin: Some(cdcmd), flags: 0},
    BuiltInCmd {name: "command", builtin: Some(commandcmd),flags: 2},
    BuiltInCmd {name: "continue", builtin: Some(breakcmd), flags: 3},
    BuiltInCmd {name: "echo", builtin: Some(echocmd), flags: 0},
    BuiltInCmd {name: "eval", builtin: None, flags: 3},
    BuiltInCmd {name: "exec", builtin: Some(execcmd), flags: 3},
    BuiltInCmd {name: "exit", builtin: Some(exitcmd), flags: 3},
    BuiltInCmd {name: "export", builtin: Some(exportcmd), flags: 7},
    BuiltInCmd {name: "false", builtin: Some(falsecmd), flags: 2},
    BuiltInCmd {name: "fg", builtin: Some(fgcmd), flags: 2},
    BuiltInCmd {name: "getopts", builtin: Some(getoptscmd), flags: 2},
    BuiltInCmd {name: "hash", builtin: Some(hashcmd), flags: 2},
    BuiltInCmd {name: "jobs", builtin: Some(jobscmd), flags: 2},
    BuiltInCmd {name: "kill", builtin: Some(killcmd), flags: 2},
    BuiltInCmd {name: "local", builtin: Some(localcmd), flags: 7},
    BuiltInCmd {name: "printf", builtin: Some(printfcmd), flags: 0},
    BuiltInCmd {name: "pwd", builtin: Some(pwdcmd), flags: 2},
    BuiltInCmd {name: "read", builtin: Some(readcmd), flags: 2},
    BuiltInCmd {name: "readonly", builtin: Some(exportcmd), flags: 7},
    BuiltInCmd {name: "return", builtin: Some(returncmd), flags: 3},
    BuiltInCmd {name: "set", builtin: Some(setcmd), flags: 3},
    BuiltInCmd {name: "shift", builtin: Some(shiftcmd), flags: 3},
    BuiltInCmd {name: "test", builtin: Some(testcmd), flags: 0},
    BuiltInCmd {name: "times", builtin: Some(timescmd), flags: 3},
    BuiltInCmd {name: "trap", builtin: Some(trapcmd), flags: 3},
    BuiltInCmd {name: "true", builtin: Some(truecmd), flags: 2},
    BuiltInCmd {name: "type", builtin: Some(typecmd), flags: 2},
    BuiltInCmd {name: "ulimit", builtin: Some(ulimitcmd), flags: 2},
    BuiltInCmd {name: "umask", builtin: Some(umaskcmd), flags: 2},
    BuiltInCmd {name: "unalias", builtin: Some(unaliascmd), flags: 2},
    BuiltInCmd {name: "unset", builtin: Some(unsetcmd), flags: 3},
    BuiltInCmd {name: "wait", builtin: Some(waitcmd), flags: 2},
];
#[macro_export]
macro_rules! ALIASCMD {
    () => {
        crate::builtins::BUILTIN_CMD[3]
    };
}
#[macro_export]
macro_rules! BGCMD {
    () => {
        crate::builtins::BUILTIN_CMD[4]
    };
}
#[macro_export]
macro_rules! BREAKCMD {
    () => {
        crate::builtins::BUILTIN_CMD[5]
    };
}
#[macro_export]
macro_rules! CDCMD {
    () => {
        crate::builtins::BUILTIN_CMD[6]
    };
}
#[macro_export]
macro_rules! COMMANDCMD {
    () => {
        crate::builtins::BUILTIN_CMD[8]
    };
}
#[macro_export]
macro_rules! DOTCMD {
    () => {
        crate::builtins::BUILTIN_CMD[0]
    };
}
#[macro_export]
macro_rules! ECHOCMD {
    () => {
        crate::builtins::BUILTIN_CMD[10]
    };
}
#[macro_export]
macro_rules! EVALCMD {
    () => {
        crate::builtins::BUILTIN_CMD[11]
    };
}
#[macro_export]
macro_rules! EXECCMD {
    () => {
        crate::builtins::BUILTIN_CMD[12]
    };
}
#[macro_export]
macro_rules! EXITCMD {
    () => {
        crate::builtins::BUILTIN_CMD[13]
    };
}
#[macro_export]
macro_rules! EXPORTCMD {
    () => {
        crate::builtins::BUILTIN_CMD[14]
    };
}
#[macro_export]
macro_rules! FALSECMD {
    () => {
        crate::builtins::BUILTIN_CMD[15]
    };
}
#[macro_export]
macro_rules! FGCMD {
    () => {
        crate::builtins::BUILTIN_CMD[16]
    };
}
#[macro_export]
macro_rules! GETOPTSCMD {
    () => {
        crate::builtins::BUILTIN_CMD[17]
    };
}
#[macro_export]
macro_rules! HASHCMD {
    () => {
        crate::builtins::BUILTIN_CMD[18]
    };
}
#[macro_export]
macro_rules! JOBSCMD {
    () => {
        crate::builtins::BUILTIN_CMD[19]
    };
}
#[macro_export]
macro_rules! KILLCMD {
    () => {
        crate::builtins::BUILTIN_CMD[20]
    };
}
#[macro_export]
macro_rules! LOCALCMD {
    () => {
        crate::builtins::BUILTIN_CMD[21]
    };
}
#[macro_export]
macro_rules! PRINTFCMD {
    () => {
        crate::builtins::BUILTIN_CMD[22]
    };
}
#[macro_export]
macro_rules! PWDCMD {
    () => {
        crate::builtins::BUILTIN_CMD[23]
    };
}
#[macro_export]
macro_rules! READCMD {
    () => {
        crate::builtins::BUILTIN_CMD[24]
    };
}
#[macro_export]
macro_rules! RETURNCMD {
    () => {
        crate::builtins::BUILTIN_CMD[26]
    };
}
#[macro_export]
macro_rules! SETCMD {
    () => {
        crate::builtins::BUILTIN_CMD[27]
    };
}
#[macro_export]
macro_rules! SHIFTCMD {
    () => {
        crate::builtins::BUILTIN_CMD[28]
    };
}
#[macro_export]
macro_rules! TESTCMD {
    () => {
        crate::builtins::BUILTIN_CMD[2]
    };
}
#[macro_export]
macro_rules! TIMESCMD {
    () => {
        crate::builtins::BUILTIN_CMD[30]
    };
}
#[macro_export]
macro_rules! TRAPCMD {
    () => {
        crate::builtins::BUILTIN_CMD[31]
    };
}
#[macro_export]
macro_rules! TRUECMD {
    () => {
        crate::builtins::BUILTIN_CMD[1]
    };
}
#[macro_export]
macro_rules! TYPECMD {
    () => {
        crate::builtins::BUILTIN_CMD[33]
    };
}
#[macro_export]
macro_rules! ULIMITCMD {
    () => {
        crate::builtins::BUILTIN_CMD[34]
    };
}
#[macro_export]
macro_rules! UMASKCMD {
    () => {
        crate::builtins::BUILTIN_CMD[35]
    };
}
#[macro_export]
macro_rules! UNALIASCMD {
    () => {
        crate::builtins::BUILTIN_CMD[36]
    };
}
#[macro_export]
macro_rules! UNSETCMD {
    () => {
        crate::builtins::BUILTIN_CMD[37]
    };
}
#[macro_export]
macro_rules! WAITCMD {
    () => {
        crate::builtins::BUILTIN_CMD[38]
    };
}

pub const BUILTIN_SPECIAL: i32 = 0x1;
pub const BUILTIN_REGULAR: i32 = 0x2;
pub const BUILTIN_ASSIGN: i32 = 0x4;
