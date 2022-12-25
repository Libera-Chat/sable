use super::*;

#[command_handler("TOPIC")]
fn handle_topic(server: &ClientServer, net: &Network, source: UserSource, cmd: &dyn Command,
                channel: wrapper::Channel, new_topic: Option<&str>) -> CommandResult
{
    if let Some(text) = new_topic
    {
        server.policy().can_set_topic(&source, &channel, &text)?;

        let details = event::details::NewChannelTopic {
            channel: channel.id(),
            text: text.to_owned(),
            setter: source.id().into()
        };
        server.add_action(CommandAction::state_change(server.ids().next_channel_topic(), details));
    }
    else if let Ok(topic) = net.topic_for_channel(channel.id())
    {
        cmd.numeric(make_numeric!(TopicIs, &channel, topic.text()));
        cmd.numeric(make_numeric!(TopicSetBy, &channel, topic.setter(), topic.timestamp()));
    }
    else
    {
        cmd.numeric(make_numeric!(NoTopic, &channel));
    }

    Ok(())
}
