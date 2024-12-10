use frame_support::assert_noop;
use frame_support::assert_ok;
use frame_system::Origin;
use pallet_identity::legacy::IdentityInfo;
use sp_core::sr25519;
use sp_runtime::BoundedVec;

use crate::mock::*;
use crate::types::Data;
use crate::Error;
use crate::Event;

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
