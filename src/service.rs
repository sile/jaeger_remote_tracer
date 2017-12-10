use std::collections::HashMap;
use std::thread;
use std::time::SystemTime;
use fibers::sync::mpsc;
use futures::{Future, Stream, Poll, Async};
use rustracing;
use rustracing::tag::Tag;
use rustracing_jaeger::{Span, Tracer};
use rustracing_jaeger::reporter::JaegerCompactReporter;
use trackable::error::Failure;

use util;

pub type SpanId = u64;

#[derive(Debug)]
pub enum Command {
    StartSpan {
        span_id: u64,
        operation_name: String,
        child_of: Option<u64>,
        follows_from: Option<u64>,
        tags: Vec<(String, String)>,
        time: Option<SystemTime>,
    },
    Finish {
        span_id: u64,
        finish_time: Option<f64>,
    },
}

#[derive(Debug)]
pub struct Service {
    tracer: Tracer,
    spans: HashMap<u64, Span>,
    command_tx: mpsc::Sender<Command>,
    command_rx: mpsc::Receiver<Command>,
}
impl Service {
    pub fn new(reporter: JaegerCompactReporter) -> Self {
        let sampler = rustracing::sampler::AllSampler;
        let (tracer, span_rx) = Tracer::new(sampler);

        thread::spawn(move || while let Ok(span) = span_rx.recv() {
            reporter.report(&[span]).expect("Cannot send report");
        });

        let (tx, rx) = mpsc::channel();
        Service {
            tracer,
            spans: HashMap::new(),
            command_tx: tx,
            command_rx: rx,
        }
    }
    pub fn handle(&self) -> ServiceHandle {
        ServiceHandle { command_tx: self.command_tx.clone() }
    }

    fn handle_command(&mut self, command: Command) {
        match command {
            Command::StartSpan {
                span_id,
                operation_name,
                child_of,
                follows_from,
                tags,
                time,
            } => {
                let mut state = None;
                let mut span = self.tracer.span(operation_name);
                if let Some(ref_span) = child_of.and_then(|id| self.spans.get(&id)) {
                    span = span.child_of(ref_span);
                    state = Some(
                        format!(
                            "{}:{:x}:0:1",
                            ref_span.context().unwrap().state().trace_id(),
                            span_id
                        ).parse()
                            .unwrap(),
                    );
                }
                if let Some(ref_span) = follows_from.and_then(|id| self.spans.get(&id)) {
                    span = span.follows_from(ref_span);
                    if state.is_none() {
                        state = Some(
                            format!(
                                "{}:{:x}:0:1",
                                ref_span.context().unwrap().state().trace_id(),
                                span_id
                            ).parse()
                                .unwrap(),
                        );
                    }
                }
                if let Some(time) = time {
                    span = span.start_time(time);
                }
                for (k, v) in tags {
                    span = span.tag(Tag::new(k, v));
                }

                let state = state.unwrap_or_else(|| {
                    format!("{:x}:{:x}:0:1", span_id, span_id).parse().unwrap()
                });
                self.spans.insert(span_id, span.start_with_state(state));
            }
            Command::Finish {
                span_id,
                finish_time,
            } => {
                if let Some(mut span) = self.spans.remove(&span_id) {
                    if let Some(finish_time) = finish_time {
                        span.set_finish_time(|| util::unixtime_to_systemtime(finish_time));
                    }
                }
            }
        }
    }
}
impl Future for Service {
    type Item = ();
    type Error = Failure;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Async::Ready(Some(command)) = self.command_rx.poll().unwrap() {
            self.handle_command(command);
        }
        Ok(Async::NotReady)
    }
}

#[derive(Debug, Clone)]
pub struct ServiceHandle {
    command_tx: mpsc::Sender<Command>,
}
impl ServiceHandle {
    pub fn send_command(&self, command: Command) {
        let _ = self.command_tx.send(command);
    }
    pub fn finish(&self, span_id: u64, finish_time: Option<f64>) {
        let _ = self.command_tx.send(Command::Finish {
            span_id,
            finish_time,
        });
    }
}
