// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

mod session;

use crate::{
    VERSION, adapters,
    contracts::{self, CATALOGUE},
};
use rmcp::{ErrorData, RoleServer, ServerHandler, model::*, service::RequestContext};
use serde_json::{Value, json};
pub use session::{AccountingService, stdio};
use std::borrow::Cow;

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

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        future::poll_fn,
        sync::{
            Arc, Mutex,
            atomic::{AtomicBool, Ordering},
        },
        task::{Context, Poll, Waker},
    };

    use rmcp::{RoleServer, model::*, transport::Transport};
    use serde_json::json;

    use super::session::{BootstrapTransport, Era};

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
    struct ClosedSink(VecDeque<ClientJsonRpcMessage>);
    impl Transport<RoleServer> for ClosedSink {
        type Error = std::io::Error;
        fn send(
            &mut self,
            _: ServerJsonRpcMessage,
        ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'static {
            std::future::ready(Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe)))
        }
        async fn receive(&mut self) -> Option<ClientJsonRpcMessage> {
            self.0.pop_front()
        }
        async fn close(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }
    #[test]
    fn a_rejection_that_cannot_be_written_ends_the_session() {
        let mut pending = pending_transport();
        let mut input = std::mem::take(&mut pending.inner.input);
        input.push_back(
            serde_json::from_value(json!({"jsonrpc":"2.0","id":3,"method":"tools/list"})).unwrap(),
        );
        let mut transport = BootstrapTransport {
            inner: ClosedSink(input),
            bootstrap: Arc::new(AtomicBool::new(false)),
            pending: None,
            era: Some(Era::Modern),
            rejection: None,
        };
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            Box::pin(transport.receive()).as_mut().poll(&mut context),
            Poll::Ready(None)
        ));
        assert_eq!(
            transport.inner.0.len(),
            1,
            "no request is read after the failed write"
        );
        assert!(transport.rejection.is_none());
        assert!(matches!(
            Box::pin(transport.close()).as_mut().poll(&mut context),
            Poll::Ready(Ok(()))
        ));
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
