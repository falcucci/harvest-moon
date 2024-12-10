use frame_support::assert_noop;
use frame_support::assert_ok;
use frame_system::Origin;
use sp_core::sr25519;

use crate::mock::*;
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
