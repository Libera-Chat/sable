use std::{path::PathBuf, collections::hash_map::Entry, ops::DerefMut};
use serde::{Serialize,Deserialize};
use serde_with::serde_as;
use std::{
    fs::File,
    collections::HashMap,
    ops::Deref,
};
use parking_lot::{RwLock, RwLockReadGuard};

use super::*;

/// A simple JSON file-backed database for testing and demonstration purposes
///
/// This is not intended to perform adequately under significant loads.
pub struct JsonDatabase
{
    filename: PathBuf,

    state: RwLock<JsonDatabaseState>,
}

#[serde_as]
#[derive(Serialize,Deserialize,Default)]
struct JsonDatabaseState
{
    #[serde_as(as = "Vec<(_,_)>")]
    accounts: HashMap<AccountId, state::Account>,

    #[serde_as(as = "Vec<(_,_)>")]
    account_auth: HashMap<AccountId, AccountAuth>,

    #[serde_as(as = "Vec<(_,_)>")]
    nick_registrations: HashMap<NickRegistrationId, state::NickRegistration>,

    #[serde_as(as = "Vec<(_,_)>")]
    channel_registrations: HashMap<ChannelRegistrationId, state::ChannelRegistration>,

    #[serde_as(as = "Vec<(_,_)>")]
    channel_accesses: HashMap<ChannelAccessId, state::ChannelAccess>,
}

#[ouroboros::self_referencing]
pub struct LockedHashMapValueIterator<'a, K, V>
    where K: 'static, V: 'static
{
    lock: RwLockReadGuard<'a, JsonDatabaseState>,
    #[borrows(lock)]
    #[covariant]
    iter: std::collections::hash_map::Values<'this, K, V>
}

impl<'a, K, V> Iterator for LockedHashMapValueIterator<'a, K, V>
    where K: 'static, V: 'static + Clone
{
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        self.with_iter_mut(|iter| iter.next().cloned())
    }
}

impl JsonDatabase
{
    fn save(&self) -> super::Result<()>
    {
        let state = self.state.read();

        let file = File::create(&self.filename).map_err(DatabaseError::from_inner)?;

        serde_json::to_writer(file, state.deref()).unwrap();
        Ok(())
    }
}

impl DatabaseConnection for JsonDatabase
{
    fn connect(conn: String) -> Result<Self>
    {
        let filename = conn.into();

        if let Ok(file) = File::open(&filename)
        {
            let state = serde_json::from_reader(file).map_err(DatabaseError::from_inner)?;

            Ok(Self {
                filename,
                state
            })
        }
        else
        {
            tracing::warn!("Couldn't open database file, starting from empty");
            Ok(Self {
                filename,
                state: Default::default()
            })
        }
    }

    fn new_account(&self, data: state::Account, mut auth: AccountAuth) -> Result<state::Account>
    {
        let mut state_guard = self.state.write();
        // Get the raw mut reference out of the guard so we can mutably borrow multiple fields
        let state = state_guard.deref_mut();
        let account_entry = state.accounts.entry(data.id);
        let auth_entry = state.account_auth.entry(data.id);

        // Just in case
        auth.account = data.id;

        let result = match (account_entry, auth_entry)
        {
            (Entry::Vacant(account_entry), Entry::Vacant(auth_entry)) => {
                auth_entry.insert(auth);
                Ok(account_entry.insert(data).clone())
            }
            _ => Err(DatabaseError::DuplicateId)
        };

        drop(state_guard);

        self.save()?;

        result
    }

    fn account(&self, id: AccountId) -> Result<state::Account>
    {
        self.state.read().accounts.get(&id).ok_or(DatabaseError::NoSuchId).cloned()
    }

    fn update_account(&self, new_data: &state::Account) -> Result<()>
    {
        let ret = match self.state.write().accounts.entry(new_data.id)
        {
            Entry::Occupied(mut entry) => {
                entry.insert(new_data.clone());
                Ok(())
            }
            Entry::Vacant(_) => Err(DatabaseError::NoSuchId)
        };

        self.save()?;
        ret
    }

    fn all_accounts(&self) -> Result<impl Iterator<Item=state::Account> + '_>
    {
        Ok(LockedHashMapValueIterator::new(self.state.read(), |state| state.accounts.values()))
    }

    fn auth_for_account(&self, id: AccountId) -> Result<AccountAuth>
    {
        self.state.read().account_auth.get(&id).ok_or(DatabaseError::NoSuchId).cloned()
    }

    fn new_nick_registration(&self, data: state::NickRegistration) -> Result<state::NickRegistration>
    {
        let ret = match self.state.write().nick_registrations.entry(data.id)
        {
            Entry::Occupied(_) => Err(DatabaseError::DuplicateId),
            Entry::Vacant(entry) => Ok(entry.insert(data).clone())
        };

        self.save()?;
        ret
    }

    fn nick_registration(&self, id: NickRegistrationId) -> Result<state::NickRegistration>
    {
        self.state.read().nick_registrations.get(&id).ok_or(DatabaseError::NoSuchId).cloned()
    }

    fn update_nick_registration(&self, new_data: &state::NickRegistration) -> Result<()>
    {
        let ret = match self.state.write().nick_registrations.entry(new_data.id)
        {
            Entry::Occupied(mut entry) => {
                entry.insert(new_data.clone());
                Ok(())
            }
            Entry::Vacant(_) => Err(DatabaseError::NoSuchId)
        };

        self.save()?;
        ret
    }

    fn all_nick_registrations(&self) -> Result<impl Iterator<Item=state::NickRegistration> + '_>
    {
        Ok(LockedHashMapValueIterator::new(self.state.read(), |state| state.nick_registrations.values()))
    }

    fn new_channel_registration(&self, data: state::ChannelRegistration, initial_access: state::ChannelAccess) -> Result<(state::ChannelRegistration, state::ChannelAccess)>
    {
        if initial_access.id.channel() != data.id
        {
            return Err(DatabaseError::InvalidData);
        }

        let mut state = self.state.write();
        let registration_entry = state.channel_registrations.entry(data.id);

        match registration_entry
        {
            Entry::Occupied(_) => Err(DatabaseError::DuplicateId),
            Entry::Vacant(entry) => {
                let ret = entry.insert(data).clone();

                // We know the access entry won't already exist because the channel registration didn't
                state.channel_accesses.insert(initial_access.id, initial_access.clone());

                drop(state);

                self.save()?;
                Ok((ret, initial_access))
            }
        }
    }

    fn channel_registration(&self, id: ChannelRegistrationId) -> Result<state::ChannelRegistration>
    {
        self.state.read().channel_registrations.get(&id).ok_or(DatabaseError::NoSuchId).cloned()
    }

    fn update_channel_registration(&self, new_data: &state::ChannelRegistration) -> Result<()>
    {
        let ret = match self.state.write().channel_registrations.entry(new_data.id)
        {
            Entry::Occupied(mut entry) => {
                entry.insert(new_data.clone());
                Ok(())
            }
            Entry::Vacant(_) => Err(DatabaseError::NoSuchId)
        };
        self.save()?;
        ret
    }

    fn all_channel_registrations(&self) -> Result<impl Iterator<Item=state::ChannelRegistration> + '_>
    {
        Ok(LockedHashMapValueIterator::new(self.state.read(), |state| state.channel_registrations.values()))
    }

    fn new_channel_access(&self, data: state::ChannelAccess) -> Result<state::ChannelAccess>
    {
        let ret = match self.state.write().channel_accesses.entry(data.id)
        {
            Entry::Occupied(_) => Err(DatabaseError::DuplicateId),
            Entry::Vacant(entry) => Ok(entry.insert(data).clone())
        };
        self.save()?;
        ret
    }

    fn channel_access(&self, id: ChannelAccessId) -> Result<state::ChannelAccess>
    {
        self.state.read().channel_accesses.get(&id).ok_or(DatabaseError::NoSuchId).cloned()
    }

    fn update_channel_access(&self, new_data: &state::ChannelAccess) -> Result<()>
    {
        let ret = match self.state.write().channel_accesses.entry(new_data.id)
        {
            Entry::Occupied(mut entry) => {
                entry.insert(new_data.clone());
                Ok(())
            }
            Entry::Vacant(_) => Err(DatabaseError::NoSuchId)
        };
        self.save()?;
        ret
    }

    fn all_channel_accesses(&self) -> Result<impl Iterator<Item=state::ChannelAccess> + '_>
    {
        Ok(LockedHashMapValueIterator::new(self.state.read(), |state| state.channel_accesses.values()))
    }
}