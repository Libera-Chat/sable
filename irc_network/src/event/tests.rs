use super::*;
use crate::id::*;
use async_std::channel;

fn drain_from(log: &mut channel::Receiver<Event>) -> Vec<Event>
{
    let mut v = Vec::new();
    
    while let Ok(e) = log.try_recv()
    {
        v.push(e);
    }
    v
}

#[test]
fn simple()
{
    let server_id = ServerId::new(1);
    let epoch_id = EpochId::new(1);
    let idgen = EventIdGenerator::new(server_id, epoch_id, 1);
    let (sender, mut receiver) = channel::unbounded::<Event>();
    let mut log = EventLog::new(idgen, Some(sender));

    let uid = UserId::new(server_id, 1);

    let e1 = log.create(uid, details::UserQuit{ message: "aaa".to_string() });
    log.add(e1.clone());
    
    let e2 = log.create(uid, details::UserQuit{ message: "bbb".to_string() });
    log.add(e2.clone());

    let entries = drain_from(&mut receiver);

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0], e1);
    assert_eq!(entries[1], e2);
}

#[test]
fn out_of_order()
{
    let server_id = ServerId::new(1);
    let epoch_id = EpochId::new(1);
    let idgen = EventIdGenerator::new(server_id, epoch_id, 1);
    let (sender, mut receiver) = channel::unbounded::<Event>();
    let mut log = EventLog::new(idgen, Some(sender));

    let uid = UserId::new(server_id, 1);

    let e1 = log.create(uid, details::UserQuit{ message: "aaa".to_string() });
    let mut e2 = log.create(uid, details::UserQuit{ message: "bbb".to_string() });

    e2.clock.update_with_id(e1.id);

    log.add(e2.clone());
    log.add(e1.clone());
    
    let entries = drain_from(&mut receiver);

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0], e1);
    assert_eq!(entries[1], e2);
}

#[test]
fn epochs()
{
    let server_id = ServerId::new(1);
    let epoch_one = EpochId::new(1);
    let epoch_two = EpochId::new(2);
    let idgen = EventIdGenerator::new(server_id, epoch_one, 1);
    let (sender, mut receiver) = channel::unbounded::<Event>();
    let mut log = EventLog::new(idgen, Some(sender));

    let uid = UserId::new(server_id, 1);

    let e1 = log.create(uid, details::UserQuit{ message: "aaa".to_string() });
    let mut e2 = log.create(uid, details::UserQuit{ message: "bbb".to_string() });

    log.set_epoch(epoch_two);

    let mut e3 = log.create(uid, details::UserQuit{ message: "ccc".to_string() });

    e2.clock.update_with_id(e1.id);
    e3.clock.update_with_id(e2.id);

    log.add(e2.clone());
    log.add(e3.clone());
    log.add(e1.clone());

    let entries = drain_from(&mut receiver);

    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0], e1);
    assert_eq!(entries[1], e2);
    assert_eq!(entries[2], e3);
}