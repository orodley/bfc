extern crate "rustc_llvm" as llvm;

use std::io;
use std::iter::AdditiveIterator;

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
        if self.is_done() {
            None
        } else {
            let c = self.text[self.pos];
            self.pos += 1;
            Some(c)
        }
    }

    fn prev_char(&self) -> Option<u8> {
        if self.pos == 0 {
            None
        } else {
            Some(self.text[self.pos - 1])
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
                    
                    match reader.prev_char() {
                        Some(c) => {
                            if (c as char) != ']' {
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

pub fn size(ast: &AST) -> uint {
    let mut sum = 0u;
    for token in ast.iter() {
        sum += match token {
            &Block(ref ast) => size(&**ast) + 2,
            _ => 1
        }
    }

    sum
}
