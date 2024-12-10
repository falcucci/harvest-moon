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

// We need to define the types used in this pallet.
pub mod types;

// Every callable function or "dispatchable" a pallet exposes must have weight
// values that correctly estimate a dispatchable's execution time. The
// benchmarking module is used to calculate weights for each dispatchable and generates this pallet's weight.rs file. Learn more about benchmarking here: https://docs.substrate.io/test/benchmark/
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
use frame_support::BoundedVec;
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
    use frame_support::sp_runtime::traits::Hash;
    use frame_support::traits::Currency;
    use frame_support::traits::ReservableCurrency;
    use frame_support::Blake2_128Concat;
    use frame_system::pallet_prelude::*;
    use scale_info::prelude::boxed::Box;
    use scale_info::prelude::vec::Vec;
    use sp_runtime::traits::IdentifyAccount;
    use sp_runtime::traits::Verify;
    use types::Commit;
    use types::Data;
    use types::Proposal;
    use types::Vote;
    use types::VoteToken;

    use super::*;

    pub type MemberCount = u32;
    pub type ProposalIndex = u32;
    pub type BlockNumber = u32;

    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    type ProposalOf<T> =
        Box<Proposal<<T as frame_system::Config>::AccountId, <T as frame_system::Config>::Block>>;

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

        /// The amount of funds that is required to have skin in a game
        #[pallet::constant]
        type BasicDeposit: Get<BalanceOf<Self>>;

        /// Maximum number of proposals allowed to be active in parallel.
        type MaxProposals: Get<ProposalIndex>;

        type Public: IdentifyAccount<AccountId = Self::AccountId>;
        type Signature: Verify<Signer = Self::Public> + Member + Decode + Encode + TypeInfo;

        /// A type representing the weights required by the dispatchables of
        /// this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::storage]
    pub type Members<T: Config> =
        CountedStorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    pub type Proposals<T: Config> =
        StorageValue<_, BoundedVec<T::Hash, T::MaxProposals>, ValueQuery>;

    #[pallet::storage]
    pub type ProposalData<T: Config> =
        StorageMap<_, Identity, T::Hash, Proposal<T::AccountId, BlockNumberFor<T>>>;

    #[pallet::storage]
    pub type Commits<T: Config> =
        StorageDoubleMap<_, Identity, T::Hash, Identity, T::AccountId, Commit<T::Signature>>;

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
        },
        Voted {
            account: T::AccountId,
            proposal_hash: T::Hash,
        },
        /// A motion (given hash) has been committed on by given account,
        /// leaving a tally (yes votes and no votes given respectively
        /// as `MemberCount`).
        Committed {
            account: T::AccountId,
            proposal_hash: T::Hash,
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
        /// Account is a already a member
        AlreadyMember,
        /// Account does not have an identity
        NoIdentity,
        /// Duplicate proposals not allowed
        DuplicateProposal,
        /// Proposal must exist
        ProposalMissing,
        /// Mismatched index
        WrongIndex,
        /// Invalid Argument was supplied
        InvalidArgument,
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
        /// Proposal Ended
        ProposalEnded,
        /// No commit has been submitted
        NoCommit,
        /// Could not verify signature of a commit
        SignatureInvalid,
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

            // check if signer is a member already | tested
            ensure!(!Self::is_member(&signer), Error::<T>::AlreadyMember);

            //check if signer has identity | tested
            ensure!(
                T::IdentityProvider::check_existence(&signer),
                Error::<T>::NoIdentity
            );

            // check if the account has enough money to deposit
            ensure!(
                T::Currency::can_reserve(&signer, T::BasicDeposit::get()),
                Error::<T>::NotEnoughFunds
            );

            T::Currency::reserve(&signer, T::BasicDeposit::get())?;
            <Members<T>>::insert(&signer, T::BasicDeposit::get());

            Self::deposit_event(Event::<T>::Joined(signer));

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::create_proposal())]
        pub fn create_proposal(
            origin: OriginFor<T>,
            community_note: Box<Data>,
            duration: BlockNumberFor<T>,
        ) -> DispatchResult {
            let signer = ensure_signed(origin)?;

            // check if signer is a member already
            ensure!(Self::is_member(&signer), Error::<T>::NotMember);
            let length_res = <Proposals<T>>::decode_len();
            if let Some(length) = length_res {
                if length == T::MaxProposals::get() as usize {
                    ensure!(false, Error::<T>::TooManyProposals);
                }
            }

            let proposal_hash = T::Hashing::hash_of(&community_note);
            let (exist, _) = Self::proposal_exist(&proposal_hash);
            ensure!(!exist, Error::<T>::DuplicateProposal);
            ensure!(
                <Proposals<T>>::try_append(proposal_hash).is_ok(),
                Error::<T>::WrongProposalLength
            );

            let end = duration + frame_system::Pallet::<T>::block_number();

            let proposal = Proposal {
                title: *community_note,
                proposer: signer.clone(),
                ayes: Vec::new(),
                nays: Vec::new(),
                end,
            };

            <ProposalData<T>>::insert(proposal_hash, proposal);
            Self::deposit_event(Event::<T>::Proposed {
                account: signer,
                proposal_hash,
            });

            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::vote())]
        pub fn vote(origin: OriginFor<T>, proposal: T::Hash, vote: Vote) -> DispatchResult {
            let signer = ensure_signed(origin)?;
            // check if signer is a member already
            ensure!(Self::is_member(&signer), Error::<T>::NotMember);
            let result = Self::already_voted_and_exist(&signer, &proposal);
            ensure!(result.is_some(), Error::<T>::ProposalMissing);
            let voted = result.unwrap();
            ensure!(!voted, Error::<T>::DuplicateVote);
            let proposal_data = <ProposalData<T>>::get(&proposal);
            ensure!(proposal_data.is_some(), Error::<T>::ProposalMissing);
            let mut proposal_data = proposal_data.unwrap();
            match vote {
                Vote::Yes => {
                    proposal_data.ayes.push(signer.clone());
                }
                Vote::No => {
                    proposal_data.nays.push(signer.clone());
                }
            };
            <ProposalData<T>>::set(proposal, Some(proposal_data));
            Self::deposit_event(Event::<T>::Voted {
                account: signer,
                proposal_hash: proposal,
            });

            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::commit_vote())]
        pub fn commit_vote(
            origin: OriginFor<T>,
            proposal: T::Hash,
            data: T::Signature,
            // number: VoteToken,
            salt: u32,
        ) -> DispatchResult {
            let signer = ensure_signed(origin)?;

            //check if signer is a member already | tested
            ensure!(Self::is_member(&signer), Error::<T>::NotMember);

            let committed = Self::already_committed_and_exist(&signer, &proposal);
            ensure!(!committed, Error::<T>::DuplicateVote);

            let proposal_data = <ProposalData<T>>::get(&proposal);
            ensure!(proposal_data.is_some(), Error::<T>::ProposalMissing);

            let proposal_data = proposal_data.unwrap();

            let current_block = frame_system::Pallet::<T>::block_number();

            ensure!(current_block < proposal_data.end, Error::<T>::ProposalEnded);

            let commit = Commit {
                signature: data,
                salt,
            };

            <Commits<T>>::insert(proposal, signer.clone(), commit);

            Self::deposit_event(Event::<T>::Committed {
                account: signer,
                proposal_hash: proposal,
            });

            Ok(())
        }

        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::reveal_vote())]
        pub fn reveal_vote(origin: OriginFor<T>, proposal: T::Hash, vote: Vote) -> DispatchResult {
            let signer = ensure_signed(origin)?;

            //check if signer is a member already | tested
            ensure!(Self::is_member(&signer), Error::<T>::NotMember);

            let result = Self::already_voted_and_exist(&signer, &proposal);
            ensure!(result.is_some(), Error::<T>::ProposalMissing);

            let voted = result.unwrap();
            ensure!(!voted, Error::<T>::DuplicateVote);

            // verify the signature
            let commit = <Commits<T>>::get(&proposal, &signer);
            ensure!(commit.is_some(), Error::<T>::NoCommit);

            let commit = commit.unwrap();
            let data = (vote.clone(), commit.salt).encode();
            let valid_sign = commit.signature.verify(data.as_slice(), &signer);
            ensure!(valid_sign, Error::<T>::SignatureInvalid);

            let proposal_data = <ProposalData<T>>::get(&proposal);
            ensure!(proposal_data.is_some(), Error::<T>::ProposalMissing);

            let mut proposal_data = proposal_data.unwrap();
            match vote {
                Vote::Yes => proposal_data.ayes.push(signer.clone()),
                Vote::No => proposal_data.nays.push(signer.clone()),
            }
            <ProposalData<T>>::insert(proposal, proposal_data);
            Self::deposit_event(Event::<T>::Voted {
                account: signer,
                proposal_hash: proposal,
            });

            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn is_member(account: &T::AccountId) -> bool { Members::<T>::contains_key(account) }

    pub fn proposal_exist(proposal: &T::Hash) -> (bool, BoundedVec<T::Hash, T::MaxProposals>) {
        let proposals = <Proposals<T>>::get();
        (proposals.contains(proposal), proposals)
    }

    pub fn already_voted_and_exist(who: &T::AccountId, proposal_hash: &T::Hash) -> Option<bool> {
        let result = <ProposalData<T>>::get(proposal_hash);
        if let Some(proposal) = result {
            Some(proposal.ayes.contains(who) || proposal.nays.contains(who))
        } else {
            None
        }
    }

    pub fn already_committed_and_exist(who: &T::AccountId, proposal_hash: &T::Hash) -> bool {
        <Commits<T>>::get(proposal_hash, who).is_some()
    }
}
