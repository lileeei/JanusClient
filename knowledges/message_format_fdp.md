# Firefox DevTools Protocol Message Format

## 1. Message Structure

### 1.1 Overview

Firefox DevTools Protocol uses JSON-formatted messages transmitted over WebSocket connections. Each message follows a specific structure based on its type:
- Request messages (client to server)
- Response messages (server to client)
- Event notifications (server to client)

### 1.2 WebSocket Frame Format

Messages are sent as text frames in the WebSocket protocol. There is no additional framing or length prefix beyond the WebSocket protocol's own framing.

## 2. Request Message Format

### 2.1 Basic Structure

```json
{
  "id": <number>,
  "method": <string>,
  "params": <object>
}
```

### 2.2 Field Descriptions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | Number | Yes | Unique identifier for correlating requests with responses |
| `method` | String | Yes | Method to invoke in "Domain.method" format |
| `params` | Object | No | Parameters for the method, can be omitted if none required |

### 2.3 Example Request

```json
{
  "id": 42,
  "method": "DOM.getDocument",
  "params": {
    "depth": 1
  }
}
```

## 3. Response Message Format

### 3.1 Basic Structure

```json
{
  "id": <number>,
  "result": <object>,
  "error": {
    "code": <number>,
    "message": <string>
  }
}
```

### 3.2 Field Descriptions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | Number | Yes | ID from the corresponding request |
| `result` | Object | * | Result data from the method call (present if successful) |
| `error` | Object | * | Error information (present if error occurred) |
| `error.code` | Number | † | Error code indicating the error type |
| `error.message` | String | † | Human-readable error description |

\* Either `result` or `error` must be present, but not both.  
† Required if `error` is present.

### 3.3 Example Success Response

```json
{
  "id": 42,
  "result": {
    "root": {
      "nodeId": 1,
      "nodeType": 9,
      "nodeName": "#document",
      "children": [
        {
          "nodeId": 2,
          "nodeType": 1,
          "nodeName": "HTML",
          "attributes": ["lang", "en"]
        }
      ]
    }
  }
}
```

### 3.4 Example Error Response

```json
{
  "id": 42,
  "error": {
    "code": -32601,
    "message": "Method DOM.getNonExistentMethod not found"
  }
}
```

## 4. Event Notification Format

### 4.1 Basic Structure

```json
{
  "method": <string>,
  "params": <object>
}
```

### 4.2 Field Descriptions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `method` | String | Yes | Event name in "Domain.event" format |
| `params` | Object | Yes | Event data (may be an empty object) |

### 4.3 Example Event

```json
{
  "method": "DOM.documentUpdated",
  "params": {}
}
```

### 4.4 Example Event with Parameters

```json
{
  "method": "Network.responseReceived",
  "params": {
    "requestId": "1000",
    "type": "Document",
    "response": {
      "url": "https://example.com/",
      "status": 200,
      "statusText": "OK",
      "headers": {
        "Content-Type": "text/html"
      },
      "mimeType": "text/html"
    }
  }
}
```

## 5. Standard Error Codes

| Code | Name | Description |
|------|------|-------------|
| -32700 | Parse error | Invalid JSON |
| -32600 | Invalid Request | The request is not a valid request object |
| -32601 | Method not found | The method does not exist |
| -32602 | Invalid params | Invalid method parameters |
| -32603 | Internal error | Internal server error |
| -32000 to -32099 | Server error | Implementation-defined server errors |

## 6. Type System

### 6.1 Common Types

| Type | JSON Representation | Description |
|------|---------------------|-------------|
| String | String | UTF-8 string |
| Number | Number | Integer or float |
| Boolean | Boolean | true or false |
| Object | Object | JSON object with properties |
| Array | Array | JSON array |
| null | null | No value |

### 6.2 Domain-Specific Types

Many domain-specific types are represented as objects with specific properties. For example, a DOM Node:

```json
{
  "nodeId": 1,
  "nodeType": 1,
  "nodeName": "HTML",
  "localName": "html",
  "nodeValue": "",
  "attributes": ["lang", "en"],
  "childNodeCount": 2
}
```

## 7. Serialization Rules

### 7.1 JSON Encoding

- All messages must be valid JSON
- Use UTF-8 encoding
- Field names use camelCase
- No comments are allowed
- No trailing commas

### 7.2 Number Representation

- JavaScript numbers are used (double-precision float)
- Integers should be represented without a decimal point when possible
- No NaN or Infinity values are allowed

### 7.3 String Escaping

Standard JSON string escaping rules apply:
- Backslash (`\`) for escaping
- Unicode escapes in the form `\uXXXX`
- Control characters must be escaped

## 8. Size Limits

- Maximum message size: 100MB (recommended limit)
- Maximum nesting depth: 1000 (recommended limit)
- Clients and servers should implement reasonable limits to prevent DoS attacks

## 9. Versioning

Firefox DevTools Protocol does not use explicit version numbers in messages. Protocol versioning is implicit based on Firefox version.

## 10. Example Message Sequences

### 10.1 Getting Document and Finding Elements

Request:
```json
{"id": 1, "method": "DOM.getDocument"}
```

Response:
```json
{"id": 1, "result": {"root": {"nodeId": 1, "nodeType": 9, "nodeName": "#document"}}}
```

Request:
```json
{"id": 2, "method": "DOM.querySelector", "params": {"nodeId": 1, "selector": "h1"}}
```

Response:
```json
{"id": 2, "result": {"nodeId": 5}}
```

### 10.2 Setting Breakpoint and Handling Paused Event

Request:
```json
{
  "id": 3, 
  "method": "Debugger.setBreakpoint", 
  "params": {
    "location": {
      "scriptId": "42",
      "lineNumber": 10
    }
  }
}
```

Response:
```json
{
  "id": 3,
  "result": {
    "breakpointId": "1:42:10:0",
    "actualLocation": {
      "scriptId": "42",
      "lineNumber": 10,
      "columnNumber": 0
    }
  }
}
```

Event:
```json
{
  "method": "Debugger.paused",
  "params": {
    "callFrames": [
      {
        "callFrameId": "0",
        "functionName": "onClick",
        "location": {
          "scriptId": "42",
          "lineNumber": 10,
          "columnNumber": 5
        }
      }
    ],
    "reason": "breakpoint"
  }
}
```

## 11. Version History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-03-26 | 1.0.0 | Initial message format specification | AI Assistant |
