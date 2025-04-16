use std::sync::LazyLock;

pub mod trusted_setup_points;

cfg_if::cfg_if! {
    if #[cfg(feature = "c-kzg")] {

        static ETHEREUM_KZG_SETTINGS: LazyLock<c_kzg::KzgSettings> = LazyLock::new(|| {
            c_kzg::KzgSettings::load_trusted_setup(
                &trusted_setup_points::G1_POINTS.0,
                &trusted_setup_points::G2_POINTS.0
            ).expect("failed to load trusted setup")
        });


        pub use c_kzg::KzgSettings;
        /// KZG Settings that allow us to specify a custom trusted setup.
        /// or use hardcoded default settings.
        #[derive(Debug, Clone, Default, PartialEq, Eq )]
        pub enum EnvKzgSettings {
            /// Default mainnet trusted setup
            #[default]
            Default,
            /// Custom trusted setup.
            Custom(std::sync::Arc<c_kzg::KzgSettings>),
        }

        impl EnvKzgSettings {
            /// Return set KZG settings.
            ///
            /// In will initialize the default settings if it is not already loaded.
            pub fn get(&self) -> &c_kzg::KzgSettings {
                match self {
                    Self::Default => {
                        &ETHEREUM_KZG_SETTINGS
                    }
                    Self::Custom(settings) => settings,
                }
            }
        }

    } else if #[cfg(feature = "kzg-rs")] {
        pub use kzg_rs::{KzgSettings,EnvKzgSettings};
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "c-kzg")]
    fn test_output_ethereum_kzg_settings() {
        let settings = &*ETHEREUM_KZG_SETTINGS;
        println!("{:?}", settings);
    }
}
