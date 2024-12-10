use codec::Encode;
use frame_support::derive_impl;
use frame_support::parameter_types;
use frame_support::PalletId;
use sp_core::sr25519;
use sp_core::ConstU128;
use sp_core::Pair;
use sp_core::Public;
use sp_runtime::traits::Convert;
use sp_runtime::traits::IdentifyAccount;
use sp_runtime::traits::IdentityLookup;
use sp_runtime::traits::Verify;
use sp_runtime::BuildStorage;
use sp_runtime::MultiSignature;

use crate as pallet_voting;
use crate::types::Vote;

type Block = frame_system::mocking::MockBlock<Test>;

pub const UNIT: u128 = 1000000000000;

/// Alias to 512-bit hash when used in the context of a transaction signature on
/// the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it
/// equivalent to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// An index to a block.
pub type BlockNumber = u64;

/// Balance of an account.
pub type Balance = u128;

/// should be random, but we leave it const for simplicity
const SALT: u32 = 10u32;

parameter_types! {
    pub const EntryFee: Balance = 30_000 * UNIT;
    pub const MaxProposals: u32 = 10u32;
    pub const RevealLength: BlockNumber = 50u64;
    pub const MinLength: BlockNumber = 100u64;
    pub const MaxTokens: u8 = 100u8;
    pub const VotingPalletId: PalletId = PalletId(*b"p/v8t1ng");
    pub const BasicDeposit: Balance = 0;
    pub const FieldDeposit: Balance = 0;
    pub const SubAccountDeposit: Balance = 0;
    pub const MaxSubAccounts: u32 = 100;
    pub const MaxAdditionalFields: u32 = 1;
    pub const MaxRegistrars: u32 = 20;
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        VotingModule: pallet_voting,
        Balances: pallet_balances,
        Identity: pallet_identity,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type AccountData = pallet_balances::AccountData<Balance>;
}

impl pallet_balances::Config for Test {
    type MaxLocks = ();
    type Balance = Balance;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<500>;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
    type RuntimeEvent = RuntimeEvent;
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = RuntimeFreezeReason;
    type ReserveIdentifier = [u8; 8];
    type FreezeIdentifier = [u8; 8];
    type MaxReserves = ();
    type MaxFreezes = ();
}

impl pallet_identity::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type Slashed = ();
    type ForceOrigin = frame_system::EnsureRoot<AccountId>;
    type WeightInfo = ();
    type BasicDeposit = BasicDeposit;
    type ByteDeposit = FieldDeposit;
    type SubAccountDeposit = SubAccountDeposit;
    type MaxSubAccounts = MaxSubAccounts;
    type IdentityInformation = pallet_identity::legacy::IdentityInfo<MaxAdditionalFields>;
    type MaxRegistrars = MaxRegistrars;
    type RegistrarOrigin = frame_system::EnsureRoot<AccountId>;
    type OffchainSignature = Signature;
    type SigningPublicKey = <Signature as Verify>::Signer;
    type UsernameAuthorityOrigin = frame_system::EnsureRoot<AccountId>;
    type PendingUsernameExpiration = MinLength;
    type MaxSuffixLength = MaxTokens;
    type MaxUsernameLength = MaxTokens;
}

pub struct VotingIdentityProvider;
impl pallet_voting::IdentityProvider<AccountId> for VotingIdentityProvider {
    fn check_existence(account: &AccountId) -> bool {
        Identity::identity(account.clone()).is_some()
    }
}

impl pallet_voting::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type IdentityProvider = VotingIdentityProvider;
    type Currency = Balances;
    type BasicDeposit = EntryFee;
    type MaxProposals = MaxProposals;
    type Public = <Signature as sp_runtime::traits::Verify>::Signer;
    type Signature = MultiSignature;
    type RevealLength = RevealLength;
    type MinLength = MinLength;
    type MaxVotingTokens = MaxTokens;
}

pub fn get_charlie() -> AccountId { get_account_id_from_seed::<sr25519::Public>("Charlie") }
pub fn get_alice() -> AccountId { get_account_id_from_seed::<sr25519::Public>("Alice") }
pub fn get_bob() -> AccountId { get_account_id_from_seed::<sr25519::Public>("Bob") }

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into();
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (get_charlie(), 20_000 * UNIT),
            (get_alice(), 1_000_000 * UNIT),
            (get_bob(), 1_000_000 * UNIT),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    t.into()
}

pub fn generate(account: &str, vote: Vote) -> (sp_core::sr25519::Signature, u32) {
    let pair: sp_core::sr25519::Pair = Pair::from_string(account, None).unwrap();
    let payload = (vote, SALT).encode();
    let payload = payload.as_slice().to_owned();
    let signed = pair.sign(&payload);
    (signed, SALT)
}

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}
