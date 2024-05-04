use super::*;

#[command_handler("TOPIC")]
async fn handle_topic(
    cmd: &dyn Command,
    server: &ClientServer,
    net: &Network,
    source: UserSource<'_>,
    response: &dyn CommandResponse,
    channel: wrapper::Channel<'_>,
    new_topic: Option<&str>,
) -> CommandResult {
    if let Some(text) = new_topic {
        server.policy().can_set_topic(&source, &channel, text)?;

        let details = event::details::NewChannelTopic {
            channel: channel.id(),
            text: text.to_owned(),
            setter: source.id().into(),
        };
        cmd.new_event_with_response(server.ids().next_channel_topic(), details)
            .await;
    } else if let Ok(topic) = net.topic_for_channel(channel.id()) {
        response.numeric(make_numeric!(TopicIs, &channel, topic.text()));
        response.numeric(make_numeric!(
            TopicSetBy,
            &channel,
            topic.setter(),
            topic.timestamp()
        ));
    } else {
        response.numeric(make_numeric!(NoTopic, &channel));
    }

    Ok(())
}
