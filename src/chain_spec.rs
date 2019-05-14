use primitives::{ed25519, sr25519, Pair};
use substrate_poa_runtime::{
	AccountId, GenesisConfig, ConsensusConfig, TimestampConfig, BalancesConfig,
	SudoConfig, IndicesConfig, ValidatorSetConfig, SessionConfig,
};
use substrate_service;

use ed25519::Public as AuthorityId;

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = substrate_service::ChainSpec<GenesisConfig>;

/// The chain specification option. This is expected to come in from the CLI and
/// is little more than one of a number of alternatives which can easily be converted
/// from a string (`--chain=...`) into a `ChainSpec`.
#[derive(Clone, Debug)]
pub enum Alternative {
	/// Whatever the current runtime is, with just Alice as an auth.
	Development,
	/// Whatever the current runtime is, with simple Alice/Bob auths.
	LocalTestnet,
}

fn authority_key(s: &str) -> AuthorityId {
	ed25519::Pair::from_string(&format!("//{}", s), None)
		.expect("static values are valid; qed")
		.public()
}

fn account_key(s: &str) -> AccountId {
	sr25519::Pair::from_string(&format!("//{}", s), None)
		.expect("static values are valid; qed")
		.public()
}

impl Alternative {
	/// Get an actual chain config from one of the alternatives.
	pub(crate) fn load(self) -> Result<ChainSpec, String> {
		Ok(match self {
			Alternative::Development => ChainSpec::from_genesis(
				"Development",
				"dev",
				|| testnet_genesis(vec![
					authority_key("Alice")
				], vec![
					account_key("Alice")
				],
					account_key("Alice")
				),
				vec![],
				None,
				None,
				None,
				None
			),
			Alternative::LocalTestnet => ChainSpec::from_genesis(
				"Local Testnet",
				"local_testnet",
				|| testnet_genesis(vec![
					authority_key("Alice"),
					authority_key("Bob"),
				], vec![
					account_key("Alice"),
					account_key("Bob"),
					account_key("Charlie"),
					account_key("Dave"),
					account_key("Eve"),
					account_key("Ferdie"),
				],
					account_key("Alice"),
				),
				vec![],
				None,
				None,
				None,
				None
			),
		})
	}

	pub(crate) fn from(s: &str) -> Option<Self> {
		match s {
			"dev" => Some(Alternative::Development),
			"" | "local" => Some(Alternative::LocalTestnet),
			_ => None,
		}
	}
}

fn testnet_genesis(_initial_authorities: Vec<AuthorityId>, _endowed_accounts: Vec<AccountId>, root_key: AccountId) -> GenesisConfig {
	// As configured in Substrate node
	// https://github.com/paritytech/substrate/blob/master/node/cli/src/chain_spec.rs
	const SECS_PER_BLOCK: u64 = 6;
	const MINUTES: u64 = 60 / SECS_PER_BLOCK;
	
	// Defining authorities again and not using the ones passed in the initial_authorities parameter.
	// This is to easily reuse them across genesis configs of several modules (see below).
	// Each item is a tuple of controller key (sr25519) and session key (ed25519).
	let authorities = vec![(account_key("Alice"), authority_key("Alice")), (account_key("Bob"), authority_key("Bob"))];

	GenesisConfig {
		consensus: Some(ConsensusConfig {
			code: include_bytes!("../runtime/wasm/target/wasm32-unknown-unknown/release/substrate_poa_runtime_wasm.compact.wasm").to_vec(),
			authorities: authorities.iter().map(|x| x.1.clone()).collect() // session keys from authorities vec declared above
		}),
		system: None,
		timestamp: Some(TimestampConfig {
			minimum_period: SECS_PER_BLOCK / 2, // due to the nature of aura the slots are 2*period
		}),
		indices: Some(IndicesConfig {
			ids: authorities.iter().map(|x| x.0.clone()).collect(), // controller keys from authorities vec declared above
		}),
		session: Some(SessionConfig {
			validators: authorities.iter().map(|x| x.0.clone()).collect(), // controller keys from authorities vec declared above
			session_length: 5 * MINUTES,
			keys: authorities.clone() // authorities vec declared above
		}),
		balances: Some(BalancesConfig {
			transaction_base_fee: 1,
			transaction_byte_fee: 0,
			existential_deposit: 500,
			transfer_fee: 0,
			creation_fee: 0,
			balances: authorities.iter().map(|x| (x.0.clone(), 1 << 60)).collect(), // controller keys from authorities vec declared above
			vesting: vec![],
		}),
		sudo: Some(SudoConfig {
			key: root_key,
		}),
		validatorset: Some(ValidatorSetConfig {
			validators: authorities // authorities vec declared above
		}),
	}
}