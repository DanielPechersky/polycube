use std::collections::BTreeSet;

use bitvec::prelude::*;

use polycube::{children, Canonicalized, Polycube};

fn main() {
    let mut generation = BTreeSet::new();
    generation.insert(Canonicalized(Polycube(1, bitvec![1])));

    for _ in 0..12 {
        generation = generation
            .into_iter()
            .flat_map(children)
            .collect::<BTreeSet<_>>();
        println!("{} cubes", generation.len());
    }
}
