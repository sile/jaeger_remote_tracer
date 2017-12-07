use futures;
use futures::future::Finished;
use htrpc::{HandleRpc, RpcServerBuilder, Result};
use htrpc::types::NeverFail;
use rustracing_jaeger::Tracer;

use rpc;
use service::ServiceHandle;

#[derive(Debug, Clone)]
pub struct Server {
    tracer: Tracer,
    service: ServiceHandle,
}
impl Server {
    pub fn new(tracer: Tracer, service: ServiceHandle) -> Self {
        Server { tracer, service }
    }
    pub fn register(self, builder: &mut RpcServerBuilder) -> Result<()> {
        track!(builder.register(self.clone(), rpc::StartSpanProcedure))?;
        track!(builder.register(self.clone(), rpc::FinishProcedure))?;
        Ok(())
    }
}
impl HandleRpc<rpc::StartSpanProcedure> for Server {
    type Future = Finished<rpc::EmptyResponse, NeverFail>;
    fn handle_rpc(self, request: rpc::StartSpanRequest) -> Self::Future {
        let rpc::StartSpanQuery {
            operation_name,
            client_span_id,
        } = request.query;
        let span = self.tracer.span(operation_name).start();
        self.service.start_span(client_span_id, span);
        futures::finished(rpc::EmptyResponse::Ok)
    }
}
impl HandleRpc<rpc::FinishProcedure> for Server {
    type Future = Finished<rpc::EmptyResponse, NeverFail>;
    fn handle_rpc(self, request: rpc::FinishRequest) -> Self::Future {
        let rpc::FinishQuery {
            client_span_id,
            finish_time,
        } = request.query;
        self.service.finish(client_span_id, finish_time);
        futures::finished(rpc::EmptyResponse::Ok)
    }
}
