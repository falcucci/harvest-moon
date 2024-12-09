// We make sure this pallet uses `no_std` for compiling to Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

// FRAME pallets require their own "mock runtimes" to be able to run unit tests.
// This module contains a mock runtime specific for testing this pallet's
// functionality.
#[cfg(test)]
mod mock;

// This module contains the unit tests for this pallet.
// Learn about pallet unit testing here: https://docs.substrate.io/test/unit-testing/
#[cfg(test)]
mod tests;

// Every callable function or "dispatchable" a pallet exposes must have weight
// values that correctly estimate a dispatchable's execution time. The
// benchmarking module is used to calculate weights for each dispatchable and generates this pallet's weight.rs file. Learn more about benchmarking here: https://docs.substrate.io/test/benchmark/
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

// All pallet logic is defined in its own module and must be annotated by the
// `pallet` attribute.
#[frame_support::pallet]
pub mod pallet {
    // Import various useful types required by all FRAME pallets.
    use frame_support::dispatch::DispatchResult;
    use frame_support::pallet_prelude::CountedStorageMap;
    use frame_support::pallet_prelude::ValueQuery;
    use frame_support::pallet_prelude::*;
    use frame_support::traits::Currency;
    use frame_support::traits::ReservableCurrency;
    use frame_support::Blake2_128Concat;
    use frame_system::pallet_prelude::*;

    use super::*;

    pub type MemberCount = u32;

    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    pub trait IdentityProvider<AccountId> {
        fn check_existence(account: &AccountId) -> bool;
    }

    /// The pallet's configuration trait.
    ///
    /// All our types and constants a pallet depends on must be declared here.
    /// These types are defined generically and made concrete when the pallet is
    /// declared in the `runtime/src/lib.rs` file of your chain.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type IdentityProvider: IdentityProvider<Self::AccountId>;
        type Currency: ReservableCurrency<Self::AccountId>;
        #[pallet::constant]
        type BasicDeposit: Get<BalanceOf<Self>>;

        /// A type representing the weights required by the dispatchables of
        /// this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::storage]
    pub type Members<T: Config> =
        CountedStorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// Events that functions in this pallet can emit.
    ///
    /// Events are a simple means of indicating to the outside world (such as
    /// dApps, chain explorers or other users) that some notable update in
    /// the runtime has occurred. In a FRAME pallet, the documentation for
    /// each event field and its parameters is added to a node's metadata so it
    /// can be used by external interfaces or tools.
    ///
    ///	The `generate_deposit` macro generates a function on `Pallet` called
    /// `deposit_event` which will convert the event type of your pallet
    /// into `RuntimeEvent` (declared in the pallet's [`Config`] trait) and
    /// deposit it using [`frame_system::Pallet::deposit_event`].
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Joined(T::AccountId),
        Left {
            account: T::AccountId,
            cashout: BalanceOf<T>,
        },
        Proposed {
            account: T::AccountId,
            proposal_hash: T::Hash,
            threshold: MemberCount,
        },
        Voted {
            account: T::AccountId,
            proposal_hash: T::Hash,
            vote: bool,
            yes: MemberCount,
            no: MemberCount,
        },
        Approved {
            proposal_hash: T::Hash,
        },
        Disapproved {
            proposal_hash: T::Hash,
        },
        Executed {
            proposal_hash: T::Hash,
            result: DispatchResult,
        },
        MemerExecuted {
            proposal_hash: T::Hash,
            result: DispatchResult,
        },
        Closed {
            proposal_hash: T::Hash,
            yes: MemberCount,
            no: MemberCount,
        },
    }

    /// Errors that can be returned by this pallet.
    ///
    /// Errors tell users that something went wrong so it's important that their
    /// naming is informative. Similar to events, error documentation is
    /// added to a node's metadata so it's equally important that they have
    /// helpful documentation associated with them.
    ///
    /// This type of runtime error can be up to 4 bytes in size should you want
    /// to return additional information.
    #[pallet::error]
    pub enum Error<T> {
        /// Account is not a member
        NotMember,
        /// Duplicate proposals not allowed
        DuplicateProposal,
        /// Proposal must exist
        ProposalMissing,
        /// Mismatched index
        WrongIndex,
        /// Duplicate vote ignored
        DuplicateVote,
        /// Members are already initialized!
        AlreadyInitialized,
        /// The close call was made too early, before the end of the voting.
        TooEarly,
        /// There can only be a maximum of `MaxProposals` active proposals.
        TooManyProposals,
        /// The given length bound for the proposal was too low.
        WrongProposalLength,
        /// Not enough funds to join the voting council
        NotEnoughFunds,
        /// The value retrieved was `None` as no value was previously set.
        NoneValue,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    /// The pallet's dispatchable functions ([`Call`]s).
    ///
    /// Dispatchable functions allows users to interact with the pallet and
    /// invoke state changes. These functions materialize as "extrinsics",
    /// which are often compared to transactions. They must always return a
    /// `DispatchResult` and be annotated with a weight and call index.
    ///
    /// The [`call_index`] macro is used to explicitly
    /// define an index for calls in the [`Call`] enum. This is useful for
    /// pallets that may introduce new dispatchables over time. If the order
    /// of a dispatchable changes, its index will also change which will
    /// break backwards compatibility.
    ///
    /// The [`weight`] macro is used to assign a weight to each call.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// .
        ///
        /// # Errors
        ///
        /// This function will return an error if .
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::join_committee())]
        pub fn join_committee(origin: OriginFor<T>) -> DispatchResult {
            let signer = ensure_signed(origin)?;
            ensure!(
                T::Currency::can_reserve(&signer, T::BasicDeposit::get()),
                Error::<T>::NotEnoughFunds
            );
            //TODO: check if signer has identity
            //TODO: check if signer is a member already
            //TODO: deposit
            <Members<T>>::insert(&signer, T::BasicDeposit::get());
            Ok(())
        }
    }
}
