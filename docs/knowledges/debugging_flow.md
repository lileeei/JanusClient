# Firefox DevTools Protocol Debugging Flow

## 1. Overview

This document describes the typical debugging workflow using the Firefox DevTools Protocol (FDP). It covers the sequence of operations from connecting to a browser instance to performing common debugging tasks.

## 2. Connection Lifecycle

### 2.1 Connection Sequence

1. **Discovery**: Find available debugging targets
2. **Connection**: Establish WebSocket connection
3. **Initialization**: Configure debugging session
4. **Interaction**: Send commands and receive responses/events
5. **Termination**: Close the debugging session

### 2.2 Discovery Phase

Before connecting, a client typically discovers available debugging targets:

```
GET http://localhost:9222/json/list
```

Example response:
```json
[
  {
    "description": "Firefox Main Page",
    "devtoolsFrontendUrl": "/devtools/inspector.html?ws=localhost:9222/devtools/browser/1234",
    "id": "1234",
    "title": "New Tab",
    "type": "page",
    "url": "about:newtab",
    "webSocketDebuggerUrl": "ws://localhost:9222/devtools/browser/1234"
  }
]
```

### 2.3 Connection Phase

The client establishes a WebSocket connection to the URL provided in `webSocketDebuggerUrl`:

```
ws://localhost:9222/devtools/browser/1234
```

### 2.4 Initialization Phase

Once connected, a client typically:

1. Gets browser information
2. Enables required domains
3. Configures event listeners

Example:
```json
{"id": 1, "method": "Browser.getVersion"}
{"id": 2, "method": "Network.enable"}
{"id": 3, "method": "DOM.enable"}
{"id": 4, "method": "CSS.enable"}
```

## 3. Common Debugging Flows

### 3.1 DOM Inspection

#### 3.1.1 Getting the Document

1. **Get Document**:
   ```json
   {"id": 5, "method": "DOM.getDocument"}
   ```

2. **Query Elements**:
   ```json
   {"id": 6, "method": "DOM.querySelector", "params": {"nodeId": 1, "selector": "#main-content"}}
   ```

3. **Get Node Properties**:
   ```json
   {"id": 7, "method": "DOM.getAttributes", "params": {"nodeId": 42}}
   ```

4. **Get Computed Styles**:
   ```json
   {"id": 8, "method": "CSS.getComputedStyleForNode", "params": {"nodeId": 42}}
   ```

#### 3.1.2 Modifying the DOM

1. **Set Attribute Value**:
   ```json
   {"id": 9, "method": "DOM.setAttributeValue", "params": {"nodeId": 42, "name": "class", "value": "highlighted"}}
   ```

2. **Remove Node**:
   ```json
   {"id": 10, "method": "DOM.removeNode", "params": {"nodeId": 42}}
   ```

### 3.2 JavaScript Debugging

#### 3.2.1 Setting Up Breakpoints

1. **Enable Debugger**:
   ```json
   {"id": 11, "method": "Debugger.enable"}
   ```

2. **Set Breakpoint by URL**:
   ```json
   {"id": 12, "method": "Debugger.setBreakpointByUrl", "params": {
     "lineNumber": 45,
     "url": "https://example.com/script.js"
   }}
   ```

3. **Set Breakpoint by Selector**:
   ```json
   {"id": 13, "method": "Debugger.setBreakpointOnFunctionCall", "params": {
     "objectId": "7FB95D7358FF12C",
     "condition": ""
   }}
   ```

#### 3.2.2 Handling Paused Execution

When execution pauses at a breakpoint, an event is fired:

```json
{
  "method": "Debugger.paused",
  "params": {
    "callFrames": [
      {
        "callFrameId": "0",
        "functionName": "processClick",
        "location": {
          "scriptId": "51",
          "lineNumber": 45,
          "columnNumber": 8
        },
        "url": "https://example.com/script.js"
      }
    ],
    "reason": "breakpoint"
  }
}
```

Client can then:

1. **Inspect Call Stack**:
   - Use the provided `callFrames` information

2. **Evaluate Expressions**:
   ```json
   {"id": 14, "method": "Debugger.evaluateOnCallFrame", "params": {
     "callFrameId": "0",
     "expression": "this.value"
   }}
   ```

3. **Control Execution**:
   ```json
   {"id": 15, "method": "Debugger.resume"}
   {"id": 16, "method": "Debugger.stepOver"}
   {"id": 17, "method": "Debugger.stepInto"}
   {"id": 18, "method": "Debugger.stepOut"}
   ```

### 3.3 Network Monitoring

#### 3.3.1 Capturing Network Requests

1. **Enable Network Monitoring**:
   ```json
   {"id": 19, "method": "Network.enable"}
   ```

2. **Handle Request Events**:
   Events received:
   ```json
   {
     "method": "Network.requestWillBeSent",
     "params": {
       "requestId": "1000.1",
       "request": {
         "url": "https://example.com/api/data",
         "method": "GET",
         "headers": {
           "User-Agent": "Mozilla/5.0...",
           "Accept": "application/json"
         }
       }
     }
   }
   ```

   ```json
   {
     "method": "Network.responseReceived",
     "params": {
       "requestId": "1000.1",
       "response": {
         "url": "https://example.com/api/data",
         "status": 200,
         "statusText": "OK",
         "headers": {
           "Content-Type": "application/json"
         },
         "mimeType": "application/json"
       }
     }
   }
   ```

3. **Get Response Body**:
   ```json
   {"id": 20, "method": "Network.getResponseBody", "params": {"requestId": "1000.1"}}
   ```

### 3.4 Console Integration

#### 3.4.1 Monitoring Console Messages

1. **Enable Console**:
   ```json
   {"id": 21, "method": "Console.enable"}
   ```

2. **Handle Console Messages**:
   ```json
   {
     "method": "Console.messageAdded",
     "params": {
       "message": {
         "level": "error",
         "text": "Uncaught TypeError: Cannot read property 'length' of undefined",
         "url": "https://example.com/script.js",
         "line": 52,
         "column": 12
       }
     }
   }
   ```

3. **Execute Code in Console**:
   ```json
   {"id": 22, "method": "Runtime.evaluate", "params": {
     "expression": "document.title",
     "contextId": 1
   }}
   ```

### 3.5 Page Navigation and Interaction

#### 3.5.1 Navigation

1. **Navigate to URL**:
   ```json
   {"id": 23, "method": "Page.navigate", "params": {"url": "https://example.com/page2"}}
   ```

2. **Handle Navigation Events**:
   ```json
   {
     "method": "Page.frameNavigated",
     "params": {
       "frame": {
         "id": "main",
         "url": "https://example.com/page2"
       }
     }
   }
   ```

3. **Reload Page**:
   ```json
   {"id": 24, "method": "Page.reload", "params": {"ignoreCache": true}}
   ```

#### 3.5.2 Page Interaction

1. **Take Screenshot**:
   ```json
   {"id": 25, "method": "Page.captureScreenshot"}
   ```

2. **Print to PDF**:
   ```json
   {"id": 26, "method": "Page.printToPDF", "params": {"landscape": true}}
   ```

## 4. Advanced Debugging Scenarios

### 4.1 Working with Multiple Targets

1. **List Available Targets**:
   ```json
   {"id": 27, "method": "Target.getTargets"}
   ```

2. **Attach to Specific Target**:
   ```json
   {"id": 28, "method": "Target.attachToTarget", "params": {"targetId": "5678"}}
   ```

3. **Create New Target**:
   ```json
   {"id": 29, "method": "Target.createTarget", "params": {"url": "https://example.com/newpage"}}
   ```

### 4.2 Performance Analysis

1. **Start Performance Monitoring**:
   ```json
   {"id": 30, "method": "Performance.enable"}
   ```

2. **Get Performance Metrics**:
   ```json
   {"id": 31, "method": "Performance.getMetrics"}
   ```

### 4.3 Storage Inspection

1. **Query Cookies**:
   ```json
   {"id": 32, "method": "Network.getCookies", "params": {"urls": ["https://example.com"]}}
   ```

2. **Clear Browser Cookies**:
   ```json
   {"id": 33, "method": "Network.clearBrowserCookies"}
   ```

## 5. Error Handling

### 5.1 Common Errors

1. **Method Not Found**:
   ```json
   {
     "id": 34,
     "error": {
       "code": -32601,
       "message": "Method not found"
     }
   }
   ```

2. **Invalid Parameters**:
   ```json
   {
     "id": 35,
     "error": {
       "code": -32602,
       "message": "Invalid parameters"
     }
   }
   ```

3. **Node Not Found**:
   ```json
   {
     "id": 36,
     "error": {
       "code": -32000,
       "message": "No node with given id found"
     }
   }
   ```

### 5.2 Recovery Strategies

1. **Reconnection**:
   - If the WebSocket connection is lost, implement an exponential backoff strategy for reconnection

2. **Context Recovery**:
   - After reconnection, re-enable necessary domains and re-establish context

3. **Error Classification**:
   - Transient errors (retry appropriate)
   - Permanent errors (report to user)
   - Protocol errors (check client implementation)

## 6. Best Practices

### 6.1 Performance Considerations

1. **Minimize DOM Queries**:
   - Fetch only necessary nodes
   - Use appropriate depth parameters

2. **Batch Commands**:
   - Group related commands where possible

3. **Be Selective with Events**:
   - Only enable domains you need
   - Disable domains when not needed

### 6.2 Security Considerations

1. **Local Connections Only**:
   - By default, limit connections to localhost

2. **User Confirmation**:
   - Require user confirmation for potentially dangerous operations

3. **Sanitize Data**:
   - Always sanitize data from the debugger before display or execution

### 6.3 Robustness

1. **Implement Timeouts**:
   - Set reasonable timeouts for all operations

2. **Handle Unexpected Events**:
   - Be prepared for unexpected event sequences

3. **Graceful Degradation**:
   - If a feature is not supported, provide fallback behavior

## 7. Complete Debugging Session Example

Below is a complete example of a debugging session from connection to disconnection:

```
1. Client discovers available targets
   GET http://localhost:9222/json/list

2. Client connects to target
   WebSocket ws://localhost:9222/devtools/browser/1234

3. Client initializes session
   -> {"id": 1, "method": "Browser.getVersion"}
   <- {"id": 1, "result": {"protocolVersion": "1.3", "product": "Firefox/95.0"}}
   
   -> {"id": 2, "method": "DOM.enable"}
   <- {"id": 2, "result": {}}
   
   -> {"id": 3, "method": "CSS.enable"}
   <- {"id": 3, "result": {}}

4. Client gets document
   -> {"id": 4, "method": "DOM.getDocument"}
   <- {"id": 4, "result": {"root": {"nodeId": 1, "nodeType": 9, "nodeName": "#document"}}}

5. Client queries for element
   -> {"id": 5, "method": "DOM.querySelector", "params": {"nodeId": 1, "selector": "#login-form"}}
   <- {"id": 5, "result": {"nodeId": 42}}

6. Client gets computed styles
   -> {"id": 6, "method": "CSS.getComputedStyleForNode", "params": {"nodeId": 42}}
   <- {"id": 6, "result": {"computedStyle": [{"name": "color", "value": "rgb(0, 0, 0)"}]}}

7. Client sets breakpoint
   -> {"id": 7, "method": "Debugger.enable"}
   <- {"id": 7, "result": {}}
   
   -> {"id": 8, "method": "Debugger.setBreakpointByUrl", "params": {"lineNumber": 50, "url": "https://example.com/script.js"}}
   <- {"id": 8, "result": {"breakpointId": "2:50:0", "locations": [{"scriptId": "51", "lineNumber": 50}]}}

8. Execution pauses at breakpoint
   <- {"method": "Debugger.paused", "params": {"callFrames": [{"callFrameId": "0", "functionName": "validateForm"}]}}

9. Client evaluates expression
   -> {"id": 9, "method": "Debugger.evaluateOnCallFrame", "params": {"callFrameId": "0", "expression": "document.getElementById('username').value"}}
   <- {"id": 9, "result": {"result": {"type": "string", "value": "user123"}}}

10. Client resumes execution
    -> {"id": 10, "method": "Debugger.resume"}
    <- {"id": 10, "result": {}}

11. Client closes session
    -> {"id": 11, "method": "Browser.close"}
    <- {"id": 11, "result": {}}
```

## 8. Version History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-03-26 | 1.0.0 | Initial debugging flow documentation | AI Assistant |
