use crate::TaikoSpecId;
use once_cell::race::OnceBox;
use revm::{
    context::Cfg,
    context_interface::ContextTr,
    handler::{EthPrecompiles, PrecompileProvider},
    interpreter::{InputsImpl, InterpreterResult},
    precompile::Precompiles,
    primitives::{hardfork::SpecId, Address},
};
use std::boxed::Box;
use std::string::String;

// Optimism precompile provider
#[derive(Debug, Clone)]
pub struct TaikoPrecompiles {
    /// Inner precompile provider is same as Ethereums.
    inner: EthPrecompiles,
    spec: TaikoSpecId,
}

impl TaikoPrecompiles {
    /// Create a new precompile provider with the given OpSpec.
    #[inline]
    pub fn new_with_spec(spec: TaikoSpecId) -> Self {
        let precompiles = default_taiko_precompiles();

        Self {
            inner: EthPrecompiles {
                precompiles,
                spec: SpecId::default(),
            },
            spec,
        }
    }

    // Precompiles getter.
    #[inline]
    pub fn precompiles(&self) -> &'static Precompiles {
        self.inner.precompiles
    }
}

/// Returns default taiko precompiles
pub fn default_taiko_precompiles() -> &'static Precompiles {
    static INSTANCE: OnceBox<Precompiles> = OnceBox::new();
    INSTANCE.get_or_init(|| {
        let precompiles = Precompiles::berlin().clone();

        Box::new(precompiles)
    })
}

impl<CTX> PrecompileProvider<CTX> for TaikoPrecompiles
where
    CTX: ContextTr<Cfg: Cfg<Spec = TaikoSpecId>>,
{
    type Output = InterpreterResult;

    #[inline]
    fn set_spec(&mut self, spec: <CTX::Cfg as Cfg>::Spec) -> bool {
        if spec == self.spec {
            return false;
        }
        *self = Self::new_with_spec(spec);
        true
    }

    #[inline]
    fn run(
        &mut self,
        context: &mut CTX,
        address: &Address,
        inputs: &InputsImpl,
        is_static: bool,
        gas_limit: u64,
    ) -> Result<Option<Self::Output>, String> {
        self.inner
            .run(context, address, inputs, is_static, gas_limit)
    }

    #[inline]
    fn warm_addresses(&self) -> Box<impl Iterator<Item = Address>> {
        self.inner.warm_addresses()
    }

    #[inline]
    fn contains(&self, address: &Address) -> bool {
        self.inner.contains(address)
    }
}

impl Default for TaikoPrecompiles {
    fn default() -> Self {
        Self::new_with_spec(TaikoSpecId::PACAYA)
    }
}
