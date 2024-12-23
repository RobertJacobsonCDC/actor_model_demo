/*!

An `IncidenceReporter` subscribes to status change messages and records them to a CSV file.

*/

use std::{
    fs::File,
    path::PathBuf,
    io::Write
};

use serde::{Deserialize, Serialize};
use csv::Writer;

use actor_model::actor::{Actor, ActorHandle};

use crate::{
    message::{Channel, Envelope, Message, RcEnvelope, Topic},
    people::{InfectionStatus, PersonID}
};

#[derive(Serialize, Deserialize, Copy, Clone)]
struct IncidenceReportItem {
    time: f64,
    person_id: PersonID,
    infection_status: InfectionStatus,
}

pub struct IncidenceReporter {
    handle: ActorHandle,
    file_name: PathBuf,
    writer: Option<Writer<File>>,
}

impl IncidenceReporter {
    // Create a new IncidenceReporter with the given file name
    pub fn new(file_name: &str) -> Self {
        let mut new_reporter = IncidenceReporter {
            handle: 0,
            file_name: PathBuf::from(file_name),
            writer: None,
        };
        new_reporter.init_writer().expect("Failed to init file writer");
        // new_reporter.write_headers().expect("Failed to write headers");

        new_reporter
    }

    // Initialize the writer (creating or opening the CSV file)
    pub fn init_writer(&mut self) -> std::io::Result<()> {
        let file = File::create(&self.file_name)?;
        let writer = Writer::from_writer(file);
        self.writer = Some(writer);
        Ok(())
    }

    // Write the headers to the CSV (based on the fields of IncidenceReportItem)
    // pub fn write_headers(&mut self) -> std::io::Result<()> {
    //     if let Some(ref mut writer) = self.writer {
    //         writer.write_record(&["name", "incidence_rate", "date"])?;
    //     }
    //     Ok(())
    // }

    // Write a row of data from an IncidenceReportItem instance to the CSV
    pub fn write_row(&mut self, item: IncidenceReportItem) -> std::io::Result<()> {
        if let Some(ref mut writer) = self.writer {
            writer.serialize(item)?;
        }
        Ok(())
    }

    // Close the writer and finalize the CSV file
    pub fn finish(&mut self) -> std::io::Result<()> {
        if let Some(ref mut writer) = self.writer {
            writer.flush()?;
        }
        Ok(())
    }
}

impl Drop for IncidenceReporter {
    fn drop(&mut self) {
        self.finish().expect("Failed to finish");
    }
}

impl Actor<Message, Topic> for IncidenceReporter {
    fn receive_message(&mut self, envelope: RcEnvelope) -> Vec<RcEnvelope> {
        // In general, we have a method that responds to every message type we know how to answer.

        // The only messages we care about are when a person changes status.

        let messages =
            match *envelope {
                Envelope {
                    channel: Channel::Topic(Topic::PersonStatus),
                    message: Some(Message::PersonStatus(person_id, infection_status)),
                    time   : Some(time), // Time is set only when it's a transition.
                    ..
                }
                => {
                    // We write a row representing the status change
                    self.write_row(
                        IncidenceReportItem{
                            time: time.0,
                            person_id,
                            infection_status,
                        }
                    ).expect("Failed to write row");
                    vec![]
                }

                _ => {
                    // A status report we don't care about
                    vec![]
                }
            };

        #[cfg(feature = "print_messages")]
        for message in &messages {
            println!("INCIDENCE REPORTER: {:?}", message);
        }

        messages
    }

    fn register(&mut self, handle: ActorHandle) -> (Vec<Channel>, Vec<RcEnvelope>) {
        self.handle = handle;

        // We respond to infection status changes. We have no initial messages.
        let subscriptions = vec![
            Channel::Topic(Topic::PersonStatus),
        ];

        (subscriptions, vec![])
    }
}
