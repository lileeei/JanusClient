# Firefox DevTools Protocol Specification

## 1. Protocol Fundamentals

### 1.1 Protocol Overview

Firefox DevTools Protocol (FDP) is a JSON-based protocol that enables external clients to communicate with Firefox's debugging functionality. It uses WebSocket for communication and implements a request-response model with event notification support.

### 1.2 Key Features

- **WebSocket-based**: Uses WebSocket for bidirectional communication
- **JSON Format**: All messages are JSON encoded
- **Request-Response Model**: Structured communication pattern
- **Event Notification**: Server-initiated messages for state changes
- **Domain Organization**: Functionality organized in domains (Browser, DOM, CSS, etc.)

## 2. Connection Establishment

### 2.1 Connection URL

```
ws://{host}:{port}/devtools/browser/{id}
```

Typical connection for local debugging:
```
ws://localhost:9222/devtools/browser/[id]
```

### 2.2 Target Discovery

Before connecting, clients need to discover available targets through:
```
http://localhost:9222/json/list
```

This returns a JSON array of available debugging targets.

## 3. Message Format

### 3.1 Request Format

```json
{
  "id": <number>,       // Message ID for matching responses
  "method": <string>,   // Method name in "Domain.method" format
  "params": {           // Optional parameters
    // Parameter fields
  }
}
```

### 3.2 Response Format

```json
{
  "id": <number>,       // Corresponding request ID
  "result": {           // Success result (mutually exclusive with error)
    // Result fields
  },
  "error": {            // Error information (mutually exclusive with result)
    "code": <number>,   // Error code
    "message": <string> // Error description
  }
}
```

### 3.3 Event Format

```json
{
  "method": <string>,   // Event name in "Domain.event" format
  "params": {           // Event parameters
    // Parameter fields
  }
}
```

## 4. Core Domains

### 4.1 Browser Domain

Controls browser-level functionality.

#### Methods

| Method | Description | Parameters | Return |
|--------|-------------|------------|--------|
| `Browser.getVersion` | Returns browser version information | None | Version info object |
| `Browser.getWindowForTarget` | Returns window for a target | `targetId` | Window ID |
| `Browser.close` | Closes browser | None | None |

#### Events

| Event | Description | Parameters |
|-------|-------------|------------|
| `Browser.windowCreated` | Fired when a new window is created | Window info |

### 4.2 Target Domain

Manages debugging targets (tabs, extensions, etc.).

#### Methods

| Method | Description | Parameters | Return |
|--------|-------------|------------|--------|
| `Target.getTargets` | Returns list of targets | None | Array of targets |
| `Target.attachToTarget` | Attaches to a target | `targetId` | Session ID |
| `Target.detachFromTarget` | Detaches from a target | `sessionId` | None |
| `Target.createTarget` | Creates a new target | `url` | Target ID |
| `Target.closeTarget` | Closes a target | `targetId` | Success boolean |

#### Events

| Event | Description | Parameters |
|-------|-------------|------------|
| `Target.targetCreated` | Fired when a target is created | Target info |
| `Target.targetDestroyed` | Fired when a target is destroyed | Target ID |
| `Target.attachedToTarget` | Fired when attached to a target | Session and target info |

### 4.3 Page Domain

Controls page operations and navigation.

#### Methods

| Method | Description | Parameters | Return |
|--------|-------------|------------|--------|
| `Page.navigate` | Navigates to URL | `url` | Navigation status |
| `Page.reload` | Reloads the page | `ignoreCache` (optional) | None |
| `Page.captureScreenshot` | Takes a screenshot | Format options | Screenshot data |
| `Page.printToPDF` | Creates PDF | PDF options | PDF data |

#### Events

| Event | Description | Parameters |
|-------|-------------|------------|
| `Page.frameNavigated` | Fired when a frame navigates | Frame info |
| `Page.loadEventFired` | Fired when load event executed | Timestamp |

### 4.4 DOM Domain

Inspects and modifies DOM structure.

#### Methods

| Method | Description | Parameters | Return |
|--------|-------------|------------|--------|
| `DOM.getDocument` | Returns document | `depth` (optional) | Node object |
| `DOM.querySelector` | Finds node by selector | `nodeId`, `selector` | NodeId |
| `DOM.querySelectorAll` | Finds all nodes by selector | `nodeId`, `selector` | Array of NodeIds |
| `DOM.setAttributeValue` | Sets attribute value | `nodeId`, `name`, `value` | None |
| `DOM.removeNode` | Removes a node | `nodeId` | None |

#### Events

| Event | Description | Parameters |
|-------|-------------|------------|
| `DOM.documentUpdated` | Fired when document updated | None |
| `DOM.attributeModified` | Fired when attribute modified | Node and attribute info |
| `DOM.childNodeInserted` | Fired when child node inserted | Node info |

### 4.5 CSS Domain

Inspects and modifies CSS styles.

#### Methods

| Method | Description | Parameters | Return |
|--------|-------------|------------|--------|
| `CSS.getMatchedStylesForNode` | Returns matched styles | `nodeId` | Style objects |
| `CSS.getComputedStyleForNode` | Returns computed style | `nodeId` | Style objects |
| `CSS.setStyleTexts` | Modifies style text | `edits` | Updated styles |

#### Events

| Event | Description | Parameters |
|-------|-------------|------------|
| `CSS.styleSheetAdded` | Fired when stylesheet added | Stylesheet info |
| `CSS.styleSheetRemoved` | Fired when stylesheet removed | Stylesheet ID |

### 4.6 Network Domain

Monitors network requests.

#### Methods

| Method | Description | Parameters | Return |
|--------|-------------|------------|--------|
| `Network.enable` | Enables network events | None | None |
| `Network.disable` | Disables network events | None | None |
| `Network.getCookies` | Returns cookies | URLs (optional) | Cookies array |
| `Network.deleteCookies` | Deletes cookies | Cookie info | None |

#### Events

| Event | Description | Parameters |
|-------|-------------|------------|
| `Network.requestWillBeSent` | Fired before request is sent | Request info |
| `Network.responseReceived` | Fired when response received | Response info |
| `Network.loadingFinished` | Fired when loading finished | Request ID, timestamp |

### 4.7 Console Domain

Accesses console messages.

#### Methods

| Method | Description | Parameters | Return |
|--------|-------------|------------|--------|
| `Console.enable` | Enables console | None | None |
| `Console.disable` | Disables console | None | None |
| `Console.clearMessages` | Clears messages | None | None |

#### Events

| Event | Description | Parameters |
|-------|-------------|------------|
| `Console.messageAdded` | Fired when message added | Message info |

### 4.8 Debugger Domain

JavaScript debugging functionality.

#### Methods

| Method | Description | Parameters | Return |
|--------|-------------|------------|--------|
| `Debugger.enable` | Enables debugger | None | None |
| `Debugger.disable` | Disables debugger | None | None |
| `Debugger.setBreakpoint` | Sets breakpoint | Location info | Breakpoint ID |
| `Debugger.removeBreakpoint` | Removes breakpoint | `breakpointId` | None |
| `Debugger.pause` | Pauses execution | None | None |
| `Debugger.resume` | Resumes execution | None | None |
| `Debugger.stepOver` | Steps over | None | None |
| `Debugger.stepInto` | Steps into | None | None |
| `Debugger.stepOut` | Steps out | None | None |
| `Debugger.evaluateOnCallFrame` | Evaluates on call frame | Frame ID, expression | Result |

#### Events

| Event | Description | Parameters |
|-------|-------------|------------|
| `Debugger.scriptParsed` | Fired when script parsed | Script info |
| `Debugger.paused` | Fired when execution paused | Call frames, reason |
| `Debugger.resumed` | Fired when execution resumed | None |

### 4.9 Runtime Domain

JavaScript runtime evaluation.

#### Methods

| Method | Description | Parameters | Return |
|--------|-------------|------------|--------|
| `Runtime.evaluate` | Evaluates expression | `expression`, options | Result |
| `Runtime.callFunctionOn` | Calls function | Object ID, function, args | Result |
| `Runtime.getProperties` | Gets object properties | Object ID, options | Properties array |

#### Events

| Event | Description | Parameters |
|-------|-------------|------------|
| `Runtime.executionContextCreated` | Fired when context created | Context info |
| `Runtime.executionContextDestroyed` | Fired when context destroyed | Context ID |

### 4.10 Performance Domain

Performance analysis functionality.

#### Methods

| Method | Description | Parameters | Return |
|--------|-------------|------------|--------|
| `Performance.enable` | Enables monitoring | None | None |
| `Performance.disable` | Disables monitoring | None | None |
| `Performance.getMetrics` | Gets metrics | None | Metrics array |

#### Events

| Event | Description | Parameters |
|-------|-------------|------------|
| `Performance.metrics` | Fired with new metrics | Metrics data |

## 5. Data Types

### 5.1 Common Types

| Type | Format | Description |
|------|--------|-------------|
| `RemoteObjectId` | String | Unique identifier for a remote object |
| `NodeId` | Integer | Unique identifier for a DOM node |
| `RequestId` | String | Unique identifier for a network request |
| `FrameId` | String | Unique identifier for a frame |
| `ScriptId` | String | Unique identifier for a script |
| `BreakpointId` | String | Unique identifier for a breakpoint |

### 5.2 DOM Node Representation

```json
{
  "nodeId": <integer>,
  "nodeType": <integer>,
  "nodeName": <string>,
  "localName": <string>,
  "nodeValue": <string>,
  "childNodeCount": <integer>,
  "attributes": [<string>, <string>, ...],
  "children": [<node>, ...],
  "documentURL": <string>
}
```

## 6. Error Codes

| Code | Description |
|------|-------------|
| -32700 | Parse error - Invalid JSON |
| -32600 | Invalid Request - Request not conforming to protocol |
| -32601 | Method not found |
| -32602 | Invalid params |
| -32603 | Internal error |
| -32000 to -32099 | Implementation-defined errors |

## 7. Usage Examples

### 7.1 Attaching to a Target

Request:
```json
{
  "id": 1,
  "method": "Target.attachToTarget",
  "params": {
    "targetId": "3FD24C667D8B404C8F1E794258A5B6C3"
  }
}
```

Response:
```json
{
  "id": 1,
  "result": {
    "sessionId": "1F3B7DFD00C1483683069FCAE293F158"
  }
}
```

### 7.2 Executing JavaScript

Request:
```json
{
  "id": 2,
  "method": "Runtime.evaluate",
  "params": {
    "expression": "document.title",
    "returnByValue": true
  }
}
```

Response:
```json
{
  "id": 2,
  "result": {
    "result": {
      "type": "string",
      "value": "Example Page Title"
    }
  }
}
```

## 8. Implementation Considerations

### 8.1 Client Implementation Guidelines

1. **Connection Management**
   - Implement WebSocket connection handling
   - Support reconnection strategies
   - Handle connection errors gracefully

2. **Message Processing**
   - Create request/response correlation
   - Implement timeout handling
   - Support event listeners

3. **Domain Implementation**
   - Organize code by domains
   - Create type-safe interfaces
   - Implement error handling

### 8.2 Performance Best Practices

1. Batch related operations when possible
2. Use targeted selectors rather than full DOM queries
3. Limit console event subscriptions when not needed
4. Clean up unused connections and resources

## 9. Version History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-03-26 | 1.0.0 | Initial specification | AI Assistant |
