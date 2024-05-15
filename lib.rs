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
        pools: StorageHashMap<PoolId, Pool>, // Storage for pools
    }

    #[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct PoolId([u8; 32]);

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Pool {
        curve_type: CurveType,          // Type of curve
        parameters: StorageVec<u128>,  // Parameters for the curve
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
                pools: StorageHashMap::new(), // Initialize storage for pools
            }
        }

        #[ink(message)]
        pub fn create_pool(&mut self, curve_type: CurveType, parameters: Vec<u128>) -> PoolId {
            let pool_id = self.env().caller(); // Get the caller's account ID
            let pool = Pool {
                curve_type,
                parameters: StorageVec::from(parameters), // Convert parameters to StorageVec
            };
            self.pools.insert(pool_id, pool); // Insert the new pool into the storage
            pool_id // Return the pool ID
        }

        #[ink(message)]
        pub fn get_pool(&self, pool_id: PoolId) -> Option<&Pool> {
            self.pools.get(&pool_id) // Get the pool by ID
        }

        #[ink(message)]
        pub fn deposit(&mut self, pool_id: PoolId, amount: u128) {
            let pool = self.pools.get_mut(&pool_id).expect("Pool does not exist");
            // Perform liquidity deposit logic here
            // This function should add liquidity to the pool
        }

        #[ink(message)]
        pub fn withdraw(&mut self, pool_id: PoolId, amount: u128) {
            let pool = self.pools.get_mut(&pool_id).expect("Pool does not exist");
            // Perform liquidity withdrawal logic here
            // This function should remove liquidity from the pool
        }

        #[ink(message)]
        pub fn set_params(&mut self, pool_id: PoolId, parameters: Vec<u128>) {
            let pool = self.pools.get_mut(&pool_id).expect("Pool does not exist");
            pool.parameters = StorageVec::from(parameters); // Update pool parameters
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

        #[test]
        fn deposit_works() {
            let mut contract = BondingCurve::new();
            let curve_type = CurveType::Linear;
            let parameters = vec![100, 50]; // example parameters
            let pool_id = contract.create_pool(curve_type, parameters.clone());
            contract.deposit(pool_id, 100);
            // Add assertion here
        }

        #[test]
        fn withdraw_works() {
            let mut contract = BondingCurve::new();
            let curve_type = CurveType::Linear;
            let parameters = vec![100, 50]; // example parameters
            let pool_id = contract.create_pool(curve_type, parameters.clone());
            contract.deposit(pool_id, 100);
            contract.withdraw(pool_id, 50);
            // Add assertion here
        }

        #[test]
        fn set_params_works() {
            let mut contract = BondingCurve::new();
            let curve_type = CurveType::Linear;
            let parameters = vec![100, 50]; // example parameters
            let pool_id = contract.create_pool(curve_type, parameters.clone());
            let new_params = vec![200, 100]; // new example parameters
            contract.set_params(pool_id, new_params.clone());
            let pool = contract.get_pool(pool_id).unwrap();
            assert_eq!(pool.parameters.as_ref(), &new_params);
        }
    }
}
