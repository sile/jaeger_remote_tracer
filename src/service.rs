use std::collections::HashMap;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use fibers::sync::mpsc;
use futures::{Future, Stream, Poll, Async};
use rustracing_jaeger::Span;
use rustracing_jaeger::reporter::JaegerCompactReporter;
use rustracing_jaeger::span::SpanReceiver;
use trackable::error::Failure;

pub type SpanId = u64;

fn unixtime_to_systemtime(unixtime: f64) -> SystemTime {
    let duration = Duration::new(unixtime as u64, (unixtime * 1_000_000_000.0) as u32);
    UNIX_EPOCH + duration
}

#[derive(Debug)]
pub enum Command {
    StartSpan { client_span_id: u64, span: Span },
    Finish {
        client_span_id: u64,
        finish_time: Option<f64>,
    },
}

#[derive(Debug)]
pub struct Service {
    spans: HashMap<u64, Span>,
    command_tx: mpsc::Sender<Command>,
    command_rx: mpsc::Receiver<Command>,
}
impl Service {
    pub fn new(span_rx: SpanReceiver, reporter: JaegerCompactReporter) -> Self {
        thread::spawn(move || while let Ok(span) = span_rx.recv() {
            reporter.report(&[span]).expect("Cannot send report");
        });

        let (tx, rx) = mpsc::channel();
        Service {
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
                client_span_id,
                span,
            } => {
                self.spans.insert(client_span_id, span);
            }
            Command::Finish {
                client_span_id,
                finish_time,
            } => {
                if let Some(mut span) = self.spans.remove(&client_span_id) {
                    if let Some(finish_time) = finish_time {
                        span.set_finish_time(|| unixtime_to_systemtime(finish_time));
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
    pub fn start_span(&self, client_span_id: u64, span: Span) {
        let _ = self.command_tx.send(Command::StartSpan {
            client_span_id,
            span,
        });
    }
    pub fn finish(&self, client_span_id: u64, finish_time: Option<f64>) {
        let _ = self.command_tx.send(Command::Finish {
            client_span_id,
            finish_time,
        });
    }
}
