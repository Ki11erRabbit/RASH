use std::fmt::Display;
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

#[derive(PartialEq)]
pub struct ncmd {
    pub r#type: i32,
    pub line_no: i32,
    pub assign: Box<Option<Node>>,
    pub args: Box<Option<Node>>,
    pub redirect: Box<Option<Node>>
}


#[derive(PartialEq)]
pub struct npipe {
    pub r#type: i32,
    pub background: i32,
    pub cmd_list: Box<Option<NodeList>>,
}


#[derive(PartialEq)]
pub struct nredir {
    pub r#type: i32,
    pub line_num: i32,
    pub node: Box<Option<Node>>,
    pub redirect: Box<Option<Node>>,
}


#[derive(PartialEq)]
pub struct nbinary {
    pub r#type: i32,
    pub ch1: Box<Option<Node>>,
    pub ch2: Box<Option<Node>>,
}

#[derive(PartialEq)]
pub struct nif {
    pub r#type: i32,
    pub test: Box<Option<Node>>,
    pub if_part: Box<Option<Node>>,
    pub else_part: Box<Option<Node>>,
}


#[derive(PartialEq)]
pub struct nfor {
    pub r#type: i32,
    pub line_num: i32,
    pub args: Box<Option<Node>>,
    pub body: Box<Option<Node>>,
    pub var: String,
}


#[derive(PartialEq)]
pub struct ncase {
    pub r#type: i32,
    pub line_num: i32,
    pub expr: Box<Option<Node>>,
    pub cases: Box<Option<Node>>,
}


#[derive(PartialEq)]
pub struct nclist {
    pub r#type: i32,
    pub next: Box<Option<Node>>,
    pub pattern: Box<Option<Node>>,
    pub body: Box<Option<Node>>,
}


#[derive(PartialEq)]
pub struct ndefun {
    pub r#type: i32,
    pub line_num: i32,
    pub text: String,
    pub body: Box<Option<Node>>
}


#[derive(PartialEq)]
pub struct narg {
    pub r#type: i32,
    pub next: Box<Option<Node>>,
    pub text: String,
    pub back_quote: Box<Option<NodeList>>,
}


#[derive(PartialEq)]
pub struct nfile {
    pub r#type: i32,
    pub next: Box<Option<Node>>,
    pub fd: i32,
    pub file_name: Box<Option<Node>>,
    pub expfname: String,
}


#[derive(PartialEq)]
pub struct ndup {
    pub r#type: i32,
    pub next: Box<Option<Node>>,
    pub fd: i32,
    pub doc: Box<Option<Node>>
}


#[derive(PartialEq)]
pub struct nhere {
    pub r#type: i32,
    pub next: Box<Option<Node>>,
    pub fd: i32,
    pub doc: Box<Option<Node>>,
}

#[derive(PartialEq)]
pub struct nnot {
    pub r#type: i32,
    pub com: Box<Option<Node>>,
}

pub union Node {
    pub r#type: i32,
    pub ncmd: std::mem::ManuallyDrop<ncmd>,
    pub npipe: std::mem::ManuallyDrop<npipe>,
    pub nredir: std::mem::ManuallyDrop<nredir>,
    pub nbinary: std::mem::ManuallyDrop<nbinary>,
    pub nif: std::mem::ManuallyDrop<nif>,
    pub nfor: std::mem::ManuallyDrop<nfor>,
    pub ncase: std::mem::ManuallyDrop<ncase>,
    pub nclist: std::mem::ManuallyDrop<nclist>,
    pub ndefun: std::mem::ManuallyDrop<ndefun>,
    pub narg: std::mem::ManuallyDrop<narg>,
    pub nfile: std::mem::ManuallyDrop<nfile>,
    pub ndup: std::mem::ManuallyDrop<ndup>,
    pub nhere: std::mem::ManuallyDrop<nhere>,
    pub nnot: std::mem::ManuallyDrop<nnot>,
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let node_type = match self {
            Node {r#type: i32} => "i32",
            Node {ncmd: ncmd } => "ncmd",
            Node {npipe: npipe } => "npipe",
            Node {nredir: nredir } => "nredir",
            Node {nbinary: nbinary } => "nbinary",
            Node {nif: nif } => "nif",
            Node {nfor: nfor } => "nfor",
            Node {ncase: ncase } => "ncase",
            Node {nclist: nclist } => "nclist",
            Node {ndefun: ndefun } => "ndefun",
            Node {narg: narg } => "narg",
            Node {ndup: ndup } => "ndup",
            Node {nhere: nhere } => "nhere",
            Node {nnot: nnot } => "nnot",
        };

        write!(f,"type: {}",node_type)
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
            self.r#type == other.r#type && self.ncmd == other.ncmd && self.npipe == other.npipe &&
                self.nredir == other.nredir && self.nbinary == other.nbinary && self.nif == other.nif &&
                self.nfor == other.nfor && self.ncase == other.ncase && self.nclist == other.nclist &&
                self.ndefun == other.ndefun && self.narg == other.narg && self.nfile == other.nfile &&
                self.ndup == other.ndup && self.nhere == other.nhere && self.nnot == other.nnot
        }
}

pub struct NodeList {
    next: Box<Option<NodeList>>,
    node: Node,
}

impl PartialEq for NodeList {
    fn eq(&self, other: &Self) -> bool {
        let mut right = other;
        let mut left = self;
        loop {
            if right.node != left.node {
                return false;
            }

            if right.next.is_some() && left.next.is_some() {
                right = &right.next.unwrap();
                left = &left.next.unwrap();
            }
            else {
                return false;
            }
        }
}
}

pub struct FuncNode {
    count: i32,
    node: Box<Node>,
}



/*
 * Make a copy of a parse tree.
 */
pub fn copy_parse_tree(node: Box<Node>) -> Box<FuncNode> {
    
    unimplemented!()
}

pub fn free_parse_tree(node: Box<Node>) {


}
