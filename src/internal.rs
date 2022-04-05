use crate::*;


#[near_bindgen]
impl StakingContract {
    pub(crate) fn internal_unstake(&mut self, account_id: AccountId, amount: u128) {
        let upgradable_account = self.accounts.get(&account_id).unwrap();
        let mut account: Account = Account::from(upgradable_account);

        assert!(amount <= account.stake_balance, "ERR_AMOUNT_MUST_LESS_THAN_STAKE_BALANCE");

        let new_reward = self.internal_calculate_account_reward(&account);

        // update account data
        account.pre_reward += new_reward;
        account.stake_balance -= amount;
        account.last_block_balance_change = env::block_index();
        account.unstake_balance += amount;
        account.unstake_start_timestamp = env::block_timestamp();
        account.unstake_available_epoch = env::epoch_height();

        if account.stake_balance == 0 {
            self.total_staker -= 1;
        }

        self.accounts.insert(&account_id, &UpgradableAccount::from(account));

        let new_contract_reward = self.internal_calculate_global_reward();
        self.pre_reward += new_contract_reward;
        self.last_block_balance_change = env::block_index();
        self.total_stake_balance -= amount;
    }

    pub(crate) fn internal_withdraw(&mut self, account_id: AccountId) -> Account {
        let upgradable_account = self.accounts.get(&account_id).unwrap();
        let account = Account::from(upgradable_account);

        assert!(account.unstake_balance > 0, "ERR_UNSTAKE_BALANCE_EQUAL_ZERO");
        assert!(account.unstake_available_epoch <= env::epoch_height(), "ERR_DISABLED_WITHDRAW");

        let new_account = Account {
            stake_balance: account.stake_balance,
            pre_reward: account.pre_reward,
            last_block_balance_change: account.last_block_balance_change,
            unstake_balance: 0,
            unstake_start_timestamp: 0,
            unstake_available_epoch: 0,
            new_account_data: account.new_account_data
        };

        self.accounts.insert(&account_id, &UpgradableAccount::from(new_account));

        account
    }

    pub(crate) fn internal_deposit_and_stake(&mut self, account_id: AccountId, amount: u128) {
        // Validate data
        let upgradable_account = self.accounts.get(&account_id);
        assert!(upgradable_account.is_some(), "ERR_ACCOUNT_NOT_FOUND");
        assert_eq!(self.paused, false, "ERR_CONTRACT_PAUSED");
        assert_eq!(self.ft_contract_id, env::predecessor_account_id(), "ERR_INVALID_FT_CONTRACT_ID");

        let mut account = Account::from(upgradable_account.unwrap());

        if account.stake_balance == 0 {
            self.total_staker += 1;
        }

        let new_reward = self.internal_calculate_account_reward(&account);

        // update account data
        account.pre_reward += new_reward;
        account.stake_balance += amount;
        account.last_block_balance_change = env::block_index();

        self.accounts.insert(&account_id, &UpgradableAccount::from(account));


        // Update pool data
        let new_contract_reward = self.internal_calculate_global_reward();
        self.total_stake_balance += amount;
        self.pre_reward += new_contract_reward;
        self.last_block_balance_change = env::block_index();
    }

    pub(crate) fn internal_register_account(&mut self, account_id: AccountId) {
        let account = Account {
            stake_balance: 0,
            pre_reward: 0,
            last_block_balance_change: env::block_index(),
            unstake_balance: 0,
            unstake_start_timestamp: 0,
            unstake_available_epoch: 0,
            new_account_data: U128(0)
        };
    
        self.accounts.insert(&account_id, &UpgradableAccount::from(account));
    }

    pub(crate) fn internal_calculate_account_reward(&self, account: &Account) -> Balance {
        let lasted_block = if self.paused {
            self.pause_in_block
        } else {
            env::block_index()
        };

        let diff_block = lasted_block - account.last_block_balance_change;
        let reward: Balance = (account.stake_balance * self.config.reward_numerator as u128 * diff_block as u128) / self.config.reward_denumerator as u128;

        reward
    }

    pub(crate) fn internal_calculate_global_reward(&self) -> Balance {
        let lasted_block = if self.paused {
            self.pause_in_block
        } else {
            env::block_index()
        };

        let diff_block = lasted_block - self.last_block_balance_change;
        let reward: Balance = (self.total_stake_balance * self.config.reward_numerator as u128 * diff_block as u128) / self.config.reward_denumerator as u128;

        reward
    }
}