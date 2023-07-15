use std::collections::BTreeSet;

use bitvec::prelude::*;

use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use polycube::{children, print_bitvec};

fn main() {
    let mut args = std::env::args();
    args.next();
    let n = args.next().unwrap().parse::<usize>().unwrap();
    let display = matches!(args.next().as_deref(), Some("-d" | "--display"));

    let mut generation = BTreeSet::new();
    generation.insert(bitvec![1]);

    println!("Gen 1: {}", generation.len());
    for g in 2..=n {
        generation = generation
            .into_par_iter()
            .flat_map(|p| children(p, g - 1))
            .collect::<BTreeSet<_>>();

        if display {
            for v in generation.iter() {
                print_bitvec(v, g);
            }
        }

        println!("Gen {g}: {}", generation.len());
    }
}
