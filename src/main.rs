use polycube::{print_bitvec, Generation};

fn main() {
    let mut args = std::env::args();
    args.next();
    let n = args.next().unwrap().parse::<usize>().unwrap();
    let display = matches!(args.next().as_deref(), Some("-d" | "--display"));

    let mut generation = Generation::default();

    for _ in 0..n {
        generation.advance();

        println!("Gen {}: {}", generation.age, generation.shapes.len());

        if display {
            for v in generation.shapes.iter() {
                print_bitvec(v, generation.age);
            }
        }
    }
}
