mod brainfuck;

fn main() {
    let ast = brainfuck::parse_file(&std::os::args()[1]).unwrap();
    let optimized = brainfuck::optimize(&*ast);
    brainfuck::write_asm(&mut std::io::stdout(), &*optimized);
}
