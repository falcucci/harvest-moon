use frame_support::assert_noop;
use frame_support::assert_ok;
use pallet_identity::legacy::IdentityInfo;
use sp_core::sr25519;
use sp_runtime::BoundedVec;

use crate::mock::generate;
use crate::mock::get_alice;
use crate::mock::get_bob;
use crate::mock::new_test_ext;
use crate::mock::Balances;
use crate::mock::Identity;
use crate::mock::MaxAdditionalFields;
use crate::mock::MaxTokens;
use crate::mock::RuntimeOrigin;
use crate::mock::System;
use crate::mock::Test;
use crate::mock::VotingModule;
use crate::types::Data;
use crate::types::Vote;
use crate::Error;
use crate::Event;
use crate::Members;
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

#[test]
fn reveal_vote_success() {
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

        System::set_block_number(System::block_number().saturating_add(20));

        let (sig, salt) = generate("//Alice", Vote::Yes);
        let results = <Proposals<Test>>::get();
        let proposal_hash = results[0];

        let sig = sp_runtime::MultiSignature::Sr25519(sig);
        let _ = VotingModule::commit_vote(origin.clone(), proposal_hash, sig, 8, salt);

        let result = VotingModule::reveal_vote(origin, proposal_hash, Vote::Yes);
        assert_ok!(result);
    });
}

#[test]
fn cannot_reveal_incorrect_vote() {
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

        System::set_block_number(System::block_number().saturating_add(20));

        let (sig, salt) = generate("//Alice", Vote::Yes);
        let results = <Proposals<Test>>::get();
        let proposal_hash = results[0];

        let sig = sp_runtime::MultiSignature::Sr25519(sig);
        let _ = VotingModule::commit_vote(origin.clone(), proposal_hash, sig, 8, salt);

        let result = VotingModule::reveal_vote(origin, proposal_hash, Vote::No);
        assert_noop!(result, Error::<Test>::SignatureInvalid);
    });
}

#[test]
fn close_vote_success() {
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

        System::set_block_number(System::block_number().saturating_add(120));
        let proposal_hash = <Proposals<Test>>::get()[0];
        let result = VotingModule::close_vote(origin, proposal_hash);
        assert_ok!(result);
    });
}

#[test]
fn cannot_close_vote_before_deadline() {
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

        let proposal_hash = <Proposals<Test>>::get()[0];
        let result = VotingModule::close_vote(origin, proposal_hash);
        assert_noop!(result, Error::<Test>::TooEarly);
    });
}

#[test]
fn close_reveal_success() {
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

        System::set_block_number(110);

        let proposal_hash = <Proposals<Test>>::get()[0];
        let _ = VotingModule::close_vote(origin.clone(), proposal_hash);

        System::set_block_number(160);

        let result = VotingModule::close_reveal(origin, proposal_hash);
        assert_ok!(result);
    });
}

#[test]
fn cannot_close_reveal_early() {
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

        System::set_block_number(110);

        let proposal_hash = <Proposals<Test>>::get()[0];
        let _ = VotingModule::close_vote(origin.clone(), proposal_hash);

        System::set_block_number(140);

        let result = VotingModule::close_reveal(origin, proposal_hash);
        assert_noop!(result, Error::<Test>::TooEarly);
    });
}

#[test]
fn cannot_close_reveal_before_vote_end() {
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

        let proposal_hash = <Proposals<Test>>::get()[0];

        System::set_block_number(140);

        let result = VotingModule::close_reveal(origin, proposal_hash);
        assert_noop!(result, Error::<Test>::RevealNotStarted);
    });
}

#[test]
fn slashed_correctly() {
    new_test_ext().execute_with(|| {
        let alice = get_alice();
        let origin_alice = RuntimeOrigin::signed(alice.clone());
        let _ = Identity::set_identity(origin_alice.clone(), Box::new(data()));
        let _ = VotingModule::join_committee(origin_alice.clone());

        let bob = get_bob();
        let origin_bob = RuntimeOrigin::signed(bob.clone());
        let _ = Identity::set_identity(origin_bob.clone(), Box::new(data()));
        let _ = VotingModule::join_committee(origin_bob.clone());

        let _ = VotingModule::create_proposal(
            origin_alice.clone(),
            Box::new(Data::Raw(BoundedVec::default())),
            100,
        );

        let (sig, salt) = generate("//Alice", Vote::Yes);
        let results = <Proposals<Test>>::get();
        let proposal_hash = results[0];

        let sig = sp_runtime::MultiSignature::Sr25519(sig);
        let _ = VotingModule::commit_vote(origin_alice, proposal_hash, sig, 8, salt);

        let (sig, salt) = generate("//Bob", Vote::No);
        let results = <Proposals<Test>>::get();
        let proposal_hash = results[0];

        let sig = sp_runtime::MultiSignature::Sr25519(sig);
        let _ = VotingModule::commit_vote(origin_bob.clone(), proposal_hash, sig, 2, salt);

        System::set_block_number(101);

        let proposal_hash = <Proposals<Test>>::get()[0];
        let _ = VotingModule::close_vote(origin_bob.clone(), proposal_hash);

        System::set_block_number(160);

        let alice_original_balance = <Members<Test>>::get(alice.clone()).reserved_balance;
        let bob_original_balance = <Members<Test>>::get(bob.clone()).reserved_balance;

        let _ = VotingModule::close_reveal(origin_bob, proposal_hash);

        let alice_current_balance = <Members<Test>>::get(alice).reserved_balance;
        let bob_current_balance = <Members<Test>>::get(bob).reserved_balance;

        let slash = bob_original_balance - bob_current_balance;
        assert!(alice_current_balance == alice_original_balance + slash);
    });
}

#[test]
#[ignore]
fn votes_deducted_and_refunded() {
    new_test_ext().execute_with(|| {
        let alice = get_alice();
        let origin_alice = RuntimeOrigin::signed(alice.clone());
        let _ = Identity::set_identity(origin_alice.clone(), Box::new(data()));
        let _ = VotingModule::join_committee(origin_alice.clone());

        let bob = get_bob();
        let origin_bob = RuntimeOrigin::signed(bob.clone());
        let _ = Identity::set_identity(origin_bob.clone(), Box::new(data()));
        let _ = VotingModule::join_committee(origin_bob.clone());

        let _ = VotingModule::create_proposal(
            origin_alice.clone(),
            Box::new(Data::Raw(BoundedVec::default())),
            100,
        );

        let results = <Proposals<Test>>::get();
        let proposal_hash = results[0];

        let (sig, salt) = generate("//Alice", Vote::Yes);
        let sig = sp_runtime::MultiSignature::Sr25519(sig);
        let _ = VotingModule::commit_vote(origin_alice.clone(), proposal_hash, sig, 8, salt);

        let alice_original_votes = <Members<Test>>::get(alice.clone()).voting_tokens;
        assert!(alice_original_votes == MaxTokens::get() - 8_u8.pow(2));

        let (sig, salt) = generate("//Bob", Vote::No);
        let sig = sp_runtime::MultiSignature::Sr25519(sig);
        let _ = VotingModule::commit_vote(origin_bob.clone(), proposal_hash, sig, 2, salt);

        let bob_original_votes = <Members<Test>>::get(bob.clone()).voting_tokens;
        assert!(bob_original_votes == MaxTokens::get() - 2_u8.pow(2));

        System::set_block_number(101);

        let proposal_hash = <Proposals<Test>>::get()[0];
        let _ = VotingModule::close_vote(origin_bob.clone(), proposal_hash);

        let _ = VotingModule::reveal_vote(origin_alice, proposal_hash, Vote::Yes);
        let _ = VotingModule::reveal_vote(origin_bob.clone(), proposal_hash, Vote::No);

        System::set_block_number(160);

        let _ = VotingModule::close_reveal(origin_bob, proposal_hash);

        let alice_tokens = <Members<Test>>::get(alice).voting_tokens;
        let bob_tokens = <Members<Test>>::get(bob).voting_tokens;

        assert!(alice_tokens == MaxTokens::get());
        assert!(bob_tokens == MaxTokens::get());
    });
}

#[test]
fn cannot_leave_while_in_vote() {
    new_test_ext().execute_with(|| {
        let alice = get_alice();
        let origin_alice = RuntimeOrigin::signed(alice);
        let _ = Identity::set_identity(origin_alice.clone(), Box::new(data()));
        let _ = VotingModule::join_committee(origin_alice.clone());

        let _ = VotingModule::create_proposal(
            origin_alice.clone(),
            Box::new(Data::Raw(BoundedVec::default())),
            100,
        );

        let results = <Proposals<Test>>::get();
        let proposal_hash = results[0];

        let (sig, salt) = generate("//Alice", Vote::Yes);
        let sig = sp_runtime::MultiSignature::Sr25519(sig);
        let _ = VotingModule::commit_vote(origin_alice.clone(), proposal_hash, sig, 8, salt);

        let result = VotingModule::leave_committee(origin_alice.clone());
        assert_noop!(result, Error::<Test>::InMotion);

        System::set_block_number(110);

        let _ = VotingModule::close_vote(origin_alice.clone(), proposal_hash);

        let result = VotingModule::leave_committee(origin_alice);
        assert_noop!(result, Error::<Test>::InMotion);
    });
}

#[test]
fn cashout() {
    new_test_ext().execute_with(|| {
        let alice = get_alice();
        let origin_alice = RuntimeOrigin::signed(alice.clone());
        let _ = Identity::set_identity(origin_alice.clone(), Box::new(data()));
        let _ = VotingModule::join_committee(origin_alice.clone());

        let bob = get_bob();
        let origin_bob = RuntimeOrigin::signed(bob);
        let _ = Identity::set_identity(origin_bob.clone(), Box::new(data()));
        let _ = VotingModule::join_committee(origin_bob.clone());

        let _ = VotingModule::create_proposal(
            origin_alice.clone(),
            Box::new(Data::Raw(BoundedVec::default())),
            100,
        );

        let results = <Proposals<Test>>::get();
        let proposal_hash = results[0];

        let (sig, salt) = generate("//Alice", Vote::Yes);
        let sig = sp_runtime::MultiSignature::Sr25519(sig);
        let _ = VotingModule::commit_vote(origin_alice.clone(), proposal_hash, sig, 8, salt);

        let (sig, salt) = generate("//Bob", Vote::No);
        let sig = sp_runtime::MultiSignature::Sr25519(sig);
        let _ = VotingModule::commit_vote(origin_bob.clone(), proposal_hash, sig, 2, salt);

        System::set_block_number(101);

        let proposal_hash = <Proposals<Test>>::get()[0];
        let _ = VotingModule::close_vote(origin_bob.clone(), proposal_hash);

        let _ = VotingModule::reveal_vote(origin_alice.clone(), proposal_hash, Vote::Yes);
        let _ = VotingModule::reveal_vote(origin_bob.clone(), proposal_hash, Vote::No);

        System::set_block_number(160);

        let _ = VotingModule::close_reveal(origin_bob, proposal_hash);

        let result = VotingModule::leave_committee(origin_alice);
        assert_ok!(result);

        assert!(Balances::reserved_balance(alice) == 0);
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
