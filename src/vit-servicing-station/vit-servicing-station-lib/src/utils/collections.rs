use std::{collections::HashMap, hash::Hash};

/// Removes all but the last occurrence of elements that
/// resolve to the same key maintaining the order in which they
/// where first found.
///
/// # Examples
///
/// ```
/// use vit_servicing_station_lib::utils::collections::dedup_by_key_keep_last;
///
/// let v = vec![(0, "a"), (0, "b"), (1, "c"), (1, "d"), (2, "e")];
///
/// assert_eq!(dedup_by_key_keep_last(v.into_iter(), |i| i.0), [(0, "b"), (1, "d"), (2, "e")]);
/// ```
pub fn dedup_by_key_keep_last<F, I, K>(iter: I, key_fn: F) -> Vec<I::Item>
where
    I: Iterator,
    F: Fn(&I::Item) -> K,
    K: Eq + Hash,
{
    let mut h = HashMap::new();
    let mut v = Vec::new();

    for i in iter {
        if let Some(prev_ix) = h.insert(key_fn(&i), v.len()) {
            v[prev_ix] = i;
        } else {
            v.push(i);
        }
    }

    v
}
