use crate::shortint::backward_compatibility::parameters::key_switching::ShortintKeySwitchingParametersVersions;
use crate::shortint::parameters::{
    DecompositionBaseLog, DecompositionLevelCount, EncryptionKeyChoice,
};
use serde::{Deserialize, Serialize};
use tfhe_versionable::Versionize;

/// A set of cryptographic parameters for homomorphic Shortint key switching.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Versionize)]
#[versionize(ShortintKeySwitchingParametersVersions)]
pub struct ShortintKeySwitchingParameters {
    pub ks_base_log: DecompositionBaseLog,
    pub ks_level: DecompositionLevelCount,
    pub destination_key: EncryptionKeyChoice,
}

impl ShortintKeySwitchingParameters {
    /// Constructs a new set of parameters for shortint key switching.
    ///
    /// # Warning
    ///
    /// Failing to fix the parameters properly would yield incorrect and insecure computation.
    /// Unless you are a cryptographer who really knows the impact of each of those parameters, you
    /// __must__ stick with the provided parameters (if any), which both offer correct results with
    /// 128 bits of security.
    pub fn new(
        ks_base_log: DecompositionBaseLog,
        ks_level: DecompositionLevelCount,
        destination_key: EncryptionKeyChoice,
    ) -> Self {
        Self {
            ks_base_log,
            ks_level,
            destination_key,
        }
    }
}

pub const PARAM_KEYSWITCH_1_1_KS_PBS_TO_2_2_KS_PBS: ShortintKeySwitchingParameters =
    ShortintKeySwitchingParameters {
        ks_level: DecompositionLevelCount(15),
        ks_base_log: DecompositionBaseLog(1),
        destination_key: EncryptionKeyChoice::Big,
    };

pub const PARAM_KEYSWITCH_MESSAGE_2_CARRY_2_KS_PBS_TUNIFORM_2M64: ShortintKeySwitchingParameters =
    ShortintKeySwitchingParameters {
        ks_level: DecompositionLevelCount(5),
        ks_base_log: DecompositionBaseLog(3),
        destination_key: EncryptionKeyChoice::Small,
    };
