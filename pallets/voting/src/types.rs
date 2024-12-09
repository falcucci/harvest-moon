use codec::Decode;
use codec::Encode;
use codec::MaxEncodedLen;
use frame_support::sp_runtime::RuntimeDebug;
use frame_support::traits::ConstU32;
use frame_support::BoundedVec;
use scale_info::prelude::vec::Vec;
use scale_info::TypeInfo;

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum Data {
    /// The data is stored directly.
    Raw(BoundedVec<u8, ConstU32<2048>>),
}

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo)]
pub struct Proposal<AccountId, BlockNumberFor> {
    pub title: Data,
    pub proposer: AccountId,
    pub ayes: Vec<AccountId>,
    pub nays: Vec<AccountId>,
    pub end: BlockNumberFor,
}
