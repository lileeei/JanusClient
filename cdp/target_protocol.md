# Chrome DevTools Protocol - Target Domain

Target Domain支持额外目标的发现并允许附加到这些目标。本文档详细记录了相关的协议内容。

## Methods (方法)

### 1. 目标管理

#### Target.activateTarget
- **描述**：激活（聚焦）目标
- **参数**：
```typescript
{
  targetId: TargetID;         // 目标ID
}
```
- **返回**：无

#### Target.attachToTarget
- **描述**：附加到指定ID的目标
- **参数**：
```typescript
{
  targetId: TargetID;         // 目标ID
  flatten: boolean;           // 启用"扁平化"访问会话，通过在命令中指定sessionId属性
}
```
- **返回**：
```typescript
{
  sessionId: SessionID;       // 分配给会话的ID
}
```

#### Target.closeTarget
- **描述**：关闭目标。如果目标是页面，页面也会被关闭
- **参数**：
```typescript
{
  targetId: TargetID;         // 目标ID
}
```
- **返回**：
```typescript
{
  success: boolean;           // 总是设置为true。如果发生错误，响应会指示协议错误
}
```
- **注意**：已废弃

### 2. 浏览器上下文管理

#### Target.createBrowserContext
- **描述**：创建新的空浏览器上下文。类似于隐身模式配置文件，但可以有多个
- **参数**：
```typescript
{
  disposeOnDetach?: boolean;  // 如果指定，在调试会话断开时处理此上下文
  proxyServer?: string;       // 代理服务器，类似于传递给--proxy-server的参数
  proxyBypassList?: string;   // 代理绕过列表，类似于传递给--proxy-bypass-list的参数
  originsWithUniversalNetworkAccess?: string[]; // 可选的源列表，授予无限制的跨源访问权限
}
```
- **返回**：
```typescript
{
  browserContextId: Browser.BrowserContextID; // 创建的上下文ID
}
```

#### Target.createTarget
- **描述**：创建新页面
- **参数**：
```typescript
{
  url: string;               // 页面将导航到的初始URL。空字符串表示about:blank
  width?: number;            // 框架宽度（DIP）
  height?: number;           // 框架高度（DIP）
  browserContextId?: Browser.BrowserContextID; // 创建页面的浏览器上下文
  enableBeginFrameControl?: boolean; // 是否通过DevTools控制此目标的BeginFrames
  newWindow?: boolean;       // 是否创建新窗口或标签页（默认false）
  background?: boolean;      // 是否在后台创建目标（默认false）
  left?: number;            // 框架左侧原点（DIP）
  top?: number;             // 框架顶部原点（DIP）
  windowState?: WindowState; // 框架窗口状态
  forTab?: boolean;         // 是否创建"tab"类型的目标
}
```
- **返回**：
```typescript
{
  targetId: TargetID;        // 打开的页面ID
}
```

### 3. 会话管理

#### Target.detachFromTarget
- **描述**：分离指定ID的会话
- **参数**：
```typescript
{
  sessionId: SessionID;      // 要分离的会话
  targetId?: TargetID;      // 已废弃
}
```
- **返回**：无

#### Target.setAutoAttach
- **描述**：控制是否自动附加到新目标
- **参数**：
```typescript
{
  autoAttach: boolean;       // 是否自动附加到目标
  waitForDebuggerOnStart: boolean; // 是否等待调试器
  flatten?: boolean;         // 是否使用扁平化会话
}
```
- **返回**：无

## Events (事件)

### Target.attachedToTarget
- **描述**：当目标被附加时触发
- **参数**：
```typescript
{
  sessionId: SessionID;      // 附加的会话ID
  targetInfo: TargetInfo;    // 目标信息
  waitingForDebugger: boolean; // 是否等待调试器
}
```

### Target.detachedFromTarget
- **描述**：当目标被分离时触发
- **参数**：
```typescript
{
  sessionId: SessionID;      // 分离的会话ID
  targetId?: TargetID;      // 目标ID
}
```

### Target.targetCreated
- **描述**：当新目标创建时触发
- **参数**：
```typescript
{
  targetInfo: TargetInfo;    // 目标信息
}
```

### Target.targetDestroyed
- **描述**：当目标被销毁时触发
- **参数**：
```typescript
{
  targetId: TargetID;        // 目标ID
}
```

### Target.targetCrashed
- **描述**：当目标崩溃时触发
- **参数**：
```typescript
{
  targetId: TargetID;        // 目标ID
  status: string;            // 终止状态
  errorCode: number;         // 错误码
}
```

## Types (类型定义)

### SessionID
- **描述**：附加调试会话的唯一标识符
- **类型**：string

### TargetID
- **类型**：string

### TargetInfo
- **类型**：object
- **属性**：
```typescript
{
  targetId: TargetID;        // 目标ID
  type: string;              // 目标类型
  title: string;             // 标题
  url: string;               // URL
  attached: boolean;         // 是否有客户端附加
  openerId?: TargetID;       // 打开者目标ID
  browserContextId?: Browser.BrowserContextID; // 浏览器上下文ID
  canAccessOpener: boolean;  // 是否可以访问打开者窗口
  openerFrameId?: Page.FrameId; // 打开者窗口的框架ID
  subtype?: string;          // 特定目标类型的其他详细信息
}
```

### WindowState (枚举)
```typescript
type WindowState = 
  | 'normal'      // 正常状态
  | 'minimized'   // 最小化
  | 'maximized'   // 最大化
  | 'fullscreen'  // 全屏
```

## 注意事项

1. Target Domain主要用于：
   - 目标管理（创建、关闭、激活目标）
   - 浏览器上下文管理（创建和管理类似隐身模式的上下文）
   - 会话管理（附加、分离会话）
   - 目标监控（通过事件监听目标的生命周期）

2. 部分功能标记为实验性(Experimental)，在使用时需要注意：
   - 某些窗口管理功能
   - 部分浏览器上下文功能
   - 特定目标类型的附加信息

3. 已废弃的功能：
   - Target.closeTarget的success返回值
   - Target.detachFromTarget中的targetId参数

## 参考链接

- [Chrome DevTools Protocol - Target Domain](https://chromedevtools.github.io/devtools-protocol/tot/Target/) 