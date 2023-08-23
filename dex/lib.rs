#![cfg_attr(not(feature = "std"), no_std)]
#![allow(non_snake_case)]

const PRECISION: u128 = 1_000_000; // Precision of 6 digits

#[ink::contract]
mod dex {
    use ink::storage::Mapping;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Zero Liquidity
        ZeroLiquidity,
        /// Amount cannot be zero!
        ZeroAmount,
        /// Insufficient amount
        InsufficientAmount,
        /// Equivalent value of tokens not provided
        NonEquivalentValue,
        /// Asset value less than threshold for contribution!
        ThresholdNotReached,
        /// Share should be less than totalShare
        InvalidShare,
        /// Insufficient pool balance
        InsufficientLiquidity,
        /// Slippage tolerance exceeded
        SlippageExceeded,
    }

    #[derive(Default)]
    #[ink(storage)]
    pub struct Dex {
        totalShares: Balance, // Stores the total amount of share issued for the pool
        totalToken1: Balance, // Stores the amount of Token1 locked in the pool
        totalToken2: Balance, // Stores the amount of Token2 locked in the pool
        shares: Mapping<AccountId, Balance>, // Stores the share holding of each provider
        token1Balance: Mapping<AccountId, Balance>, // Stores the token1 balance of each user
        token2Balance: Mapping<AccountId, Balance>, // Stores the token2 balance of each user
        fees: Balance,        // Percent of trading fees charged on trade
    }

    #[ink(impl)]
    impl Dex {
        // Ensures that the _qty is non-zero and the user has enough balance
        fn validAmountCheck(
            &self,
            _balance: &Mapping<AccountId, Balance>,
            _qty: Balance,
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            let my_balance = *_balance.get(&caller).unwrap_or(&0);

            match _qty {
                0 => Err(Error::ZeroAmount),
                _ if _qty > my_balance => Err(Error::InsufficientAmount),
                _ => Ok(()),
            }
        }

        // Returns the liquidity constant of the pool
        fn getK(&self) -> Balance {
            self.totalToken1 * self.totalToken2
        }

        // Used to restrict withdraw & swap feature till liquidity is added to the pool
        fn activePool(&self) -> Result<(), Error> {
            match self.getK() {
                0 => Err(Error::ZeroLiquidity),
                _ => Ok(()),
            }
        }
    }

    impl Dex {
        /// Constructs a new AMM instance
        /// @param _fees: valid interval -> [0,1000)
        #[ink(constructor)]
        pub fn new(_fees: Balance) -> Self {
            // Sets fees to zero if not in valid range
            Self {
                fees: if _fees >= 1000 { 0 } else { _fees },
                ..Default::default()
            }
        }

        /// Sends free token(s) to the invoker
        #[ink(message)]
        pub fn faucet(&mut self, _amountToken1: Balance, _amountToken2: Balance) {
            let caller = self.env().caller();
            let token1 = *self.token1Balance.get(&caller).unwrap_or(&0);
            let token2 = *self.token2Balance.get(&caller).unwrap_or(&0);

            self.token1Balance.insert(caller, token1 + _amountToken1);
            self.token2Balance.insert(caller, token2 + _amountToken2);
        }

        /// Returns the balance of the user
        #[ink(message)]
        pub fn getMyHoldings(&self) -> (Balance, Balance, Balance) {
            let caller = self.env().caller();
            let token1 = *self.token1Balance.get(&caller).unwrap_or(&0);
            let token2 = *self.token2Balance.get(&caller).unwrap_or(&0);
            let myShares = *self.shares.get(&caller).unwrap_or(&0);
            (token1, token2, myShares)
        }

        /// Returns the amount of tokens locked in the pool,total shares issued & trading fee param
        #[ink(message)]
        pub fn getPoolDetails(&self) -> (Balance, Balance, Balance, Balance) {
            (
                self.totalToken1,
                self.totalToken2,
                self.totalShares,
                self.fees,
            )
        }
    }
}
