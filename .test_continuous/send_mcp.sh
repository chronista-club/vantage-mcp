#!/bin/bash
# MCPリクエストを送信してレスポンスを取得

REQUEST=$1
echo "$REQUEST" | nc -w 2 127.0.0.1 12800 2>/dev/null | tail -1
