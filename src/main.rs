extern crate clap;
extern crate jaeger_remote_tracer;
extern crate fibers;
extern crate futures;
extern crate htrpc;
extern crate rustracing;
extern crate rustracing_jaeger;
#[macro_use]
extern crate trackable;

use clap::{App, Arg};
use fibers::{Spawn, Executor, ThreadPoolExecutor};
use futures::Future;
use jaeger_remote_tracer::service::Service;
use trackable::error::Failure;

fn main() {
    let matches = App::new("jaeger_remote_tracer")
        .arg(
            Arg::with_name("SERVICE_NAME")
                .long("service")
                .takes_value(true)
                .default_value("jaeger_remote_trace"),
        )
        .arg(
            Arg::with_name("HTTP_PORT")
                .long("http_port")
                .short("p")
                .takes_value(true)
                .default_value("8888"),
        )
        .get_matches();
    let service_name = matches.value_of("SERVICE_NAME").unwrap();
    let http_port = matches.value_of("HTTP_PORT").unwrap();

    let bind_addr = format!("0.0.0.0:{}", http_port).parse().unwrap();
    let mut rpc_server = htrpc::RpcServerBuilder::new(bind_addr);

    let reporter = track_try_unwrap!(rustracing_jaeger::reporter::JaegerCompactReporter::new(
        service_name,
    ));
    let service = Service::new(reporter);
    track_try_unwrap!(
        jaeger_remote_tracer::server::Server::new(service.handle()).register(&mut rpc_server)
    );

    let executor = ThreadPoolExecutor::new().unwrap();
    executor.spawn(rpc_server.start(executor.handle()).map_err(
        |e| panic!("{}", e),
    ));
    executor.spawn(service.map_err(|e| panic!("{}", e)));
    track_try_unwrap!(executor.run().map_err(Failure::from_error));
}
