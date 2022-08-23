/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use buck2_build_api::query::aquery::evaluator::get_aquery_evaluator;
use buck2_common::dice::cells::HasCellResolver;
use buck2_events::dispatch::span_async;
use buck2_query::query::syntax::simple::eval::values::QueryEvaluationResult;
use buck2_server_ctx::command_end::command_end;
use buck2_server_ctx::ctx::ServerCommandContextTrait;
use buck2_server_ctx::pattern::target_platform_from_client_context;
use cli_proto::AqueryRequest;
use cli_proto::AqueryResponse;

use crate::query::printer::QueryResultPrinter;
use crate::query::printer::ShouldPrintProviders;

pub async fn aquery_command(
    ctx: Box<dyn ServerCommandContextTrait>,
    req: cli_proto::AqueryRequest,
) -> anyhow::Result<cli_proto::AqueryResponse> {
    let metadata = ctx.request_metadata()?;
    let start_event = buck2_data::CommandStart {
        metadata: metadata.clone(),
        data: Some(buck2_data::AqueryCommandStart {}.into()),
    };
    span_async(start_event, async {
        let result = aquery(ctx, req).await;
        let end_event = command_end(metadata, &result, buck2_data::AqueryCommandEnd {});
        (result, end_event)
    })
    .await
}

async fn aquery(
    mut server_ctx: Box<dyn ServerCommandContextTrait>,
    request: AqueryRequest,
) -> anyhow::Result<AqueryResponse> {
    let ctx = server_ctx.dice_ctx().await?;
    let cell_resolver = ctx.get_cell_resolver().await?;

    let output_configuration = QueryResultPrinter::from_request_options(
        &cell_resolver,
        &request.output_attributes,
        request.unstable_output_format,
    )?;

    let AqueryRequest {
        query,
        query_args,
        context,
        ..
    } = request;

    let global_target_platform = target_platform_from_client_context(
        context.as_ref(),
        &cell_resolver,
        server_ctx.working_dir(),
    )
    .await?;

    let evaluator = get_aquery_evaluator(
        &ctx,
        server_ctx.working_dir(),
        server_ctx.project_root().clone(),
        global_target_platform,
    )
    .await?;

    let query_result = evaluator.eval_query(&query, &query_args).await?;

    let mut stdout = server_ctx.stdout()?;

    let result = match query_result {
        QueryEvaluationResult::Single(targets) => {
            output_configuration
                .print_single_output(&mut stdout, targets, false, ShouldPrintProviders::No)
                .await
        }
        QueryEvaluationResult::Multiple(results) => {
            output_configuration
                .print_multi_output(&mut stdout, results, false, ShouldPrintProviders::No)
                .await
        }
    };
    let error_messages = match result {
        Ok(_) => vec![],
        Err(e) => vec![format!("{:#}", e)],
    };
    Ok(AqueryResponse { error_messages })
}
