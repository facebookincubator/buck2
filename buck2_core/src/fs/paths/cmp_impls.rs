/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

//!
//! General macros useful for path declaration
//!

use std::cmp;

///
/// Generates ['cmp::PartialEq'] and ['cmp::PartialOrd'] for the `lhs` and `rhs`
/// types, where `ty` is the unowned, reference path type.
macro_rules! impl_cmp {
    ($lhs:ty, $rhs:ty, $ty:ty) => {
        impl cmp::PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                <$ty as cmp::PartialEq>::eq(self, other)
            }
        }

        impl cmp::PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                <$ty as cmp::PartialEq>::eq(self, other)
            }
        }

        impl cmp::PartialOrd<$rhs> for $lhs {
            #[inline]
            fn partial_cmp(&self, other: &$rhs) -> Option<cmp::Ordering> {
                <$ty as cmp::PartialOrd>::partial_cmp(self, other)
            }
        }

        impl cmp::PartialOrd<$lhs> for $rhs {
            #[inline]
            fn partial_cmp(&self, other: &$lhs) -> Option<cmp::Ordering> {
                <$ty as cmp::PartialOrd>::partial_cmp(self, other)
            }
        }
    };
}

///
/// Generates ['cmp::PartialEq'] and ['cmp::PartialOrd'] for the `lhs` and `rhs`
/// string types, where `ty` is the unowned, reference path type.
macro_rules! impl_cmp_str {
    ($lhs:ty, $rhs:ty, $ty:ty) => {
        impl cmp::PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                match <$ty>::new(other) {
                    Ok(other) => <$ty as cmp::PartialEq>::eq(self, other),
                    _ => false,
                }
            }
        }

        impl cmp::PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                match <$ty>::new(self) {
                    Ok(this) => <$ty as cmp::PartialEq>::eq(this, other),
                    _ => false,
                }
            }
        }

        impl cmp::PartialOrd<$rhs> for $lhs {
            #[inline]
            fn partial_cmp(&self, other: &$rhs) -> Option<cmp::Ordering> {
                match <$ty>::new(other) {
                    Ok(other) => <$ty as cmp::PartialOrd>::partial_cmp(self, other),
                    _ => None,
                }
            }
        }

        impl cmp::PartialOrd<$lhs> for $rhs {
            #[inline]
            fn partial_cmp(&self, other: &$lhs) -> Option<cmp::Ordering> {
                match <$ty>::new(self) {
                    Ok(this) => <$ty as cmp::PartialOrd>::partial_cmp(this, other),
                    _ => None,
                }
            }
        }
    };
}

use crate::fs::paths::{ForwardRelativePath, ForwardRelativePathBuf};

impl_cmp!(
    ForwardRelativePathBuf,
    ForwardRelativePath,
    ForwardRelativePath
);
impl_cmp!(
    ForwardRelativePathBuf,
    &'_ ForwardRelativePath,
    ForwardRelativePath
);

impl_cmp_str!(ForwardRelativePathBuf, str, ForwardRelativePath);
impl_cmp_str!(ForwardRelativePathBuf, &'_ str, ForwardRelativePath);
impl_cmp_str!(ForwardRelativePathBuf, String, ForwardRelativePath);
impl_cmp_str!(ForwardRelativePath, str, ForwardRelativePath);
impl_cmp_str!(ForwardRelativePath, &'_ str, ForwardRelativePath);
impl_cmp_str!(ForwardRelativePath, String, ForwardRelativePath);
impl_cmp_str!(&'_ ForwardRelativePath, str, ForwardRelativePath);
impl_cmp_str!(&'_ ForwardRelativePath, String, ForwardRelativePath);

use crate::fs::paths::{AbsPath, AbsPathBuf};

impl_cmp!(AbsPathBuf, AbsPath, AbsPath);
impl_cmp!(AbsPathBuf, &'_ AbsPath, AbsPath);

impl_cmp_str!(AbsPathBuf, str, AbsPath);
impl_cmp_str!(AbsPathBuf, &'_ str, AbsPath);
impl_cmp_str!(AbsPathBuf, String, AbsPath);
impl_cmp_str!(AbsPath, str, AbsPath);
impl_cmp_str!(AbsPath, &'_ str, AbsPath);
impl_cmp_str!(AbsPath, String, AbsPath);
impl_cmp_str!(&'_ AbsPath, str, AbsPath);
impl_cmp_str!(&'_ AbsPath, String, AbsPath);

use crate::package::{PackageRelativePath, PackageRelativePathBuf};

impl_cmp!(
    PackageRelativePathBuf,
    PackageRelativePath,
    PackageRelativePath
);
impl_cmp!(
    PackageRelativePathBuf,
    &'_ PackageRelativePath,
    PackageRelativePath
);

impl_cmp_str!(PackageRelativePathBuf, str, PackageRelativePath);
impl_cmp_str!(PackageRelativePathBuf, &'_ str, PackageRelativePath);
impl_cmp_str!(PackageRelativePathBuf, String, PackageRelativePath);
impl_cmp_str!(PackageRelativePath, str, PackageRelativePath);
impl_cmp_str!(PackageRelativePath, &'_ str, PackageRelativePath);
impl_cmp_str!(PackageRelativePath, String, PackageRelativePath);
impl_cmp_str!(&'_ PackageRelativePath, str, PackageRelativePath);
impl_cmp_str!(&'_ PackageRelativePath, String, PackageRelativePath);

use crate::fs::project::{ProjectRelativePath, ProjectRelativePathBuf};

impl_cmp!(
    ProjectRelativePathBuf,
    ProjectRelativePath,
    ProjectRelativePath
);
impl_cmp!(
    ProjectRelativePathBuf,
    &'_ ProjectRelativePath,
    ProjectRelativePath
);

impl_cmp_str!(ProjectRelativePathBuf, str, ProjectRelativePath);
impl_cmp_str!(ProjectRelativePathBuf, &'_ str, ProjectRelativePath);
impl_cmp_str!(ProjectRelativePathBuf, String, ProjectRelativePath);
impl_cmp_str!(ProjectRelativePath, str, ProjectRelativePath);
impl_cmp_str!(ProjectRelativePath, &'_ str, ProjectRelativePath);
impl_cmp_str!(ProjectRelativePath, String, ProjectRelativePath);
impl_cmp_str!(&'_ ProjectRelativePath, str, ProjectRelativePath);
impl_cmp_str!(&'_ ProjectRelativePath, String, ProjectRelativePath);
