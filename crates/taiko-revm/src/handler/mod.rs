//!Handler related to Optimism chain
use crate::{api::exec::TaikoContextTr, transaction::TaikoTransactionError, TaikoHaltReason};
use revm::{
    context_interface::result::{EVMError, FromStringError},
    handler::{handler::EvmTrError, EvmTr, Frame, FrameResult, Handler, MainnetHandler},
    inspector::{Inspector, InspectorEvmTr, InspectorFrame, InspectorHandler},
    interpreter::{interpreter::EthInterpreter, FrameInput},
};

pub mod post_execution;
pub mod pre_execution;

pub struct TaikoHandler<EVM, ERROR, FRAME> {
    pub mainnet: MainnetHandler<EVM, ERROR, FRAME>,
    pub _phantom: core::marker::PhantomData<(EVM, ERROR, FRAME)>,
}

impl<EVM, ERROR, FRAME> TaikoHandler<EVM, ERROR, FRAME> {
    pub fn new() -> Self {
        Self {
            mainnet: MainnetHandler::default(),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<EVM, ERROR, FRAME> Default for TaikoHandler<EVM, ERROR, FRAME> {
    fn default() -> Self {
        Self::new()
    }
}

pub trait IsTxError {
    fn is_tx_error(&self) -> bool;
}

impl<DB, TX> IsTxError for EVMError<DB, TX> {
    fn is_tx_error(&self) -> bool {
        matches!(self, EVMError::Transaction(_))
    }
}

impl<EVM, ERROR, FRAME> Handler for TaikoHandler<EVM, ERROR, FRAME>
where
    EVM: EvmTr<Context: TaikoContextTr>,
    ERROR: EvmTrError<EVM> + From<TaikoTransactionError> + FromStringError + IsTxError,
    // TODO `FrameResult` should be a generic trait.
    // TODO `FrameInit` should be a generic.
    FRAME: Frame<Evm = EVM, Error = ERROR, FrameResult = FrameResult, FrameInit = FrameInput>,
{
    type Evm = EVM;
    type Error = ERROR;
    type Frame = FRAME;
    type HaltReason = TaikoHaltReason;

    /// Deducts maximum possible fee and transfer value from caller's balance.
    ///
    /// Unused fees are returned to caller after execution completes.
    #[inline]
    fn validate_against_state_and_deduct_caller(
        &self,
        evm: &mut Self::Evm,
    ) -> Result<(), Self::Error> {
        pre_execution::validate_against_state_and_deduct_caller(evm.ctx())
    }

    /// Returns unused gas costs to the transaction sender's account.
    #[inline]
    fn reimburse_caller(
        &self,
        evm: &mut Self::Evm,
        exec_result: &mut <Self::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
        post_execution::reimburse_caller(evm.ctx(), exec_result.gas_mut()).map_err(From::from)
    }

    /// Transfers transaction fees to the block beneficiary's account.
    #[inline]
    fn reward_beneficiary(
        &self,
        evm: &mut Self::Evm,
        exec_result: &mut <Self::Frame as Frame>::FrameResult,
    ) -> Result<(), Self::Error> {
        post_execution::reward_beneficiary(evm.ctx(), exec_result.gas_mut()).map_err(From::from)
    }

    // !!! VALIDATION
}

impl<EVM, ERROR, FRAME> InspectorHandler for TaikoHandler<EVM, ERROR, FRAME>
where
    EVM: InspectorEvmTr<
        Context: TaikoContextTr,
        Inspector: Inspector<<<Self as Handler>::Evm as EvmTr>::Context, EthInterpreter>,
    >,
    ERROR: EvmTrError<EVM> + From<TaikoTransactionError> + FromStringError + IsTxError,
    // TODO `FrameResult` should be a generic trait.
    // TODO `FrameInit` should be a generic.
    FRAME: InspectorFrame<
        Evm = EVM,
        Error = ERROR,
        FrameResult = FrameResult,
        FrameInit = FrameInput,
        IT = EthInterpreter,
    >,
{
    type IT = EthInterpreter;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{api::default_ctx::TaikoContext, DefaultTaiko, TaikoBuilder, TaikoSpecId};
    use alloy_primitives::uint;
    use revm::{
        context::{BlockEnv, Context, TransactionType},
        context_interface::result::InvalidTransaction,
        database::InMemoryDB,
        database_interface::EmptyDB,
        handler::EthFrame,
        interpreter::{CallOutcome, Gas, InstructionResult, InterpreterResult},
        primitives::{bytes, Address, Bytes, B256},
        state::AccountInfo,
    };
    use rstest::rstest;
    use std::boxed::Box;

    /// Creates frame result.
    fn call_last_frame_return(
        ctx: TaikoContext<EmptyDB>,
        instruction_result: InstructionResult,
        gas: Gas,
    ) -> Gas {
        let mut evm = ctx.build_taiko();

        let mut exec_result = FrameResult::Call(CallOutcome::new(
            InterpreterResult {
                result: instruction_result,
                output: Bytes::new(),
                gas,
            },
            0..0,
        ));

        let mut handler =
            TaikoHandler::<_, EVMError<_, TaikoTransactionError>, EthFrame<_, _, _>>::new();

        handler
            .last_frame_result(&mut evm, &mut exec_result)
            .unwrap();
        handler.refund(&mut evm, &mut exec_result, 0);
        *exec_result.gas()
    }

    #[test]
    fn test_revert_gas() {
        let ctx = Context::taiko()
            .modify_tx_chained(|tx| {
                tx.base.gas_limit = 100;
            })
            .modify_cfg_chained(|cfg| cfg.spec = TaikoSpecId::PACAYA);

        let gas = call_last_frame_return(ctx, InstructionResult::Revert, Gas::new(90));
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spent(), 10);
        assert_eq!(gas.refunded(), 0);
    }

    #[test]
    fn test_consume_gas() {
        let ctx = Context::taiko()
            .modify_tx_chained(|tx| {
                tx.base.gas_limit = 100;
            })
            .modify_cfg_chained(|cfg| cfg.spec = TaikoSpecId::PACAYA);

        let gas = call_last_frame_return(ctx, InstructionResult::Stop, Gas::new(90));
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spent(), 10);
        assert_eq!(gas.refunded(), 0);
    }

    #[test]
    fn test_consume_gas_with_refund() {
        let ctx = Context::taiko()
            .modify_tx_chained(|tx| {
                tx.base.gas_limit = 100;
            })
            .modify_cfg_chained(|cfg| cfg.spec = TaikoSpecId::PACAYA);

        let mut ret_gas = Gas::new(90);
        ret_gas.record_refund(20);

        let gas = call_last_frame_return(ctx.clone(), InstructionResult::Stop, ret_gas);
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spent(), 10);
        assert_eq!(gas.refunded(), 2); // min(20, 10/5)

        let gas = call_last_frame_return(ctx, InstructionResult::Revert, ret_gas);
        assert_eq!(gas.remaining(), 90);
        assert_eq!(gas.spent(), 10);
        assert_eq!(gas.refunded(), 0);
    }

    #[test]
    fn test_consume_gas_deposit_tx() {
        let ctx = Context::taiko()
            .modify_tx_chained(|tx| {
                tx.base.gas_limit = 100;
                tx.is_anchor = false;
            })
            .modify_cfg_chained(|cfg| cfg.spec = TaikoSpecId::PACAYA);
        let gas = call_last_frame_return(ctx, InstructionResult::Stop, Gas::new(90));
        assert_eq!(gas.remaining(), 0);
        assert_eq!(gas.spent(), 100);
        assert_eq!(gas.refunded(), 0);
    }
}
