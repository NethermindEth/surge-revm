use crate::{L1BlockInfo, TaikoSpecId, TaikoTransaction};
use revm::{
    context::{BlockEnv, CfgEnv, TxEnv},
    database_interface::EmptyDB,
    Context, Journal, MainContext,
};

/// Type alias for the default context type of the TaikoEvm.
pub type TaikoContext<DB> =
    Context<BlockEnv, TaikoTransaction<TxEnv>, CfgEnv<TaikoSpecId>, DB, Journal<DB>, L1BlockInfo>;

/// Trait that allows for a default context to be created.
pub trait DefaultTaiko {
    /// Create a default context.
    fn taiko() -> TaikoContext<EmptyDB>;
}

impl DefaultTaiko for TaikoContext<EmptyDB> {
    fn taiko() -> Self {
        Context::mainnet()
            .with_tx(TaikoTransaction::default())
            .with_cfg(CfgEnv::new_with_spec(TaikoSpecId::PACAYA))
            .with_chain(L1BlockInfo::default())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::api::builder::TaikoBuilder;
    use revm::{
        inspector::{InspectEvm, NoOpInspector},
        ExecuteEvm,
    };

    #[test]
    fn default_run_op() {
        let ctx = Context::taiko();
        // convert to optimism context
        let mut evm = ctx.build_taiko_with_inspector(NoOpInspector {});
        // execute
        let _ = evm.replay();
        // inspect
        let _ = evm.inspect_replay();
    }
}
