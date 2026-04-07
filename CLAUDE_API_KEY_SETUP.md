# Using Claude API Key with Atlas AI CLI

This guide explains how to set up and use your Claude API key with the Atlas AI CLI.

## Quick Start

### Option 1: Environment Variable (Recommended)

Set the `ANTHROPIC_API_KEY` environment variable before running atlas:

```bash
export ANTHROPIC_API_KEY="sk-ant-xxx..."
atlas
```

### Option 2: Using the /login Command

You can use the `/login` command within the interactive atlas CLI to provide your API key:

```bash
atlas
> /login
```

When prompted, paste your Claude API key and press Enter.

### Option 3: Store in .env File

Create a `.env` file in your project directory:

```bash
ANTHROPIC_API_KEY=sk-ant-xxx...
```

Then run atlas from that directory:

```bash
atlas
```

## Logging Out

To clear your API key from the current session, use the `/logout` command:

```bash
atlas
> /logout
```

This removes the API key from the current session. It does not affect permanently configured keys (environment variables or .env files from outside the CLI).

## Getting Your Claude API Key

1. Go to https://console.anthropic.com/
2. Sign in with your Anthropic account (create one if needed)
3. Navigate to the **API Keys** section
4. Click **Create Key**
5. Copy your new API key
6. **Important**: Store this key securely. Never commit it to version control.

## Environment Variables Supported

The Atlas CLI supports the following environment variables for authentication:

- `ANTHROPIC_API_KEY` - Your Anthropic API key (primary)
- `ANTHROPIC_AUTH_TOKEN` - Optional OAuth token for enterprise setups

## Switching Models

Once authenticated, you can switch between different Claude models:

```bash
> /model claude-opus-4-6
```

## Testing Your Setup

To verify your API key is working:

```bash
atlas "Hello, what is 2+2?"
```

Or in interactive mode:

```bash
atlas
> What is 2+2?
```

## Troubleshooting

### Error: "Missing credentials for Anthropic"

This means the API key environment variable is not set. Make sure you:
1. Have set `ANTHROPIC_API_KEY` before running atlas
2. Are using a valid API key from https://console.anthropic.com/

### Error: "Invalid API key"

Your API key may be:
- Expired or revoked
- Incorrect
- From the wrong account

Generate a new key from the Anthropic console and try again.

### Connection errors

Check that:
1. You have internet connectivity
2. Anthropic's API is not experiencing outages
3. Your firewall/proxy allows connections to `api.anthropic.com`

## Security Best Practices

- Never commit `ANTHROPIC_API_KEY` to version control
- Use `.env` files (added to `.gitignore`) for local development
- For CI/CD environments, use secret management systems
- Rotate your API keys regularly
- Use environment variables in production, not hardcoded strings

## Advanced: Multiple Keys

If you need to work with multiple API keys:

```bash
# Temporarily switch keys
ANTHROPIC_API_KEY="key-1" atlas command1
ANTHROPIC_API_KEY="key-2" atlas command2
```

## Support

For more information about Anthropic API:
- Documentation: https://docs.anthropic.com/
- API Reference: https://docs.anthropic.com/reference/
- Support: https://support.anthropic.com/


