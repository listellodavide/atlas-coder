# WebDynamic Tool

## Overview

The `WebDynamic` tool is a new advanced web fetching capability that extends the existing `WebFetch` and `WebSearch` tools by adding JavaScript execution support using Playwright. This tool can handle modern dynamic websites that rely on JavaScript and React rendering to display content.

## Problem It Solves

Traditional web tools (`WebFetch`, `WebSearch`) have a limitation: they cannot execute JavaScript. Modern websites heavily rely on client-side rendering and JavaScript execution to:
- Generate DOM dynamically
- Load content via AJAX/fetch requests
- Render React/Vue/Angular components
- Execute interactive features

`WebDynamic` bridges this gap by using Playwright to:
- Launch a headless browser
- Navigate to URLs
- Execute JavaScript
- Wait for DOM to render
- Extract rendered content

## Usage

### Basic Example

```json
{
  "tool": "WebDynamic",
  "input": {
    "url": "https://example.com/dynamic-page",
    "prompt": "Extract the main content from this page",
    "wait_time_ms": 5000
  }
}
```

### With CSS Selector Waiting

```json
{
  "tool": "WebDynamic",
  "input": {
    "url": "https://example.com/dynamic-page",
    "prompt": "Get the data table content",
    "wait_time_ms": 3000,
    "selector": ".data-table tbody"
  }
}
```

### Extract Full HTML

```json
{
  "tool": "WebDynamic",
  "input": {
    "url": "https://example.com/spa-app",
    "prompt": "Get the rendered HTML structure",
    "wait_time_ms": 8000,
    "extract_html": true
  }
}
```

## Input Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `url` | string | Yes | - | The URL to fetch (must be valid HTTP/HTTPS) |
| `prompt` | string | Yes | - | Description of what information to extract |
| `wait_time_ms` | integer | No | 5000 | Milliseconds to wait for JavaScript execution (0-999999) |
| `selector` | string | No | - | CSS selector to wait for before extraction |
| `extract_html` | boolean | No | false | If true, extracts full HTML; if false, extracts text content |

## Output

The tool returns a structured response:

```json
{
  "bytes": 45234,
  "code": 200,
  "codeText": "OK",
  "result": "Extracted and summarized content based on the prompt...",
  "durationMs": 7543,
  "url": "https://example.com/dynamic-page",
  "jsExecuted": true
}
```

### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `bytes` | integer | Size of extracted content in bytes |
| `code` | integer | HTTP status code (200 for success) |
| `codeText` | string | Human-readable status ("OK") |
| `result` | string | Extracted and processed content |
| `durationMs` | integer | Total execution time in milliseconds |
| `url` | string | Final URL after navigation (includes redirects) |
| `jsExecuted` | boolean | Always true when successful |

## Advantages Over WebFetch

| Feature | WebFetch | WebDynamic |
|---------|----------|-----------|
| JavaScript Execution | ❌ | ✅ |
| Dynamic Content | ❌ | ✅ |
| React/Vue/Angular | ❌ | ✅ |
| AJAX/Fetch Requests | ❌ | ✅ |
| CSS Selector Waiting | ❌ | ✅ |
| HTML Extraction | ❌ | ✅ |
| Text Extraction | ✅ | ✅ |
| Performance | Fast | Slower (requires browser) |
| Resource Usage | Low | Higher |

## Use Cases

1. **React/Vue/Angular Applications**: Extract content from single-page applications
2. **Infinite Scroll Pages**: Wait for content to load dynamically
3. **API-Rendered Content**: Websites that load data via AJAX
4. **Interactive Elements**: Extract data from modals, dropdowns, etc.
5. **Weather Sites**: Fetch current weather with JavaScript rendering
6. **Real-time Data**: Get live-updated content from dynamic dashboards
7. **E-commerce**: Extract product details from dynamic product pages

## Example Use Cases

### 1. Check Weather with JavaScript-Rendered Data

```json
{
  "url": "https://weather.example.com/london",
  "prompt": "Get the current temperature, min/max, and weather conditions",
  "wait_time_ms": 3000
}
```

**Result**: Successfully extracts dynamically rendered weather data

### 2. Extract Data from React Dashboard

```json
{
  "url": "https://app.example.com/dashboard",
  "prompt": "Extract all metrics and KPIs from the dashboard",
  "wait_time_ms": 5000,
  "selector": ".metrics-container"
}
```

**Result**: Waits for the React component to render, then extracts all visible metrics

### 3. Get Rendered HTML Structure

```json
{
  "url": "https://spa.example.com",
  "prompt": "Get the complete page structure",
  "wait_time_ms": 8000,
  "extract_html": true
}
```

**Result**: Returns the full rendered HTML for parsing or analysis

## Configuration

### Dependencies

The tool requires:
- `playwright = "0.0.20"` (Rust)
- `tokio` with async runtime (already in tools crate)

These are automatically added to the `Cargo.toml` when the tool is integrated.

### Browser Support

WebDynamic uses Chromium via Playwright. The browser is launched in headless mode for efficiency.

## Limitations

1. **Performance**: Significantly slower than `WebFetch` due to browser overhead
2. **Resource Usage**: Higher memory and CPU requirements
3. **Timeout**: 20 seconds for navigation, configurable wait times
4. **User Agents**: May be detected and blocked by some anti-bot systems
5. **Session State**: Each request is a fresh browser session (no persistent cookies)

## Error Handling

Common errors and their meanings:

| Error | Cause | Solution |
|-------|-------|----------|
| "Failed to initialize Playwright" | Browser not available | Ensure Playwright is properly installed |
| "Failed to navigate to URL" | Invalid URL or network issue | Check URL format and network connectivity |
| "Failed to extract text" | JavaScript error in page | Increase `wait_time_ms` |
| "Failed to get page URL" | Navigation failed | Check if URL is accessible |

## Best Practices

1. **Start with WebFetch**: Use `WebFetch` first; only use `WebDynamic` when necessary
2. **Optimize Wait Times**: Use the minimum `wait_time_ms` needed to avoid unnecessary delays
3. **Use Selectors**: Specify CSS selectors to wait for specific elements instead of fixed delays
4. **Extract Efficiently**: Set `extract_html: false` for text extraction (smaller, faster)
5. **Error Handling**: Handle timeouts gracefully with fallback logic
6. **Rate Limiting**: Be respectful to servers; don't abuse with rapid requests
7. **User Agent Spoofing**: The tool uses a standard user agent; some sites may still block it

## Integration Details

The tool is integrated into the tools crate with:
- **Tool Name**: `WebDynamic`
- **Permission Mode**: `ReadOnly`
- **Allowed Subagents**: Explore, Plan, Verification, atlas-guide, default
- **Async Support**: Uses Tokio runtime for non-blocking execution

## Testing

The tool can be tested with:

```bash
cargo test -p tools -- --nocapture web_dynamic
```

## Future Enhancements

Potential improvements for future versions:
- Cookie/session persistence
- Custom request headers
- JavaScript injection
- Screenshot capability
- Form interaction support
- Network request inspection
- Performance profiling

## Related Tools

- **WebFetch**: Fast static content extraction (no JavaScript)
- **WebSearch**: Web search with result parsing
- **Bash**: Execute local commands (can use curl/wget)

## References

- Playwright Documentation: https://playwright.dev/
- Rust Playwright Crate: https://crates.io/crates/playwright
- Atlas Tools Crate: `/rust/crates/tools`

