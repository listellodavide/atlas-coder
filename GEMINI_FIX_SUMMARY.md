# Gemini API Integration Fix - Summary

## Problem
When using Atlas CLI with Gemini models, users were receiving a 400 Bad Request error from the Gemini API:

```
Invalid JSON payload received. Unknown name "max_tokens": Cannot find field.
Invalid JSON payload received. Unknown name "messages": Cannot find field.
Invalid JSON payload received. Unknown name "stream": Cannot find field.
Invalid JSON payload received. Unknown name "tool_choice": Cannot find field.
Invalid JSON payload received. Unknown name "function" at 'tools[0]': Cannot find field.
```

This indicated that requests were being sent in OpenAI format instead of Gemini's format.

## Root Cause
The Gemini API has a completely different request/response format than OpenAI:

### OpenAI Format (incorrect for Gemini):
```json
{
  "model": "gpt-4",
  "max_tokens": 2048,
  "messages": [...],
  "stream": false,
  "tool_choice": "auto",
  "tools": [...]
}
```

### Gemini Format (correct):
```json
{
  "contents": [...],
  "system_instruction": {...},
  "generationConfig": {
    "maxOutputTokens": 2048
  }
}
```

## Solution
Updated `/mnt/projects/projects_2026/atlas-ai/rust/crates/api/src/providers/openai_compat.rs` to:

### 1. Detect Gemini Requests
Added dual detection in `send_raw_request()` method:
```rust
let is_gemini = self.config.provider_name == "Gemini" 
    || request.model.starts_with("gemini/");
```

This ensures Gemini models are detected whether they're explicitly configured or detected by model name prefix.

### 2. Route to Correct Request Builder
```rust
let request_body = if is_gemini {
    build_gemini_request(request)
} else {
    build_chat_completion_request(request, self.config())
};
```

### 3. Proper Endpoint Formatting
- Gemini uses: `https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent`
- Model name is extracted from the request and replaces the `{model}` placeholder
- Example: `gemini/gemini-3.1-pro-preview` → endpoint becomes `.../models/gemini-3.1-pro-preview:generateContent`

### 4. Authentication Method
- Gemini uses API key as query parameter: `?key=YOUR_API_KEY`
- Other providers use Bearer token: `Authorization: Bearer YOUR_API_KEY`

```rust
if is_gemini {
    req = req.query(&[("key", &self.api_key)]);
} else {
    req = req.bearer_auth(&self.api_key);
}
```

### 5. Request Format Builder
The `build_gemini_request()` function properly formats requests:
- Converts messages to Gemini's `contents` format
- Places system prompts in `system_instruction` field
- Uses `generationConfig` for `maxOutputTokens`
- Explicitly excludes OpenAI fields (tools, tool_choice, stream, max_tokens)

### 6. Response Parsing
The `normalize_gemini_response()` function:
- Parses Gemini's response structure (`candidates`, `content`, `parts`)
- Combines multiple text parts into unified response
- Maps finish reasons: `STOP` → `end_turn`, `MAX_TOKENS` → `max_tokens`, etc.
- Extracts token usage from `usage_metadata`

## Testing
All unit tests pass (9/9):
- ✓ `builds_gemini_request_correctly` - Verifies request format
- ✓ `endpoint_builder_accepts_base_urls_and_full_endpoints` - Verifies URL building
- ✓ `normalizes_stop_reasons` - Verifies response parsing
- ✓ All other OpenAI/xAI compatibility tests still pass

## Files Modified
- `/mnt/projects/projects_2026/atlas-ai/rust/crates/api/src/providers/openai_compat.rs`
  - Enhanced `send_raw_request()` with dual detection
  - Enhanced `send_message()` with dual detection
  - Added comprehensive `build_gemini_request()` function
  - Added `normalize_gemini_response()` function
  - Added `normalize_gemini_finish_reason()` function
  - Added `gemini()` configuration method to `OpenAiCompatConfig`

## How to Use
1. Set GEMINI_API_KEY environment variable:
   ```bash
   export GEMINI_API_KEY='your-api-key'
   ```

2. Use the /gemini command in Atlas:
   ```
   > /login gemini
   > /gemini gemini-3.1-pro-preview
   > hello what is your name?
   ```

The system now correctly:
- Detects Gemini models by prefix ("gemini/")
- Routes them to Gemini-specific request builder
- Uses proper authentication and endpoint
- Formats all requests in Gemini's native format
- Parses responses correctly

## Build Status
✓ Code compiles without errors
✓ All tests pass
✓ Ready for end-to-end testing with live Gemini API

