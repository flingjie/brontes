use std::{pin::Pin, sync::Arc};

use brontes_database_libmdbx::{implementation::tx::LibmdbxTx, Libmdbx};
use brontes_types::structured_trace::TxTrace;
pub use brontes_types::traits::TracingProvider;
use futures::Future;
use reth_db::mdbx::RO;
use reth_interfaces::provider::ProviderResult;
use reth_primitives::{Address, BlockNumberOrTag, Header, B256};
use tokio::{sync::mpsc::UnboundedSender, task::JoinError};

use self::parser::TraceParser;
use crate::{
    executor::{Executor, TaskKind},
    init_trace,
};

#[cfg(feature = "dyn-decode")]
mod dyn_decode;

pub mod parser;
mod utils;
use brontes_metrics::{trace::types::TraceMetricEvent, PoirotMetricEvents};
#[allow(dead_code)]
pub(crate) const UNKNOWN: &str = "unknown";
#[allow(dead_code)]
pub(crate) const RECEIVE: &str = "receive";
#[allow(dead_code)]
pub(crate) const FALLBACK: &str = "fallback";
use reth_primitives::BlockId;

// #[async_trait::async_trait]
// impl TracingProvider for Provider<Http<Client>> {
//     async fn eth_call(
//         &self,
//         request: CallRequest,
//         block_number: Option<BlockId>,
//         state_overrides: Option<StateOverride>,
//         block_overrides: Option<Box<BlockOverrides>>,
//     ) -> ProviderResult<Bytes> {
//         todo!()
//     }
//
//     async fn block_hash_for_id(&self, _block_num: u64) ->
// ProviderResult<Option<B256>> {         todo!()
//     }
//
//     #[cfg(not(feature = "local"))]
//     fn best_block_number(&self) -> ProviderResult<u64> {
//         todo!()
//     }
//
//     #[cfg(feature = "local")]
//     async fn best_block_number(&self) -> ProviderResult<u64> {
//         todo!()
//     }
//
//     async fn replay_block_transactions(
//         &self,
//         _block_id: BlockId,
//     ) -> EthResult<Option<Vec<TxTrace>>> {
//         todo!()
//     }
//
//     async fn block_receipts(
//         &self,
//         _number: BlockNumberOrTag,
//     ) -> ProviderResult<Option<Vec<TransactionReceipt>>> {
//         todo!()
//     }
//
//     async fn header_by_number(&self, _number: BlockNumber) ->
// ProviderResult<Option<Header>> {         todo!()
//     }
// }

pub type ParserFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Option<(Vec<TxTrace>, Header)>, JoinError>> + Send + 'a>>;

pub struct Parser<'a, T: TracingProvider> {
    executor: Executor,
    parser:   TraceParser<'a, T>,
}

impl<'a, T: TracingProvider> Parser<'a, T> {
    pub fn new(
        metrics_tx: UnboundedSender<PoirotMetricEvents>,
        libmdbx: &'a Libmdbx,
        tracing: T,
        should_fetch: Box<dyn Fn(&Address, &LibmdbxTx<RO>) -> bool + Send + Sync>,
    ) -> Self {
        let executor = Executor::new();

        let parser =
            TraceParser::new(libmdbx, should_fetch, Arc::new(tracing), Arc::new(metrics_tx));

        Self { executor, parser }
    }

    #[cfg(feature = "local")]
    pub async fn get_latest_block_number(&self) -> ProviderResult<u64> {
        self.parser.tracer.best_block_number().await
    }

    pub fn get_tracer(&self) -> Arc<T> {
        self.parser.get_tracer()
    }

    #[cfg(not(feature = "local"))]
    pub fn get_latest_block_number(&self) -> ProviderResult<u64> {
        self.parser.tracer.best_block_number()
    }

    pub async fn get_block_hash_for_number(&self, block_num: u64) -> ProviderResult<Option<B256>> {
        self.parser.tracer.block_hash_for_id(block_num.into()).await
    }

    /// executes the tracing of a given block
    pub fn execute(&self, block_num: u64) -> ParserFuture {
        // Safety: This is safe as the Arc ensures immutability.
        // This will satisfy its lifetime scope do to the lifetime itself living longer
        // than the process that runs brontes.
        let parser: &'static TraceParser<'static, T> = unsafe { std::mem::transmute(&self.parser) };

        Box::pin(
            self.executor
                .spawn_result_task_as(parser.execute_block(block_num), TaskKind::Default),
        ) as ParserFuture
    }
}
