use std::{
    collections::{BTreeMap, VecDeque},
    io::Write,
};

use clap::Args;
use dora_core::config::InputMapping;
use dora_message::id::{DataId, NodeId};
use serde::Serialize;
use tabwriter::TabWriter;
use tokio::runtime::Builder;

use crate::{
    command::{
        Executable, default_tracing,
        topic::selector::{DataflowSelector, TopicSelector},
    },
    formatting::OutputFormat,
};

/// List topics.
#[derive(Debug, Args)]
pub struct List {
    #[clap(flatten)]
    selector: DataflowSelector,
    /// Output format
    #[clap(long, value_name = "FORMAT", default_value_t = OutputFormat::Table)]
    pub format: OutputFormat,
}
impl Executable for List {
    fn execute(self) -> eyre::Result<()> {
        default_tracing()?;

        list(self.selector, self.format)
    }
}

#[derive(Serialize)]
struct OutputEntry {
    node: NodeId,
    name: DataId,
    subscribers: Vec<String>,
}

fn list(selector: DataflowSelector, format: OutputFormat) -> eyre::Result<()> {
    let (_session, _dataflow_id, descriptor) = selector.resolve()?;

    let mut subscribers = BTreeMap::<(&NodeId, &DataId), Vec<(&NodeId, &DataId)>>::new();
    for node in &descriptor.nodes {
        for (data, input) in &node.inputs {
            if let InputMapping::User(user) = &input.mapping {
                subscribers
                    .entry((&user.source, &user.output))
                    .or_default()
                    .push((&node.id, data));
            }
        }
    }

    let mut entries = Vec::new();
    for node in &descriptor.nodes {
        for output in &node.outputs {
            entries.push(OutputEntry {
                node: node.id.clone(),
                name: output.clone(),
                subscribers: subscribers
                    .remove(&(&node.id, output))
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(node, data)| format!("{node}/{data}"))
                    .collect(),
            });
        }
    }

    match format {
        OutputFormat::Table => {
            let mut tw = TabWriter::new(std::io::stdout().lock());
            tw.write_all(b"Node\tName\tSubscribers\n")?;
            for entry in entries {
                tw.write_all(
                    format!(
                        "{}\t{}\t{}\n",
                        entry.node,
                        entry.name,
                        entry.subscribers.join(", ")
                    )
                    .as_bytes(),
                )?;
            }
            tw.flush()?;
        }
        OutputFormat::Json => {
            for entry in entries {
                println!("{}", serde_json::to_string(&entry)?);
            }
        }
    }

    Ok(())
}
