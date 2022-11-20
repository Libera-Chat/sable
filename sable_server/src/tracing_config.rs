use crate::config::*;
use tracing_subscriber::{
    prelude::*,
    filter::filter_fn,
    Layer,
    registry::LookupSpan,
};
use tracing_core::LevelFilter;
use tracing::Subscriber;
use std::convert::Into;

use std::{
    io::{
        Error as IoError
    },
    path::Path,
};

fn build_target<S>(conf: LogEntry, dir: impl AsRef<Path>) -> Result<Box<dyn Layer<S> + Send + Sync + 'static>, IoError>
    where S: Subscriber + Send + Sync,
          for<'span> S: LookupSpan<'span>,

{
    let layer = match &conf.target
    {
        LogTarget::File { filename } =>
        {
            tracing_subscriber::fmt::layer()
                                     .with_writer(tracing_appender::rolling::daily(dir, filename))
                                     .with_ansi(false)
                                     .boxed()
        }
        LogTarget::Builtin(BuiltinLogTarget::Stdout) =>
        {
            tracing_subscriber::fmt::layer().with_writer(std::io::stdout).boxed()
        }
        LogTarget::Builtin(BuiltinLogTarget::Stderr) =>
        {
            tracing_subscriber::fmt::layer().with_writer(std::io::stderr).boxed()
        }
    };

    let filter = filter_fn(move |metadata| {
        let level: tracing_core::LevelFilter = if let Some(level) = conf.level { level.into() } else { LevelFilter::TRACE };
        metadata.level() <= &level &&
            (
                conf.modules.is_empty() ||
                    if let Some(module) = metadata.module_path() {
                        conf.modules.iter().any(|m| module.starts_with(m))
                    } else {
                        true
                    }
            )
    });

    Ok(layer.with_filter(filter).boxed())
}

pub fn build_subscriber(conf: LoggingConfig) -> Result<impl Subscriber, IoError>
{
    let mut layers = Vec::new();

    for target in conf.targets
    {
        layers.push(build_target(target, &conf.dir)?);
    }

    // The global filter is for excluding overly verbose messages from external modules - its default
    // needs to be permissive so that individual log targets can filter as they need to
    let filter = tracing_subscriber::filter::Targets::new()
                    .with_default(conf.default_level.unwrap_or(LogLevel::Trace))
                    .with_targets(conf.module_levels);

    let console = conf.console_address.map(|addr| console_subscriber::ConsoleLayer::builder().server_addr(addr).spawn());

    Ok(tracing_subscriber::registry()
            .with(console)
            .with(filter)
            .with(layers)
        )
}