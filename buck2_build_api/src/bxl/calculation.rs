//! DICE calculations for bxl

use std::sync::Arc;

use async_trait::async_trait;
use buck2_common::result::SharedError;
use buck2_common::result::SharedResult;
use dice::DiceComputations;

use crate::bxl::result::BxlResult;
use crate::bxl::types::BxlKey;

#[derive(Debug, thiserror::Error)]
enum BxlCalculationError {
    #[error("BxlCalculationDyn is not configured (internal error)")]
    NoBxl,
}

#[async_trait]
pub trait BxlCalculationDyn: Send + Sync + 'static {
    async fn eval_bxl(&self, ctx: &DiceComputations, bxl: BxlKey) -> SharedResult<Arc<BxlResult>>;
}

/// Implementation which can be used when bxl crate is not available.
pub struct BxlCalculationNoBxl;

#[async_trait]
impl BxlCalculationDyn for BxlCalculationNoBxl {
    async fn eval_bxl(
        &self,
        _ctx: &DiceComputations,
        _bxl: BxlKey,
    ) -> SharedResult<Arc<BxlResult>> {
        Err(SharedError::new(BxlCalculationError::NoBxl))
    }
}

#[async_trait]
pub trait BxlCalculation {
    async fn eval_bxl<'a>(&self, bxl: BxlKey) -> SharedResult<Arc<BxlResult>>;
}

#[async_trait]
impl BxlCalculation for DiceComputations {
    async fn eval_bxl<'a>(&self, bxl: BxlKey) -> SharedResult<Arc<BxlResult>> {
        self.global_data()
            .get::<&'static dyn BxlCalculationDyn>()
            .map_err(SharedError::new)?
            .eval_bxl(self, bxl)
            .await
    }
}
