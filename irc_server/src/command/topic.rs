use super::*;

command_handler!("TOPIC" => TopicHandler {
    fn min_parameters(&self) -> usize { 1 }

    fn handle_user(&mut self, source: &wrapper::User, cmd: &ClientCommand) -> CommandResult
    {
        let channel_name = ChannelName::from_str(&cmd.args[0])?;
        let channel = self.server.network().channel_by_name(&channel_name)?;

        if cmd.args.len() > 1
        {
            let text = cmd.args[1].clone();

            self.server.policy().can_set_topic(source, &channel, &text)?;

            let details = event::details::NewChannelTopic {
                channel: channel.id(),
                text: text,
                setter: source.id().into()
            };
            self.action(CommandAction::state_change(self.server.ids().next_channel_topic(), details))?;
        }
        else
        {
            if let Ok(topic) = self.server.network().topic_for_channel(channel.id())
            {
                cmd.response(&numeric::TopicIs::new(&channel, topic.text()))?;
                cmd.response(&numeric::TopicSetBy::new(&channel, topic.setter(), topic.timestamp()))?;
            }
            else
            {
                cmd.response(&numeric::NoTopic::new(&channel))?;
            }
        }

        Ok(())
    }
});