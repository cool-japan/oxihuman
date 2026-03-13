// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub fn chunks_of<T: Clone>(v: &[T], size: usize) -> Vec<Vec<T>> {
    if size == 0 {
        return vec![];
    }
    v.chunks(size).map(|c| c.to_vec()).collect()
}

pub fn zip_with_vecs<A: Clone, B: Clone, C>(a: &[A], b: &[B], f: impl Fn(&A, &B) -> C) -> Vec<C> {
    a.iter().zip(b).map(|(x, y)| f(x, y)).collect()
}

pub fn flat_map_vec<T: Clone, U>(v: &[T], f: impl Fn(&T) -> Vec<U>) -> Vec<U> {
    v.iter().flat_map(f).collect()
}

pub fn take_while_vec<T: Clone>(v: &[T], f: impl Fn(&T) -> bool) -> Vec<T> {
    v.iter().take_while(|x| f(x)).cloned().collect()
}

pub fn drop_while_vec<T: Clone>(v: &[T], f: impl Fn(&T) -> bool) -> Vec<T> {
    v.iter().skip_while(|x| f(x)).cloned().collect()
}

pub fn partition_vec<T: Clone>(v: &[T], f: impl Fn(&T) -> bool) -> (Vec<T>, Vec<T>) {
    let mut yes = Vec::new();
    let mut no = Vec::new();
    for item in v {
        if f(item) {
            yes.push(item.clone());
        } else {
            no.push(item.clone());
        }
    }
    (yes, no)
}

pub fn intersperse<T: Clone>(v: &[T], sep: T) -> Vec<T> {
    if v.is_empty() {
        return vec![];
    }
    let mut out = Vec::with_capacity(v.len() * 2 - 1);
    for (i, item) in v.iter().enumerate() {
        if i > 0 {
            out.push(sep.clone());
        }
        out.push(item.clone());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunks_of() {
        /* split vec into chunks */
        let v = vec![1, 2, 3, 4, 5];
        let c = chunks_of(&v, 2);
        assert_eq!(c, vec![vec![1, 2], vec![3, 4], vec![5]]);
    }

    #[test]
    fn test_zip_with_vecs() {
        /* zip two vecs with a function */
        let a = vec![1, 2, 3];
        let b = vec![4, 5, 6];
        let r = zip_with_vecs(&a, &b, |x, y| x + y);
        assert_eq!(r, vec![5, 7, 9]);
    }

    #[test]
    fn test_flat_map_vec() {
        /* flat_map over a vec */
        let v = vec![1, 2, 3];
        let r = flat_map_vec(&v, |x| vec![*x, *x * 10]);
        assert_eq!(r, vec![1, 10, 2, 20, 3, 30]);
    }

    #[test]
    fn test_take_while_vec() {
        /* take while condition holds */
        let v = vec![1, 2, 3, 4, 5];
        let r = take_while_vec(&v, |x| *x < 4);
        assert_eq!(r, vec![1, 2, 3]);
    }

    #[test]
    fn test_drop_while_vec() {
        /* drop while condition holds */
        let v = vec![1, 2, 3, 4, 5];
        let r = drop_while_vec(&v, |x| *x < 3);
        assert_eq!(r, vec![3, 4, 5]);
    }

    #[test]
    fn test_partition_vec() {
        /* partition into two vecs */
        let v = vec![1, 2, 3, 4, 5];
        let (evens, odds) = partition_vec(&v, |x| x % 2 == 0);
        assert_eq!(evens, vec![2, 4]);
        assert_eq!(odds, vec![1, 3, 5]);
    }

    #[test]
    fn test_intersperse() {
        /* intersperse separator */
        let v = vec![1, 2, 3];
        let r = intersperse(&v, 0);
        assert_eq!(r, vec![1, 0, 2, 0, 3]);
    }

    #[test]
    fn test_chunks_empty() {
        /* empty input gives empty output */
        let v: Vec<i32> = vec![];
        assert_eq!(chunks_of(&v, 2), Vec::<Vec<i32>>::new());
    }
}
