// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use std::{
    borrow::Cow,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

use rmcp::{ErrorData, RoleServer, ServerHandler, model::*, service::RequestContext};
use serde_json::json;

use super::AccountingServer;

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
pub(super) struct BootstrapError {
    id: RequestId,
    custom: Option<String>,
    metadata: Option<String>,
    legacy: bool,
}
pub(super) type PendingSend<E> = Pin<Box<dyn Future<Output = Result<(), E>> + Send>>;
pub(super) struct BootstrapTransport<T: rmcp::transport::Transport<RoleServer>> {
    pub(super) inner: T,
    pub(super) bootstrap: Arc<AtomicBool>,
    pub(super) pending: Option<BootstrapError>,
    pub(super) era: Option<Era>,
    // receive() is selected alongside other SDK events. Keep a consumed
    // request's response future here so cancellation cannot discard it.
    pub(super) rejection: Option<PendingSend<T::Error>>,
}
#[derive(Clone, Copy)]
pub(super) enum Era {
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
fn record_bootstrap<T: rmcp::transport::Transport<RoleServer>>(
    transport: &mut BootstrapTransport<T>,
    request: &JsonRpcRequest<ClientRequest>,
    present: bool,
    era: Era,
) {
    if transport.bootstrap.load(Ordering::Relaxed)
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
        transport.pending = Some(BootstrapError {
            id: request.id.clone(),
            custom: legacy_method(&request.request, era),
            metadata: metadata_message(request.request.get_meta(), present),
            legacy: matches!(era, Era::Legacy),
        });
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
                record_bootstrap(self, request, present, era);
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
fn legacy_method(request: &ClientRequest, era: Era) -> Option<String> {
    match request {
        ClientRequest::ListToolsRequest(_)
        | ClientRequest::CallToolRequest(_)
        | ClientRequest::ListResourcesRequest(_)
        | ClientRequest::ListResourceTemplatesRequest(_)
        | ClientRequest::ReadResourceRequest(_)
        | ClientRequest::ListPromptsRequest(_)
        | ClientRequest::GetPromptRequest(_) => None,
        _ if matches!(era, Era::Legacy) => Some(request.method().to_owned()),
        _ => None,
    }
}

fn rewrite(
    result: &mut Result<ServerResult, ErrorData>,
    inline: bool,
    version: Option<ProtocolVersion>,
    method: &str,
    metadata_message: Option<String>,
) {
    if inline
        && version.is_some_and(|v| v >= ProtocolVersion::V_2026_07_28)
        && let Err(error) = &mut *result
        && error.code == ErrorCode(-32022)
        && let Some(data) = &mut error.data
    {
        data["supported"] = json!(["2026-07-28"]);
    }
    if let Err(error) = &mut *result
        && error.code == ErrorCode::METHOD_NOT_FOUND
    {
        error.message = "Method not found".into();
        error.data = Some(json!(method));
    }
    if let Err(error) = &mut *result
        && error.code == ErrorCode::INVALID_PARAMS
        && error
            .message
            .starts_with("request _meta is missing or has malformed required fields:")
        && let Some(message) = metadata_message
    {
        error.message = message.into();
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
        rewrite(&mut result, inline, version, &method, metadata_message);
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
