#![warn(rust_2018_idioms)]
#![warn(clippy::pedantic)]

use std::borrow::{Borrow, Cow};
use std::env;
use std::error;
use std::fs::File;
use std::path::PathBuf;

use clap::{crate_name, crate_version, value_parser, Arg, Command};
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct EventArg<'a> {
    #[serde(default)]
    #[serde(borrow)]
    function_args: Cow<'a, str>,
    #[serde(borrow)]
    location: Option<Cow<'a, str>>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
enum EventType {
    #[serde(rename = "B")]
    Begin,
    #[serde(rename = "E")]
    End,
}

#[derive(Debug, Deserialize, Serialize)]
struct Event<'a> {
    #[serde(default)]
    args: Option<EventArg<'a>>,
    #[serde(default)]
    cat: String,
    #[serde(default)]
    #[serde(borrow)]
    name: Cow<'a, str>,
    ph: EventType,
    pid: u32,
    tid: u32,
    ts: u32,
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let matches = Command::new(crate_name!())
        .version(crate_version!())
        .arg(
            Arg::new("f")
                .required(true)
                .value_parser(value_parser!(PathBuf)),
        )
        .get_matches();

    if let Some(value) = matches.get_one::<PathBuf>("f") {
        let mut stack = Vec::<Event<'_>>::new();
        let file = File::open(value)?;
        let deserializer = Deserializer::from_reader(file);

        let stream = deserializer.into_iter::<Vec<Event<'_>>>();

        for value in stream {
            for event in value? {
                if event.ph == EventType::Begin {
                    stack.push(event);
                } else if event.ph == EventType::End {
                    let trace = stack
                        .iter()
                        .map(|e| e.name.borrow())
                        .collect::<Vec<&str>>()
                        .join(";");
                    if let Some(begin_event) = stack.pop() {
                        println!("{} {}", trace, event.ts - begin_event.ts);
                    }
                }
            }
        }
    }

    Ok(())
}
