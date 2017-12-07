use futures::Future;
use htrpc::{self, Procedure, RpcRequest, RpcResponse, BodyReader, ReadBody, FutureExt};
use htrpc::types::{EntryPoint, HttpMethod};

#[derive(Debug)]
pub struct StartSpanProcedure;
impl Procedure for StartSpanProcedure {
    type Request = StartSpanRequest;
    type Response = EmptyResponse;
    fn method() -> HttpMethod {
        HttpMethod::Get
    }
    fn entry_point() -> EntryPoint {
        htrpc_entry_point!["start_span"]
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartSpanRequest {
    pub query: StartSpanQuery,
}
impl RpcRequest for StartSpanRequest {
    fn body(&mut self) -> Vec<u8> {
        Vec::new()
    }
    fn read_body(self, body: BodyReader) -> ReadBody<Self> {
        discard_body(self, body)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EmptyResponse {
    Ok,
    InternalServerError,
}
impl RpcResponse for EmptyResponse {
    fn body(&mut self) -> Box<AsRef<[u8]> + Send + 'static> {
        Box::new([])
    }
    fn set_body(&mut self, _bytes: Vec<u8>) {}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartSpanQuery {
    pub operation_name: String,
    pub client_span_id: u64,
    #[serde(default)]
    pub child_of: Option<u64>,
    #[serde(default)]
    pub follows_from: Option<u64>,
    #[serde(default)]
    pub tags: String, // ${KEY}:${VALUE},...
    #[serde(default)]
    pub time: Option<f64>,
}

#[derive(Debug)]
pub struct FinishProcedure;
impl Procedure for FinishProcedure {
    type Request = FinishRequest;
    type Response = EmptyResponse;
    fn method() -> HttpMethod {
        HttpMethod::Get
    }
    fn entry_point() -> EntryPoint {
        htrpc_entry_point!["finish"]
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinishRequest {
    pub query: FinishQuery,
}
impl RpcRequest for FinishRequest {
    fn body(&mut self) -> Vec<u8> {
        Vec::new()
    }
    fn read_body(self, body: BodyReader) -> ReadBody<Self> {
        discard_body(self, body)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinishQuery {
    pub client_span_id: u64,
    #[serde(default)]
    pub time: Option<f64>,
}

fn discard_body<T: Send + 'static>(this: T, body: BodyReader) -> ReadBody<T> {
    let future = body.read_all_bytes().map_err(htrpc::Error::from).map(
        move |(r, _)| (r, this),
    );
    Box::new(future)
}
