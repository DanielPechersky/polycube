use std::collections::BTreeSet;

use bitvec::prelude::*;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Canonicalized(pub Polycube);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Polycube(pub usize, pub BitVec);

pub fn children(Canonicalized(Polycube(n, bitvec)): &Canonicalized) -> BTreeSet<Canonicalized> {
    let mut children = BTreeSet::new();
    let (parent, placements) =
        potential_cube_placements(Canonicalized(Polycube(*n, bitvec.clone())));
    for i in placements.iter_ones() {
        let mut new_polycube = parent.clone();
        new_polycube.set(i, true);
        let canonicalized = canonicalize(Polycube(n + 1, new_polycube), n + 2);
        children.insert(canonicalized);
    }
    children
}

#[test]
fn children_test() {
    let c = Canonicalized(Polycube(1, bitvec![1]));
    let (ns, vs): (Vec<_>, Vec<_>) = children(&c)
        .into_iter()
        .map(|Canonicalized(Polycube(n, v))| (n, v))
        .unzip();
    assert_eq!(vs, vec![bitvec![1, 1, 0, 0]]);
    assert_eq!(ns, vec![2]);
}

#[test]
fn second_generation_children_test() {
    let c = Canonicalized(Polycube(2, bitvec![1, 1, 0, 0]));
    let (ns, vs): (Vec<_>, Vec<_>) = children(&c)
        .into_iter()
        .map(|Canonicalized(Polycube(n, v))| (n, v))
        .unzip();
    assert_eq!(
        vs,
        vec![
            bitvec![1, 1, 0, 1, 0, 0, 0, 0, 0],
            bitvec![1, 1, 1, 0, 0, 0, 0, 0, 0]
        ]
    );
    assert_eq!(ns, vec![3, 3]);
}

pub fn canonicalize(p: Polycube, bitvec_side_length: usize) -> Canonicalized {
    let c = rotations(p, bitvec_side_length)
        .into_iter()
        .map(|p| move_top_left(p, bitvec_side_length))
        .max_by_key(|Polycube(_, v)| v.clone())
        .unwrap();

    let Polycube(n, v) = c;
    let mut chunks = v.chunks(bitvec_side_length);
    let mut v = BitVec::with_capacity(n.pow(2));
    for _ in 0..n {
        let c = chunks.next().unwrap();
        v.extend_from_bitslice(&c[0..n]);
    }

    assert_eq!(v.len(), n.pow(2));

    Canonicalized(Polycube(n, v))
}

#[test]
fn canonicalize_test() {
    let p = Polycube(2, bitvec![0, 1, 0, 1]);
    let c = canonicalize(p, 2);
    assert_eq!(c, Canonicalized(Polycube(2, bitvec![1, 1, 0, 0])));

    let p = Polycube(2, bitvec![0, 0, 0, 0, 1, 0, 0, 1, 0]);
    let c = canonicalize(p, 3);
    assert_eq!(c, Canonicalized(Polycube(2, bitvec![1, 1, 0, 0])));

    let p = Polycube(3, bitvec![1, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    let c = canonicalize(p, 4);
    assert_eq!(
        c,
        Canonicalized(Polycube(3, bitvec![1, 1, 0, 1, 0, 0, 0, 0, 0]))
    );
}

pub fn move_top_left(Polycube(n, mut bitvec): Polycube, bitvec_side_length: usize) -> Polycube {
    let leading_rows = bitvec.leading_zeros() / bitvec_side_length;
    for i in 0..bitvec_side_length {
        if (0..bitvec_side_length).any(|j| bitvec[j * bitvec_side_length + i]) {
            bitvec.shift_left(i + leading_rows * bitvec_side_length);
            break;
        }
    }
    Polycube(n, bitvec)
}

#[test]
fn move_top_left_test() {
    let p = Polycube(2, bitvec![0, 1, 0, 1]);
    let c = move_top_left(p, 2);
    assert_eq!(c, Polycube(2, bitvec![1, 0, 1, 0]));

    let p = Polycube(2, bitvec![0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    let c = move_top_left(p, 4);
    assert_eq!(
        c,
        Polycube(2, bitvec![1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
    );

    let p = Polycube(3, bitvec![0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0]);
    let c = move_top_left(p, 4);
    assert_eq!(
        c,
        Polycube(3, bitvec![1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
    );
}

pub fn rotations(p: Polycube, sl: usize) -> [Polycube; 4] {
    let r1 = rotate90(p.clone(), sl);
    let r2 = rotate90(r1.clone(), sl);
    let r3 = rotate90(r2.clone(), sl);
    [p, r1, r2, r3]
}

pub fn rotate90(Polycube(n, mut bitvec): Polycube, sl: usize) -> Polycube {
    transpose(sl, &mut bitvec);
    reverse_rows(sl, &mut bitvec);
    Polycube(n, bitvec)
}

fn reverse_rows(n: usize, bitslice: &mut BitSlice) {
    bitslice.chunks_exact_mut(n).for_each(|row| row.reverse());
}

fn transpose(n: usize, bitslice: &mut BitSlice) {
    for j in 0..n {
        for k in 0..j {
            let (a, b) = (index(n, j, k), index(n, k, j));
            bitslice.swap(a, b);
        }
    }
}

fn index(n: usize, j: usize, k: usize) -> usize {
    j * n + k
}

pub fn print_bitvec(n: usize, bitvec: &BitSlice) {
    for j in 0..n {
        for k in 0..n {
            let Some(v) = bitvec.get(index(n, j, k)) else {
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

pub fn potential_cube_placements(c: Canonicalized) -> (BitVec, BitVec) {
    let n = c.0 .0 + 2;
    let original = pad_out(c);
    let mut placements: BitVec = bitvec![0; n.pow(2)];

    // left
    placements |= &original[1..];

    // right
    placements[1..] |= &original;

    // up
    placements |= &original[n..];

    // down
    placements[n..] |= &original;

    placements &= !original.clone();

    (original, placements)
}

#[test]
fn potential_cube_placements_test() {
    let c = Canonicalized(Polycube(1, bitvec![1]));
    let p = potential_cube_placements(c);
    assert_eq!(
        p,
        (
            bitvec![0, 0, 0, 0, 1, 0, 0, 0, 0],
            bitvec![0, 1, 0, 1, 0, 1, 0, 1, 0]
        )
    );
}

fn pad_out(Canonicalized(Polycube(n, bitvec)): Canonicalized) -> BitVec {
    let mut r = bitvec![0; (n + 2).pow(2)];
    for i in 1..(n + 2 - 1) {
        r[index(n + 2, i, 1)..index(n + 2, i, n + 1)] |=
            &bitvec[index(n, i - 1, 0)..index(n, i - 1, n)]
    }
    r
}

#[test]
fn pad_out_test() {
    let c = Canonicalized(Polycube(1, bitvec![1]));
    let p = pad_out(c);
    assert_eq!(p, bitvec![0, 0, 0, 0, 1, 0, 0, 0, 0]);

    let c = Canonicalized(Polycube(2, bitvec![1, 1, 1, 0]));
    let p = pad_out(c);
    assert_eq!(p, bitvec![0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0]);
}
