use lsp_server::Request as ServerRequest;
use lsp_server::{Connection, ErrorCode, Message, RequestId, Response};

pub mod command;
pub mod parser;

use anyhow::Result;
use lsp_types::ServerCapabilities;
use lsp_types::request::{GotoDefinition, Request};
use rustc_hash::FxHashMap;

struct Context {}

impl Context {
    pub fn new() -> Self {
        Context {}
    }
}

trait LSPOperation {
    fn initialize(&self, capabilities: &ServerCapabilities) -> Result<()>;
    fn send_ok<T: serde::Serialize>(&self, id: RequestId, result: &T) -> Result<()>;
    fn send_err(&self, id: &RequestId, code: ErrorCode, msg: &str) -> Result<()>;
    fn dispatch_request(&self, ctx: &mut Context, req: &ServerRequest) -> Result<()>;
}

impl LSPOperation for Connection {
    fn send_ok<T: serde::Serialize>(&self, id: RequestId, result: &T) -> Result<()> {
        self.sender.send(Message::Response(Response {
            id,
            result: Some(serde_json::to_value(result)?),
            error: None,
        }))?;

        Ok(())
    }

    fn send_err(&self, id: &RequestId, code: ErrorCode, msg: &str) -> Result<()> {
        let resp = Response {
            id: id.clone(),
            result: None,
            error: Some(lsp_server::ResponseError {
                code: code as i32,
                message: msg.into(),
                data: None,
            }),
        };
        self.sender.send(Message::Response(resp))?;

        Ok(())
    }

    fn dispatch_request(&self, ctx: &mut Context, req: &ServerRequest) -> Result<()> {
        match req.method.as_str() {
            GotoDefinition::METHOD => {}
            _ => self.send_err(&req.id, ErrorCode::MethodNotFound, "Method not found")?,
        }
        todo!()
    }

    fn initialize(&self, capabilities: &ServerCapabilities) -> Result<()> {
        let init_value = serde_json::json!({
            "capabilities": capabilities,
            "offsetEncoding": ["utf-8"],
        });
        let init_params = self.initialize(init_value)?;

        Ok(())
    }
}

fn main() -> Result<()> {
    let (conn, io_thread) = Connection::stdio();
    let mut ctx = Context::new();

    for msg in &conn.receiver {
        match msg {
            Message::Request(req) => {
                if conn.handle_shutdown(&req)? {
                    break;
                }
                conn.dispatch_request(&mut ctx, &req)?;
            }
            Message::Response(resp) => todo!(),
            Message::Notification(noti) => todo!(),
        }
    }

    io_thread.join()?;
    Ok(())
}
