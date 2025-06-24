use crate::{evm::TaikoEvm, transaction::TaikoTxTr, L1BlockInfo, TaikoSpecId};
use revm::{
    context::{Cfg, JournalOutput},
    context_interface::{Block, JournalTr},
    handler::instructions::EthInstructions,
    interpreter::interpreter::EthInterpreter,
    Context, Database,
};

/// Trait that allows for optimism OpEvm to be built.
pub trait TaikoBuilder: Sized {
    /// Type of the context.
    type Context;

    /// Build the op.
    fn build_taiko(
        self,
    ) -> TaikoEvm<Self::Context, (), EthInstructions<EthInterpreter, Self::Context>>;

    /// Build the op with an inspector.
    fn build_taiko_with_inspector<INSP>(
        self,
        inspector: INSP,
    ) -> TaikoEvm<Self::Context, INSP, EthInstructions<EthInterpreter, Self::Context>>;
}

impl<BLOCK, TX, CFG, DB, JOURNAL> TaikoBuilder for Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>
where
    BLOCK: Block,
    TX: TaikoTxTr,
    CFG: Cfg<Spec = TaikoSpecId>,
    DB: Database,
    JOURNAL: JournalTr<Database = DB, FinalOutput = JournalOutput>,
{
    type Context = Self;

    fn build_taiko(
        self,
    ) -> TaikoEvm<Self::Context, (), EthInstructions<EthInterpreter, Self::Context>> {
        TaikoEvm::new(self, ())
    }

    fn build_taiko_with_inspector<INSP>(
        self,
        inspector: INSP,
    ) -> TaikoEvm<Self::Context, INSP, EthInstructions<EthInterpreter, Self::Context>> {
        TaikoEvm::new(self, inspector)
    }
}
