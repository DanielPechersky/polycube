use std::collections::BTreeSet;

use bitvec::prelude::*;

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use polycube::{children, print_bitvec, Canonicalized, Polycube};

fn main() {
    let mut args = std::env::args();
    args.next();
    let n = args.next().unwrap().parse::<usize>().unwrap();
    let display = matches!(args.next().as_deref(), Some("-d" | "--display"));

    let mut generation = BTreeSet::new();
    generation.insert(Canonicalized(Polycube(1, bitvec![1])));

    println!("Gen 1: {}", generation.len());
    for g in 2..=n {
        generation = generation
            .par_iter()
            .flat_map(children)
            .collect::<BTreeSet<_>>();

        if display {
            for Canonicalized(Polycube(n, v)) in generation.iter() {
                print_bitvec(*n, v);
            }
        }

        println!("Gen {g}: {}", generation.len());
    }
}
