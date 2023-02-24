
pub const NCMD: i32 = 0;
pub const NPIPE: i32 = 1;
pub const NREDIR: i32 = 2;
pub const NBACKGND: i32 = 3;
pub const NSUBSHELL: i32 = 4;
pub const NAND: i32 = 5;
pub const NOR: i32 = 6;
pub const NSEMI: i32 = 7;
pub const NIF: i32 = 8;
pub const NWHILE: i32 = 9;
pub const NUNTIL: i32 = 10;
pub const NFOR: i32 = 11;
pub const NCASE: i32 = 12;
pub const NCLIST: i32 = 13;
pub const NDEFUN: i32 = 14;
pub const NARG: i32 = 15;
pub const NTO: i32 = 16;
pub const NCLOBBER: i32 = 17;
pub const NFROM: i32 = 18;
pub const NFROMTO: i32 = 19;
pub const NAPPEND: i32 = 20;
pub const NTOFD: i32 = 21;
pub const NFROMFD: i32 = 22;
pub const NHERE: i32 = 23;
pub const NXHERE: i32 = 24;
pub const NNOT: i32 = 25;

pub struct ncmd {
    r#type: i32,
    line_no: i32,
    assign: Box<Node>,
    args: Box<Node>,
    redirect: Box<Node>
}


pub struct npipe {
    r#type: i32,
    background: i32,
    cmd_list: Box<NodeList>,
}


pub struct nredir {
    r#type: i32,
    line_num: i32,
    node: Box<Node>,
    redirect: Box<Node>,
}


pub struct nbinary {
    r#type: i32,
    ch1: Box<Node>,
    ch2: Box<Node>,
}

pub struct nif {
    r#type: i32,
    test: Box<Node>,
    if_part: Box<Node>,
    else_part: Box<Node>,
}


pub struct nfor {
    r#type: i32,
    line_num: i32,
    args: Box<Node>,
    var: String,
}


pub struct ncase {
    r#type: i32,
    line_num: i32,
    expr: Box<Node>,
    cases: Box<Node>,
}


pub struct nclist {
    r#type: i32,
    next: Box<Node>,
    pattern: Box<Node>,
    body: Box<Node>,
}


pub struct ndefun {
    r#type: i32,
    line_num: i32,
    text: String,
    body: Box<Node>
}


pub struct narg {
    r#type: i32,
    next: Box<Node>,
    text: String,
    back_quote: Box<NodeList>,
}


pub struct nfile {
    r#type: i32,
    next: Box<Node>,
    fd: i32,
    file_name: Box<Node>,
    expfname: String,
}


pub struct ndup {
    r#type: i32,
    next: Box<Node>,
    fd: i32,
    doc: Box<Node>
}


pub struct nhere {
    r#type: i32,
    next: Box<Node>,
    fd: i32,
    doc: Box<Node>,
}

pub struct nnot {
    r#type: i32,
    com: Box<Node>,
}

pub union Node {
    r#type: i32,
    ncmd: std::mem::ManuallyDrop<ncmd>,
    npipe: std::mem::ManuallyDrop<npipe>,
    nredir: std::mem::ManuallyDrop<nredir>,
    nbinary: std::mem::ManuallyDrop<nbinary>,
    nif: std::mem::ManuallyDrop<nif>,
    nfor: std::mem::ManuallyDrop<nfor>,
    ncase: std::mem::ManuallyDrop<ncase>,
    nclist: std::mem::ManuallyDrop<nclist>,
    ndefun: std::mem::ManuallyDrop<ndefun>,
    narg: std::mem::ManuallyDrop<narg>,
    nfile: std::mem::ManuallyDrop<nfile>,
    ndup: std::mem::ManuallyDrop<ndup>,
    nhere: std::mem::ManuallyDrop<nhere>,
    nnot: std::mem::ManuallyDrop<nnot>,
}

pub struct NodeList {
    next: Box<NodeList>,
    node: Node,
}

pub struct FuncNode {
    count: i32,
    node: Box<Node>,
}



/*
 * Make a copy of a parse tree.
 */
pub fn copy_parse_tree (node: Box<Node>) -> Box<FuncNode> {
        
}
