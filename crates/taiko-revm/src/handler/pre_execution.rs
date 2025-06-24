use revm::{
    context::{result::InvalidTransaction, Block, Cfg, ContextTr, JournalTr, Transaction},
    handler::pre_execution::validate_account_nonce_and_code,
    Database,
};

use crate::{api::exec::TaikoContextTr, transaction::TaikoTxTr};

#[inline]
/// The same as mainnet`s `validate_against_state_and_deduct_caller`, but with
/// additional `charge_gas` parameter for taiko anchor transactions.
pub fn validate_against_state_and_deduct_caller<
    CTX: TaikoContextTr,
    ERROR: From<InvalidTransaction> + From<<CTX::Db as Database>::Error>,
>(
    context: &mut CTX,
) -> Result<(), ERROR> {
    let basefee = context.block().basefee() as u128;
    let blob_price = context.block().blob_gasprice().unwrap_or_default();
    let is_balance_check_disabled = context.cfg().is_balance_check_disabled();
    let is_eip3607_disabled = context.cfg().is_eip3607_disabled();
    let is_nonce_check_disabled = context.cfg().is_nonce_check_disabled();
    let charge_gas = !context.tx().is_anchor();

    let (tx, journal) = context.tx_journal();

    // Load caller's account.
    let caller_account = journal.load_account_code(tx.caller())?.data;

    validate_account_nonce_and_code(
        &mut caller_account.info,
        tx.nonce(),
        tx.kind().is_call(),
        is_eip3607_disabled,
        is_nonce_check_disabled,
    )?;

    let max_balance_spending = tx.max_balance_spending()?;

    // Check if account has enough balance for `gas_limit * max_fee`` and value transfer.
    // Transfer will be done inside `*_inner` functions.
    if is_balance_check_disabled {
        // Make sure the caller's balance is at least the value of the transaction.
        caller_account.info.balance = caller_account.info.balance.max(tx.value());
    } else if max_balance_spending > caller_account.info.balance {
        return Err(InvalidTransaction::LackOfFundForMaxFee {
            fee: Box::new(max_balance_spending),
            balance: Box::new(caller_account.info.balance),
        }
        .into());
    } else {
        let effective_balance_spending = tx
            .effective_balance_spending(basefee, blob_price)
            .expect("effective balance is always smaller than max balance so it can't overflow");

        // subtracting max balance spending with value that is going to be deducted later in the call.
        let gas_balance_spending = effective_balance_spending - tx.value();

        if charge_gas {
            caller_account.info.balance = caller_account
                .info
                .balance
                .saturating_sub(gas_balance_spending);
        }
    }

    // Touch account so we know it is changed.
    caller_account.mark_touch();
    Ok(())
}
