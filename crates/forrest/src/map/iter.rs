// SPDX-FileCopyrightText: 2022 Profian Inc. <opensource@profian.com>
// SPDX-License-Identifier: Apache-2.0

use alloc::{vec, vec::Vec};
use core::iter::FusedIterator;

use digest::Digest;

use super::node::Node;

/// An iterator over map items
pub struct Iter<'a, D: Digest, K, V> {
    stack: Vec<(&'a Node<D, K, V>, usize)>,
    total: usize,
    index: usize,
}

impl<'a, D: Digest, K, V> Iter<'a, D, K, V> {
    pub(crate) fn new(root: &'a Node<D, K, V>, total: usize) -> Self {
        Self {
            stack: vec![(root, 0)],
            total,
            index: 0,
        }
    }
}

impl<'a, D: Digest, K, V> ExactSizeIterator for Iter<'a, D, K, V> {}
impl<'a, D: Digest, K, V> FusedIterator for Iter<'a, D, K, V> {}

impl<'a, D: Digest, K, V> Iterator for Iter<'a, D, K, V> {
    type Item = (&'a K, &'a V);

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.total - self.index;
        (size, Some(size))
    }

    fn next(&mut self) -> Option<Self::Item> {
        match self.stack.pop() {
            None => None,

            Some((node, idx @ 0 | idx @ 1)) => match node {
                Node::Leaf(leaf) => {
                    self.index += 1;
                    Some((&leaf.0, &leaf.1))
                }

                Node::Fork(fork) => {
                    if idx == 0 {
                        self.stack.push((node, 1));
                    }

                    if let Some(link) = fork[idx].as_deref() {
                        self.stack.push((&link.node, 0));
                    }

                    self.next()
                }
            },

            _ => unreachable!(),
        }
    }
}
