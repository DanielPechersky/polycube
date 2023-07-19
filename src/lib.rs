use std::collections::HashSet;

use bitvec::{prelude::*, slice::IterOnes};
use ouroboros::self_referencing;

#[derive(Clone)]
pub struct Generation {
    pub shapes: HashSet<BitVec>,
    pub age: usize,
}

impl Default for Generation {
    fn default() -> Self {
        Self {
            shapes: [bitvec![1]].into(),
            age: 1,
        }
    }
}

impl Generation {
    pub fn advance(&mut self) {
        use rayon::prelude::{IntoParallelIterator, ParallelIterator};

        let shapes = std::mem::take(&mut self.shapes);

        let shapes = shapes
            .into_par_iter()
            .flat_map_iter(|p| children(p, self.age))
            .collect();

        self.shapes = shapes;
        self.age += 1;
    }
}

#[self_referencing]
struct PlacementsIter {
    placements: BitVec,

    #[borrows(placements)]
    #[not_covariant]
    iter: IterOnes<'this, usize, Lsb0>,
}

impl Iterator for PlacementsIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        self.with_iter_mut(|iter| iter.next())
    }
}

pub fn children(parent: BitVec, generation: usize) -> impl Iterator<Item = BitVec> {
    let (parent, placements) = potential_cube_placements(parent, generation);

    let iter = PlacementsIterBuilder {
        placements: placements.clone(),
        iter_builder: |placements| placements.iter_ones(),
    }
    .build();

    iter.map(move |i| {
        let mut new_polycube = parent.clone();
        new_polycube.set(i, true);
        nudge_top_left(&mut new_polycube, generation + 2);
        let new_polycube = crop(&new_polycube, generation + 2, generation + 1);
        canonicalize(new_polycube, generation + 1)
    })
}

#[test]
fn children_test() {
    let mut children: Vec<_> = children(bitvec![1], 1).collect();
    children.sort();
    children.dedup();
    assert_eq!(children, vec![bits![1, 1, 0, 0]]);
}

#[test]
fn second_generation_children_test() {
    let mut children: Vec<_> = children(bitvec![1, 1, 0, 0], 2).collect();
    children.sort();
    children.dedup();
    assert_eq!(
        children,
        vec![
            bits![1, 1, 0, 1, 0, 0, 0, 0, 0],
            bits![1, 1, 1, 0, 0, 0, 0, 0, 0]
        ]
    );
}

pub fn canonicalize(polycube: BitVec, side_length: usize) -> BitVec {
    rotations(polycube, side_length)
        .into_iter()
        .map(|mut p| {
            move_top_left(&mut p, side_length);
            p
        })
        .max()
        .unwrap()
}

pub fn crop(bits: &BitSlice, from: usize, to: usize) -> BitVec {
    assert!(from >= to);

    if from == to {
        return bits.to_bitvec();
    }

    let mut chunks = bits.chunks(from);
    let mut v = BitVec::with_capacity(to.pow(2));
    for _ in 0..to {
        let c = chunks.next().unwrap();
        v.extend_from_bitslice(&c[0..to]);
    }
    assert_eq!(v.len(), to.pow(2));
    v
}

#[test]
fn canonicalize_test() {
    let c = canonicalize(bitvec![0, 1, 0, 1], 2);
    assert_eq!(c, bits![1, 1, 0, 0]);

    let c = canonicalize(bitvec![0, 0, 0, 0, 1, 0, 0, 1, 0], 3);
    assert_eq!(c, bits![1, 1, 0, 0, 0, 0, 0, 0, 0]);

    let c = canonicalize(bitvec![1, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 4);
    assert_eq!(c, bits![1, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
}

pub fn move_top_left(bits: &mut BitSlice, side_length: usize) {
    let leading_rows = leading_rows(bits, side_length);
    let leading_cols = leading_cols(bits, side_length);

    bits.shift_left(leading_cols + leading_rows * side_length);
}

fn leading_rows(bits: &mut BitSlice, side_length: usize) -> usize {
    bits.leading_zeros() / side_length
}

fn leading_cols(bits: &mut BitSlice, side_length: usize) -> usize {
    (0..side_length)
        .find(|&i| (0..side_length).any(|j| bits[i + j * side_length]))
        .unwrap_or(side_length)
}

pub fn nudge_top_left(bits: &mut BitSlice, side_length: usize) {
    let nudge_up = bits.leading_zeros() >= side_length;
    let nudge_left = (0..side_length).all(|i| !bits[i * side_length]);
    bits.shift_left(usize::from(nudge_left) + usize::from(nudge_up) * side_length);
}

#[test]
fn move_top_left_test() {
    let mut b = bitvec![0, 1, 0, 1];
    move_top_left(&mut b, 2);
    assert_eq!(b, bits![1, 0, 1, 0]);

    let mut b = bitvec![0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    move_top_left(&mut b, 4);
    assert_eq!(b, bits![1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

    let mut b = bitvec![0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0];
    move_top_left(&mut b, 4);
    assert_eq!(b, bits![1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
}

pub fn rotations(b: BitVec, side_length: usize) -> [BitVec; 4] {
    let r1 = rotate90(&b, side_length);
    let r2 = rotate90(&r1, side_length);
    let r3 = rotate90(&r2, side_length);
    [b, r1, r2, r3]
}

pub fn rotate90(b: &BitSlice, side_length: usize) -> BitVec {
    let s = side_length;
    let mut out = bitvec![0; s.pow(2)];
    for i in 0..s {
        for j in 0..s {
            let (idx_out, idx_in) = (index(s, i, j), index(s, j, s - i - 1));
            out.set(idx_out, b[idx_in]);
        }
    }
    out
}

fn index(n: usize, i: usize, j: usize) -> usize {
    i * n + j
}

pub fn print_bitvec(bitvec: &BitSlice, side_length: usize) {
    for i in 0..side_length {
        for j in 0..side_length {
            let Some(v) = bitvec.get(index(side_length, i, j)) else {
                    println!("❌");
                    println!();
                    return;
                };
            let c = if *v { '█' } else { '.' };
            print!("{c}");
        }
        println!();
    }
    println!();
}

pub fn potential_cube_placements(bitvec: BitVec, side_length: usize) -> (BitVec, BitVec) {
    let original = pad_all_sides(bitvec, side_length);
    let side_length = side_length + 2;
    let mut placements: BitVec = bitvec![0; side_length.pow(2)];

    // left
    placements |= &original[1..];

    // right
    placements[1..] |= &original;

    // up
    placements |= &original[side_length..];

    // down
    placements[side_length..] |= &original;

    let original = !original;
    placements &= &original;
    let original = !original;

    (original, placements)
}

#[test]
fn potential_cube_placements_test() {
    let p = potential_cube_placements(bitvec![1], 1);
    assert_eq!(
        p,
        (
            bitvec![0, 0, 0, 0, 1, 0, 0, 0, 0],
            bitvec![0, 1, 0, 1, 0, 1, 0, 1, 0]
        )
    );
}

fn pad_all_sides(bitvec: BitVec, side_length: usize) -> BitVec {
    let s = side_length;
    let mut r = bitvec![0; (s + 2).pow(2)];
    for i in 1..(s + 2 - 1) {
        r[index(s + 2, i, 1)..index(s + 2, i, s + 1)] |=
            &bitvec[index(s, i - 1, 0)..index(s, i - 1, s)]
    }
    r
}

#[test]
fn pad_out_test() {
    let p = pad_all_sides(bitvec![1], 1);
    assert_eq!(p, bitvec![0, 0, 0, 0, 1, 0, 0, 0, 0]);

    let p = pad_all_sides(bitvec![1, 1, 1, 0], 2);
    assert_eq!(p, bitvec![0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0]);
}
