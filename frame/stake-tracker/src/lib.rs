// This file is part of Substrate.

// Copyright (C) 2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::pallet::Pallet;
use frame_election_provider_support::VoteWeight;
use frame_support::traits::{Currency, CurrencyToVote, Defensive};
use pallet::Config;
use sp_runtime::DispatchResult;
use sp_staking::{OnStakingUpdate, Stake, StakingInterface};

/// The balance type of this pallet.
pub type BalanceOf<T> = <<T as Config>::Staking as StakingInterface>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use crate::*;
	use frame_election_provider_support::{SortedListProvider, VoteWeight};
	use sp_staking::StakingInterface;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// This has to come from Staking::Currency
		type Currency: Currency<Self::AccountId>;

		type Staking: StakingInterface;

		type VoterList: SortedListProvider<Self::AccountId, Score = VoteWeight>;

		type TargetList: SortedListProvider<Self::AccountId, Score = VoteWeight>;
	}
}

impl<T: Config> Pallet<T> {
	/// The total balance that can be slashed from a stash account as of right now.
	pub(crate) fn slashable_balance_of(who: &T::AccountId) -> BalanceOf<T> {
		// Weight note: consider making the stake accessible through stash.
		T::Staking::stake(who).map(|l| l.active).unwrap_or_default()
	}

	pub(crate) fn to_vote(balance: BalanceOf<T>) -> VoteWeight {
		let total_issuance = T::Currency::total_issuance();
		T::Staking::CurrencyToVote::to_vote(balance, total_issuance)
	}
}

impl<T: Config> OnStakingUpdate<T::AccountId, BalanceOf<T>> for Pallet<T> {
	fn on_update_ledger(
		who: &T::AccountId,
		prev_stake: Stake<T::AccountId, BalanceOf<T>>,
	) -> DispatchResult {
		let prev_active = prev_stake.map(|s| s.active);
		let current_stake = T::Staking::stake(who)?;

		let update_target_list = |who: &T::AccountId| {
			use sp_std::cmp::Ordering;
			match ledger.active.cmp(&prev_active) {
				Ordering::Greater => {
					let _ = T::TargetList::on_increase(who, current_stake.active - prev_active)
						.defensive();
				},
				Ordering::Less => {
					let _ = T::TargetList::on_decrease(who, prev_active - current_stake.active)
						.defensive();
				},
				Ordering::Equal => Ok(()),
			};
		};

		// if this is a nominator
		if let Some(targets) = T::Staking::nominations(&current_stake.stash) {
			// update the target list.
			for target in targets {
				update_target_list(&target)?;
			}

			// update the voter list.
			let _ =
				T::VoterList::on_update(&current_stake.stash, Self::to_vote(current_stake.active))
					.defensive_proof("any nominator should have an entry in the voter list.")?;
		}

		if T::Staking::is_validator(&current_stake.stash) {
			update_target_list(&current_stake.stash)?;

			let _ =
				T::VoterList::on_update(&current_stake.stash, Self::to_vote(current_stake.active))
					.defensive_proof("any validator should have an entry in the voter list.")?;
		}

		Ok(())
	}

	fn on_nominator_add(who: &T::AccountId, prev_nominations: Vec<T::AccountId>) -> DispatchResult {
		// if Some(nominations) = T::Staking::nominations(who) {
		// 	return Ok(())
		// }
		// T::VoterList::on_insert(who.clone(), Self::weight_of(stash)).defensive();
		Ok(())
	}

	fn on_validator_add(who: &T::AccountId) -> DispatchResult {
		todo!()
	}

	fn on_validator_remove(who: &T::AccountId) -> DispatchResult {
		todo!()
	}

	fn on_nominator_remove(who: &T::AccountId, nominations: Vec<T::AccountId>) -> DispatchResult {
		todo!()
	}

	fn on_reaped(who: &T::AccountId) -> DispatchResult {
		todo!()
	}
}