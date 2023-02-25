#![warn(rust_2018_idioms)]
#![warn(clippy::pedantic)]

use std::env;
use std::error;
use std::fs::File;

use clap::{crate_name, crate_version, Arg, Command};
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct EventArg {
    #[serde(default)]
    function_args: String,
    location: String,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
enum EventType {
    #[serde(rename = "B")]
    Begin,
    #[serde(rename = "E")]
    End,
}

#[derive(Debug, Deserialize, Serialize)]
struct Event {
    #[serde(default)]
    args: Option<EventArg>,
    #[serde(default)]
    cat: String,
    #[serde(default)]
    name: String,
    ph: EventType,
    pid: u32,
    tid: u32,
    ts: u32,
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let matches = Command::new(crate_name!())
        .version(crate_version!())
        .arg(Arg::new("f").takes_value(true))
        .get_matches();

    if let Some(value) = matches.value_of("f") {
        let mut stack = Vec::<Event>::new();
        let file = File::open(value)?;
        let deserializer = Deserializer::from_reader(file);

        let stream = deserializer.into_iter::<Vec<Event>>();

        for value in stream {
            let value = value?;
            //     match value {
            //         Value::Array(ref array) => for a in array {},
            //         Value::Number(ref num) => {}
            //         Value::Object(ref obj) => {}
            //         _ => panic!("not considered"),
            //     }
            for event in value {
                // println!("{:?}", event);
                if event.ph == EventType::Begin {
                    stack.push(event);
                } else if event.ph == EventType::End {
                    let trace = stack
                        .iter()
                        .map(|e| e.name.clone())
                        .collect::<Vec<String>>()
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
