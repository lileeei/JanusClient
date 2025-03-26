# Firefox DevTools Protocol Domain Reference

## 1. Protocol Domain Organization

Firefox DevTools Protocol organizes functionality into logical domains. Each domain represents a specific area of browser debugging functionality, with its own methods and events.

## 2. Browser Domain

Controls browser-level operations.

### 2.1 Methods

| Method | Description | Parameters | Return |
|--------|-------------|------------|--------|
| `Browser.getVersion` | Returns browser version info | None | `{protocolVersion, product, revision, userAgent, jsVersion}` |
| `Browser.getWindowForTarget` | Gets browser window for target | `{targetId}` | `{windowId, bounds}` |
| `Browser.close` | Closes browser instance | None | None |

### 2.2 Events

| Event | Description | Parameters |
|-------|-------------|------------|
| `Browser.windowCreated` | Fired when window created | `{windowId}` |
| `Browser.windowClosed` | Fired when window closed | `{windowId}` |

### 2.3 Types

```typescript
interface Version {
    protocolVersion: string;
    product: string;
    revision: string;
    userAgent: string;
    jsVersion: string;
}

interface WindowBounds {
    left: number;
    top: number;
    width: number;
    height: number;
    windowState: "normal" | "minimized" | "maximized" | "fullscreen";
}
```

## 3. Target Domain

Supports multiple debugging contexts management.

### 3.1 Methods

| Method | Description | Parameters | Return |
|--------|-------------|------------|--------|
| `Target.getTargets` | Returns all targets | None | `{targetInfos[]}` |
| `Target.attachToTarget` | Attaches to target | `{targetId, flatten?}` | `{sessionId}` |
| `Target.detachFromTarget` | Detaches from target | `{sessionId, targetId}` | None |
| `Target.createTarget` | Creates a target | `{url, width?, height?, browserContextId?}` | `{targetId}` |
| `Target.closeTarget` | Closes a target | `{targetId}` | `{success}` |

### 3.2 Events

| Event | Description | Parameters |
|-------|-------------|------------|
| `Target.targetCreated` | Fired when target created | `{targetInfo}` |
| `Target.targetDestroyed` | Fired when target destroyed | `{targetId}` |
| `Target.attachedToTarget` | Fired when attached to target | `{sessionId, targetInfo}` |
| `Target.detachedFromTarget` | Fired when detached from target | `{sessionId, targetId}` |

### 3.3 Types

```typescript
interface TargetInfo {
    targetId: string;
    type: "page" | "iframe" | "worker" | "other";
    title: string;
    url: string;
    attached: boolean;
    browserContextId?: string;
}
```

## 4. Page Domain

Page-related operations.

### 4.1 Methods

| Method | Description | Parameters | Return |
|--------|-------------|------------|--------|
| `Page.navigate` | Navigates to URL | `{url, referrer?, transitionType?}` | `{frameId, loaderId, errorText?}` |
| `Page.reload` | Reloads page | `{ignoreCache?, scriptToEvaluateOnLoad?}` | None |
| `Page.captureScreenshot` | Takes screenshot | `{format?, quality?, clip?}` | `{data}` (Base64) |
| `Page.printToPDF` | Generates PDF | `{landscape?, printBackground?, ...}` | `{data}` (Base64) |

### 4.2 Events

| Event | Description | Parameters |
|-------|-------------|------------|
| `Page.frameNavigated` | Fired when frame navigated | `{frame}` |
| `Page.frameStartedLoading` | Fired when frame starts loading | `{frameId}` |
| `Page.frameStoppedLoading` | Fired when frame stops loading | `{frameId}` |
| `Page.loadEventFired` | Fired when load event triggered | `{timestamp}` |
| `Page.domContentEventFired` | Fired when DOMContentLoaded | `{timestamp}` |

### 4.3 Types

```typescript
interface Frame {
    id: string;
    parentId?: string;
    loaderId: string;
    name?: string;
    url: string;
    securityOrigin: string;
    mimeType: string;
}
```

## 5. DOM Domain

Document Object Model operations.

### 5.1 Methods

| Method | Description | Parameters | Return |
|--------|-------------|------------|--------|
| `DOM.getDocument` | Returns document | `{depth?, pierce?}` | `{root}` |
| `DOM.querySelector` | Executes selector | `{nodeId, selector}` | `{nodeId}` |
| `DOM.querySelectorAll` | Executes selector | `{nodeId, selector}` | `{nodeIds[]}` |
| `DOM.getAttributes` | Gets attributes | `{nodeId}` | `{attributes[]}` |
| `DOM.setAttributeValue` | Sets attribute | `{nodeId, name, value}` | None |
| `DOM.removeAttribute` | Removes attribute | `{nodeId, name}` | None |
| `DOM.getOuterHTML` | Gets outerHTML | `{nodeId}` | `{outerHTML}` |
| `DOM.setOuterHTML` | Sets outerHTML | `{nodeId, outerHTML}` | None |
| `DOM.removeNode` | Removes node | `{nodeId}` | None |

### 5.2 Events

| Event | Description | Parameters |
|-------|-------------|------------|
| `DOM.documentUpdated` | Fired on document update | None |
| `DOM.attributeModified` | Fired on attribute change | `{nodeId, name, value}` |
| `DOM.attributeRemoved` | Fired on attribute removal | `{nodeId, name}` |
| `DOM.childNodeCountUpdated` | Fired when child count changes | `{nodeId, childNodeCount}` |
| `DOM.childNodeInserted` | Fired when node inserted | `{parentNodeId, previousNodeId, node}` |
| `DOM.childNodeRemoved` | Fired when node removed | `{parentNodeId, nodeId}` |

### 5.3 Types

```typescript
interface Node {
    nodeId: NodeId;
    nodeType: number;
    nodeName: string;
    localName: string;
    nodeValue: string;
    childNodeCount?: number;
    children?: Node[];
    attributes?: string[];
    documentURL?: string;
    baseURL?: string;
}

type NodeId = number;
```

## 6. CSS Domain

CSS style operations.

### 6.1 Methods

| Method | Description | Parameters | Return |
|--------|-------------|------------|--------|
| `CSS.getMatchedStylesForNode` | Gets styles | `{nodeId}` | `{inlineStyle, attributesStyle, matchedCSSRules, inherited, pseudoElements}` |
| `CSS.getComputedStyleForNode` | Gets computed style | `{nodeId}` | `{computedStyle}` |
| `CSS.getInlineStylesForNode` | Gets inline styles | `{nodeId}` | `{inlineStyle, attributesStyle}` |
| `CSS.setStyleTexts` | Modifies styles | `{edits[]}` | `{styles[]}` |
| `CSS.getStyleSheetText` | Gets stylesheet | `{styleSheetId}` | `{text}` |
| `CSS.setStyleSheetText` | Sets stylesheet | `{styleSheetId, text}` | None |

### 6.2 Events

| Event | Description | Parameters |
|-------|-------------|------------|
| `CSS.styleSheetAdded` | Fired when sheet added | `{header}` |
| `CSS.styleSheetRemoved` | Fired when sheet removed | `{styleSheetId}` |
| `CSS.styleSheetChanged` | Fired when sheet changed | `{styleSheetId}` |

### 6.3 Types

```typescript
interface CSSStyle {
    styleSheetId?: StyleSheetId;
    cssProperties: CSSProperty[];
    shorthandEntries: ShorthandEntry[];
    cssText?: string;
    range?: SourceRange;
}

interface CSSProperty {
    name: string;
    value: string;
    important?: boolean;
    implicit?: boolean;
    text?: string;
    parsedOk?: boolean;
    disabled?: boolean;
    range?: SourceRange;
}

type StyleSheetId = string;
```

## 7. Network Domain

Network activity monitoring.

### 7.1 Methods

| Method | Description | Parameters | Return |
|--------|-------------|------------|--------|
| `Network.enable` | Enables network | `{maxTotalBufferSize?, maxResourceBufferSize?}` | None |
| `Network.disable` | Disables network | None | None |
| `Network.getResponseBody` | Gets response | `{requestId}` | `{body, base64Encoded}` |
| `Network.getCookies` | Gets cookies | `{urls?}` | `{cookies[]}` |
| `Network.deleteCookies` | Deletes cookie | `{name, url, domain?, path?}` | None |
| `Network.clearBrowserCache` | Clears cache | None | None |
| `Network.clearBrowserCookies` | Clears cookies | None | None |

### 7.2 Events

| Event | Description | Parameters |
|-------|-------------|------------|
| `Network.requestWillBeSent` | Fired before request | `{requestId, request, timestamp, initiator, ...}` |
| `Network.responseReceived` | Fired on response | `{requestId, response, timestamp, ...}` |
| `Network.dataReceived` | Fired on data chunk | `{requestId, timestamp, dataLength, encodedDataLength}` |
| `Network.loadingFinished` | Fired on completion | `{requestId, timestamp, encodedDataLength}` |
| `Network.loadingFailed` | Fired on failure | `{requestId, timestamp, errorText, canceled?}` |

### 7.3 Types

```typescript
interface Request {
    url: string;
    method: string;
    headers: Headers;
    postData?: string;
    hasPostData?: boolean;
    mixedContentType?: "none" | "optionally-blockable" | "blockable";
    initialPriority: "VeryLow" | "Low" | "Medium" | "High" | "VeryHigh";
}

interface Response {
    url: string;
    status: number;
    statusText: string;
    headers: Headers;
    mimeType: string;
    requestHeaders?: Headers;
    connectionReused: boolean;
    connectionId: number;
    fromDiskCache?: boolean;
    fromServiceWorker?: boolean;
    encodedDataLength: number;
    timing?: ResourceTiming;
    securityState: "unknown" | "neutral" | "insecure" | "secure" | "info";
}

type Headers = {[key: string]: string};
```

## 8. Console Domain

Console message operations.

### 8.1 Methods

| Method | Description | Parameters | Return |
|--------|-------------|------------|--------|
| `Console.enable` | Enables console | None | None |
| `Console.disable` | Disables console | None | None |
| `Console.clearMessages` | Clears messages | None | None |

### 8.2 Events

| Event | Description | Parameters |
|-------|-------------|------------|
| `Console.messageAdded` | Fired when message added | `{message}` |
| `Console.messageRepeatCountUpdated` | Fired on repeat count change | `{count, timestamp}` |
| `Console.messagesCleared` | Fired when messages cleared | None |

### 8.3 Types

```typescript
interface ConsoleMessage {
    source: "xml" | "javascript" | "network" | "console-api" | "storage" | "appcache" | "rendering" | "security" | "other" | "deprecation" | "worker";
    level: "log" | "warning" | "error" | "debug" | "info";
    text: string;
    url?: string;
    line?: number;
    column?: number;
}
```

## 9. Debugger Domain

JavaScript debugging.

### 9.1 Methods

| Method | Description | Parameters | Return |
|--------|-------------|------------|--------|
| `Debugger.enable` | Enables debugger | None | None |
| `Debugger.disable` | Disables debugger | None | None |
| `Debugger.setBreakpoint` | Sets breakpoint | `{location, condition?}` | `{breakpointId, actualLocation}` |
| `Debugger.removeBreakpoint` | Removes breakpoint | `{breakpointId}` | None |
| `Debugger.setBreakpointByUrl` | Sets URL breakpoint | `{url, lineNumber, condition?, ...}` | `{breakpointId, locations[]}` |
| `Debugger.pause` | Pauses execution | None | None |
| `Debugger.resume` | Resumes execution | None | None |
| `Debugger.stepOver` | Steps over | None | None |
| `Debugger.stepInto` | Steps into | None | None |
| `Debugger.stepOut` | Steps out | None | None |
| `Debugger.evaluateOnCallFrame` | Evaluates expression | `{callFrameId, expression, ...}` | `{result, exceptionDetails?}` |

### 9.2 Events

| Event | Description | Parameters |
|-------|-------------|------------|
| `Debugger.scriptParsed` | Fired when script parsed | `{scriptId, url, startLine, startColumn, ...}` |
| `Debugger.paused` | Fired when paused | `{callFrames[], reason, data?}` |
| `Debugger.resumed` | Fired when resumed | None |
| `Debugger.breakpointResolved` | Fired when breakpoint resolved | `{breakpointId, location}` |

### 9.3 Types

```typescript
interface Location {
    scriptId: ScriptId;
    lineNumber: number;
    columnNumber?: number;
}

interface CallFrame {
    callFrameId: string;
    functionName: string;
    location: Location;
    url: string;
    scopeChain: Scope[];
    this: RemoteObject;
    returnValue?: RemoteObject;
}

type ScriptId = string;
type BreakpointId = string;
```

## 10. Runtime Domain

Runtime evaluation.

### 10.1 Methods

| Method | Description | Parameters | Return |
|--------|-------------|------------|--------|
| `Runtime.evaluate` | Evaluates expression | `{expression, objectGroup?, ...}` | `{result, exceptionDetails?}` |
| `Runtime.callFunctionOn` | Calls function | `{functionDeclaration, objectId?, arguments?, ...}` | `{result, exceptionDetails?}` |
| `Runtime.getProperties` | Gets properties | `{objectId, ownProperties?, ...}` | `{result[]}` |
| `Runtime.releaseObject` | Releases object | `{objectId}` | None |
| `Runtime.releaseObjectGroup` | Releases objects | `{objectGroup}` | None |

### 10.2 Events

| Event | Description | Parameters |
|-------|-------------|------------|
| `Runtime.executionContextCreated` | Fired when context created | `{context}` |
| `Runtime.executionContextDestroyed` | Fired when context destroyed | `{executionContextId}` |
| `Runtime.executionContextsCleared` | Fired when contexts cleared | None |
| `Runtime.exceptionThrown` | Fired when exception thrown | `{timestamp, exceptionDetails}` |
| `Runtime.consoleAPICalled` | Fired on console API call | `{type, args[], executionContextId, ...}` |

### 10.3 Types

```typescript
interface RemoteObject {
    type: "object" | "function" | "undefined" | "string" | "number" | "boolean" | "symbol" | "bigint";
    subtype?: "array" | "null" | "node" | "regexp" | "date" | "map" | "set" | "weakmap" | "weakset" | "iterator" | "error";
    className?: string;
    value?: any;
    unserializableValue?: "Infinity" | "-Infinity" | "-0" | "NaN";
    description?: string;
    objectId?: string;
}

interface ExecutionContextDescription {
    id: number;
    origin: string;
    name: string;
    auxData?: {[key: string]: string};
}

interface PropertyDescriptor {
    name: string;
    value?: RemoteObject;
    writable?: boolean;
    get?: RemoteObject;
    set?: RemoteObject;
    configurable: boolean;
    enumerable: boolean;
    wasThrown?: boolean;
    isOwn?: boolean;
    symbol?: RemoteObject;
}
```

## 11. Performance Domain

Performance analysis.

### 11.1 Methods

| Method | Description | Parameters | Return |
|--------|-------------|------------|--------|
| `Performance.enable` | Enables performance | None | None |
| `Performance.disable` | Disables performance | None | None |
| `Performance.getMetrics` | Gets metrics | None | `{metrics[]}` |

### 11.2 Events

| Event | Description | Parameters |
|-------|-------------|------------|
| `Performance.metrics` | Fired with metrics | `{metrics[], title}` |

### 11.3 Types

```typescript
interface Metric {
    name: string;
    value: number;
}
```

## 12. Storage Domain

Browser storage operations.

### 12.1 Methods

| Method | Description | Parameters | Return |
|--------|-------------|------------|--------|
| `Storage.clearDataForOrigin` | Clears storage | `{origin, storageTypes}` | None |
| `Storage.getCookies` | Gets cookies | `{browserContextId?}` | `{cookies[]}` |
| `Storage.setCookies` | Sets cookies | `{cookies[], browserContextId?}` | None |
| `Storage.clearCookies` | Clears cookies | `{browserContextId?}` | None |

### 12.2 Types

```typescript
interface Cookie {
    name: string;
    value: string;
    domain: string;
    path: string;
    expires: number;
    size: number;
    httpOnly: boolean;
    secure: boolean;
    session: boolean;
    sameSite?: "Strict" | "Lax" | "None";
}
```

## 13. Accessibility Domain

Accessibility information.

### 13.1 Methods

| Method | Description | Parameters | Return |
|--------|-------------|------------|--------|
| `Accessibility.getPartialAXTree` | Gets accessibility tree | `{nodeId?, backendNodeId?, objectId?}` | `{nodes[]}` |

### 13.2 Types

```typescript
interface AXNode {
    nodeId: AXNodeId;
    ignored: boolean;
    role?: AXValue;
    name?: AXValue;
    description?: AXValue;
    value?: AXValue;
    properties?: AXProperty[];
    childIds?: AXNodeId[];
    backendDOMNodeId?: BackendNodeId;
}

type AXNodeId = string;
```

## 14. Security Domain

Security information.

### 14.1 Methods

| Method | Description | Parameters | Return |
|--------|-------------|------------|--------|
| `Security.enable` | Enables security | None | None |
| `Security.disable` | Disables security | None | None |
| `Security.showCertificateViewer` | Shows certificate | None | None |

### 14.2 Events

| Event | Description | Parameters |
|-------|-------------|------------|
| `Security.securityStateChanged` | Fired on security change | `{securityState, schemeIsCryptographic, ...}` |
| `Security.certificateError` | Fired on certificate error | `{eventId, errorType, requestURL}` |

### 14.3 Types

```typescript
type SecurityState = "unknown" | "neutral" | "insecure" | "secure" | "info" | "insecure-broken";
```

## 15. Protocol Type System

### 15.1 Primitive Types

| Type | Description | JSON Representation |
|------|-------------|---------------------|
| `string` | UTF-16 string | JSON string |
| `number` | Double-precision floating point | JSON number |
| `boolean` | Boolean value | JSON boolean |
| `integer` | 32-bit signed integer | JSON number |
| `object` | Unstructured JSON object | JSON object |
| `any` | Any type | Any JSON value |
| `array` | Ordered collection | JSON array |

### 15.2 Special Types

| Type | Description | Example |
|------|-------------|---------|
| `binary` | Base64-encoded data | `"data:image/png;base64,iVBORw0KGgo..."` |
| `enum` | String enumeration | `"log"`, `"warning"`, `"error"` |
| `timestamp` | Time in milliseconds | `1622548800000` |

### 15.3 Type References

Types can be referenced across domains using the notation:
- `Domain.TypeName` (e.g., `DOM.NodeId`)

## 16. Additional Information

### 16.1 Protocol Extensions

Firefox DevTools Protocol may include Firefox-specific extensions not found in the Chrome DevTools Protocol. These extensions are indicated in the domain documentation.

### 16.2 Experimental Domains

Some domains or methods may be marked as experimental, meaning they:
- May change without notice
- May be removed in future versions
- May have incomplete functionality
- May not work in all Firefox versions

### 16.3 Protocol Version History

| Firefox Version | Protocol Version | Major Changes |
|-----------------|------------------|---------------|
| 60 | 1.0 | Initial protocol version |
| 70 | 1.1 | Added Network domain improvements |
| 80 | 1.2 | Added Performance domain |
| 90 | 1.3 | Enhanced Debugger capabilities |
| 95 | 1.4 | Added Storage domain improvements |

## 17. Error Codes Reference

| Error Code | Name | Description |
|------------|------|-------------|
| -32700 | Parse error | Invalid JSON was received |
| -32600 | Invalid Request | The JSON sent is not a valid Request object |
| -32601 | Method not found | The method does not exist / is not available |
| -32602 | Invalid params | Invalid method parameter(s) |
| -32603 | Internal error | Internal JSON-RPC error |
| -32000 | Server error | Implementation-defined server error |
| -32001 | Session not found | The specified session does not exist |
| -32002 | Node not found | The specified DOM node does not exist |
| -32003 | Object not found | The specified JavaScript object does not exist |
| -32004 | Breakpoint not found | The specified breakpoint does not exist |

## 18. Version History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-03-26 | 1.0.0 | Initial domain reference documentation | AI Assistant |
