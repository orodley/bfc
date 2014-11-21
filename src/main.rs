mod brainfuck;

fn main() {
    let ast = brainfuck::parse_file(&std::os::args()[1]);
    println!("AST = {}", ast);
    let optimized = brainfuck::optimize(&*ast.unwrap());
    println!("Optimized AST = {}", optimized);
}
