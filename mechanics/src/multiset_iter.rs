use std::marker::PhantomData;

/// Return all unique combinations of the provided keys with provided multiplicity of k
/// k.
pub fn multiset_k_combination_iter<A, K, F>(
    keys: A,
    multiplicity: F,
    k: usize,
) -> impl Iterator<Item = Vec<(K, usize)>> + Clone
where
    A: AsRef<[K]> + Clone,
    K: Clone,
    F: Fn(&K) -> usize + Clone,
{
    // Start out already-done if the required `k` is unsatisfiable.
    let done = keys.as_ref().iter().map(&multiplicity).sum::<usize>() < k;
    let mut current = vec![0; keys.as_ref().len()];

    if !done {
        __fill_to_k(k, |idx| multiplicity(&keys.as_ref()[idx]), &mut current);
    }

    MultisetKIter {
        keys,
        multiplicity,
        current,
        done,
        _k: PhantomData,
    }
}

#[derive(Clone)]
struct MultisetKIter<A, K, F>
where
    A: AsRef<[K]> + Clone,
    K: Clone,
    F: Fn(&K) -> usize + Clone,
{
    /// The keys that we are computing the multiset over. Note that we will use these keys
    /// irrespective of the actual keys in the provided `multiplicity`, so in practice we are using the
    /// sub-multiset spanned by `keys`.
    keys: A,
    /// The multiplicity of each key in the multiset
    multiplicity: F,
    /// The current allocation of items in the multiset. Indices correspond to indices in `keys`;
    /// elements are the count of values in `keys`. If !done, the sum of elements should always be
    /// equal to `k`.
    current: Vec<usize>,
    /// Whether this iterator has been fully consumed.
    done: bool,

    _k: PhantomData<K>,
}

impl<A, K, F> Iterator for MultisetKIter<A, K, F>
where
    A: AsRef<[K]> + Clone,
    K: Clone,
    F: Fn(&K) -> usize + Clone,
{
    type Item = Vec<(K, usize)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        // 0. Compute the requisite output (to avoid off-by-one)
        let output = Some(
            self.current
                .iter()
                .enumerate()
                .filter(|(_, ct)| **ct > 0)
                .map(|(idx, ct)| (self.keys.as_ref()[idx].clone(), *ct))
                .collect(),
        );

        // Attempt to increment the least-significant key-slots

        // 1. Find the first nonzero value
        let mut i = self.current.iter().position(|x| *x != 0).unwrap();

        // 2. Set it to zero (stashing the previous value for later use)
        let mut l = self.current[i];
        self.current[i] = 0;

        // 3. Increment and carry
        loop {
            i += 1;

            // If we went past the last key, we're done!
            if i >= self.current.len() {
                self.done = true;
                break;
            }

            if self.current[i] == (self.multiplicity)(&self.keys.as_ref()[i]) {
                // Carry if we need to increment past the multiplicity of the key
                l += self.current[i];
                self.current[i] = 0;
            } else {
                l -= 1;
                self.current[i] += 1;
                break;
            }
        }

        // 4. Fill in the lower key-slots
        __fill_to_k(
            l,
            |idx| (self.multiplicity)(&self.keys.as_ref()[idx]),
            &mut self.current,
        );

        output
    }
}

/// Fill `current` from the least-significant key-slot with `k` items.
fn __fill_to_k(k: usize, multiplicity: impl Fn(usize) -> usize, state: &mut [usize]) {
    let mut i = 0;
    let mut n = k;
    loop {
        let m = multiplicity(i);
        if n <= m {
            debug_assert_eq!(state[i], 0);
            state[i] = n;
            break;
        }

        state[i] = m;
        n -= m;
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::multiset_k_combination_iter;

    #[test]
    fn test_multiset_iter() {
        let multiset = [('a', 10), ('b', 4), ('c', 3)]
            .into_iter()
            .collect::<BTreeMap<_, _>>();
        let res = multiset_k_combination_iter(
            &multiset.keys().rev().collect::<Vec<_>>(),
            |k| multiset[*k],
            6,
        )
        .map(|v| v.into_iter().map(|(c, ct)| (*c, ct)).collect::<Vec<_>>())
        .collect::<Vec<_>>();
        assert_eq!(
            res,
            vec![
                vec![('c', 3), ('b', 3)],
                vec![('c', 2), ('b', 4)],
                vec![('c', 3), ('b', 2), ('a', 1)],
                vec![('c', 2), ('b', 3), ('a', 1)],
                vec![('c', 1), ('b', 4), ('a', 1)],
                vec![('c', 3), ('b', 1), ('a', 2)],
                vec![('c', 2), ('b', 2), ('a', 2)],
                vec![('c', 1), ('b', 3), ('a', 2)],
                vec![('b', 4), ('a', 2)],
                vec![('c', 3), ('a', 3)],
                vec![('c', 2), ('b', 1), ('a', 3)],
                vec![('c', 1), ('b', 2), ('a', 3)],
                vec![('b', 3), ('a', 3)],
                vec![('c', 2), ('a', 4)],
                vec![('c', 1), ('b', 1), ('a', 4)],
                vec![('b', 2), ('a', 4)],
                vec![('c', 1), ('a', 5)],
                vec![('b', 1), ('a', 5)],
                vec![('a', 6)],
            ]
        )
    }
}
