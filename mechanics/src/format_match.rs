use std::collections::{BTreeMap, HashSet, VecDeque};
use std::iter::Peekable;

use anyhow::Error;

use crate::ordered_card::{AdjacentTupleSizes, MatchingCards, OrderedCard};

pub fn find_format_matches(
    format: Vec<AdjacentTupleSizes>,
    cards: BTreeMap<OrderedCard, usize>,
) -> impl Iterator<Item = Vec<MatchingCards>> {
    let mut queue = VecDeque::new();

    let (requirements, init) = FormatMatchState::empty(&format);
    queue.push_back(QueueItem::State(init));

    FormatMatchIterator {
        format: requirements,
        format_seq: format,
        queue,
        cards,
        visited: HashSet::new(),
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
struct FormatMatchState {
    /// We use this convulated matching so that FormatMatchState is the same
    /// regardless of the order in which we fill the requested formats, but not
    /// the same if we fill them with similar-but-not-identical values.
    /// This is a map from:
    /// format-request to
    ///     card mapping to
    ///         number of times we used that card mapping
    assigned_formats_so_far: BTreeMap<AdjacentTupleSizes, BTreeMap<MatchingCards, usize>>,
}

impl FormatMatchState {
    fn empty(format: &[AdjacentTupleSizes]) -> (BTreeMap<AdjacentTupleSizes, usize>, Self) {
        let mut required_formats = BTreeMap::new();
        for f in format {
            *required_formats.entry(f.clone()).or_default() += 1;
        }

        (
            required_formats,
            FormatMatchState {
                assigned_formats_so_far: BTreeMap::new(),
            },
        )
    }

    fn with_additional_formats(
        &self,
        fmt: impl IntoIterator<Item = Vec<(OrderedCard, usize)>>,
    ) -> Self {
        let mut s = self.clone();
        for fmt in fmt {
            let adj_tup = fmt.iter().map(|(_, v)| *v).collect();
            *s.assigned_formats_so_far
                .entry(adj_tup)
                .or_default()
                .entry(fmt)
                .or_default() += 1;
        }
        s
    }

    fn contained_cards(&self) -> BTreeMap<OrderedCard, usize> {
        let mut cards = BTreeMap::new();

        for v in self.assigned_formats_so_far.values() {
            for (vv, ct) in v {
                for (card, count) in vv {
                    *cards.entry(*card).or_default() += *count * ct;
                }
            }
        }

        cards
    }

    fn into_matching_cards(
        mut self,
        format_seq: &[AdjacentTupleSizes],
    ) -> Result<Vec<MatchingCards>, Error> {
        let mut output = Vec::with_capacity(format_seq.len());

        for adj_req in format_seq {
            let options = self
                .assigned_formats_so_far
                .get_mut(adj_req)
                .ok_or_else(|| anyhow::anyhow!("Missing requested format bucket"))?;

            let v = options
                .keys()
                .next_back()
                .ok_or_else(|| anyhow::anyhow!("No remaining matchings for format"))?
                .clone();

            let ct = options.get_mut(&v).unwrap();
            if *ct == 1 {
                options.remove(&v);
            } else {
                *ct -= 1;
            }

            output.push(v);
        }

        Ok(output)
    }
}

enum QueueItem {
    State(FormatMatchState),
    Enqueue(Peekable<Box<dyn Iterator<Item = FormatMatchState>>>),
}

struct FormatMatchIterator {
    format: BTreeMap<AdjacentTupleSizes, usize>,
    format_seq: Vec<AdjacentTupleSizes>,
    cards: BTreeMap<OrderedCard, usize>,
    visited: HashSet<FormatMatchState>,
    queue: VecDeque<QueueItem>,
}

impl FormatMatchIterator {
    /// Compute the remaining un-matched requests.
    ///
    /// Returns the number of remaining matches needed for the request, along
    /// with the request.
    fn remaining<'a, 'b: 'a>(
        &'a self,
        state: &'b FormatMatchState,
    ) -> impl Iterator<Item = (&'a AdjacentTupleSizes, usize)> + 'a {
        self.format.iter().flat_map(|(shape, count)| {
            // we need `count` values
            let filled = state
                .assigned_formats_so_far
                .get(shape)
                .map(|c| c.values().sum::<usize>())
                .unwrap_or_default();
            if filled < *count {
                Some((shape, *count - filled))
            } else {
                None
            }
        })
    }

    fn find_possible_tractors(
        &self,
        r: &AdjacentTupleSizes,
        c_ct: &impl Fn(OrderedCard) -> usize,
    ) -> Vec<Vec<(OrderedCard, usize)>> {
        assert!(r.len() > 1);
        let mut to_append = vec![];
        for card in self
            .cards
            .iter()
            .rev()
            .filter(|(_, ct)| **ct >= r[0])
            .map(|(c, _)| *c)
        {
            // Do a small DFS to find the sequential adjacent tuples
            let mut rr = r.clone();
            rr.reverse();
            let mut stk = vec![(card, rr, vec![])];

            while let Some((next_card, mut remaining_tuples, mut seq_so_far)) = stk.pop() {
                let ct = self.cards.get(&next_card).copied().unwrap_or_default() - c_ct(next_card);
                let next = remaining_tuples.pop().unwrap();
                if ct < next {
                    continue;
                }
                seq_so_far.push((next_card, next));
                if remaining_tuples.is_empty() {
                    to_append.push(seq_so_far);
                } else {
                    // We may need to examine multiple potential
                    // successors in the case of the trump number
                    // outside the trump suit -- e.g. if the trump
                    // number is 2, there are three potential 2x2
                    // tractors starting at A.
                    for s in next_card.successor() {
                        stk.push((s, remaining_tuples.clone(), seq_so_far.clone()));
                    }
                }
            }
        }
        to_append
    }
}

impl Iterator for FormatMatchIterator {
    type Item = Vec<MatchingCards>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(qi) = self.queue.pop_front() {
            match qi {
                QueueItem::State(n) => {
                    if self.visited.contains(&n) {
                        continue;
                    }
                    self.visited.insert(n.clone());

                    let contained_cards = n.contained_cards();
                    let c_ct = |c| contained_cards.get(&c).copied().unwrap_or_default();

                    let remaining = self.remaining(&n).collect::<Vec<_>>();
                    if remaining.is_empty() {
                        let cards = n.into_matching_cards(&self.format_seq).unwrap();
                        return Some(cards);
                    }
                    let expansion = remaining
                        .into_iter()
                        .map(|(r, copies)| {
                            // If it's not complete, for each incomplete format, try filling it
                            // in for each possible variant -- we'll end up filling variants in
                            // all possible orders.

                            if r.is_empty() {
                                // Not supposed to happen
                                unreachable!("Got an empty tuple request!");
                            }

                            // Special case tractor handling, because tractors can overlap (so using combinations is not ideal)
                            if r.len() > 1 {
                                let possible_tractors = self.find_possible_tractors(r, &c_ct);
                                (
                                    possible_tractors
                                        .into_iter()
                                        .map(|t| (t, 1))
                                        .collect::<Vec<_>>(),
                                    1,
                                )
                            } else {
                                // For tuples, they're guaranteed not to overlap, so we can use
                                // combinations helpers to speed things up by simultaneously filling in
                                // more than one at a time.
                                (
                                    self.cards
                                        .iter()
                                        .rev()
                                        .map(|(c, ct)| (vec![(*c, r[0])], (ct - c_ct(*c)) / r[0]))
                                        .filter(|(_, ct)| *ct > 0)
                                        .collect(),
                                    copies,
                                )
                            }
                        })
                        .collect::<Vec<_>>();
                    // We should consider expansion if and only if all remaining
                    // components are satisfiable, since only fully satisfied records are produced.
                    //
                    // expansion contains one element per `remaining`, where each element is
                    // (
                    //   [
                    //     (option1, multiplicity1),
                    //     (option2, multiplicity2),
                    //     ...
                    //   ],
                    //   required_count
                    // )

                    if expansion
                        .iter()
                        .all(|(e, c)| e.iter().map(|(_, ct)| ct).sum::<usize>() >= *c)
                    {
                        // If there are multiple requested copies of the potential
                        // expansion, try to move multiple along at once.
                        for (e, ct) in expansion {
                            let n_ = n.clone();
                            let e_ = e.clone();
                            let iter = crate::multiset_iter::multiset_k_combination_iter(
                                (0..e.len()).collect::<Vec<_>>(),
                                move |idx| e[*idx].1,
                                ct,
                            )
                            .map(move |combination| {
                                let assignments: Vec<_> = combination
                                    .into_iter()
                                    .flat_map(|(idx, multiplicity)| {
                                        (0..multiplicity)
                                            .map(|_| e_[idx].0.clone())
                                            .collect::<Vec<_>>()
                                    })
                                    .collect();
                                n_.with_additional_formats(assignments)
                            });

                            let iter: Box<dyn Iterator<Item = FormatMatchState>> = Box::new(iter);
                            self.queue.push_back(QueueItem::Enqueue(iter.peekable()));
                        }
                    }
                }
                QueueItem::Enqueue(mut iter) => {
                    if let Some(next) = iter.next() {
                        if iter.peek().is_some() {
                            self.queue.push_front(QueueItem::Enqueue(iter));
                        }
                        if !self.visited.contains(&next) {
                            self.queue.push_front(QueueItem::State(next));
                        }
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::ordered_card::OrderedCard;
    use crate::types::{cards::*, Card, Number, Suit, Trump};

    use super::find_format_matches;

    const TRUMP: Trump = Trump::Standard {
        number: Number::Four,
        suit: Suit::Spades,
    };
    macro_rules! oc {
        ($card:expr) => {
            OrderedCard {
                card: $card,
                trump: TRUMP,
            }
        };
    }

    #[test]
    fn test_tuple_format_match() {
        let counts: BTreeMap<_, _> = vec![
            (oc!(S_2), 2),
            (oc!(S_3), 3),
            (oc!(S_5), 3),
            (oc!(Card::BigJoker), 1),
        ]
        .into_iter()
        .collect();

        let v = find_format_matches(vec![vec![1]], counts.clone()).collect::<Vec<_>>();

        assert_eq!(
            v,
            vec![
                vec![vec![(oc!(Card::BigJoker), 1)]],
                vec![vec![(oc!(S_5), 1)]],
                vec![vec![(oc!(S_3), 1)]],
                vec![vec![(oc!(S_2), 1)]],
            ]
        );

        let v = find_format_matches(vec![vec![2]], counts).collect::<Vec<_>>();

        assert_eq!(
            v,
            vec![
                vec![vec![(oc!(S_5), 2)]],
                vec![vec![(oc!(S_3), 2)]],
                vec![vec![(oc!(S_2), 2)]],
            ]
        );
    }

    #[test]
    fn test_multiple_tuple_format_match() {
        let counts: BTreeMap<_, _> = vec![
            (oc!(S_2), 2),
            (oc!(S_3), 3),
            (oc!(S_5), 3),
            (oc!(Card::BigJoker), 1),
        ]
        .into_iter()
        .collect();

        let v = find_format_matches(vec![vec![1], vec![1]], counts.clone()).collect::<Vec<_>>();
        assert_eq!(
            v[0],
            vec![vec![(oc!(Card::BigJoker), 1)], vec![(oc!(S_5), 1)]]
        );
        // There are 9 unique choices of two cards:
        // HJ, 5
        // HJ, 3
        // HJ, 2
        // 5, 5
        // 5, 3
        // 5, 2
        // 3, 3
        // 3, 2
        // 2, 2
        assert_eq!(v.len(), 9);

        let v = find_format_matches(vec![vec![2], vec![2]], counts).collect::<Vec<_>>();

        // There are 3 unique choices of two pairsA
        // 55, 33
        // 55, 22
        // 33, 22
        assert_eq!(v[0], vec![vec![(oc!(S_5), 2)], vec![(oc!(S_3), 2)]]);
        assert_eq!(v.len(), 3);
    }

    #[test]
    fn test_tractor_format_match() {
        let counts = vec![
            (oc!(S_2), 2),
            (oc!(S_3), 3),
            (oc!(S_5), 3),
            (oc!(Card::BigJoker), 1),
        ]
        .into_iter()
        .collect();

        let v = find_format_matches(vec![vec![2, 2]], counts).collect::<Vec<_>>();

        assert_eq!(
            v,
            vec![
                vec![vec![(oc!(S_3), 2), (oc!(S_5), 2)]],
                vec![vec![(oc!(S_2), 2), (oc!(S_3), 2)]],
            ]
        );
    }

    #[test]
    fn test_multiple_tractor_format_match() {
        let counts = vec![
            (oc!(S_2), 4),
            (oc!(S_3), 4),
            (oc!(S_5), 4),
            (oc!(S_6), 4),
            (oc!(Card::BigJoker), 1),
        ]
        .into_iter()
        .collect();

        let v = find_format_matches(vec![vec![2, 2], vec![2, 2]], counts).collect::<Vec<_>>();

        assert_eq!(
            v[0],
            vec![
                vec![(oc!(S_5), 2), (oc!(S_6), 2)],
                vec![(oc!(S_5), 2), (oc!(S_6), 2)]
            ]
        );
        // 5566, 5566
        // 5566, 3355
        // 5566, 2233
        // 3355, 3355
        // 3355, 2233
        // 2233, 2233
        assert_eq!(v.len(), 6);
    }

    #[test]
    fn test_very_large_tractor_throw() {
        let counts = vec![
            (oc!(S_2), 10),
            (oc!(S_3), 10),
            (oc!(S_5), 10),
            (oc!(S_6), 10),
            (oc!(Card::BigJoker), 4),
        ]
        .into_iter()
        .collect();
        let fmt = vec![vec![4, 4], vec![3, 3], vec![1], vec![3]];

        let v = find_format_matches(fmt, counts).collect::<Vec<_>>();

        assert_eq!(
            v[0],
            vec![
                vec![(oc!(S_5), 4), (oc!(S_6), 4)],
                vec![(oc!(S_5), 3), (oc!(S_6), 3)],
                vec![(oc!(Card::BigJoker), 1)],
                vec![(oc!(Card::BigJoker), 3)],
            ]
        );
        assert_eq!(v.len(), 215);
    }

    #[test]
    fn test_really_long_silly_requirements() {
        let counts = vec![
            (oc!(S_9), 10),
            (oc!(S_10), 10),
            (oc!(S_J), 10),
            (oc!(S_Q), 10),
            (oc!(S_K), 10),
            (oc!(S_A), 10),
            (oc!(Card::SmallJoker), 4),
            (oc!(Card::BigJoker), 4),
        ]
        .into_iter()
        .collect();
        let fmt = (0..10).map(|_| vec![1]).collect();

        let v = find_format_matches(fmt, counts).collect::<Vec<_>>();

        assert_eq!(
            v[0],
            vec![
                vec![(oc!(Card::BigJoker), 1)],
                vec![(oc!(Card::BigJoker), 1)],
                vec![(oc!(Card::BigJoker), 1)],
                vec![(oc!(Card::BigJoker), 1)],
                vec![(oc!(Card::SmallJoker), 1)],
                vec![(oc!(Card::SmallJoker), 1)],
                vec![(oc!(Card::SmallJoker), 1)],
                vec![(oc!(Card::SmallJoker), 1)],
                vec![(oc!(S_A), 1)],
                vec![(oc!(S_A), 1)],
            ]
        );
        assert_eq!(v.len(), 17865);
    }
}
