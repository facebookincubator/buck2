/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

//! Integrations of `buck2_error::Error` with `anyhow::Error` and `StdError`.

use std::error::request_value;
use std::error::Error as StdError;
use std::fmt;
use std::sync::Arc;

use mappable_rc::Marc;
use ref_cast::RefCast;

use crate::error::ErrorKind;
use crate::root::ErrorRoot;

// This implementation is fairly magic and is what allows us to bypass the issue with conflicting
// implementations between `anyhow::Error` and `T: StdError`. The `T: Into<anyhow::Error>` bound is
// what we actually make use of in the implementation, while the other bound is needed to make sure
// this impl does not accidentally cover too many types. Importantly, this impl does not conflict
// with `T: From<T>`
impl<T: fmt::Debug + fmt::Display + Sync + Send + 'static> From<T> for crate::Error
where
    T: Into<anyhow::Error>,
    Result<(), T>: anyhow::Context<(), T>,
{
    #[track_caller]
    fn from(value: T) -> crate::Error {
        let source_location =
            crate::source_location::from_file(std::panic::Location::caller().file(), None);
        // `Self` may be an `anyhow::Error` or any `StdError`. We'll check by downcasting
        let mut e = Some(value);
        let r: &mut dyn std::any::Any = &mut e;
        if let Some(e) = r.downcast_mut::<Option<anyhow::Error>>() {
            return recover_crate_error(Marc::new(e.take().unwrap()), source_location);
        }

        // Otherwise, we'll use the strategy for `StdError`
        let anyhow = e.unwrap().into();
        recover_crate_error(Marc::new(anyhow), source_location)
    }
}

fn maybe_add_context_from_metadata(mut e: crate::Error, context: &dyn StdError) -> crate::Error {
    if let Some(metadata) = request_value::<ProvidableContextMetadata>(context) {
        if let Some(category) = metadata.category {
            e = e.context(category);
        }
        if !metadata.tags.is_empty() {
            e = e.tag(metadata.tags.iter().copied());
        }
        e
    } else {
        e
    }
}

pub(crate) fn recover_crate_error(
    value: Marc<anyhow::Error>,
    source_location: Option<String>,
) -> crate::Error {
    // Instead of just turning this into an error root, we will first check if this error has any
    // information associated with it that would allow us to recover more structure.
    let mut context_stack = Vec::new();
    let mut cur: Marc<dyn StdError + 'static> = Marc::map(value.clone(), AsRef::as_ref);
    // We allow this to appear more than once in the context chain, however we always use the
    // bottom-most value
    let mut source_location = source_location;
    let base = 'base: loop {
        // Handle the `cur` error
        if let Some(base) = cur.downcast_ref::<CrateAsStdError>() {
            break base.0.clone();
        }

        let context_metadata = if let Some(metadata) =
            request_value::<ProvidableContextMetadata>(&*cur)
            && (metadata.check_error_type)(&*cur).is_some()
        {
            source_location = crate::source_location::from_file(
                metadata.source_file,
                metadata.source_location_extra,
            );
            Some(metadata)
        } else {
            None
        };

        if let Some(metadata) = request_value::<ProvidableRootMetadata>(&*cur)
            && (metadata.check_error_type)(&*cur).is_some()
        {
            // FIXME(JakobDegen): `Marc` needs `try_map` here too
            let cur = Marc::map(cur, |e| (metadata.check_error_type)(e).unwrap());
            let e = crate::Error(Arc::new(ErrorKind::Root(ErrorRoot::new(
                cur.clone(),
                metadata.typ,
                source_location,
                metadata.action_error,
            ))));
            break 'base maybe_add_context_from_metadata(e, cur.as_ref());
        }

        context_stack.push((cur.clone(), context_metadata));

        // Compute the next element in the source chain
        if let Ok(new_cur) = Marc::try_map(cur.clone(), |e| e.source()) {
            cur = new_cur;
            continue;
        }

        // The error was not created directly from a `buck2_error::Error` or with a
        // `ProvidableRootMetadata`. However, if may have only `ProvidableContextMetadata`, so
        // check for that possibility
        while let Some((e, context_metadata)) = context_stack.pop() {
            let Some(context_metadata) = context_metadata else {
                continue;
            };
            // The `unwrap` is ok because we checked this condition when initially constructing the `context_metadata`
            let e = Marc::map(e, |e| (context_metadata.check_error_type)(e).unwrap());
            let val = crate::Error(Arc::new(ErrorKind::Root(ErrorRoot::new(
                e.clone(),
                None,
                source_location,
                None,
            ))));
            break 'base maybe_add_context_from_metadata(val, e.as_ref());
        }
        // This error was not created with any useful metadata on it, so there's nothing smart we can do
        return crate::Error(Arc::new(ErrorKind::Root(ErrorRoot::new_anyhow(
            value,
            source_location,
        ))));
    };
    // We were able to convert the error into a `buck2_error::Error` in some non-trivial way. We'll
    // now need to add back any context that is not included in the `base` buck2_error yet.
    let mut e = base;
    for (context_value, _) in context_stack.into_iter().rev() {
        // First, just add the value directly. This value is only used for formatting
        e = e.context(&context_value);
        // Now add any additional information from the metadata, if it's available
        e = maybe_add_context_from_metadata(e, context_value.as_ref());
    }
    e
}

impl From<crate::Error> for anyhow::Error {
    fn from(value: crate::Error) -> Self {
        Into::into(CrateAsStdError(value))
    }
}

#[derive(derive_more::Display, RefCast)]
#[repr(transparent)]
pub(crate) struct CrateAsStdError(pub(crate) crate::Error);

impl fmt::Debug for CrateAsStdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl StdError for CrateAsStdError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &*self.0.0 {
            ErrorKind::Root(r) => r.source(),
            ErrorKind::WithContext(_, r) | ErrorKind::Emitted(_, r) => {
                Some(CrateAsStdError::ref_cast(r))
            }
        }
    }
}

pub type CheckErrorType =
    for<'a> fn(&'a (dyn StdError + 'static)) -> Option<&'a (dyn StdError + Send + Sync + 'static)>;

/// This can be `provide`d by an error to inject buck2-specific information about it.
///
/// Currently intended for macro use only, might make sense to allow it more generally in the
/// future.
#[derive(Clone)]
pub struct ProvidableRootMetadata {
    pub typ: Option<crate::ErrorType>,
    /// Some errors will transitively call `Provide` for their sources. That means that even when a
    /// `request_value` call returns `Some`, the value might actually be provided by something
    /// further down the source chain. We work around this issue by calling this function to confirm
    /// that the value was indeed provided by the element of the source chain we're currently
    /// inspecting.
    ///
    /// We also reuse this to get a `Send + Sync` reference to our error, since `source()` does not
    /// give us that.
    pub check_error_type: CheckErrorType,

    /// The protobuf ActionError, if the root was an action error
    pub action_error: Option<buck2_data::ActionError>,
}

/// Like `ProvidableRootMetadata`, but for "context-like" metadata that can appear on the error more
/// than once.
#[derive(Clone)]
pub struct ProvidableContextMetadata {
    /// Technically this should be in the `ProvidableRootMetadata`. However, we allow it to appear
    /// multiple times in the context and just pick the last one. There's no benefit to being picky.
    pub source_file: &'static str,
    /// Extra information to add to the end of the source location - typically a type/variant name,
    /// and the same thing as gets passed to `buck2_error::source_location::from_file`.
    pub source_location_extra: Option<&'static str>,
    pub category: Option<crate::Category>,
    pub tags: Vec<crate::ErrorTag>,
    /// See `ProvidableRootMetadata`
    pub check_error_type: CheckErrorType,
}

impl ProvidableRootMetadata {
    pub const fn gen_check_error_type<E: StdError + Send + Sync + 'static>() -> CheckErrorType {
        |e| e.downcast_ref::<E>().map(|e| e as _)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Request;

    use super::*;
    use crate as buck2_error;
    use crate::error::ErrorKind;

    #[derive(Debug, derive_more::Display)]
    struct TestError;

    impl StdError for TestError {}

    fn check_equal(mut a: &crate::Error, mut b: &crate::Error) {
        loop {
            match (&*a.0, &*b.0) {
                (ErrorKind::Root(a), ErrorKind::Root(b)) => {
                    // Avoid comparing vtable pointers
                    assert!(a.test_equal(b));
                    return;
                }
                (
                    ErrorKind::WithContext(a_context, a_inner),
                    ErrorKind::WithContext(b_context, b_inner),
                ) => {
                    a_context.assert_eq(b_context);
                    a = a_inner;
                    b = b_inner;
                }
                (ErrorKind::Emitted(_, a_inner), ErrorKind::Emitted(_, b_inner)) => {
                    a = a_inner;
                    b = b_inner;
                }
                (_, _) => {
                    panic!("Left side did not match right: {:?} {:?}", a, b)
                }
            }
        }
    }

    #[test]
    fn test_rountrip_no_context() {
        let e = crate::Error::new(TestError).context("context 1");
        let e2 = crate::Error::from(anyhow::Error::from(e.clone()));
        check_equal(&e, &e2);
    }

    #[test]
    fn test_rountrip_with_context() {
        let e = crate::Error::new(TestError).context("context 1");
        let e2 = crate::Error::from(anyhow::Error::from(e.clone()).context("context 2"));
        let e3 = e.context("context 2");
        check_equal(&e2, &e3);
    }

    #[derive(Debug, derive_more::Display)]
    struct FullMetadataError;

    impl StdError for FullMetadataError {
        fn provide<'a>(&'a self, request: &mut Request<'a>) {
            request
                .provide_value(ProvidableRootMetadata {
                    typ: Some(crate::ErrorType::Watchman),
                    check_error_type: ProvidableRootMetadata::gen_check_error_type::<Self>(),
                    action_error: None,
                })
                .provide_value(ProvidableContextMetadata {
                    source_file: file!(),
                    source_location_extra: Some("FullMetadataError"),
                    tags: vec![
                        crate::ErrorTag::WatchmanTimeout,
                        crate::ErrorTag::StarlarkFail,
                        crate::ErrorTag::WatchmanTimeout,
                    ],
                    category: Some(crate::Category::User),
                    check_error_type: ProvidableRootMetadata::gen_check_error_type::<Self>(),
                });
        }
    }

    #[test]
    fn test_metadata() {
        for e in [
            FullMetadataError.into(),
            crate::Error::new(FullMetadataError),
        ] {
            assert_eq!(e.get_category(), Some(crate::Category::User));
            assert_eq!(e.get_error_type(), Some(crate::ErrorType::Watchman));
            assert_eq!(
                e.source_location(),
                Some("buck2_error/src/any.rs::FullMetadataError")
            );
            assert_eq!(
                &e.get_tags(),
                &[
                    crate::ErrorTag::StarlarkFail,
                    crate::ErrorTag::WatchmanTimeout
                ]
            );
        }
    }

    #[test]
    fn test_metadata_through_anyhow() {
        let e: anyhow::Error = FullMetadataError.into();
        let e = e.context("anyhow");
        let e: crate::Error = e.into();
        assert_eq!(e.get_category(), Some(crate::Category::User));
        assert!(format!("{:?}", e).contains("anyhow"));
    }

    #[derive(Debug, thiserror::Error)]
    #[error("wrapper")]
    struct WrapperError(#[source] FullMetadataError);

    #[test]
    fn test_metadata_through_wrapper() {
        let e: crate::Error = WrapperError(FullMetadataError).into();
        assert_eq!(e.get_category(), Some(crate::Category::User));
        assert!(format!("{:?}", e).contains("wrapper"));
    }

    #[derive(Debug, buck2_error_derive::Error)]
    #[buck2(infra)]
    #[error("wrapper2")]
    struct FullMetadataContextWrapperError(#[source] FullMetadataError);

    #[test]
    fn test_context_in_wrapper() {
        let e: crate::Error = FullMetadataContextWrapperError(FullMetadataError).into();
        assert_eq!(e.get_category(), Some(crate::Category::Infra));
        assert_eq!(e.get_error_type(), Some(crate::ErrorType::Watchman));
        assert_eq!(
            e.source_location(),
            Some("buck2_error/src/any.rs::FullMetadataError")
        );
        assert!(format!("{:?}", e).contains("wrapper2"));
    }

    #[derive(Debug, buck2_error_derive::Error)]
    #[buck2(user)]
    #[error("unused")]
    struct UserMetadataError;

    #[derive(Debug, buck2_error_derive::Error)]
    #[buck2(infra)]
    #[error("unused")]
    struct InfraMetadataWrapperError(#[source] UserMetadataError);

    #[test]
    fn test_no_root_metadata_context() {
        let e = InfraMetadataWrapperError(UserMetadataError);
        let e: crate::Error = e.into();
        assert_eq!(e.get_category(), Some(crate::Category::Infra));
    }
}
