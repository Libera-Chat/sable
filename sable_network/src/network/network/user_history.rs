use super::*;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricUserStore {
    #[serde_as(as = "Vec<(_,_)>")]
    users: HashMap<HistoricUserId, state::HistoricUser>,
}

impl HistoricUserStore {
    /// Construct an empty store
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    /// Add a user to the store.
    ///
    /// This (or one of the update variants) should be called by the [`Network`] for every change
    /// to a client-protocol-visible attribute of the user object.
    ///
    /// The provided timestamp will be applied to the previous historic user corresponding to this
    /// user object, to indicate when it ceased to be relevant, and the user object's serial number
    /// will be incremented.
    pub fn add(
        &mut self,
        user: &mut state::User,
        timestamp: i64,
        nickname: Nickname,
        account: Option<Nickname>,
    ) {
        if let Some(existing) = self
            .users
            .get_mut(&HistoricUserId::new(user.id, user.serial))
        {
            existing.timestamp = Some(timestamp);
        }

        user.serial += 1;

        let historic_user = HistoricUser {
            id: user.id,
            serial: user.serial,
            nickname,
            user: user.user,
            visible_host: user.visible_host,
            realname: user.realname,
            away_reason: user.away_reason,
            account,
            timestamp: None,
        };

        let new_id = HistoricUserId::new(user.id, user.serial);

        self.users.insert(new_id, historic_user);
    }

    /// Update the details of a user that's already in the store, reusing the existing nickname and account
    pub fn update(&mut self, user: &mut state::User, timestamp: i64) -> HistoricUserId {
        let old_id = HistoricUserId::new(user.id, user.serial);

        let Some(existing) = self.get_user(user) else {
            return old_id;
        };

        let nickname = existing.nickname;
        let account = existing.account;

        self.add(user, timestamp, nickname, account);
        old_id
    }

    /// Update the details of a user that's already in the store, reusing the existing account
    pub fn update_nick(
        &mut self,
        user: &mut state::User,
        timestamp: i64,
        nickname: Nickname,
    ) -> HistoricUserId {
        let old_id = HistoricUserId::new(user.id, user.serial);

        let Some(existing) = self.get_user(user) else {
            return old_id;
        };

        let account = existing.account;

        self.add(user, timestamp, nickname, account);
        old_id
    }

    /// Update the details of a user that's already in the store, reusing the existing nickname
    pub fn update_account(
        &mut self,
        user: &mut state::User,
        timestamp: i64,
        account: Option<Nickname>,
    ) -> HistoricUserId {
        let old_id = HistoricUserId::new(user.id, user.serial);

        let Some(existing) = self.get_user(user) else {
            return old_id;
        };

        let nickname = existing.nickname;

        self.add(user, timestamp, nickname, account);
        old_id
    }

    /// Look up a historic user by ID
    pub fn get(&self, id: &HistoricUserId) -> Option<&HistoricUser> {
        self.users.get(id)
    }

    /// Get the historic user representing the current state of the given user object
    pub fn get_user(&self, user: &state::User) -> Option<&HistoricUser> {
        self.users.get(&HistoricUserId::new(user.id, user.serial))
    }

    /// Expire user objects whose last-relevant time is older than the given timestamp
    pub fn expire_users(
        &mut self,
        min_timestamp: i64,
    ) -> impl Iterator<Item = (HistoricUserId, HistoricUser)> + '_ {
        self.users
            .extract_if(move |_, user| user.timestamp.is_some_and(|ts| ts < min_timestamp))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricNickStore {
    data: HashMap<Nickname, VecDeque<HistoricUserId>>,
}

const WHOWAS_LENGTH: usize = 8;

impl HistoricNickStore {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Iterate over entries for a given nickname
    pub fn get<'a>(
        &'a self,
        nick: &Nickname,
        net: &'a Network,
    ) -> impl Iterator<Item = &'a HistoricUser> + 'a {
        self.data
            .get(nick)
            .map(move |vec| vec.iter().filter_map(move |id| net.historic_users.get(id)))
            .into_iter()
            .flatten()
    }

    /// Add an entry for the given nickname
    pub fn add(&mut self, nick: &Nickname, id: HistoricUserId) {
        let vec = self
            .data
            .entry(*nick)
            .or_insert_with(|| VecDeque::with_capacity(WHOWAS_LENGTH));

        if vec.len() == vec.capacity() {
            vec.pop_back();
        }

        vec.push_front(id);
    }

    /// Remove expired entries for which the provided predicate returns false
    pub fn retain(&mut self, pred: impl Fn(&HistoricUserId) -> bool) {
        for vec in self.data.values_mut() {
            vec.retain(&pred);
        }
    }
}
