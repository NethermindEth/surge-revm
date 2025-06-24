use alloy_primitives::U256;
use revm::{
    context::{Block, Cfg, ContextTr, JournalTr},
    interpreter::Gas,
    Database,
};

use crate::{api::exec::TaikoContextTr, transaction::TaikoTxTr, TaikoSpecId};

/// Reimburse the caller for the gas used in the transaction if tx is not anchor.
pub fn reimburse_caller<CTX: TaikoContextTr>(
    context: &mut CTX,
    gas: &mut Gas,
) -> Result<(), <CTX::Db as Database>::Error> {
    if context.tx().is_anchor() {
        // Anchor transactions do not reimburse gas.
        return Ok(());
    }

    revm::handler::post_execution::reimburse_caller(context, gas)
}

#[inline]
pub fn reward_beneficiary<CTX: TaikoContextTr>(
    context: &mut CTX,
    gas: &mut Gas,
) -> Result<(), <CTX::Db as Database>::Error> {
    if context.tx().is_anchor() {
        // Anchor transactions do not reward the beneficiary.
        return Ok(());
    }

    revm::handler::post_execution::reward_beneficiary(context, gas)?;

    if context.cfg().spec() == TaikoSpecId::ONTAKE {
        reward_beneficiary_ontake(context, gas)
    } else {
        reward_beneficiary_hekla(context, gas)
    }
}

fn reward_beneficiary_hekla<CTX: TaikoContextTr>(
    context: &mut CTX,
    gas: &Gas,
) -> Result<(), <CTX::Db as Database>::Error> {
    let treasury = context.tx().treasury();
    let basefee = context.block().basefee();

    let mut treasury_account = context.journal().load_account(treasury)?;
    treasury_account.mark_touch();
    treasury_account.info.balance = treasury_account
        .info
        .balance
        .saturating_add(U256::from(basefee) * U256::from(gas.spent() - gas.refunded() as u64));
    Ok(())
}

/*
https://github.com/taikoxyz/taiko-geth/blob/60551be44eb3080be9d0ba0c6cf01c6e2a47caf5/core/state_transition.go#L475-L482
Ontake upgrade for basefee sharing:
    totalFee := new(big.Int).Mul(st.evm.Context.BaseFee, new(big.Int).SetUint64(st.gasUsed()))
    feeCoinbase := new(big.Int).Div(
        new(big.Int).Mul(totalFee, new(big.Int).SetUint64(uint64(st.msg.BasefeeSharingPctg))),
        new(big.Int).SetUint64(100),
    )
    feeTreasury := new(big.Int).Sub(totalFee, feeCoinbase)
    st.state.AddBalance(st.getTreasuryAddress(), uint256.MustFromBig(feeTreasury))
    st.state.AddBalance(st.evm.Context.Coinbase, uint256.MustFromBig(feeCoinbase))
*/
fn reward_beneficiary_ontake<CTX: TaikoContextTr>(
    context: &mut CTX,
    gas: &Gas,
) -> Result<(), <CTX::Db as Database>::Error> {
    let basefee_ratio = context.tx().basefee_ratio();
    let treasury = context.tx().treasury();
    let basefee = context.block().basefee();

    let mut treasury_account = context.journal().load_account(treasury)?;
    treasury_account.mark_touch();
    let total_fee = U256::from(basefee) * U256::from(gas.spent() - gas.refunded() as u64);
    let fee_coinbase = total_fee * U256::from(basefee_ratio) / U256::from(100);
    let fee_treasury = total_fee - fee_coinbase;
    treasury_account.info.balance = treasury_account.info.balance.saturating_add(fee_treasury);

    let beneficiary = context.block().beneficiary();
    let mut coinbase_account = context.journal().load_account(beneficiary)?;
    coinbase_account.mark_touch();
    coinbase_account.info.balance = coinbase_account.info.balance.saturating_add(fee_coinbase);
    Ok(())
}
