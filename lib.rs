#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod bonding_curve {
    use ink_storage::{
        collections::{HashMap as StorageHashMap, Vec as StorageVec},
        lazy::Lazy,
    };

    #[ink(storage)]
    pub struct BondingCurve {
        pools: StorageHashMap<PoolId, Pool>,
    }

    #[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct PoolId([u8; 32]);

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Pool {
        curve_type: CurveType,
        parameters: StorageVec<u128>,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum CurveType {
        Linear,
        Exponential,
        Logarithmic,
        Custom,
    }

    impl BondingCurve {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                pools: StorageHashMap::new(),
            }
        }

        #[ink(message)]
        pub fn create_pool(&mut self, curve_type: CurveType, parameters: Vec<u128>) -> PoolId {
            let pool_id = self.env().caller();
            let pool = Pool {
                curve_type,
                parameters: StorageVec::from(parameters),
            };
            self.pools.insert(pool_id, pool);
            pool_id
        }

        #[ink(message)]
        pub fn get_pool(&self, pool_id: PoolId) -> Option<&Pool> {
            self.pools.get(&pool_id)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn create_pool_works() {
            let mut contract = BondingCurve::new();
            let curve_type = CurveType::Linear;
            let parameters = vec![100, 50]; // example parameters
            let pool_id = contract.create_pool(curve_type, parameters.clone());
            let pool = contract.get_pool(pool_id).unwrap();
            assert_eq!(pool.curve_type, curve_type);
            assert_eq!(pool.parameters.as_ref(), &parameters);
        }
    }
}
