use futures;
use futures::future::Finished;
use htrpc::{HandleRpc, RpcServerBuilder, Result};
use htrpc::types::NeverFail;

use rpc;
use service::{ServiceHandle, Command};
use util;

#[derive(Debug, Clone)]
pub struct Server {
    service: ServiceHandle,
}
impl Server {
    pub fn new(service: ServiceHandle) -> Self {
        Server { service }
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
            child_of,
            follows_from,
            tags,
            time,
        } = request.query;
        self.service.send_command(Command::StartSpan {
            client_span_id,
            operation_name,
            child_of,
            follows_from,
            tags: tags.split(",")
                .filter(|kv| !kv.is_empty())
                .map(|kv| {
                    let mut i = kv.splitn(2, ":");
                    (
                        i.next().unwrap_or("").to_owned(),
                        i.next().unwrap_or("").to_owned(),
                    )
                })
                .collect(),
            time: time.map(util::unixtime_to_systemtime),
        });
        futures::finished(rpc::EmptyResponse::Ok)
    }
}
impl HandleRpc<rpc::FinishProcedure> for Server {
    type Future = Finished<rpc::EmptyResponse, NeverFail>;
    fn handle_rpc(self, request: rpc::FinishRequest) -> Self::Future {
        let rpc::FinishQuery {
            client_span_id,
            time,
        } = request.query;
        self.service.finish(client_span_id, time);
        futures::finished(rpc::EmptyResponse::Ok)
    }
}
