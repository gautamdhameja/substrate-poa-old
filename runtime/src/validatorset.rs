use support::{decl_module, decl_storage, decl_event, StorageMap, ensure, dispatch::Result};
use rstd::prelude::*;
use system::{ensure_signed};
use session;

pub trait Trait: system::Trait + session::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as ValidatorSet {
		Validators get(validators) config(): map T::AccountId => T::SessionKey;
		AddProposals get(add_proposals): map (T::AccountId, T::SessionKey) => bool;
		Votes get(votes): map (T::AccountId, T::SessionKey) => Vec<T::AccountId>;
	}
	extra_genesis_skip_phantom_data_field;
}

decl_event!(
  pub enum Event<T> where AccountId = <T as system::Trait>::AccountId, 
  SessionKey = <T as consensus::Trait>::SessionKey {
	  // New validator proposed. First argument is the AccountId of proposer.
	  ValidatorProposed(AccountId, AccountId, SessionKey),
	  
	  // New validator added.
	  ValidatorAdded(AccountId, SessionKey),
  }
);

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event<T>() = default;

		/// Propose a new validator to be added.
		/// 
		/// Can only be called by an existing validator.
		pub fn propose_validator(origin, account_id: T::AccountId, session_key: T::SessionKey) -> Result {
			let who = ensure_signed(origin)?;
			ensure!(<Validators<T>>::exists(who.clone()), "Access Denied!");
			ensure!(!<Validators<T>>::exists(account_id.clone()), "Already a validator.");

			if <AddProposals<T>>::exists((account_id.clone(), session_key.clone())) {
				let votes = <Votes<T>>::get((account_id.clone(), session_key.clone()));
				let v = votes.into_iter().find(|x| x == &who);
				ensure!(v == None, "You have already proposed this validator.");
			} else {
				<AddProposals<T>>::insert((account_id.clone(), session_key.clone()), true);
			}

			<Votes<T>>::mutate((account_id.clone(), session_key.clone()), |vote_list| {
				vote_list.push(who.clone());
			});
			
			Self::deposit_event(RawEvent::ValidatorProposed(who, account_id, session_key));
			Ok(())
		}

		/// Verifies if all existing validators have proposed the new validator
		/// and then adds the new validator.
		/// 
		/// New validator's session key should be set in session module before calling this.
		pub fn resolve_add_validator(origin, account_id: T::AccountId, session_key: T::SessionKey) -> Result {
			let _who = ensure_signed(origin)?;

			ensure!(!<Validators<T>>::exists(account_id.clone()), "Already a validator.");
			ensure!(<AddProposals<T>>::exists((account_id.clone(), session_key.clone())), 
				"Proposal to add this validator does not exist.");
			
			let votes = <Votes<T>>::get((account_id.clone(), session_key.clone()));
			let current_count = <session::Module<T>>::validator_count();
			if votes.len() as u32 == current_count {
				Self::add_new_authority(account_id.clone(), session_key.clone())?;
			}

			Self::deposit_event(RawEvent::ValidatorAdded(account_id, session_key));
			Ok(())
		}

		/// Add a new validator using root/sudo privileges.
		/// 
		/// New validator's session key should be set in session module before calling this.
		pub fn add_validator(account_id: T::AccountId, session_key: T::SessionKey) -> Result {
			ensure!(!<Validators<T>>::exists(account_id.clone()), "Already a validator.");
			
			Self::add_new_authority(account_id.clone(), session_key.clone())?;

			Self::deposit_event(RawEvent::ValidatorAdded(account_id, session_key));
			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {
	// Adds new authority in the consensus module.
	fn add_new_authority(account_id: T::AccountId, session_key: T::SessionKey) -> Result {
		// Add new validator in session module.
		let mut current_validators = <session::Module<T>>::validators();
		current_validators.push(account_id.clone());
		<session::Module<T>>::set_validators(&current_validators);

		// rotate session for new set of validators to take effect
		<session::Module<T>>::rotate_session(true, false);
		<Validators<T>>::insert(account_id, session_key);
		
		Ok(())
	}
}