// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

pub mod client;
pub mod handlers;
pub mod tools;

pub use client::{AiClient, AiConfig, AiError, Message, ToolCall};
pub use handlers::{chat, ChatRequest, ChatResponse};
pub use tools::{execute_tool, tool_definitions};
