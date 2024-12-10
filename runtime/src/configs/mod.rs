// This is free and unencumbered software released into the public domain.
//
// Anyone is free to copy, modify, publish, use, compile, sell, or
// distribute this software, either in source code form or as a compiled
// binary, for any purpose, commercial or non-commercial, and by any
// means.
//
// In jurisdictions that recognize copyright laws, the author or authors
// of this software dedicate any and all copyright interest in the
// software to the public domain. We make this dedication for the benefit
// of the public at large and to the detriment of our heirs and
// successors. We intend this dedication to be an overt act of
// relinquishment in perpetuity of all present and future rights to this
// software under copyright law.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
// IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
// OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
// ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
// OTHER DEALINGS IN THE SOFTWARE.
//
// For more information, please refer to <http://unlicense.org>

// Substrate and Polkadot dependencies
use frame_support::derive_impl;
use frame_support::parameter_types;
use frame_support::traits::ConstBool;
use frame_support::traits::ConstU128;
use frame_support::traits::ConstU32;
use frame_support::traits::ConstU64;
use frame_support::traits::ConstU8;
use frame_support::traits::VariantCountOf;
use frame_support::weights::constants::RocksDbWeight;
use frame_support::weights::constants::WEIGHT_REF_TIME_PER_SECOND;
use frame_support::weights::IdentityFee;
use frame_support::weights::Weight;
use frame_system::limits::BlockLength;
use frame_system::limits::BlockWeights;
use pallet_transaction_payment::ConstFeeMultiplier;
use pallet_transaction_payment::FungibleAdapter;
use pallet_transaction_payment::Multiplier;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_runtime::traits::One;
use sp_runtime::Perbill;
use sp_version::RuntimeVersion;

use super::AccountId;
use super::Aura;
use super::Balance;
use super::Balances;
use super::Block;
use super::BlockNumber;
use super::Hash;
use super::Nonce;
use super::PalletInfo;
use super::Runtime;
use super::RuntimeCall;
use super::RuntimeEvent;
use super::RuntimeFreezeReason;
use super::RuntimeHoldReason;
use super::RuntimeOrigin;
use super::RuntimeTask;
use super::System;
use super::EXISTENTIAL_DEPOSIT;
use super::SLOT_DURATION;
use super::VERSION;
use crate::Signature;
use crate::DAYS;

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    pub const Version: RuntimeVersion = VERSION;

    /// We allow for 2 seconds of compute with a 6 second average block time.
    pub RuntimeBlockWeights: BlockWeights = BlockWeights::with_sensible_defaults(
        Weight::from_parts(2u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
        NORMAL_DISPATCH_RATIO,
    );
    pub RuntimeBlockLength: BlockLength = BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    pub const SS58Prefix: u8 = 42;

    pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
    pub const CouncilMaxProposals: u32 = 100;
    pub const CouncilMaxMembers: u32 = 100;
    pub MaxProposalWeight: Weight = Perbill::from_percent(50) * RuntimeBlockWeights::get().max_block;

    pub const BasicDeposit: Balance = 0;
    pub const FieldDeposit: Balance = 0;
    pub const SubAccountDeposit: Balance = 0;
    pub const MaxSubAccounts: u32 = 100;
    pub const MaxAdditionalFields: u32 = 100;
    pub const MaxRegistrars: u32 = 20;
}

/// The default types are being injected by
/// [`derive_impl`](`frame_support::derive_impl`) from
/// [`SoloChainDefaultConfig`](`struct@
/// frame_system::config_preludes::SolochainDefaultConfig`), but overridden as
/// needed.
#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig)]
impl frame_system::Config for Runtime {
    /// The block type for the runtime.
    type Block = Block;
    /// Block & extrinsics weights: base values and limits.
    type BlockWeights = RuntimeBlockWeights;
    /// The maximum length of a block (in bytes).
    type BlockLength = RuntimeBlockLength;
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The type for storing how many extrinsics an account has signed.
    type Nonce = Nonce;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// Maximum number of block number to block hash mappings to keep (oldest
    /// pruned first).
    type BlockHashCount = BlockHashCount;
    /// The weight of database operations that the runtime can invoke.
    type DbWeight = RocksDbWeight;
    /// Version of the runtime.
    type Version = Version;
    /// The data to be stored in an account.
    type AccountData = pallet_balances::AccountData<Balance>;
    /// This is used as an identifier of the chain. 42 is the generic substrate
    /// prefix.
    type SS58Prefix = SS58Prefix;
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_aura::Config for Runtime {
    type AuthorityId = AuraId;
    type DisabledValidators = ();
    type MaxAuthorities = ConstU32<32>;
    type AllowMultipleBlocksPerSlot = ConstBool<false>;
    type SlotDuration = pallet_aura::MinimumPeriodTimesTwo<Runtime>;
}

impl pallet_grandpa::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;

    type WeightInfo = ();
    type MaxAuthorities = ConstU32<32>;
    type MaxNominators = ConstU32<0>;
    type MaxSetIdSessionEntries = ConstU64<0>;

    type KeyOwnerProof = sp_core::Void;
    type EquivocationReportSystem = ();
}

impl pallet_timestamp::Config for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = Aura;
    type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
    type WeightInfo = ();
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
    type FreezeIdentifier = RuntimeFreezeReason;
    type MaxFreezes = VariantCountOf<RuntimeFreezeReason>;
    type RuntimeHoldReason = RuntimeHoldReason;
    type AccountStore = System;
    type RuntimeFreezeReason = RuntimeHoldReason;
}

parameter_types! {
    pub FeeMultiplier: Multiplier = Multiplier::one();
}

impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = FungibleAdapter<Balances, ()>;
    type OperationalFeeMultiplier = ConstU8<5>;
    type WeightToFee = IdentityFee<Balance>;
    type LengthToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
}

impl pallet_sudo::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;
}

impl pallet_collective::Config for Runtime {
    type RuntimeOrigin = RuntimeOrigin;
    type Proposal = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type MotionDuration = CouncilMotionDuration;
    type MaxMembers = CouncilMaxMembers;
    type MaxProposals = CouncilMaxProposals;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type SetMembersOrigin = frame_system::EnsureRoot<AccountId>;
    type MaxProposalWeight = MaxProposalWeight;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

// ensure that at least half of the council votes for
type EnsureRootOrHalfCouncil = frame_support::traits::EitherOfDiverse<
    frame_system::EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionMoreThan<AccountId, (), 1, 2>,
>;

impl pallet_identity::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type BasicDeposit = BasicDeposit;
    type ByteDeposit = FieldDeposit;
    type SubAccountDeposit = SubAccountDeposit;
    type IdentityInformation = pallet_identity::legacy::IdentityInfo<MaxAdditionalFields>;
    type OffchainSignature = Signature;
    type SigningPublicKey = <Signature as sp_runtime::traits::Verify>::Signer;
    type UsernameAuthorityOrigin = frame_system::EnsureRoot<AccountId>;
    type PendingUsernameExpiration = ConstU32<{ 7 * DAYS }>;
    type MaxSuffixLength = ConstU32<16>;
    type MaxUsernameLength = ConstU32<32>;
    type Currency = Balances;
    type Slashed = ();
    type ForceOrigin = EnsureRootOrHalfCouncil;
    type RegistrarOrigin = EnsureRootOrHalfCouncil;
    type MaxRegistrars = MaxRegistrars;
    type MaxSubAccounts = MaxSubAccounts;
    type WeightInfo = pallet_identity::weights::SubstrateWeight<Runtime>;
}

pub struct VotingIdentityProvider;

impl pallet_voting::IdentityProvider<AccountId> for VotingIdentityProvider {
    fn check_existence(account: &AccountId) -> bool {
        crate::Identity::identity(account.clone()).is_some()
    }
}

pub const UNIT: u128 = 1000000000000;

parameter_types! {
    pub const EntryFee: Balance = 30_000 * UNIT;
    pub const MaxProposals: u32 = 10u32;
}

/// Configure the pallet-voting in pallets/voting.
impl pallet_voting::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = pallet_voting::weights::SubstrateWeight<Runtime>;
    type IdentityProvider = VotingIdentityProvider;
    type Currency = Balances;
    type BasicDeposit = EntryFee;
    type MaxProposals = MaxProposals;
}
