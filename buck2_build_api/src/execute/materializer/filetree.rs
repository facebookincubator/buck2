/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

//! Data structure akin to a map, but where the key is a sequence.
//!
//! Restrictions:
//! - Storing both a key A and a key B which is a prefix of A is not possible.
//!
//! Special operations:
//! - Using key A to search for a value at a key B when B is a prefix of A.
//!
//! This is useful when artifacts are directories and we need to query the map
//! to figure out which artifact a path belongs to. E.g. we have an artifact at
//! "foo/bar", and we need to find out which artifact "foo/bar/c" belongs to.

use std::borrow::Borrow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::Hash;

use buck2_core::fs::paths::FileNameBuf;

pub type FileTree<V> = DataTree<FileNameBuf, V>;

/// Tree that stores data in the leaves. Think of the key as the path to the
/// leaf containing the value. The data/value is of type `V`, and each edge
/// is of type `K` (making the key to a value a sequence of `K`).
///
/// # Example
/// ```
/// use buck2_core::fs::paths::{FileNameBuf, ForwardRelativePathBuf};
/// use buck2_build_api::execute::materializer::filetree::DataTree;
///
/// let path = ForwardRelativePathBuf::unchecked_new("foo/bar".to_owned());
/// let contents = "contents_of_foobar".to_owned();
///
/// let mut file_path_to_contents: DataTree<FileNameBuf, String> = DataTree::new();
/// file_path_to_contents.insert(
///     path.iter().map(|f| f.to_owned()),
///     contents.clone(),
/// );
///
/// assert_eq!(file_path_to_contents.prefix_get_mut(&mut path.iter()).as_deref(), Some(&contents));
/// ```
#[derive(Debug)]
pub enum DataTree<K, V> {
    /// Stores data of type `V` with key of type `Iterator<Item = K>`.
    Tree(HashMap<K, DataTree<K, V>>),
    Data(V),
}

impl<K: Eq + Hash, V> DataTree<K, V> {
    pub fn new() -> Self {
        Self::Tree(HashMap::new())
    }

    /// Gets the value at `key` or one of its prefixes, and returns it.
    ///
    /// When a value is found and [`Some`] is returned, it's guaranteed that
    /// only enough of `key` to find the returned value was consumed.
    /// E.g. if `key` is (A, B, C, D) and there's a value present at (A, B),
    /// then after this method returns (C, D) can still be consumed from `key`.
    ///
    /// There are no guarantees on how much is consumed from `key` when
    /// [`None`] is returned.
    pub fn prefix_get<'a, I, Q>(&self, key: &mut I) -> Option<&V>
    where
        K: 'a + Borrow<Q>,
        Q: 'a + Hash + Eq + ?Sized,
        I: Iterator<Item = &'a Q>,
    {
        if let Self::Data(data) = self {
            // return early so we don't consume from key_iter unnecessarily
            return Some(data);
        }
        let mut node = self;
        for k in key {
            node = match node.children().unwrap().get(k) {
                None => return None,
                Some(node) => match node {
                    Self::Tree(_) => node,
                    Self::Data(data) => return Some(data),
                },
            };
        }
        None
    }

    /// Similar to `prefix_get`, but takes and returns `&mut`.
    pub fn prefix_get_mut<'a, I, Q>(&mut self, key: &mut I) -> Option<&mut V>
    where
        K: 'a + Borrow<Q>,
        Q: 'a + Hash + Eq + ?Sized,
        I: Iterator<Item = &'a Q>,
    {
        if let Self::Data(data) = self {
            // return early so we don't consume from key_iter unnecessarily
            return Some(data);
        }
        let mut node = self;
        for k in key {
            node = match node.children_mut().unwrap().get_mut(k) {
                None => return None,
                Some(node) => match node {
                    Self::Tree(_) => node,
                    Self::Data(data) => return Some(data),
                },
            };
        }
        None
    }

    /// Inserts a key-value pair into the tree.
    ///
    /// If there is already a key in the map that is a prefix of the inserted
    /// key, that key is removed.
    ///
    /// If the inserted key is a prefix of one or more keys in the map, all
    /// those keys are removed.
    pub fn insert<I: Iterator<Item = K>>(&mut self, mut key: I, value: V) {
        if let Some(k) = key.next() {
            if matches!(self, Self::Data(_)) {
                *self = Self::new();
            }
            let child = match self.children_mut().unwrap().entry(k) {
                Entry::Occupied(e) => e.into_mut(),
                Entry::Vacant(e) => e.insert(Self::new()),
            };
            child.insert(key, value);
        } else {
            *self = Self::Data(value);
        }
    }

    /// Removes a key from the tree, returning the value at the key if the key
    /// was previously in the tree.
    pub fn remove<'a, I, Q>(&mut self, mut key: I) -> Option<V>
    where
        K: 'a + Borrow<Q>,
        Q: 'a + Hash + Eq + ?Sized,
        I: Iterator<Item = &'a Q>,
    {
        if matches!(self, Self::Data(_)) {
            return match std::mem::replace(self, Self::new()) {
                Self::Data(data) => Some(data),
                _ => unreachable!(),
            };
        }
        if let Some(k) = key.next() {
            if let Some(node) = self.children_mut().unwrap().get_mut(k) {
                let data = node.remove(key);
                let remove_node = match node {
                    Self::Tree(children) => children.is_empty(),
                    Self::Data(_) => true,
                };
                if remove_node {
                    self.children_mut().unwrap().remove(k);
                }
                return data;
            }
        }
        None
    }

    fn children(&self) -> Option<&HashMap<K, DataTree<K, V>>> {
        match self {
            Self::Tree(children) => Some(children),
            Self::Data(_) => None,
        }
    }

    fn children_mut(&mut self) -> Option<&mut HashMap<K, DataTree<K, V>>> {
        match self {
            Self::Tree(children) => Some(children),
            Self::Data(_) => None,
        }
    }
}
