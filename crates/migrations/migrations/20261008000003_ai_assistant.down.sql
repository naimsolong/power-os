-- SPDX-License-Identifier: MIT
-- Copyright (c) 2026 Power OS Contributors

DROP INDEX IF EXISTS idx_ai_tool_call_session;
DROP INDEX IF EXISTS idx_ai_message_session;
DROP INDEX IF EXISTS idx_ai_session_workspace;

DROP TABLE IF EXISTS ai_tool_call;
DROP TABLE IF EXISTS ai_message;
DROP TABLE IF EXISTS ai_session;
