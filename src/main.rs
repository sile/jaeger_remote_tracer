extern crate clap;
extern crate jaeger_remote_tracer;
extern crate fibers;
extern crate futures;
extern crate htrpc;
extern crate rustracing;
extern crate rustracing_jaeger;
#[macro_use]
extern crate trackable;

use fibers::{Spawn, Executor, ThreadPoolExecutor};
use futures::Future;
use jaeger_remote_tracer::service::Service;
use trackable::error::Failure;

fn main() {
    let bind_addr = "0.0.0.0:8888".parse().unwrap();
    let mut rpc_server = htrpc::RpcServerBuilder::new(bind_addr);

    let sampler = rustracing::sampler::AllSampler;
    let (tracer, span_rx) = rustracing_jaeger::Tracer::new(sampler);
    let reporter = track_try_unwrap!(rustracing_jaeger::reporter::JaegerCompactReporter::new(
        "TODO_service_name",
    ));
    let service = Service::new(span_rx, reporter);
    track_try_unwrap!(
        jaeger_remote_tracer::server::Server::new(tracer, service.handle())
            .register(&mut rpc_server)
    );

    let executor = ThreadPoolExecutor::new().unwrap();
    executor.spawn(rpc_server.start(executor.handle()).map_err(
        |e| panic!("{}", e),
    ));
    executor.spawn(service.map_err(|e| panic!("{}", e)));
    track_try_unwrap!(executor.run().map_err(Failure::from_error));
}
