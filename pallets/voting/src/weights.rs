// Executed Command:
// ../../target/release/harvest-moon
// benchmark
// pallet
// --chain
// dev
// --pallet
// pallet_voting
// --extrinsic
// *
// --steps=50
// --repeat=20
// --wasm-execution=compiled
// --output
// pallets/voting/src/weights.rs
// --voting
// ../../.maintain/frame-weight-voting.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]

use core::marker::PhantomData;

use frame_support::traits::Get;
use frame_support::weights::constants::RocksDbWeight;
use frame_support::weights::Weight;

/// Weight functions needed for pallet_voting.
pub trait WeightInfo {
    fn join_committee() -> Weight;
    fn leave_committee() -> Weight;
    fn create_proposal() -> Weight;
    fn cause_error() -> Weight;
    fn close_vote() -> Weight;
    fn close_reveal() -> Weight;
    fn commit_vote() -> Weight;
    fn reveal_vote() -> Weight;
}

/// Weights for pallet_voting using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    /// Storage: VotingModule Something (r:0 w:1)
    /// Proof: VotingModule Something (max_values: Some(1), max_size: Some(4),
    /// added: 499, mode: MaxEncodedLen)
    fn join_committee() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 8_000_000 picoseconds.
        Weight::from_parts(10_000_000, 0).saturating_add(T::DbWeight::get().writes(1_u64))
    }

    fn leave_committee() -> Weight {
        Weight::from_parts(10_000_000, 0).saturating_add(T::DbWeight::get().writes(1_u64))
    }

    fn create_proposal() -> Weight {
        Weight::from_parts(10_000_000, 0).saturating_add(T::DbWeight::get().writes(1_u64))
    }

    fn close_vote() -> Weight {
        Weight::from_parts(10_000_000, 0).saturating_add(T::DbWeight::get().writes(1_u64))
    }

    fn close_reveal() -> Weight {
        Weight::from_parts(10_000_000, 0).saturating_add(T::DbWeight::get().writes(1_u64))
    }

    fn commit_vote() -> Weight {
        Weight::from_parts(10_000_000, 0).saturating_add(T::DbWeight::get().writes(1_u64))
    }

    fn reveal_vote() -> Weight {
        Weight::from_parts(10_000_000, 0).saturating_add(T::DbWeight::get().writes(1_u64))
    }

    /// Storage: VotingModule Something (r:1 w:1)
    /// Proof: VotingModule Something (max_values: Some(1), max_size: Some(4),
    /// added: 499, mode: MaxEncodedLen)
    fn cause_error() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `32`
        //  Estimated: `1489`
        // Minimum execution time: 6_000_000 picoseconds.
        Weight::from_parts(6_000_000, 1489)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn join_committee() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 8_000_000 picoseconds.
        Weight::from_parts(10_000_000, 0).saturating_add(RocksDbWeight::get().writes(1_u64))
    }

    fn leave_committee() -> Weight {
        Weight::from_parts(10_000_000, 0).saturating_add(RocksDbWeight::get().writes(1_u64))
    }

    fn create_proposal() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 10_000_000 picoseconds.
        Weight::from_parts(10_000_000, 0).saturating_add(RocksDbWeight::get().writes(1_u64))
    }

    fn close_vote() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 10_000_000 picoseconds.
        Weight::from_parts(10_000_000, 0).saturating_add(RocksDbWeight::get().writes(1_u64))
    }

    fn close_reveal() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 10_000_000 picoseconds.
        Weight::from_parts(10_000_000, 0).saturating_add(RocksDbWeight::get().writes(1_u64))
    }

    fn commit_vote() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 10_000_000 picoseconds.
        Weight::from_parts(10_000_000, 0).saturating_add(RocksDbWeight::get().writes(1_u64))
    }

    fn reveal_vote() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 10_000_000 picoseconds.
        Weight::from_parts(10_000_000, 0).saturating_add(RocksDbWeight::get().writes(1_u64))
    }

    /// Storage: VotingModule Something (r:1 w:1)
    /// Proof: VotingModule Something (max_values: Some(1), max_size: Some(4),
    /// added: 499, mode: MaxEncodedLen)
    fn cause_error() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `32`
        //  Estimated: `1489`
        // Minimum execution time: 6_000_000 picoseconds.
        Weight::from_parts(6_000_000, 1489)
            .saturating_add(RocksDbWeight::get().reads(1_u64))
            .saturating_add(RocksDbWeight::get().writes(1_u64))
    }
}
