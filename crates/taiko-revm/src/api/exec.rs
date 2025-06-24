use crate::{
    evm::TaikoEvm, handler::TaikoHandler, transaction::TaikoTxTr, L1BlockInfo, TaikoHaltReason,
    TaikoSpecId, TaikoTransactionError,
};
use revm::{
    context::{ContextSetters, JournalOutput},
    context_interface::{
        result::{EVMError, ExecutionResult, ResultAndState},
        Cfg, ContextTr, Database, JournalTr,
    },
    handler::{
        instructions::EthInstructions, system_call::SystemCallEvm, EthFrame, EvmTr, Handler,
        PrecompileProvider, SystemCallTx,
    },
    inspector::{InspectCommitEvm, InspectEvm, Inspector, InspectorHandler, JournalExt},
    interpreter::{interpreter::EthInterpreter, InterpreterResult},
    primitives::{Address, Bytes},
    DatabaseCommit, ExecuteCommitEvm, ExecuteEvm,
};

// Type alias for Taiko context
pub trait TaikoContextTr:
    ContextTr<
    Journal: JournalTr<FinalOutput = JournalOutput>,
    Tx: TaikoTxTr,
    Cfg: Cfg<Spec = TaikoSpecId>,
    Chain = L1BlockInfo,
>
{
}

impl<T> TaikoContextTr for T where
    T: ContextTr<
        Journal: JournalTr<FinalOutput = JournalOutput>,
        Tx: TaikoTxTr,
        Cfg: Cfg<Spec = TaikoSpecId>,
        Chain = L1BlockInfo,
    >
{
}

/// Type alias for the error type of the OpEvm.
type OpError<CTX> = EVMError<<<CTX as ContextTr>::Db as Database>::Error, TaikoTransactionError>;

impl<CTX, INSP, PRECOMPILE> ExecuteEvm
    for TaikoEvm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, PRECOMPILE>
where
    CTX: TaikoContextTr + ContextSetters,
    PRECOMPILE: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    type Output = Result<ResultAndState<TaikoHaltReason>, OpError<CTX>>;

    type Tx = <CTX as ContextTr>::Tx;

    type Block = <CTX as ContextTr>::Block;

    fn set_tx(&mut self, tx: Self::Tx) {
        self.0.ctx.set_tx(tx);
    }

    fn set_block(&mut self, block: Self::Block) {
        self.0.ctx.set_block(block);
    }

    fn replay(&mut self) -> Self::Output {
        let mut h = TaikoHandler::<_, _, EthFrame<_, _, _>>::new();
        h.run(self)
    }
}

impl<CTX, INSP, PRECOMPILE> ExecuteCommitEvm
    for TaikoEvm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, PRECOMPILE>
where
    CTX: TaikoContextTr<Db: DatabaseCommit> + ContextSetters,
    PRECOMPILE: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    type CommitOutput = Result<ExecutionResult<TaikoHaltReason>, OpError<CTX>>;

    fn replay_commit(&mut self) -> Self::CommitOutput {
        self.replay().map(|r| {
            self.ctx().db().commit(r.state);
            r.result
        })
    }
}

impl<CTX, INSP, PRECOMPILE> InspectEvm
    for TaikoEvm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, PRECOMPILE>
where
    CTX: TaikoContextTr<Journal: JournalExt> + ContextSetters,
    INSP: Inspector<CTX, EthInterpreter>,
    PRECOMPILE: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    type Inspector = INSP;

    fn set_inspector(&mut self, inspector: Self::Inspector) {
        self.0.inspector = inspector;
    }

    fn inspect_replay(&mut self) -> Self::Output {
        let mut h = TaikoHandler::<_, _, EthFrame<_, _, _>>::new();
        h.inspect_run(self)
    }
}

impl<CTX, INSP, PRECOMPILE> InspectCommitEvm
    for TaikoEvm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, PRECOMPILE>
where
    CTX: TaikoContextTr<Journal: JournalExt, Db: DatabaseCommit> + ContextSetters,
    INSP: Inspector<CTX, EthInterpreter>,
    PRECOMPILE: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    fn inspect_replay_commit(&mut self) -> Self::CommitOutput {
        self.inspect_replay().map(|r| {
            self.ctx().db().commit(r.state);
            r.result
        })
    }
}

impl<CTX, INSP, PRECOMPILE> SystemCallEvm
    for TaikoEvm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, PRECOMPILE>
where
    CTX: TaikoContextTr<Tx: SystemCallTx> + ContextSetters,
    PRECOMPILE: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    fn transact_system_call(
        &mut self,
        system_contract_address: Address,
        data: Bytes,
    ) -> Self::Output {
        self.set_tx(CTX::Tx::new_system_tx(data, system_contract_address));
        let mut h = TaikoHandler::<_, _, EthFrame<_, _, _>>::new();
        h.run_system_call(self)
    }
}
