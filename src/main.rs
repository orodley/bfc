mod brainfuck;

fn main() {
    let ast = brainfuck::parse_file(&std::os::args()[1]).unwrap();
    println!("AST = {}", ast);
    let optimized = brainfuck::optimize(&*ast);
    println!("Optimized AST = {}", optimized);
    println!("Un-optimized size = {}, optimized size = {}",
         brainfuck::size(&*ast), brainfuck::size(&*optimized));
}
