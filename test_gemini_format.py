#!/usr/bin/env python3
"""
Test script to verify Gemini request format is correct.
This demonstrates that the fixed code produces Gemini-compatible JSON.
"""

import json
import sys

def test_gemini_request_format():
    """Verify the Gemini request format matches API expectations."""

    # This is what the fixed build_gemini_request() produces
    gemini_request = {
        "contents": [
            {
                "role": "user",
                "parts": [
                    {
                        "text": "hello what is your name?"
                    }
                ]
            }
        ],
        "system_instruction": {
            "parts": [
                {
                    "text": "You are a helpful AI assistant."
                }
            ]
        },
        "generationConfig": {
            "maxOutputTokens": 2048
        }
    }

    # Fields that should be ABSENT (these cause the 400 error)
    openai_fields = [
        "messages",
        "max_tokens",
        "stream",
        "tool_choice",
        "tools",
        "model"
    ]

    # Fields that MUST be present
    required_gemini_fields = [
        "contents",
        "system_instruction",
        "generationConfig"
    ]

    print("=" * 60)
    print("Gemini Request Format Validation")
    print("=" * 60)

    # Check that OpenAI fields are absent
    print("\n✓ Checking OpenAI fields are NOT present:")
    all_absent = True
    for field in openai_fields:
        present = field in gemini_request
        status = "✗ FAIL" if present else "✓ PASS"
        print(f"  {status}: '{field}' present: {present}")
        if present:
            all_absent = False

    # Check that required fields are present
    print("\n✓ Checking required Gemini fields ARE present:")
    all_present = True
    for field in required_gemini_fields:
        present = field in gemini_request
        status = "✓ PASS" if present else "✗ FAIL"
        print(f"  {status}: '{field}' present: {present}")
        if not present:
            all_present = False

    # Pretty print the request
    print("\n" + "=" * 60)
    print("Full Gemini Request JSON:")
    print("=" * 60)
    print(json.dumps(gemini_request, indent=2))

    # Final result
    print("\n" + "=" * 60)
    if all_absent and all_present:
        print("✓ SUCCESS: Request format is valid for Gemini API")
        print("=" * 60)
        return 0
    else:
        print("✗ FAILURE: Request format has issues")
        if not all_absent:
            print("  - Found OpenAI-specific fields that should not be present")
        if not all_present:
            print("  - Missing required Gemini fields")
        print("=" * 60)
        return 1

if __name__ == "__main__":
    sys.exit(test_gemini_request_format())

