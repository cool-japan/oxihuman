#![allow(dead_code)]

/// Binary search utilities for sorted slices.
#[allow(dead_code)]
pub struct BinarySearch;

/// Performs binary search on a sorted slice, returns the index if found.
#[allow(dead_code)]
pub fn binary_search_sorted(data: &[f64], target: f64) -> Option<usize> {
    let mut lo = 0usize;
    let mut hi = data.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if (data[mid] - target).abs() < f64::EPSILON {
            return Some(mid);
        } else if data[mid] < target {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    None
}

/// Returns the index of the first element >= target.
#[allow(dead_code)]
pub fn lower_bound(data: &[f64], target: f64) -> usize {
    let mut lo = 0usize;
    let mut hi = data.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if data[mid] < target {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo
}

/// Returns the index of the first element > target.
#[allow(dead_code)]
pub fn upper_bound(data: &[f64], target: f64) -> usize {
    let mut lo = 0usize;
    let mut hi = data.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if data[mid] <= target {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo
}

/// Returns indices (lo, hi) bounding the range [lo_val, hi_val].
#[allow(dead_code)]
pub fn binary_search_range(data: &[f64], lo_val: f64, hi_val: f64) -> (usize, usize) {
    (lower_bound(data, lo_val), upper_bound(data, hi_val))
}

/// Returns the index of the element closest to target.
#[allow(dead_code)]
pub fn search_closest(data: &[f64], target: f64) -> Option<usize> {
    if data.is_empty() {
        return None;
    }
    let pos = lower_bound(data, target);
    if pos == 0 {
        return Some(0);
    }
    if pos >= data.len() {
        return Some(data.len() - 1);
    }
    if (data[pos] - target).abs() < (data[pos - 1] - target).abs() {
        Some(pos)
    } else {
        Some(pos - 1)
    }
}

/// Returns the insertion point to keep the slice sorted.
#[allow(dead_code)]
pub fn search_insertion_point(data: &[f64], target: f64) -> usize {
    lower_bound(data, target)
}

/// Counts elements in [lo_val, hi_val].
#[allow(dead_code)]
pub fn search_count_in_range(data: &[f64], lo_val: f64, hi_val: f64) -> usize {
    let (lo, hi) = binary_search_range(data, lo_val, hi_val);
    hi.saturating_sub(lo)
}

/// Returns true if the slice is sorted in ascending order.
#[allow(dead_code)]
pub fn is_sorted_ascending(data: &[f64]) -> bool {
    data.windows(2).all(|w| w[0] <= w[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_search_found() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(binary_search_sorted(&data, 3.0), Some(2));
    }

    #[test]
    fn test_binary_search_not_found() {
        let data = vec![1.0, 2.0, 4.0, 5.0];
        assert_eq!(binary_search_sorted(&data, 3.0), None);
    }

    #[test]
    fn test_lower_bound() {
        let data = vec![1.0, 2.0, 2.0, 3.0, 5.0];
        assert_eq!(lower_bound(&data, 2.0), 1);
    }

    #[test]
    fn test_upper_bound() {
        let data = vec![1.0, 2.0, 2.0, 3.0, 5.0];
        assert_eq!(upper_bound(&data, 2.0), 3);
    }

    #[test]
    fn test_binary_search_range() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(binary_search_range(&data, 2.0, 4.0), (1, 4));
    }

    #[test]
    fn test_search_closest() {
        let data = vec![1.0, 3.0, 5.0, 7.0];
        assert_eq!(search_closest(&data, 4.0), Some(1));
        assert_eq!(search_closest(&data, 6.0), Some(2));
    }

    #[test]
    fn test_search_insertion_point() {
        let data = vec![1.0, 3.0, 5.0];
        assert_eq!(search_insertion_point(&data, 2.0), 1);
        assert_eq!(search_insertion_point(&data, 4.0), 2);
    }

    #[test]
    fn test_search_count_in_range() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(search_count_in_range(&data, 2.0, 4.0), 3);
    }

    #[test]
    fn test_is_sorted_ascending() {
        assert!(is_sorted_ascending(&[1.0, 2.0, 3.0]));
        assert!(!is_sorted_ascending(&[3.0, 1.0, 2.0]));
    }

    #[test]
    fn test_empty_slice() {
        assert_eq!(binary_search_sorted(&[], 1.0), None);
        assert_eq!(search_closest(&[], 1.0), None);
        assert!(is_sorted_ascending(&[]));
    }
}
