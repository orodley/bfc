mod brainfuck;

fn main() {
    let ast = brainfuck::parse_file(&std::os::args()[1]);
    println!("AST = {}", ast);
}
