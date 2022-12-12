use super::*;
use sable_network::prelude::*;

impl<DB: DatabaseConnection> ServicesServer<DB>
{
    pub(crate) fn register_user(&self, account_name: Nickname, password: String) -> RemoteServerResponse
    {
        let new_account_id = self.node.ids().next_account();

        let Ok(password_hash) = bcrypt::hash(password, bcrypt::DEFAULT_COST) else {
            tracing::error!(?account_name, "Failed to hash password for new account");

            return RemoteServerResponse::Error("Failed to hash password".to_string());
        };

        let account_data = state::Account {
            id: new_account_id,
            name: account_name,
        };
        let auth_data = AccountAuth {
            account: new_account_id,
            password_hash
        };

        match self.db.new_account(account_data, auth_data)
        {
            Ok(new_account) =>
            {
                tracing::debug!(?new_account, "Successfully created account");
                let id = new_account.id;
                self.node.submit_event(id, AccountUpdate { data: Some(new_account) });
                RemoteServerResponse::LogUserIn(id)
            }
            Err(DatabaseError::DuplicateId | DatabaseError::DuplicateName) =>
            {
                tracing::debug!(?account_name, "Duplicate account name/id");
                RemoteServerResponse::AlreadyExists
            }
            Err(error) =>
            {
                tracing::error!(?error, "Error creating account");
                RemoteServerResponse::Error("Unknown error".to_string())
            }
        }
    }

    pub(crate) fn user_login(&self, account_id: AccountId, password: String) -> RemoteServerResponse
    {
        let Ok(auth) = self.db.auth_for_account(account_id) else {
            tracing::error!(?account_id, "Error looking up account");
            return RemoteServerResponse::Error("Couldn't look up account".to_string());
        };

        match bcrypt::verify(password, &auth.password_hash)
        {
            Ok(true) => {
                tracing::debug!("login successful");
                RemoteServerResponse::LogUserIn(account_id)
            }
            Ok(false) => {
                tracing::debug!("wrong password");
                RemoteServerResponse::InvalidCredentials
            }
            Err(_) => RemoteServerResponse::Error("Couldn't verify password".to_string())
        }
    }
}