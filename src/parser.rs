use crate::nodes::Node;
use crate::token;

#[macro_export]
macro_rules! Node_EOF {
    () => {
        unsafe {
            Some(Box::new( crate::nodes::Node { r#type: crate::parser::TOKEN_PUSH_BACK }))
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

pub fn is_assignment(text: &str) -> bool {

    let end = end_of_name(text);
    if end.is_none() {
        return false;
    }
    if end.unwrap().chars().nth(0) == Some('=') {
        return true;
    }

    false
    
}

#[inline]
fn is_name(chr: char) -> bool {
    chr.is_alphabetic() || chr == '_'
}

#[inline]
fn is_in_name(chr: char) -> bool {
    chr.is_alphanumeric() || chr == '_'
}


/*
 * Return of a legal variable name (a letter or underscore followed by zero or
 * more letters, underscores, and digits).
 */
fn end_of_name(text: &str) -> Option<String> {
    
    if !is_name(text.chars().nth(0).unwrap()) {
        return Some(text.to_string());
    }

    for i in 0..text.len() {
        if !is_in_name(text.chars().nth(i).unwrap()) {
            return Some(text.get(i..).unwrap().to_string());
        }
    }
    None
}
