use itertools::Itertools;

mod data;
mod utils;

use pretty_assertions::assert_eq;
use sable_network::prelude::*;
use utils::stringify;

#[ignore]
#[test]
fn event_reordering() {
    tracing_subscriber::fmt::init();

    // It's important to build this first and clone for each ordering, so that the
    // hash parameters for all the networks are the same and their elements end up
    // in the same order
    let empty_network = utils::empty_network();

    let events = data::sample_events::sample_events();
    let mut orderings = events.iter().permutations(events.len()).enumerate();

    let mut reference_network = empty_network.clone();
    build_network_from(&mut reference_network, orderings.next().unwrap().1);

    for (num, ordering) in orderings
    //    let num = 720;
    //    let ordering = events.iter().permutations(events.len()).nth(num).unwrap();
    {
        println!("=== Ordering {} ===", num);
        let mut test_network = empty_network.clone();
        build_network_from(&mut test_network, ordering);
        assert_eq!(stringify(&test_network), stringify(&reference_network));
    }
}

fn build_network_from<'a>(network: &mut Network, events: impl IntoIterator<Item = &'a Event>) {
    let id_generator = ObjectIdGenerator::new(ServerId::new(0));
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
    let mut event_log = EventLog::new(id_generator, Some(sender));
    event_log.set_clock(data::sample_events::initial_clock());
    let noop_receiver = utils::receiver::NoOpUpdateReceiver;

    for event in events {
        println!("Adding id {:?} to log", event.id);
        event_log.add(event.clone());
    }

    while let Ok(event) = receiver.try_recv() {
        println!("Got id {:?} from log", event.id);
        network.apply(&event, &noop_receiver).unwrap();
    }
}
