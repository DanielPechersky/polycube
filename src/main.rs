use std::collections::BTreeSet;

use bitvec::prelude::*;

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use polycube::{children, Canonicalized, Polycube};

fn main() {
    let mut generation = BTreeSet::new();
    generation.insert(Canonicalized(Polycube(1, bitvec![1])));

    println!("Gen 1: {}", generation.len());
    for g in 2..22 {
        generation = generation
            .par_iter()
            .flat_map(children)
            .collect::<BTreeSet<_>>();
        println!("Gen {g}: {}", generation.len());
    }
}
