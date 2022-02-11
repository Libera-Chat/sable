use crate::*;
use arrayvec::ArrayString;
use hashers::fnv::*;
use std::hash::{Hash,Hasher};


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