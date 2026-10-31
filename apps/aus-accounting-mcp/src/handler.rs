use crate::{
    VERSION, adapters,
    contracts::{self, CATALOGUE},
};
use rmcp::{ErrorData, RoleServer, ServerHandler, model::*, service::RequestContext};
use serde_json::{Value, json};
use std::{
    borrow::Cow,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

#[derive(Clone, Default)]
pub struct AccountingServer;

impl AccountingServer {
    fn envelope(&self, mut value: Value, context: &RequestContext<RoleServer>) -> Value {
        if context
            .protocol_version()
            .is_some_and(|v| v >= ProtocolVersion::V_2026_07_28)
        {
            value["resultType"] = json!("complete");
            value["_meta"]["io.modelcontextprotocol/serverInfo"] =
                json!(ServerHandler::get_info(self).server_info);
        }
        value
    }
    pub fn call(&self, name: &str, arguments: &Value) -> Value {
        if let Err(error) = contracts::check_arguments(name, arguments) {
            return contracts::error(error);
        }
        let args = match crate::validation::arguments(name, arguments) {
            Ok(args) => args,
            Err(error) => return contracts::error(format!("Error executing tool {name}: {error}")),
        };
        match adapters::evaluate(name, &args).and_then(|result| {
            contracts::validate_output(name, &result)?;
            Ok(result)
        }) {
            Ok(result) => {
                let compact = if name == "build_payday_super_evidence_pack"
                    && args["response_detail"] == "compact"
                {
                    Some(format!(
                        "Review exit code: {} (0: no exception indicators; 2: review required).\nFour private review files are in structuredContent.files. Read their warnings and retain the exact file contents. No decisions or sign-off recorded.\n{}\n{}",
                        result["review_exit_code"],
                        result["disclaimer"].as_str().unwrap(),
                        result["caveats"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|v| v.as_str().unwrap())
                            .collect::<Vec<_>>()
                            .join("\n")
                    ))
                } else {
                    None
                };
                let mut response = contracts::success(result);
                if let Some(text) = compact {
                    response["content"][0]["text"] = json!(text);
                }
                response
            }
            Err(error) => contracts::error(format!("Error executing tool {name}: {error}")),
        }
    }
}

impl ServerHandler for AccountingServer {
    fn get_info(&self) -> ServerConfig {
        let mut info = CATALOGUE["initialize"].clone();
        info["serverInfo"]["version"] = json!(VERSION);
        info["capabilities"] = json!({"prompts":{"listChanged":true},"resources":{"listChanged":true,"subscribe":true},"tools":{"listChanged":true}});
        serde_json::from_value(info).expect("embedded initialize metadata")
    }
    async fn initialize(
        &self,
        request: InitializeRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, ErrorData> {
        context.peer.set_peer_info(request.clone());
        let mut result = self.negotiate_initialize(&request)?;
        result.capabilities =
            serde_json::from_value(CATALOGUE["initialize"]["capabilities"].clone()).unwrap();
        Ok(result)
    }
    fn accepted_subscription_filter(
        &self,
        requested: &SubscriptionFilter,
    ) -> Option<SubscriptionFilter> {
        Some(requested.clone())
    }
    fn supported_protocol_versions(&self) -> Cow<'static, [ProtocolVersion]> {
        Cow::Borrowed(&[
            ProtocolVersion::V_2024_11_05,
            ProtocolVersion::V_2025_03_26,
            ProtocolVersion::V_2025_06_18,
            ProtocolVersion::V_2025_11_25,
            ProtocolVersion::V_2026_07_28,
        ])
    }
    async fn discover(&self, _: RequestContext<RoleServer>) -> Result<DiscoverResult, ErrorData> {
        let mut info = ServerHandler::get_info(self);
        // The original Python SDK publishes its modern subscription capability
        // shape separately from the legacy initialize flags.
        info.capabilities=serde_json::from_value(json!({"prompts":{"listChanged":true},"resources":{"listChanged":true,"subscribe":true},"tools":{"listChanged":true}})).unwrap();
        Ok(DiscoverResult::from_server_info(
            vec![ProtocolVersion::V_2026_07_28],
            info,
        ))
    }
    async fn list_tools(
        &self,
        _: Option<PaginatedRequestParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(
            serde_json::from_value(self.envelope(json!({"tools":CATALOGUE["tools"]}), &context))
                .unwrap(),
        )
    }
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let envelope = self.envelope(
            self.call(&request.name, &json!(request.arguments.unwrap_or_default())),
            &context,
        );
        // rmcp's deserialiser turns a number beyond u64 into a float; keep the exact value.
        let structured = envelope.get("structuredContent").cloned();
        let mut result: CallToolResult = serde_json::from_value(envelope).unwrap();
        result.structured_content = structured;
        Ok(result.into())
    }
    async fn list_resources(
        &self,
        _: Option<PaginatedRequestParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        Ok(serde_json::from_value(
            self.envelope(json!({"resources":CATALOGUE["resources"]}), &context),
        )
        .unwrap())
    }
    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, ErrorData> {
        let result = crate::resources::result(&request.uri)
            .map_err(|e| ErrorData::invalid_params(e, Some(json!({"uri":request.uri}))))?;
        Ok(
            serde_json::from_value::<ReadResourceResult>(self.envelope(result, &context))
                .unwrap()
                .into(),
        )
    }
    async fn list_resource_templates(
        &self,
        _: Option<PaginatedRequestParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, ErrorData> {
        Ok(
            serde_json::from_value(self.envelope(json!({"resourceTemplates":[]}), &context))
                .unwrap(),
        )
    }
    async fn list_prompts(
        &self,
        _: Option<PaginatedRequestParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, ErrorData> {
        Ok(
            serde_json::from_value(
                self.envelope(json!({"prompts":CATALOGUE["prompts"]}), &context),
            )
            .unwrap(),
        )
    }
    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResponse, ErrorData> {
        // The MCP wire type is a map of strings; the Python SDK refuses anything else here.
        if request
            .arguments
            .iter()
            .flatten()
            .any(|(_, value)| !value.is_string())
        {
            return Err(ErrorData::invalid_params(
                "Invalid request parameters",
                Some(json!("")),
            ));
        }
        let arguments = request.arguments.map(|a| a.into_iter().collect());
        let result = crate::prompts::get(&request.name, arguments.as_ref())
            .map_err(|e| ErrorData::new(ErrorCode(0), e, None))?;
        Ok(
            serde_json::from_value::<GetPromptResult>(self.envelope(result, &context))
                .unwrap()
                .into(),
        )
    }
    async fn on_custom_request(
        &self,
        request: CustomRequest,
        _: RequestContext<RoleServer>,
    ) -> Result<CustomResult, ErrorData> {
        Err(ErrorData::new(
            ErrorCode::METHOD_NOT_FOUND,
            "Method not found",
            Some(json!(request.method)),
        ))
    }
}

/// The SDK validates inline versions before dispatch. Python's modern lifecycle
/// advertises only its modern revision; its legacy handshake supports four more.
/// Keep the SDK's dispatch and checks, adapting only that error's advertised list.
pub struct AccountingService {
    server: AccountingServer,
    bootstrap: Arc<AtomicBool>,
    subscriptions: AtomicUsize,
}
pub fn stdio() -> (
    AccountingService,
    impl rmcp::transport::Transport<RoleServer, Error = std::io::Error>,
) {
    let bootstrap = Arc::new(AtomicBool::new(true));
    let (read, write) = rmcp::transport::stdio();
    let transport = BootstrapTransport {
        inner: rmcp::transport::async_rw::AsyncRwTransport::new(read, write),
        bootstrap: bootstrap.clone(),
        pending: None,
        era: None,
        rejection: None,
    };
    (
        AccountingService {
            server: AccountingServer,
            bootstrap,
            subscriptions: AtomicUsize::new(0),
        },
        transport,
    )
}

/// Preserve one original bootstrap error that rmcp emits before service dispatch.
/// Once a session starts, all SDK metadata validation remains authoritative.
#[derive(Clone)]
struct MetaPresence(bool);
fn metadata_message(meta: &RequestMetaObject, present: bool) -> Option<String> {
    if !present {
        return Some("params._meta must be an object carrying the required 'io.modelcontextprotocol/protocolVersion' and 'io.modelcontextprotocol/clientCapabilities' envelope keys".into());
    }
    let missing = [
        "io.modelcontextprotocol/protocolVersion",
        "io.modelcontextprotocol/clientCapabilities",
    ]
    .into_iter()
    .filter(|key| !meta.contains_key(*key))
    .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Some(format!(
            "params._meta is missing the required envelope key(s): {}",
            missing.join(", ")
        ));
    }
    if !meta["io.modelcontextprotocol/protocolVersion"].is_string() {
        return Some("the protocol-version envelope value must be a string".into());
    }
    None
}
struct BootstrapError {
    id: RequestId,
    custom: Option<String>,
    metadata: Option<String>,
    legacy: bool,
}
type PendingSend<E> = Pin<Box<dyn Future<Output = Result<(), E>> + Send>>;
struct BootstrapTransport<T: rmcp::transport::Transport<RoleServer>> {
    inner: T,
    bootstrap: Arc<AtomicBool>,
    pending: Option<BootstrapError>,
    era: Option<Era>,
    // receive() is selected alongside other SDK events. Keep a consumed
    // request's response future here so cancellation cannot discard it.
    rejection: Option<PendingSend<T::Error>>,
}
#[derive(Clone, Copy)]
enum Era {
    Legacy,
    Modern,
}
const VERSION_META: &str = "io.modelcontextprotocol/protocolVersion";
impl Era {
    fn opened_by(request: &ClientRequest) -> Self {
        if !matches!(request, ClientRequest::InitializeRequest(_))
            && request.get_meta().contains_key(VERSION_META)
        {
            Self::Modern
        } else {
            Self::Legacy
        }
    }
    fn rejection(self, request: &ClientRequest, present: bool) -> Option<ErrorData> {
        match (self, request) {
            (Self::Modern, ClientRequest::InitializeRequest(request)) => Some(ErrorData::new(
                ErrorCode(-32022),
                "connection is serving the 2026-07-28 protocol; the initialize handshake is not accepted",
                Some(
                    json!({"supported":["2026-07-28"],"requested":request.params.protocol_version}),
                ),
            )),
            (Self::Legacy, request)
                if !matches!(request, ClientRequest::InitializeRequest(_))
                    && request.get_meta().contains_key(VERSION_META) =>
            {
                Some(ErrorData::new(
                    ErrorCode::INVALID_REQUEST,
                    "this connection serves the handshake protocol era; requests carrying the 2026-07-28 envelope are not accepted on it",
                    None,
                ))
            }
            (Self::Modern, request) => {
                let meta = request.get_meta();
                if let Some(message) = metadata_message(meta, present) {
                    return Some(ErrorData::invalid_params(message, None));
                }
                if meta[VERSION_META] != "2026-07-28" {
                    return Some(ErrorData::new(
                        ErrorCode(-32022),
                        "Unsupported protocol version",
                        Some(json!({"supported":["2026-07-28"],"requested":meta[VERSION_META]})),
                    ));
                }
                // rmcp permits ping during bootstrap; Python's modern era has
                // already removed that method, including before discovery.
                if matches!(request, ClientRequest::PingRequest(_)) {
                    return Some(ErrorData::new(
                        ErrorCode::METHOD_NOT_FOUND,
                        "Method not found",
                        Some(json!("ping")),
                    ));
                }
                None
            }
            _ => None,
        }
    }
}
impl<T: rmcp::transport::Transport<RoleServer>> rmcp::transport::Transport<RoleServer>
    for BootstrapTransport<T>
{
    type Error = T::Error;
    fn send(
        &mut self,
        mut message: ServerJsonRpcMessage,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'static {
        if let ServerJsonRpcMessage::Error(error) = &mut message
            && self
                .pending
                .as_ref()
                .is_some_and(|p| Some(&p.id) == error.id.as_ref())
        {
            let pending = self.pending.take().unwrap();
            if self.bootstrap.load(Ordering::Relaxed)
                && error.error.code == ErrorCode::INVALID_PARAMS
            {
                if let Some(method) = pending.custom {
                    error.error = ErrorData::new(
                        ErrorCode::METHOD_NOT_FOUND,
                        "Method not found",
                        Some(json!(method)),
                    );
                } else if pending.legacy {
                    error.error =
                        ErrorData::invalid_params("Invalid request parameters", Some(json!("")));
                } else if let Some(message) = pending.metadata {
                    error.error.message = message.into();
                }
            }
        }
        self.inner.send(message)
    }
    async fn receive(&mut self) -> Option<ClientJsonRpcMessage> {
        loop {
            if let Some(sending) = self.rejection.as_mut() {
                let sent = sending.await.is_ok();
                self.rejection = None;
                if !sent {
                    return None;
                }
            }
            let mut message = self.inner.receive().await?;
            if let ClientJsonRpcMessage::Request(request) = &mut message {
                let present = request
                    .request
                    .extensions()
                    .get::<RequestMetaObject>()
                    .is_some();
                let era = *self
                    .era
                    .get_or_insert_with(|| Era::opened_by(&request.request));
                if let Some(error) = era.rejection(&request.request, present) {
                    self.rejection = Some(Box::pin(
                        self.inner
                            .send(ServerJsonRpcMessage::error(error, Some(request.id.clone()))),
                    ));
                    continue;
                }
                request
                    .request
                    .extensions_mut()
                    .insert(MetaPresence(present));
                if self.bootstrap.load(Ordering::Relaxed)
                    && !request
                        .request
                        .get_meta()
                        .missing_required_keys(&ProtocolVersion::V_2026_07_28)
                        .is_empty()
                    && !matches!(
                        request.request,
                        ClientRequest::InitializeRequest(_) | ClientRequest::PingRequest(_)
                    )
                {
                    self.pending = Some(BootstrapError {
                        id: request.id.clone(),
                        custom: match &request.request {
                            ClientRequest::ListToolsRequest(_)
                            | ClientRequest::CallToolRequest(_)
                            | ClientRequest::ListResourcesRequest(_)
                            | ClientRequest::ListResourceTemplatesRequest(_)
                            | ClientRequest::ReadResourceRequest(_)
                            | ClientRequest::ListPromptsRequest(_)
                            | ClientRequest::GetPromptRequest(_) => None,
                            _ if matches!(era, Era::Legacy) => {
                                Some(request.request.method().to_owned())
                            }
                            _ => None,
                        },
                        metadata: metadata_message(request.request.get_meta(), present),
                        legacy: matches!(era, Era::Legacy),
                    });
                }
            }
            return Some(message);
        }
    }
    fn close(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send {
        // A parked send may own the writer mutex while flushing. Release it
        // before close tries to acquire that same mutex during SDK shutdown.
        self.rejection.take();
        self.inner.close()
    }
}
/// A slot belongs to the request future, so aborting or dropping that future
/// releases it even when cancellation bypasses the handler's normal return.
struct SubscriptionSlot<'a>(&'a AtomicUsize);
impl SubscriptionSlot<'_> {
    fn reserve(count: &AtomicUsize) -> Result<SubscriptionSlot<'_>, ErrorData> {
        count
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
                (n < 1024).then_some(n + 1)
            })
            .map(|_| SubscriptionSlot(count))
            .map_err(|_| {
                ErrorData::new(
                    ErrorCode::INTERNAL_ERROR,
                    "Subscription limit reached",
                    None,
                )
            })
    }
}
impl Drop for SubscriptionSlot<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}
impl rmcp::service::Service<RoleServer> for AccountingService {
    async fn handle_request(
        &self,
        request: ClientRequest,
        context: RequestContext<RoleServer>,
    ) -> Result<ServerResult, ErrorData> {
        if !matches!(request, ClientRequest::DiscoverRequest(_)) {
            self.bootstrap.store(false, Ordering::Relaxed);
        }
        let method = request.method().to_owned();
        let inline = !matches!(request, ClientRequest::InitializeRequest(_));
        let version = context.meta.protocol_version();
        let metadata_message = metadata_message(
            &context.meta,
            context
                .extensions
                .get::<MetaPresence>()
                .is_some_and(|p| p.0),
        );
        let _slot = if matches!(request, ClientRequest::SubscriptionsListenRequest(_))
            && version.as_ref() == Some(&ProtocolVersion::V_2026_07_28)
            && context
                .meta
                .missing_required_keys(&ProtocolVersion::V_2026_07_28)
                .is_empty()
        {
            Some(SubscriptionSlot::reserve(&self.subscriptions)?)
        } else {
            None
        };
        let mut result =
            rmcp::service::Service::handle_request(&self.server, request, context).await;
        if inline
            && version.is_some_and(|v| v >= ProtocolVersion::V_2026_07_28)
            && let Err(error) = &mut result
            && error.code == ErrorCode(-32022)
            && let Some(data) = &mut error.data
        {
            data["supported"] = json!(["2026-07-28"]);
        }
        if let Err(error) = &mut result
            && error.code == ErrorCode::METHOD_NOT_FOUND
        {
            error.message = "Method not found".into();
            error.data = Some(json!(method));
        }
        if let Err(error) = &mut result
            && error.code == ErrorCode::INVALID_PARAMS
            && error
                .message
                .starts_with("request _meta is missing or has malformed required fields:")
            && let Some(message) = metadata_message
        {
            error.message = message.into();
        }
        result
    }
    async fn handle_notification(
        &self,
        notification: ClientNotification,
        context: rmcp::service::NotificationContext<RoleServer>,
    ) -> Result<(), ErrorData> {
        rmcp::service::Service::handle_notification(&self.server, notification, context).await
    }
    fn get_info(&self) -> ServerConfig {
        ServerHandler::get_info(&self.server)
    }
    fn supported_protocol_versions(&self) -> Cow<'static, [ProtocolVersion]> {
        ServerHandler::supported_protocol_versions(&self.server)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::transport::Transport;
    use std::{
        collections::VecDeque,
        future::poll_fn,
        sync::Mutex,
        task::{Context, Poll, Waker},
    };

    struct PendingSink {
        input: VecDeque<ClientJsonRpcMessage>,
        ready: Arc<AtomicBool>,
        sent: Arc<Mutex<Vec<ServerJsonRpcMessage>>>,
        writing: Arc<AtomicBool>,
    }
    struct WriteGuard(Arc<AtomicBool>);
    impl Drop for WriteGuard {
        fn drop(&mut self) {
            self.0.store(false, Ordering::Relaxed);
        }
    }
    impl Transport<RoleServer> for PendingSink {
        type Error = std::io::Error;
        fn send(
            &mut self,
            message: ServerJsonRpcMessage,
        ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'static {
            let ready = self.ready.clone();
            let sent = self.sent.clone();
            let writing = self.writing.clone();
            async move {
                writing.store(true, Ordering::Relaxed);
                let _guard = WriteGuard(writing);
                poll_fn(|_| {
                    if ready.load(Ordering::Relaxed) {
                        Poll::Ready(())
                    } else {
                        Poll::Pending
                    }
                })
                .await;
                sent.lock().unwrap().push(message);
                Ok(())
            }
        }
        async fn receive(&mut self) -> Option<ClientJsonRpcMessage> {
            self.input.pop_front()
        }
        async fn close(&mut self) -> Result<(), Self::Error> {
            assert!(
                !self.writing.load(Ordering::Relaxed),
                "pending send still holds writer"
            );
            Ok(())
        }
    }
    fn pending_transport() -> BootstrapTransport<PendingSink> {
        let ready = Arc::new(AtomicBool::new(false));
        let sent = Arc::new(Mutex::new(Vec::new()));
        let input = VecDeque::from([serde_json::from_value(json!({
            "jsonrpc":"2.0","id":2,"method":"initialize","params":{
                "protocolVersion":"2025-11-25","capabilities":{},
                "clientInfo":{"name":"synthetic-cancellation-check","version":"1"}
            }
        }))
        .unwrap()]);
        BootstrapTransport {
            inner: PendingSink {
                input,
                ready: ready.clone(),
                sent: sent.clone(),
                writing: Arc::new(AtomicBool::new(false)),
            },
            bootstrap: Arc::new(AtomicBool::new(false)),
            pending: None,
            era: Some(Era::Modern),
            rejection: None,
        }
    }
    #[test]
    fn close_releases_a_parked_rejection_before_closing_writer() {
        let mut transport = pending_transport();
        let mut context = Context::from_waker(Waker::noop());
        assert!(
            Box::pin(transport.receive())
                .as_mut()
                .poll(&mut context)
                .is_pending()
        );
        assert!(transport.inner.writing.load(Ordering::Relaxed));
        assert!(matches!(
            Box::pin(transport.close()).as_mut().poll(&mut context),
            Poll::Ready(Ok(()))
        ));
        assert!(!transport.inner.writing.load(Ordering::Relaxed));
    }
    #[test]
    fn rejected_request_survives_receive_cancellation_during_send() {
        let mut transport = pending_transport();
        let ready = transport.inner.ready.clone();
        let sent = transport.inner.sent.clone();
        let mut context = Context::from_waker(Waker::noop());
        {
            let mut receive = Box::pin(transport.receive());
            assert!(receive.as_mut().poll(&mut context).is_pending());
        }
        assert!(transport.inner.input.is_empty());
        assert!(sent.lock().unwrap().is_empty());
        ready.store(true, Ordering::Relaxed);
        assert!(matches!(
            Box::pin(transport.receive()).as_mut().poll(&mut context),
            Poll::Ready(None)
        ));
        let sent = sent.lock().unwrap();
        assert_eq!(sent.len(), 1);
        let response = serde_json::to_value(&sent[0]).unwrap();
        assert_eq!(response["id"], 2);
        assert_eq!(response["error"]["code"], -32022);
    }
}
