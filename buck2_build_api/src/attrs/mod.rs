/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

//! This module provides support for implementing and consuming attr
//! types.
//!
//! Support for implementing new attribute types is in the `val` module.
//!
//! Attribute values are present in 4 different states:
//!   1. initial starlark value - the value passed to the rule function
//!   2. coerced value - the value captured after processing the build file. At this point, the type has
//!      been checked (i.e. for an attr.list(attr.string()), we've confirmed that (1) was an iterable of strings).
//!      This is done when invoking a rule function, so it has no access to configuration or information from
//!      other build files.
//!   3. configured value - this is roughly (2) with a specific configuration attached to all configurable
//!      values. For example a dep or a source (where the source is an output) are configurable (they both are basically
//!      targets).
//!   4. resolved value - this is the "resolved" attribute once again as a starlark value and as provided to a rule
//!      implementation function.
//!
//! Attribute coercion happens immediately on declaring a build target (via
//! invoking a rule function). It will validate the types of the
//! attribute values and do some simple conversions (ex. parse strings to
//! target labels).
//!
//! Attribute configuration happens when the configuration for a target
//! becomes available. There are two primary operations that happen
//! here: select resolution and target configuration.
//!
//! Attribute resolution happens just before invoking the rule
//! implementation. At this point, the context has access to all the
//! providers that are needed (based on inspection of the configured value)
//! and anything that requires a provider can be resolved to its final
//! value.
//!
//! Generally an attribute type is specified as its "resolved" type that the
//! rule implementation requests. For example, a rule that wants a list of files
//! for its sources would specify `attr.list(attr.file())` and would
//! receive a list of files in the implementation. The intermediate form of that
//! may be strings or targets or some other thing (e.g. a lazy glob, maybe).

use buck2_node::attrs::attr_type::attr_literal::AttrLiteral;
use buck2_node::attrs::coerced_attr::CoercedAttr;
use starlark_map::small_map;

pub(crate) mod analysis;
pub mod attr_type;
pub mod coerced_attr;
pub mod configured_attr;

#[cfg(test)]
pub(crate) mod testing;
#[cfg(test)]
mod tests;

/// A simple map that maintains insertion order.
pub type OrderedMap<K, V> = small_map::SmallMap<K, V>;
pub type OrderedMapEntry<'a, K, V> = small_map::Entry<'a, K, V>;
pub type OrderedMapOccupiedEntry<'a, K, V> = small_map::OccupiedEntry<'a, K, V>;
pub type OrderedMapVacantEntry<'a, K, V> = small_map::VacantEntry<'a, K, V>;
