//! Benchmarking setup for pallet-voting.
#![cfg(feature = "runtime-benchmarks")]
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

use super::*;
#[allow(unused)]
use crate::Pallet as Voting;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn do_something() {
        let value = 100u32;
        let caller: T::AccountId = whitelisted_caller();
        #[extrinsic_call]
        do_something(RawOrigin::Signed(caller), value);

        assert_eq!(Something::<T>::get(), Some(value));
    }

    #[benchmark]
    fn cause_error() {
        Something::<T>::put(100u32);
        let caller: T::AccountId = whitelisted_caller();
        #[extrinsic_call]
        cause_error(RawOrigin::Signed(caller));

        assert_eq!(Something::<T>::get(), Some(101u32));
    }

    impl_benchmark_test_suite!(Voting, crate::mock::new_test_ext(), crate::mock::Test);
}
