use crate::prelude::*;
use arrayvec::ArrayString;
use hashers::fnv::*;
use std::hash::{Hash,Hasher};

/// Generate a hashed nickname for the given user ID, to be used
/// for the loser of a nickname collision
pub(crate) fn hashed_nick_for(id: UserId) -> Nickname
{
    let mut hasher = FNV1aHasher32::default();
    id.hash(&mut hasher);
    let hash = hasher.finish().to_string();
    let mut newnick = ArrayString::new();
    for c in hash.chars()
    {
        // If we fill up the arraystring, just stop adding. We won't, though, as long as
        // the hash is 32-bit and the nicklen is >10
        if newnick.try_push(c).is_err()
        {
            break;
        }
    }
    Nickname::new_for_collision(newnick).unwrap()
}

/// Generate a hashed channel name for the given channel ID, to be used
/// for the loser of a name collision
pub(crate) fn hashed_channel_name_for(id: ChannelId) -> ChannelName
{
    let mut hasher = FNV1aHasher32::default();
    id.hash(&mut hasher);
    let hash = hasher.finish().to_string();
    let mut name = ArrayString::from("&").unwrap();
    for c in hash.chars()
    {
        // If we fill up the arraystring, just stop adding. We won't, though, as long as
        // the hash is 32-bit and the channellen is >10
        if name.try_push(c).is_err()
        {
            break;
        }
    }
    ChannelName::new(name).unwrap()
}
