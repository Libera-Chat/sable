use super::*;

impl<DB: DatabaseConnection> ServicesServer<DB> {
    pub(crate) fn register_user(&self, account_name: Nickname, password: String) -> CommandResult {
        let new_account_id = self.node.ids().next_account();

        let password_hash = match self.config.password_hash.hash(&password) {
            Ok(password_hash) => password_hash,
            Err(error) => {
                tracing::error!(
                    ?account_name,
                    "Failed to hash password for new account: {}",
                    error
                );

                return Err("Failed to hash password".into());
            }
        };

        let account_data = state::Account {
            id: new_account_id,
            name: account_name,
            authorised_fingerprints: Vec::new(),
        };
        let auth_data = AccountAuth {
            account: new_account_id,
            password_hash,
        };

        match self.db.new_account(account_data, auth_data) {
            Ok(new_account) => {
                tracing::debug!(?new_account, "Successfully created account");
                let id = new_account.id;
                self.node.submit_event(
                    id,
                    AccountUpdate {
                        data: Some(new_account),
                    },
                );
                Ok(RemoteServerResponse::LogUserIn(id))
            }
            Err(DatabaseError::DuplicateId | DatabaseError::DuplicateName) => {
                tracing::debug!(?account_name, "Duplicate account name/id");
                Ok(RemoteServerResponse::AlreadyExists)
            }
            Err(error) => {
                tracing::error!(?error, "Error creating account");
                Err("Unknown error".into())
            }
        }
    }

    pub(crate) fn user_login(&self, account_id: AccountId, password: String) -> CommandResult {
        let Ok(auth) = self.db.auth_for_account(account_id) else {
            tracing::error!(?account_id, "Error looking up account");
            return Err("Couldn't look up account".into());
        };

        match bcrypt::verify(password, &auth.password_hash) {
            Ok(true) => {
                tracing::debug!("login successful");
                Ok(RemoteServerResponse::LogUserIn(account_id))
            }
            Ok(false) => {
                tracing::debug!("wrong password");
                Ok(RemoteServerResponse::InvalidCredentials)
            }
            Err(_) => Err("Couldn't verify password".into()),
        }
    }

    pub(crate) fn user_add_fp(&self, account_id: AccountId, fp: String) -> CommandResult {
        if self.node.network().account_with_fingerprint(&fp).is_some() {
            return Err("Duplicate fingerprint".into());
        }

        let Ok(mut account) = self.db.account(account_id) else {
            tracing::error!(?account_id, "Error looking up account");
            return Err("Couldn't look up account".into());
        };

        account.authorised_fingerprints.push(fp);

        self.db.update_account(&account)?;
        self.node.submit_event(
            account.id,
            event::AccountUpdate {
                data: Some(account),
            },
        );

        Ok(RemoteServerResponse::Success)
    }

    pub(crate) fn user_del_fp(&self, account_id: AccountId, fp: String) -> CommandResult {
        let Ok(mut account) = self.db.account(account_id) else {
            tracing::error!(?account_id, "Error looking up account");
            return Err("Couldn't look up account".into());
        };

        account.authorised_fingerprints.retain(|f| f != &fp);

        self.db.update_account(&account)?;
        self.node.submit_event(
            account.id,
            event::AccountUpdate {
                data: Some(account),
            },
        );

        Ok(RemoteServerResponse::Success)
    }
}
