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
    #[serde(borrow)]
    detail: Option<Cow<'a, str>>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
enum EventType {
    #[serde(rename = "B")]
    Begin,
    #[serde(rename = "E")]
    End,
    #[serde(rename = "X")]
    Complete,
    #[serde(rename = "i")]
    Instant,
    #[serde(rename = "C")]
    Counter,
    #[serde(rename = "P")]
    Sample,
    #[serde(rename = "M")]
    Metadata,
    #[serde(rename = "R")]
    Mark,
    #[serde(rename = "c")]
    ClockSync,
}

#[derive(Debug, Deserialize, Serialize)]
struct Event<'a> {
    #[serde(default)]
    args: Option<EventArg<'a>>,
    /// event category
    #[serde(default)]
    cat: String,
    #[serde(default)]
    #[serde(borrow)]
    name: Cow<'a, str>,
    /// event type
    ph: EventType,
    /// process ID
    pid: u32,
    /// thread ID
    tid: u32,
    /// timestamp in microseconds
    ts: u32,
    /// duration in microseconds, exists only for events of type [`EventType::Complete`]
    dur: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Top<'a> {
    #[serde(borrow)]
    trace_events: Vec<Event<'a>>,
}

fn process_event<'a>(event: Event<'a>, stack: &mut Vec<Event<'a>>) {
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
    } else if event.ph == EventType::Complete {
        println!(
            "{} {} {}",
            event.name,
            if let Some(ref a) = event.args {
                if let Some(ref d) = a.detail {
                    <Cow<'_, str> as Borrow<str>>::borrow(d)
                } else {
                    ""
                }
            } else {
                ""
            },
            event.dur.unwrap()
        );
    }
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let matches = Command::new(crate_name!())
        .version(crate_version!())
        .arg(
            Arg::new("f")
                .required(true)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("object")
                .required(false)
                .long("object")
                .num_args(0),
        )
        .get_matches();

    if let Some(value) = matches.get_one::<PathBuf>("f") {
        let mut stack = Vec::<Event<'_>>::new();
        let file = File::open(value)?;
        let deserializer = Deserializer::from_reader(&file);

        if matches.contains_id("object") {
            let deserializer = Deserializer::from_reader(&file);
            let stream = deserializer.into_iter::<Top<'_>>();
            for value in stream {
                for event in value?.trace_events {
                    process_event(event, &mut stack);
                }
            }
        } else {
            let stream = deserializer.into_iter::<Vec<Event<'_>>>();
            for value in stream {
                for event in value? {
                    process_event(event, &mut stack);
                }
            }
        }
    }

    Ok(())
}
