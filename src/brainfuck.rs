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

pub fn parse_file(filename: &String) -> io::IoResult<Box<AST>> {
    let path = Path::new(filename);
    let file = try!(io::File::open(&path));
    let mut file = io::BufferedReader::new(file);

    Ok(read_ast(&mut file))
}

fn read_ast(reader: &mut io::BufferedReader<io::File>) -> Box<AST> {
    let mut ast = Vec::new();
    loop {
        match reader.read_char() {
            Ok(c) => {
                match c {
                    '>' => ast.push(MovePointer(1)),
                    '<' => ast.push(MovePointer(-1)),
                    '+' => ast.push(ChangeValue(1)),
                    '-' => ast.push(ChangeValue(-1)),
                    '.' => ast.push(OutputValue),
                    ',' => ast.push(InputValue),
                    _ => (),
                };
            }
            Err(..) => return box ast
        }
    }
}
