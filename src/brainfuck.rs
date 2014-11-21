extern crate "rustc_llvm" as llvm;

use std::io;

type AST = Vec<ASTNode>;

#[deriving(Show)]
enum ASTNode {
    MovePointer(int),
    ChangeValue(int),
    OutputValue,
    InputValue,
    Block(Box<AST>),
}

struct Parser {
    text: Vec<u8>,
    pos: uint,
}

impl Parser {
    fn new(file: &mut io::File) -> io::IoResult<Parser> {
        let text = try!(file.read_to_end());
        Ok(Parser {
            text: text,
            pos: 0,
        })
    }

    fn read_char(&mut self) -> Option<u8> {
        if self.pos >= self.text.len() {
            None
        } else {
            let c = self.text[self.pos];
            self.pos += 1;
            Some(c)
        }
    }

    fn peek_char(&self) -> Option<u8> {
        if self.pos >= self.text.len() {
            None
        } else {
            Some(self.text[self.pos])
        }
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
            Some(ch) => match ch as char {
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
                }
                _ => (),
            },
            None => break,
        }
    }

    Some(box ast)
}
