# Chrome DevTools Protocol - Browser Domain

Browser Domain定义了浏览器管理相关的方法和事件。本文档详细记录了相关的协议内容。

## Methods (方法)

### 1. 基础浏览器操作

#### Browser.close()
- **描述**：优雅地关闭浏览器
- **参数**：无
- **返回**：无

#### Browser.getVersion()
- **描述**：获取版本信息
- **参数**：无
- **返回**：
```typescript
{
  protocolVersion: string;      // 协议版本
  product: string;              // 产品名称，如"Chrome/94.0.4606.71"
  revision: string;             // 浏览器修订版本
  userAgent: string;            // User Agent字符串
  jsVersion: string;            // JavaScript引擎版本
}
```

#### Browser.crash() (实验性)
- **描述**：在主线程上崩溃浏览器
- **参数**：无
- **返回**：无

#### Browser.crashGpuProcess() (实验性)
- **描述**：崩溃GPU进程
- **参数**：无
- **返回**：无

### 2. 权限管理

#### Browser.setPermission()
- **描述**：为指定源设置权限设置
- **参数**：
```typescript
{
  permission: {                 // 权限描述符
    name: string;              // 权限名称
    sysex?: boolean;           // 系统专属权限标志
    userVisibleOnly?: boolean; // 用户可见权限标志
    allowWithoutSanitization?: boolean;
    panTiltZoom?: boolean;     // 是否包含PTZ控制
  };
  setting: PermissionSetting;   // 权限设置（'granted'|'denied'|'prompt'）
  origin: string;               // 源地址
  browserContextId?: string;    // 浏览器上下文ID
}
```
- **返回**：无

#### Browser.grantPermissions()
- **描述**：授予特定权限给指定源
- **参数**：
```typescript
{
  permissions: PermissionType[]; // 权限类型数组
  origin?: string;              // 源地址
  browserContextId?: string;    // 浏览器上下文ID
}
```
- **返回**：无

#### Browser.resetPermissions()
- **描述**：重置所有权限管理
- **参数**：
```typescript
{
  browserContextId?: string;    // 浏览器上下文ID
}
```
- **返回**：无

### 3. 窗口管理

#### Browser.getWindowBounds()
- **描述**：获取浏览器窗口的位置和大小
- **参数**：
```typescript
{
  windowId: WindowID;          // 窗口ID
}
```
- **返回**：
```typescript
{
  bounds: {                    // 窗口边界信息
    left?: number;            // 左边距
    top?: number;             // 顶边距
    width?: number;           // 宽度
    height?: number;          // 高度
    windowState?: WindowState; // 窗口状态
  }
}
```

#### Browser.setWindowBounds()
- **描述**：设置浏览器窗口的位置和大小
- **参数**：
```typescript
{
  windowId: WindowID;          // 窗口ID
  bounds: {                    // 窗口边界信息
    left?: number;
    top?: number;
    width?: number;
    height?: number;
    windowState?: WindowState;
  }
}
```
- **返回**：无

### 4. 下载管理

#### Browser.setDownloadBehavior()
- **描述**：设置下载行为
- **参数**：
```typescript
{
  behavior: string;            // 下载行为 ('allow'|'deny'|'allowAndName')
  browserContextId?: string;   // 浏览器上下文ID
  downloadPath?: string;       // 下载路径
  eventsEnabled?: boolean;     // 是否启用事件
}
```
- **返回**：无

#### Browser.cancelDownload()
- **描述**：取消进行中的下载
- **参数**：
```typescript
{
  guid: string;               // 下载的唯一标识符
  browserContextId?: string;  // 浏览器上下文ID
}
```
- **返回**：无

### 5. 诊断和调试

#### Browser.getHistogram()
- **描述**：获取指定名称的Chrome直方图
- **参数**：
```typescript
{
  name: string;               // 直方图名称
  delayMs?: number;          // 延迟毫秒数
}
```
- **返回**：
```typescript
{
  histogram: {
    name: string;            // 直方图名称
    sum: number;             // 总和
    count: number;           // 计数
    buckets: Array<{        // 存储桶数组
      low: number;          // 下限
      high: number;         // 上限
      count: number;        // 计数
    }>
  }
}
```

## Events (事件)

### Browser.downloadProgress
- **描述**：下载进度更新事件
- **参数**：
```typescript
{
  guid: string;              // 下载的唯一标识符
  totalBytes: number;        // 总字节数
  receivedBytes: number;     // 已接收字节数
  state: string;            // 下载状态
}
```

### Browser.downloadWillBegin
- **描述**：下载即将开始事件
- **参数**：
```typescript
{
  frameId: string;          // 框架ID
  guid: string;             // 下载的唯一标识符
  url: string;              // 下载URL
  suggestedFilename: string; // 建议的文件名
}
```

## Types (类型定义)

### PermissionType (枚举)
可能的值包括：
```typescript
type PermissionType = 
  | 'accessibilityEvents'
  | 'audioCapture'
  | 'backgroundSync'
  | 'backgroundFetch'
  | 'clipboardReadWrite'
  | 'clipboardSanitizedWrite'
  | 'displayCapture'
  | 'durableStorage'
  | 'flash'
  | 'geolocation'
  | 'midi'
  | 'midiSysex'
  | 'nfc'
  | 'notifications'
  | 'paymentHandler'
  | 'periodicBackgroundSync'
  | 'protectedMediaIdentifier'
  | 'sensors'
  | 'videoCapture'
  | 'videoCapturePanTiltZoom'
  | 'idleDetection'
  | 'wakeLockScreen'
  | 'wakeLockSystem'
```

### WindowState (枚举)
```typescript
type WindowState = 
  | 'normal'      // 正常状态
  | 'minimized'   // 最小化
  | 'maximized'   // 最大化
  | 'fullscreen'  // 全屏
```

### PermissionSetting (枚举)
```typescript
type PermissionSetting = 
  | 'granted'     // 已授权
  | 'denied'      // 已拒绝
  | 'prompt'      // 提示
```

### BrowserCommandId (枚举)
```typescript
type BrowserCommandId =
  | 'openTabSearch'
  | 'closeTabSearch'
  | 'toggleTabSearch'
  | 'openTabFind'
  | 'closeTabFind'
  | 'toggleTabFind'
  | 'selectNextTab'
  | 'selectPreviousTab'
  | 'closeTab'
  | 'reloadTab'
  | 'duplicateTab'
```

### WindowID (类型)
```typescript
type WindowID = number;
```

### Bounds (类型)
```typescript
interface Bounds {
  left?: number;
  top?: number;
  width?: number;
  height?: number;
  windowState?: WindowState;
}
```

### DownloadProgressEvent (类型)
```typescript
interface DownloadProgressEvent {
  guid: string;           // 下载的唯一标识符
  totalBytes: number;     // 总字节数
  receivedBytes: number;  // 已接收字节数
  state: "inProgress" | "completed" | "canceled" | "interrupted";
}
```

## 注意事项

1. 所有标记为实验性(Experimental)的API都可能在未来版本中发生变化，使用时需要注意。
2. 这些API提供了完整的浏览器控制能力，包括：
   - 基础浏览器操作（启动、关闭、版本信息）
   - 权限管理（设置、授予、重置权限）
   | 窗口管理（获取和设置窗口位置、大小）
   - 下载管理（控制下载行为、监控下载进度）
   - 诊断和调试功能（获取性能直方图等）

### 实验性功能
以下功能被标记为实验性(Experimental)，在未来版本中可能会发生变化：

#### Methods
- Browser.cancelDownload
- Browser.crash
- Browser.crashGpuProcess
- Browser.executeBrowserCommand
- Browser.getBrowserCommandLine
- Browser.getHistogram
- Browser.getHistograms
- Browser.getWindowBounds
- Browser.getWindowForTarget
- Browser.grantPermissions
- Browser.setDockTile
- Browser.setDownloadBehavior
- Browser.setPermission
- Browser.setWindowBounds

#### Events
- Browser.downloadProgress
- Browser.downloadWillBegin

### 废弃功能
目前Browser Domain中没有被标记为废弃(Deprecated)的功能。

## 参考链接

- [Chrome DevTools Protocol Viewer - Browser Domain](https://chromedevtools.github.io/devtools-protocol/tot/Browser) 