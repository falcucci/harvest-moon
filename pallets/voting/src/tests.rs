use frame_support::assert_noop;
use frame_support::assert_ok;
use frame_system::Origin;
use pallet_identity::legacy::IdentityInfo;
use sp_core::sr25519;
use sp_runtime::BoundedVec;

use crate::mock::generate;
use crate::mock::get_alice;
use crate::mock::get_bob;
use crate::mock::new_test_ext;
use crate::mock::Identity;
use crate::mock::MaxAdditionalFields;
use crate::mock::RuntimeOrigin;
use crate::mock::System;
use crate::mock::Test;
use crate::mock::VotingModule;
use crate::types::Data;
use crate::types::Vote;
use crate::Error;
use crate::Event;
use crate::Proposals;

#[test]
fn not_join_without_identity() {
    new_test_ext().execute_with(|| {
        let alice = get_alice();
        let origin = RuntimeOrigin::signed(alice);
        assert_noop!(
            VotingModule::join_committee(origin),
            Error::<Test>::NoIdentity
        );
    });
}

#[test]
fn join_with_identity() {
    new_test_ext().execute_with(|| {
        let alice = get_alice();
        let origin = RuntimeOrigin::signed(alice);
        let result = Identity::set_identity(origin.clone(), Box::new(data()));
        assert!(result.is_ok());
        assert_ok!(VotingModule::join_committee(origin));
    });
}

#[test]
fn disallow_action_for_non_members() {
    new_test_ext().execute_with(|| {
        let bob_origin = RuntimeOrigin::signed(get_bob());
        let _ = Identity::set_identity(bob_origin.clone(), Box::new(data()));

        let result = VotingModule::create_proposal(
            bob_origin,
            Box::new(Data::Raw(BoundedVec::default())),
            100,
        );
        assert_noop!(result, Error::<Test>::NotMember);
    });
}

#[test]
fn create_proposal_success() {
    new_test_ext().execute_with(|| {
        let alice = get_alice();
        let origin = RuntimeOrigin::signed(alice);
        let _ = Identity::set_identity(origin.clone(), Box::new(data()));

        let _ = VotingModule::join_committee(origin.clone());

        let result =
            VotingModule::create_proposal(origin, Box::new(Data::Raw(BoundedVec::default())), 100);
        assert_ok!(result);

        let results = <Proposals<Test>>::get();
        assert!(results.len() == 1);
    });
}

#[test]
fn no_proposal_duplicates() {
    new_test_ext().execute_with(|| {
        let alice = get_alice();
        let origin = RuntimeOrigin::signed(alice);
        let _ = Identity::set_identity(origin.clone(), Box::new(data()));

        let _ = VotingModule::join_committee(origin.clone());

        let _ = VotingModule::create_proposal(
            origin.clone(),
            Box::new(Data::Raw(BoundedVec::default())),
            100,
        );
        let result =
            VotingModule::create_proposal(origin, Box::new(Data::Raw(BoundedVec::default())), 100);

        assert_noop!(result, Error::<Test>::DuplicateProposal);
    });
}

#[test]
fn submit_commits() {
    new_test_ext().execute_with(|| {
        let alice = get_alice();
        let origin = RuntimeOrigin::signed(alice);
        let _ = Identity::set_identity(origin.clone(), Box::new(data()));

        let _ = VotingModule::join_committee(origin.clone());

        let _ = VotingModule::create_proposal(
            origin.clone(),
            Box::new(Data::Raw(BoundedVec::default())),
            100,
        );

        let (sig, salt) = generate("//Alice", Vote::Yes);
        let results = <Proposals<Test>>::get();
        let proposal_hash = results[0];

        let sig = sp_runtime::MultiSignature::Sr25519(sig);
        let result = VotingModule::commit_vote(origin, proposal_hash, sig, 8, salt);
        assert_ok!(result);
    });
}

fn cannot_submit_votes_more_than_have() {
    new_test_ext().execute_with(|| {
        let alice = get_alice();
        let origin = RuntimeOrigin::signed(alice);
        let _ = Identity::set_identity(origin.clone(), Box::new(data()));

        let _ = VotingModule::join_committee(origin.clone());

        let _ = VotingModule::create_proposal(
            origin.clone(),
            Box::new(Data::Raw(BoundedVec::default())),
            100,
        );

        let (sig, salt) = generate("//Alice", Vote::Yes);
        let results = <Proposals<Test>>::get();
        let proposal_hash = results[0];

        let sig = sp_runtime::MultiSignature::Sr25519(sig);
        let result = VotingModule::commit_vote(origin, proposal_hash, sig, 11, salt);
        assert_noop!(result, Error::<Test>::NotEnoughVotingTokens);
    });
}

#[test]
fn cannot_commit_after_deadline() {
    new_test_ext().execute_with(|| {
        let alice = get_alice();
        let origin = RuntimeOrigin::signed(alice);
        let _ = Identity::set_identity(origin.clone(), Box::new(data()));

        let _ = VotingModule::join_committee(origin.clone());

        let _ = VotingModule::create_proposal(
            origin.clone(),
            Box::new(Data::Raw(BoundedVec::default())),
            100,
        );

        System::set_block_number(System::block_number().saturating_add(105));

        let (sig, salt) = generate("//Alice", Vote::Yes);
        let results = <Proposals<Test>>::get();
        let proposal_hash = results[0];

        let sig = sp_runtime::MultiSignature::Sr25519(sig);
        let result = VotingModule::commit_vote(origin, proposal_hash, sig, 5, salt);
        assert_noop!(result, Error::<Test>::VoteEnded);
    });
}

fn data() -> IdentityInfo<MaxAdditionalFields> {
    IdentityInfo {
        display: pallet_identity::Data::Raw(b"ten".to_vec().try_into().unwrap()),
        additional: BoundedVec::default(),
        legal: Default::default(),
        web: Default::default(),
        riot: Default::default(),
        twitter: Default::default(),
        email: Default::default(),
        pgp_fingerprint: Default::default(),
        image: Default::default(),
    }
}
