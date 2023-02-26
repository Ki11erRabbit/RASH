use crate::nodes::Node;
use crate::token;

#[macro_export]
macro_rules! Node_EOF {
    () => {
        unsafe {
            Box::new(Some( crate::nodes::Node { r#type: crate::parser::TOKEN_PUSH_BACK }))
        }
    };
}

static mut LAST_TOKEN: i32 = 0;
pub static mut TOKEN_PUSH_BACK: i32 = 0;
static mut WHICH_PROMPT: i32 = 0;      // 1 == PS1, 2 == PS2
static mut CHECK_KWD: i32 = 0;


#[inline]
pub fn eof() -> bool {
    unsafe {
        return (TOKEN_PUSH_BACK != 0) && (LAST_TOKEN != 0) == (token::TEOF == 0) 
    }
}

pub fn parse_cmd(interact: i32) -> Box<Option<Node>> {

    Box::new(Some(Node { r#type: 0}))
}



pub fn expand_str(ps: &str) -> String {

    "hello world".to_string()
}

pub fn fix_redir(node: Option<Box<Node>>, text: &str, err: i32) {

}
