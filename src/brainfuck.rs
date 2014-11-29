extern crate "rustc_llvm" as llvm;

use std::io;

type AST = Vec<ASTNode>;

#[deriving(Show, Clone)]
enum ASTNode {
    MovePointer(int),
    ChangeValue(int),
    OutputValue,
    InputValue,
    Block(Box<AST>),
}

struct Parser {
    text: String,
    pos: uint,
}

impl Parser {
    fn new(file: &mut io::File) -> io::IoResult<Parser> {
        let text = try!(file.read_to_string());
        Ok(Parser {
            text: text,
            pos: 0,
        })
    }

    fn read_char(&mut self) -> Option<char> {
        if self.is_done() {
            None
        } else {
            let c = self.text.as_slice().char_at(self.pos);
            self.pos += 1;
            Some(c)
        }
    }

    fn prev_char(&self) -> Option<char> {
        if self.pos == 0 {
            None
        } else {
            Some(self.text.as_slice().char_at(self.pos - 1))
        }
    }

    #[inline]
    fn is_done(&self) -> bool {
        self.pos >= self.text.len()
    }
}

pub fn parse_file(filename: &String) -> Option<Box<AST>> {
    let path = Path::new(filename);
    let mut file = match io::File::open(&path) {
        Ok(f) => f,
        Err(..) => return None,
    };
    let mut parser = match Parser::new(&mut file) {
        Ok(p) => p,
        Err(..) => return None,
    };

    read_ast(&mut parser)
}

// TODO: This should be a result type with parser errors rather than an Option
fn read_ast(reader: &mut Parser) -> Option<Box<AST>> {
    let mut ast = Vec::new();
    loop {
        match reader.read_char() {
            Some(ch) => match ch {
                '>' => ast.push(MovePointer(1)),
                '<' => ast.push(MovePointer(-1)),
                '+' => ast.push(ChangeValue(1)),
                '-' => ast.push(ChangeValue(-1)),
                '.' => ast.push(OutputValue),
                ',' => ast.push(InputValue),
                '[' => {
                    match read_ast(reader) {
                        Some(block_ast) => ast.push(Block(block_ast)),
                        None => return None
                    }
                    
                    match reader.prev_char() {
                        Some(c) => {
                            if c != ']' {
                                // TODO: error about unmatched '['
                                return None;
                            }
                        }
                        None => return None,
                    }
                }
                ']' => break,
                _ => (),
            },
            None => break,
        }
    }

    Some(box ast)
}

/// Merges together adjacent +'s and -'s, and <'s and >'s
pub fn optimize(ast: &AST) -> Box<AST> {
    let mut new_ast = Vec::new();

    let mut prev = OutputValue; // start on something that doesn't merge
    for token in ast.iter() {
        match token {
            &MovePointer(x) => match prev {
                MovePointer(y) => {
                    new_ast.pop();
                    new_ast.push(MovePointer(x + y));
                }
                _ => new_ast.push(MovePointer(x)),
            },
            &ChangeValue(x) => match prev {
                ChangeValue(y) => {
                    new_ast.pop();
                    new_ast.push(ChangeValue(x + y));
                }
                _ => new_ast.push(ChangeValue(x)),
            },
            &Block(ref ast) => new_ast.push(Block(optimize(&**ast))),
            _ => new_ast.push((*token).clone()),
        }

        // unwrap is safe as all of the above push a value
        prev = new_ast.last().unwrap().clone();
    }

    box new_ast
}

// Code generation
struct Codegen<'a> {
    writer: &'a mut io::Writer+'a,
    next_block: int,
}

static HEADER: &'static str = include_str!("header.s");
static FOOTER: &'static str = include_str!("footer.s");

pub fn write_asm(writer: &mut io::Writer, ast: &AST) -> io::IoResult<()> {
    let mut codegen = Codegen{ writer: writer, next_block: 0 };
    try!(codegen.writer.write_line(HEADER));
    try!(write_asm_for_ast(&mut codegen, ast));
    try!(codegen.writer.write_line(FOOTER));
    Ok(())
}

fn write_asm_for_ast(codegen: &mut Codegen, ast: &AST) -> io::IoResult<()> {
    for node in ast.iter() {
        try!(write_asm_for_node(codegen, node));
    }

    Ok(())
}

fn write_asm_for_node(codegen: &mut Codegen, node: &ASTNode) -> io::IoResult<()> {
    match *node {
        MovePointer(x) =>
            try!(writeln!(codegen.writer, "    addq    ${}, (ptr)", x)),
        ChangeValue(x) => {
            try!(writeln!(codegen.writer, "    movq    (ptr), %rax"));
            try!(writeln!(codegen.writer, "    addb    ${}, (%rax)", x));
        }
        OutputValue =>
            try!(codegen.writer.write_line("    call    write")),
        InputValue =>
            try!(codegen.writer.write_line("    call    read")),
        Block(ref block) => {
            let n = codegen.next_block;
            codegen.next_block += 1;
            try!(writeln!(codegen.writer, "block{}_start:", n));
            try!(writeln!(codegen.writer, "    movq    (ptr), %rax"));
            try!(writeln!(codegen.writer, "    cmpb    $0, (%rax)"));
            try!(writeln!(codegen.writer, "    je      block{}_end", n));
            try!(write_asm_for_ast(codegen, &**block));
            try!(writeln!(codegen.writer, "    jmp     block{}_start", n));
            try!(writeln!(codegen.writer, "block{}_end:", n));
        }
    }

    Ok(())
}
