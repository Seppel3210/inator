/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! The powerset construction algorithm for constructing an equivalent DFA from an arbitrary NFA.
//! Also known as the subset construction algorithm.

use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};

use crate::{nfa::Graph as Nfa, Compiled as Dfa};

/// Type for transitions from _subsets_ of states to _subsets_ of states.
type SubsetStates<I> = BTreeMap<
    BTreeSet<usize>, // <-- Subset of NFA states (will become one DFA state)
    (
        BTreeMap<I, (BTreeSet<usize>, Option<&'static str>)>, // <-- Transitions
        bool,                                                 // <-- Accepting or not
    ),
>;

impl<I: Clone + Ord> Nfa<I> {
    /// Powerset construction algorithm mapping subsets of states to DFA nodes.
    #[inline]
    pub(crate) fn subsets(self) -> Dfa<I> {
        // Map which _subsets_ of states transition to which _subsets_ of states
        let mut subset_states = SubsetStates::new();
        let initial_state = traverse(
            &self,
            self.initial.iter().copied().collect(),
            &mut subset_states,
        );

        // Fix an ordering on subsets so each can be a DFA state
        let mut ordered: Vec<_> = subset_states.keys().collect();
        ordered.sort_unstable();

        // Check that binary_search works
        #[cfg(test)]
        {
            for (i, subset) in ordered.iter().enumerate() {
                assert_eq!(ordered.binary_search(subset), Ok(i));
            }
        }

        // Construct the vector of subset-mapped states
        let states = ordered
            .iter()
            .map(|&subset| {
                let &(ref states, accepting) = unwrap!(subset_states.get(subset));
                crate::dfa::State {
                    transitions: states
                        .iter()
                        .map(|(token, &(ref set, fn_name))| {
                            (
                                token.clone(),
                                (unwrap!(ordered.binary_search(&set)), fn_name),
                            )
                        })
                        .collect(),
                    accepting,
                }
            })
            .collect();

        // Wrap it in a DFA
        Dfa {
            states,
            initial: unwrap!(ordered.binary_search(&&initial_state)),
        }
    }
}

/// Map which _subsets_ of states transition to which _subsets_ of states.
/// Return the expansion of the original `queue` argument after taking all epsilon transitions.
#[inline]
fn traverse<I: Clone + Ord>(
    nfa: &Nfa<I>,
    queue: Vec<usize>,
    subset_states: &mut SubsetStates<I>,
) -> BTreeSet<usize> // <-- Return the set of states after taking epsilon transitions
{
    // Take all epsilon transitions immediately
    let post_epsilon = nfa.take_all_epsilon_transitions(queue);

    // Check if we've already seen this subset
    let tmp = match subset_states.entry(post_epsilon.clone()) {
        std::collections::btree_map::Entry::Occupied(_) => return post_epsilon,
        std::collections::btree_map::Entry::Vacant(empty) => empty,
    };

    // Get all _states_ from indices
    let subset = post_epsilon.iter().map(|&i| get!(nfa.states, i));

    // For now, so we can't get stuck in a cycle, cache an empty map
    let _ = tmp.insert((BTreeMap::new(), subset.clone().any(|state| state.accepting)));

    // Calculate the next superposition of states WITHOUT EPSILON TRANSITIONS YET
    let mut transitions = BTreeMap::<I, (BTreeSet<usize>, Option<&'static str>)>::new();
    for state in subset {
        for (token, &(ref map, fn_name)) in &state.non_epsilon {
            match transitions.entry(token.clone()) {
                Entry::Vacant(entry) => {
                    let _ = entry.insert((map.clone(), fn_name));
                }
                Entry::Occupied(entry) => {
                    let &mut (ref mut set, ref mut existing_fn_name) = entry.into_mut();
                    assert_eq!(fn_name, *existing_fn_name, "MESSAGE TODO");
                    set.extend(map.iter().copied());
                }
            }
        }
    }

    // Now, follow epsilon transitions AND recurse
    for &mut (ref mut dst, _) in transitions.values_mut() {
        *dst = traverse(nfa, dst.iter().copied().collect(), subset_states);
    }

    // Rewrite the empty map we wrote earlier with the actual transitions
    unwrap!(subset_states.get_mut(&post_epsilon)).0 = transitions;

    post_epsilon
}
