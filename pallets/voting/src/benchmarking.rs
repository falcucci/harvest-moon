//! Benchmarking setup for pallet-voting
#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::v2::*;

use super::*;

#[benchmarks]
mod benchmarks {
    use frame_system::RawOrigin;

    use super::*;
    #[cfg(test)]
    use crate::pallet::Pallet as Voting;

    #[benchmark]
    fn do_something() {
        let caller: T::AccountId = whitelisted_caller();
        #[extrinsic_call]
        do_something(RawOrigin::Signed(caller), 100);

        assert_eq!(
            Something::<T>::get().map(|v| v.block_number),
            Some(100u32.into())
        );
    }

    #[benchmark]
    fn cause_error() {
        Something::<T>::put(CompositeStruct {
            block_number: 100u32.into(),
        });
        let caller: T::AccountId = whitelisted_caller();
        #[extrinsic_call]
        cause_error(RawOrigin::Signed(caller));

        assert_eq!(
            Something::<T>::get().map(|v| v.block_number),
            Some(101u32.into())
        );
    }

    impl_benchmark_test_suite!(Voting, crate::mock::new_test_ext(), crate::mock::Test);
}
