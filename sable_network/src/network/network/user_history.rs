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
    /// This should be called by the [`Network`] for every change to a client-protocol-visible attribute
    /// of the user object.
    pub fn add(&mut self, user: &mut state::User, nickname: Nickname, account: Option<Nickname>) {
        user.serial += 1;

        let historic_user = HistoricUser {
            id: user.id,
            nickname,
            user: user.user,
            visible_host: user.visible_host,
            realname: user.realname,
            away_reason: user.away_reason,
            account,
        };

        let new_id = HistoricUserId::new(user.id, user.serial);

        self.users.insert(new_id, historic_user);
    }

    /// Update the details of a user that's already in the store, reusing the existing nickname and account
    pub fn update(&mut self, user: &mut state::User) {
        let Some(existing) = self.get_user(&user) else {
            return;
        };

        let nickname = existing.nickname;
        let account = existing.account;

        self.add(user, nickname, account);
    }

    /// Update the details of a user that's already in the store, reusing the existing account
    pub fn update_nick(&mut self, user: &mut state::User, nickname: Nickname) {
        let Some(existing) = self.get_user(&user) else {
            return;
        };

        let account = existing.account;

        self.add(user, nickname, account);
    }

    /// Update the details of a user that's already in the store, reusing the existing nickname
    pub fn update_account(&mut self, user: &mut state::User, account: Option<Nickname>) {
        let Some(existing) = self.get_user(&user) else {
            return;
        };

        let nickname = existing.nickname;

        self.add(user, nickname, account);
    }

    /// Look up a historic user by ID
    pub fn get(&self, id: &HistoricUserId) -> Option<&HistoricUser> {
        self.users.get(id)
    }

    /// Get the historic user representing the current state of the given user object
    pub fn get_user(&self, user: &state::User) -> Option<&HistoricUser> {
        self.users.get(&HistoricUserId::new(user.id, user.serial))
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

    pub fn get<'a>(
        &'a self,
        nick: &Nickname,
        net: &'a Network,
    ) -> impl Iterator<Item = &HistoricUser> + 'a {
        self.data
            .get(nick)
            .map(move |vec| vec.iter().filter_map(move |id| net.historic_users.get(id)))
            .into_iter()
            .flatten()
    }

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
}
