// We make sure this pallet uses `no_std` for compiling to Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::pallet_prelude::BlockNumberFor;
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
use codec::alloc::borrow::ToOwned;
use frame_support::traits::ReservableCurrency;
use frame_support::BoundedVec;
use scale_info::prelude::vec;
use scale_info::prelude::vec::Vec;
use sp_runtime::traits::AccountIdConversion;
use sp_runtime::traits::CheckedDiv;
use sp_runtime::traits::Get;
use sp_runtime::DispatchError;
pub use weights::*;

// All pallet logic is defined in its own module and must be annotated by the
// `pallet` attribute.
#[frame_support::pallet]
pub mod pallet {
    use core::cmp::Ordering;

    // Import various useful types required by all FRAME pallets.
    use frame_support::dispatch::DispatchResult;
    use frame_support::pallet_prelude::CountedStorageMap;
    use frame_support::pallet_prelude::ValueQuery;
    use frame_support::pallet_prelude::*;
    use frame_support::sp_runtime::traits::Hash;
    use frame_support::traits::Currency;
    use frame_support::traits::PalletsInfoAccess;
    use frame_support::traits::ReservableCurrency;
    use frame_support::Blake2_128Concat;
    use frame_support::PalletId;
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

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    // type ProposalOf<T> =
    //     Box<Proposal<<T as frame_system::Config>::AccountId, <T as
    // frame_system::Config>::Block>>;

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

        /// The length of reveal phase
        #[pallet::constant]
        type RevealLength: Get<BlockNumberFor<Self>>;

        /// Minimum length of proposal
        #[pallet::constant]
        type MinLength: Get<BlockNumberFor<Self>>;

        /// Minimum length of proposal
        #[pallet::constant]
        type MaxVotingTokens: Get<u8>;

        /// Maximum number of proposals allowed to be active in parallel.
        #[pallet::constant]
        type MaxProposals: Get<ProposalIndex>;

        type Public: IdentifyAccount<AccountId = Self::AccountId>;
        type Signature: Verify<Signer = Self::Public> + Member + Decode + Encode + TypeInfo;

        /// The council's pallet id, used for deriving its sovereign account ID.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// A type representing the weights required by the dispatchables of
        /// this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::storage]
    pub type Proposals<T: Config> =
        StorageValue<_, BoundedVec<T::Hash, T::MaxProposals>, ValueQuery>;

    #[pallet::storage]
    pub type ProposalData<T: Config> =
        StorageMap<_, Identity, T::Hash, Proposal<T::AccountId, BlockNumberFor<T>, BalanceOf<T>>>;

    #[pallet::storage]
    pub type Commits<T: Config> =
        StorageDoubleMap<_, Identity, T::AccountId, Identity, T::Hash, Commit<T::Signature>>;

    #[pallet::storage]
    pub type Members<T: Config> =
        CountedStorageMap<_, Identity, T::AccountId, VoteToken, ValueQuery>;

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
        ///
        NotEnoughVotingTokens,
        /// Voting phase ended
        VoteEnded,
        /// Vote has already ended when trying to close it
        VoteAlreadyEnded,
        /// Reveal phase ended
        RevealEnded,
        /// Reveal phase has not yet started
        RevealNotStarted,
        /// No commit has been submitted
        NoCommit,
        /// Could not verify signature of a commit
        SignatureInvalid,
        /// The voter is in the middle of vote
        InMotion,
        /// Proposal is still going
        NotFinished,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        _phantom: core::marker::PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            // Create Treasury account
            let account_id = <Pallet<T>>::account_id();
            let min = T::Currency::minimum_balance();
            if T::Currency::free_balance(&account_id) < min {
                let _ = T::Currency::make_free_balance_be(&account_id, min);
            }
        }
    }

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

            //deposit 100 voting tokens to the voter
            Self::deposit_votes(&signer, 100);

            Self::deposit_event(Event::<T>::Joined(signer));

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::leave_committee())]
        pub fn leave_committee(origin: OriginFor<T>) -> DispatchResult {
            let signer = ensure_signed(origin)?;
            //check if signer has identity | tested
            ensure!(
                T::IdentityProvider::check_existence(&signer),
                Error::<T>::NoIdentity
            );
            let active_votes = <Commits<T>>::iter_prefix_values(signer.clone()).count();
            ensure!(active_votes == 0, Error::<T>::InMotion);
            let balance = T::Currency::unreserve(&signer, T::Currency::reserved_balance(&signer));
            //remove entries
            <Members<T>>::remove(signer.clone());
            Self::deposit_event(Event::<T>::Left {
                account: signer,
                cashout: balance,
            });
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::create_proposal())]
        pub fn create_proposal(
            origin: OriginFor<T>,
            community_note: Box<Data>,
            duration: BlockNumberFor<T>,
        ) -> DispatchResult {
            let signer = ensure_signed(origin)?;

            if duration < T::MinLength::get() {
                ensure!(false, Error::<T>::WrongProposalLength);
            }

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
                Error::<T>::TooManyProposals
            );

            let end = duration + frame_system::Pallet::<T>::block_number();

            let proposal = Proposal {
                title: *community_note,
                proposer: signer.clone(),
                ayes: 0,
                nays: 0,
                poll_end: end,
                reveal_end: None,
                votes: Vec::new(),
                revealed: Vec::new(),
                payout: BalanceOf::<T>::default(),
            };

            <ProposalData<T>>::insert(proposal_hash, proposal);
            Self::deposit_event(Event::<T>::Proposed {
                account: signer,
                proposal_hash,
            });

            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::close_vote())]
        pub fn close_vote(origin: OriginFor<T>, proposal: T::Hash) -> DispatchResult {
            let signer = ensure_signed(origin)?;

            // check if signer is a member already
            ensure!(Self::is_member(&signer), Error::<T>::NotMember);

            let proposal_data = <ProposalData<T>>::get(&proposal);
            ensure!(proposal_data.is_some(), Error::<T>::ProposalMissing);

            let mut proposal_data = proposal_data.unwrap();
            ensure!(
                proposal_data.reveal_end.is_none(),
                Error::<T>::VoteAlreadyEnded
            );

            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(
                proposal_data.poll_end <= current_block,
                Error::<T>::TooEarly
            );

            let current_block = frame_system::Pallet::<T>::block_number();
            proposal_data.reveal_end = Some(current_block + T::RevealLength::get());

            <ProposalData<T>>::insert(proposal, proposal_data);

            Ok(())
        }

        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::close_reveal())]
        pub fn close_reveal(origin: OriginFor<T>, proposal: T::Hash) -> DispatchResult {
            let signer = ensure_signed(origin)?;

            //check if signer is a member already | tested
            ensure!(Self::is_member(&signer), Error::<T>::NotMember);

            let proposal_data = <ProposalData<T>>::get(&proposal);
            ensure!(proposal_data.is_some(), Error::<T>::ProposalMissing);

            let mut proposal_data = proposal_data.unwrap();
            ensure!(
                proposal_data.reveal_end.is_some(),
                Error::<T>::RevealNotStarted
            );

            let reveal_end = proposal_data.reveal_end.unwrap();
            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(reveal_end <= current_block, Error::<T>::TooEarly);

            // refund voting tokens to voters
            for (_, (account, votes, _)) in proposal_data.votes.iter().enumerate() {
                let amount = u8::pow(*votes, 2);
                Self::deposit_votes(account, amount);
            }

            //deduce winning side
            let result = proposal_data.ayes.cmp(&proposal_data.nays);
            let pot_address = Self::account_id();
            let amount: BalanceOf<T>;
            match result {
                Ordering::Greater => {
                    let losers: Vec<T::AccountId> = proposal_data
                        .votes
                        .iter()
                        .filter(|entry| entry.2 == Vote::No)
                        .map(|entry| entry.0.clone())
                        .collect();
                    amount = Self::slash_voting_side(losers, &pot_address)?;
                    let winners: Vec<T::AccountId> = proposal_data
                        .votes
                        .iter()
                        .filter(|entry| entry.2 == Vote::Yes)
                        .map(|entry| entry.0.clone())
                        .collect();
                    Self::reward_voting_side(winners, &pot_address, amount)?;
                }
                Ordering::Less => {
                    let losers: Vec<T::AccountId> = proposal_data
                        .votes
                        .iter()
                        .filter(|entry| entry.2 == Vote::Yes)
                        .map(|entry| entry.0.clone())
                        .collect();
                    amount = Self::slash_voting_side(losers, &pot_address)?;
                    let winners: Vec<T::AccountId> = proposal_data
                        .votes
                        .iter()
                        .filter(|entry| entry.2 == Vote::No)
                        .map(|entry| entry.0.clone())
                        .collect();
                    Self::reward_voting_side(winners, &pot_address, amount)?;
                }
                Ordering::Equal => {
                    let losers: Vec<T::AccountId> =
                        proposal_data.votes.iter().map(|entry| entry.0.clone()).collect();
                    amount = Self::slash_voting_side(losers, &pot_address)?;
                    Self::reward_voting_side(
                        vec![proposal_data.clone().proposer],
                        &pot_address,
                        amount,
                    )?;
                }
            }
            proposal_data.payout = amount;
            <ProposalData<T>>::insert(&proposal, proposal_data);

            Ok(())
        }

        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::reveal_vote())]
        pub fn reveal_vote(origin: OriginFor<T>, proposal: T::Hash, vote: Vote) -> DispatchResult {
            let signer = ensure_signed(origin)?;

            //check if signer is a member already | tested
            ensure!(Self::is_member(&signer), Error::<T>::NotMember);

            //verify the signature
            let commit = <Commits<T>>::take(&signer, &proposal);
            ensure!(commit.is_some(), Error::<T>::NoCommit);
            let commit = commit.unwrap();

            let data = (vote.clone(), commit.salt).encode();
            let valid_sign = commit.signature.verify(data.as_slice(), &signer);
            ensure!(valid_sign, Error::<T>::SignatureInvalid);

            let proposal_data = <ProposalData<T>>::get(&proposal);
            ensure!(proposal_data.is_some(), Error::<T>::ProposalMissing);
            let mut proposal_data = proposal_data.unwrap();

            let reveal_exist = proposal_data.reveal_end;
            if let Some(reveal_end) = reveal_exist {
                let current_block = frame_system::Pallet::<T>::block_number();
                // if voter decides to reveal votes after the end, he will just be slashed
                // the voter is incentivised to perform this action in order to refund voting
                // tokens or to cash out
                if current_block > reveal_end {
                    let pot_address = Self::account_id();
                    let _ = Self::slash_voting_side(vec![signer.clone()], &pot_address)?;
                    let amount = u8::pow(commit.number, 2);
                    Self::deposit_votes(&signer, amount);
                    //probably need to refund, but let it be additional punishment
                    return Ok(());
                }
            }

            let voted = Self::already_voted(&signer, &proposal_data);
            ensure!(!voted, Error::<T>::DuplicateVote);

            match vote {
                Vote::Yes => proposal_data.ayes += commit.number as u32,
                Vote::No => proposal_data.nays += commit.number as u32,
            }

            proposal_data.votes.push((signer.clone(), commit.number, vote.clone()));
            proposal_data.revealed.push(signer.clone());

            <ProposalData<T>>::insert(proposal, proposal_data);

            Self::deposit_event(Event::<T>::Voted {
                account: signer,
                proposal_hash: proposal,
            });

            Ok(())
        }

        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::commit_vote())]
        pub fn commit_vote(
            origin: OriginFor<T>,
            proposal: T::Hash,
            data: T::Signature,
            number: VoteToken,
            salt: u32,
        ) -> DispatchResult {
            let signer = ensure_signed(origin)?;
            //check if signer is a member already | tested
            ensure!(Self::is_member(&signer), Error::<T>::NotMember);

            if number == 0 {
                ensure!(false, Error::<T>::InvalidArgument);
            }

            let committed = Self::already_committed_and_exist(&signer, &proposal);
            ensure!(!committed, Error::<T>::DuplicateVote);

            let proposal_data = <ProposalData<T>>::get(&proposal);
            ensure!(proposal_data.is_some(), Error::<T>::ProposalMissing);
            let proposal_data = proposal_data.unwrap();

            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(
                current_block < proposal_data.poll_end,
                Error::<T>::VoteEnded
            );

            let mut tokens_to_take: u8 = number;
            if number > 1 {
                tokens_to_take = number.pow(2);
            }

            let enough_tokens = Self::decrease_votes(&signer, tokens_to_take);
            ensure!(enough_tokens, Error::<T>::NotEnoughVotingTokens);

            let commit = Commit {
                signature: data,
                salt,
                number,
            };
            <Commits<T>>::insert(signer.clone(), proposal, commit);

            Self::deposit_event(Event::<T>::Committed {
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

    pub fn already_voted(
        who: &T::AccountId,
        proposal: &types::Proposal<T::AccountId, BlockNumberFor<T>, BalanceOf<T>>,
    ) -> bool {
        proposal.revealed.contains(who)
    }

    pub fn already_committed_and_exist(who: &T::AccountId, proposal_hash: &T::Hash) -> bool {
        <Commits<T>>::get(who, proposal_hash).is_some()
    }

    /// Deposit voting tokens to the account and make sure it does not exceed
    /// the limit
    pub fn deposit_votes(who: &T::AccountId, tokens: u8) {
        <Members<T>>::mutate(who, |balance| {
            *balance += tokens;
            if *balance > 100u8 {
                *balance = 100u8;
            }
        });
    }

    /// tries to decrease the voting tokens of a specific account by specified
    /// amount. Returns false if account does not have enough voting tokens
    pub fn decrease_votes(who: &T::AccountId, amount: u8) -> bool {
        <Members<T>>::try_mutate(who, |balance| {
            if *balance < amount {
                return Err(());
            }
            *balance -= amount;
            Ok(())
        })
        .is_ok()
    }

    /// Slashes the losing side, puts money in a pot and returns the total
    /// amount slashed
    pub fn slash_voting_side(
        voters: Vec<T::AccountId>,
        pot: &T::AccountId,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let mut balance: BalanceOf<T> = BalanceOf::<T>::default();
        for voter in voters {
            let denominator: BalanceOf<T> = 10u8.into();
            let slash = T::Currency::reserved_balance(&voter)
                .checked_div(&denominator.clone())
                .get_or_insert(BalanceOf::<T>::default())
                .to_owned();
            T::Currency::repatriate_reserved(
                &voter,
                pot,
                slash,
                frame_support::traits::BalanceStatus::Reserved,
            )?;
            balance += slash;
        }
        Ok(balance)
    }
    /// Rewards evenly every member from the pot with the provided sum
    pub fn reward_voting_side(
        voters: Vec<T::AccountId>,
        pot: &T::AccountId,
        total: BalanceOf<T>,
    ) -> Result<(), DispatchError> {
        let len = voters.len() as u32;
        let share = total / len.into();
        for voter in voters {
            T::Currency::repatriate_reserved(
                pot,
                &voter,
                share,
                frame_support::traits::BalanceStatus::Reserved,
            )?;
        }
        Ok(())
    }

    /// Intermediate
    pub fn account_id() -> T::AccountId { T::PalletId::get().into_account_truncating() }
}
