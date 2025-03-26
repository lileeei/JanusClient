# Chrome DevTools Protocol - Page Domain

Page Domain定义了与被检查页面相关的所有操作和事件。本文档详细记录了相关的协议内容。

## Methods (方法)

### 1. 页面生命周期控制

#### Page.enable()
- **描述**：启用页面域的事件通知
- **参数**：无
- **返回**：无

#### Page.disable()
- **描述**：禁用页面域的事件通知
- **参数**：无
- **返回**：无

#### Page.close()
- **描述**：关闭当前页面
- **参数**：无
- **返回**：无

#### Page.stopLoading()
- **描述**：强制停止页面加载
- **参数**：无
- **返回**：无

#### Page.setLifecycleEventsEnabled()
- **描述**：启用/禁用生命周期事件
- **参数**：
```typescript
{
  enabled: boolean;          // 是否启用生命周期事件
}
```
- **返回**：无

### 2. 页面导航与历史

#### Page.navigate()
- **描述**：导航到指定URL
- **参数**：
```typescript
{
  url: string;                // 目标URL
  referrer?: string;         // 引用页URL
  transitionType?: string;   // 转换类型
  frameId?: string;          // 框架ID
}
```
- **返回**：
```typescript
{
  frameId: string;           // 导航的框架ID
  loaderId: string;          // 加载器ID
  errorText?: string;        // 错误信息（如果导航失败）
}
```

#### Page.reload()
- **描述**：重新加载当前页面
- **参数**：
```typescript
{
  ignoreCache?: boolean;     // 是否忽略缓存
  scriptToEvaluateOnLoad?: string; // 页面加载时执行的脚本
}
```
- **返回**：无

#### Page.getNavigationHistory()
- **描述**：获取导航历史
- **参数**：无
- **返回**：导航历史条目列表

#### Page.navigateToHistoryEntry()
- **描述**：导航到指定的历史条目
- **参数**：
```typescript
{
  entryId: number;           // 历史条目ID
}
```
- **返回**：无

#### Page.resetNavigationHistory()
- **描述**：重置导航历史
- **参数**：无
- **返回**：无

### 3. 页面内容操作

#### Page.captureScreenshot()
- **描述**：捕获页面截图
- **参数**：
```typescript
{
  format?: string;           // 图片格式（默认png）
  quality?: number;          // 图片质量（仅用于jpeg）
  clip?: {                   // 裁剪区域
    x: number;
    y: number;
    width: number;
    height: number;
    scale?: number;
  };
  fromSurface?: boolean;     // 是否从表面捕获
  captureBeyondViewport?: boolean; // 是否捕获视口之外的内容
}
```
- **返回**：
```typescript
{
  data: string;              // Base64编码的图片数据
}
```

#### Page.printToPDF()
- **描述**：生成页面的PDF
- **参数**：
```typescript
{
  landscape?: boolean;       // 是否横向
  displayHeaderFooter?: boolean; // 是否显示页眉页脚
  printBackground?: boolean; // 是否打印背景
  scale?: number;           // 缩放比例
  paperWidth?: number;      // 纸张宽度
  paperHeight?: number;     // 纸张高度
  marginTop?: number;       // 上边距
  marginBottom?: number;    // 下边距
  marginLeft?: number;      // 左边距
  marginRight?: number;     // 右边距
  pageRanges?: string;      // 页面范围
  headerTemplate?: string;  // 页眉模板
  footerTemplate?: string;  // 页脚模板
  preferCSSPageSize?: boolean; // 是否优先使用CSS页面尺寸
}
```
- **返回**：
```typescript
{
  data: string;              // Base64编码的PDF数据
}
```

#### Page.setDocumentContent()
- **描述**：设置页面HTML内容
- **参数**：
```typescript
{
  frameId: string;          // 框架ID
  html: string;             // HTML内容
}
```
- **返回**：无

### 4. 脚本执行控制

#### Page.addScriptToEvaluateOnNewDocument()
- **描述**：添加在新文档中执行的脚本
- **参数**：
```typescript
{
  source: string;           // 脚本源代码
  worldName?: string;       // 执行上下文名称
}
```
- **返回**：
```typescript
{
  identifier: string;       // 脚本标识符
}
```

#### Page.removeScriptToEvaluateOnNewDocument()
- **描述**：移除在新文档中执行的脚本
- **参数**：
```typescript
{
  identifier: string;       // 脚本标识符
}
```
- **返回**：无

#### Page.createIsolatedWorld()
- **描述**：创建独立的JavaScript执行环境
- **参数**：
```typescript
{
  frameId: string;          // 框架ID
  worldName?: string;       // 执行环境名称
  grantUniveralAccess?: boolean; // 是否授予通用访问权限
}
```
- **返回**：
```typescript
{
  executionContextId: number; // 执行上下文ID
}
```

### 5. 应用程序管理

#### Page.getAppManifest()
- **描述**：获取Web应用程序清单
- **参数**：无
- **返回**：
```typescript
{
  url: string;              // 清单URL
  errors: array;            // 解析错误
  data?: string;           // 清单内容
  parsed?: object;         // 解析后的清单对象
}
```

#### Page.bringToFront()
- **描述**：将页面带到前台
- **参数**：无
- **返回**：无

### 6. 对话框处理

#### Page.handleJavaScriptDialog()
- **描述**：处理JavaScript对话框
- **参数**：
```typescript
{
  accept: boolean;          // 是否接受对话框
  promptText?: string;      // 提示文本（用于prompt对话框）
}
```
- **返回**：无

### 7. 安全设置

#### Page.setBypassCSP()
- **描述**：设置是否绕过内容安全策略
- **参数**：
```typescript
{
  enabled: boolean;         // 是否启用CSP绕过
}
```
- **返回**：无

### 8. 文件选择器设置

#### Page.setInterceptFileChooserDialog()
- **描述**：设置是否拦截文件选择器对话框
- **参数**：
```typescript
{
  enabled: boolean;         // 是否启用拦截
}
```
- **返回**：无

### 9. 框架和布局相关

#### Page.getFrameTree()
- **描述**：获取当前页面的框架树结构
- **参数**：无
- **返回**：
```typescript
{
  frameTree: {
    frame: Frame;           // 框架信息
    childFrames?: FrameTree[]; // 子框架
  }
}
```

#### Page.getLayoutMetrics()
- **描述**：获取页面布局度量信息
- **参数**：无
- **返回**：
```typescript
{
  layoutViewport: {         // 布局视口
    pageX: number;         // 页面X坐标
    pageY: number;         // 页面Y坐标
    clientWidth: number;   // 客户端宽度
    clientHeight: number;  // 客户端高度
  };
  visualViewport: {         // 可视视口
    offsetX: number;       // X偏移
    offsetY: number;       // Y偏移
    pageX: number;         // 页面X坐标
    pageY: number;         // 页面Y坐标
    clientWidth: number;   // 客户端宽度
    clientHeight: number;  // 客户端高度
    scale: number;        // 缩放比例
  };
  contentSize: {           // 内容大小
    width: number;        // 宽度
    height: number;       // 高度
  }
}
```

### 10. 实验性功能

#### Page.generateTestReport()
- **描述**：生成测试报告
- **参数**：
```typescript
{
  message: string;          // 报告消息
  group?: string;          // 报告分组
}
```
- **返回**：无

#### Page.waitForDebugger()
- **描述**：等待调试器连接
- **参数**：无
- **返回**：无

#### Page.setWebLifecycleState()
- **描述**：设置页面Web生命周期状态
- **参数**：
```typescript
{
  state: string;           // 生命周期状态
}
```
- **返回**：无

#### Page.getPermissionsPolicyState()
- **描述**：获取权限策略状态
- **参数**：
```typescript
{
  frameId: string;         // 框架ID
}
```
- **返回**：权限策略状态列表

#### Page.getOriginTrials()
- **描述**：获取源试验信息
- **参数**：
```typescript
{
  frameId: string;         // 框架ID
}
```
- **返回**：源试验信息列表

### 11. 屏幕广播控制

#### Page.startScreencast()
- **描述**：开始屏幕广播
- **参数**：
```typescript
{
  format?: string;         // 图像格式
  quality?: number;        // 图像质量
  maxWidth?: number;       // 最大宽度
  maxHeight?: number;      // 最大高度
  everyNthFrame?: number; // 每N帧捕获一次
}
```
- **返回**：无

#### Page.stopScreencast()
- **描述**：停止屏幕广播
- **参数**：无
- **返回**：无

#### Page.screencastFrameAck()
- **描述**：确认接收到屏幕广播帧
- **参数**：
```typescript
{
  sessionId: number;       // 会话ID
}
```
- **返回**：无

### 12. 广告拦截

#### Page.setAdBlockingEnabled()
- **描述**：启用/禁用广告拦截
- **参数**：
```typescript
{
  enabled: boolean;        // 是否启用广告拦截
}
```
- **返回**：无

### 13. 字体设置

#### Page.setFontFamilies()
- **描述**：设置字体族
- **参数**：
```typescript
{
  fontFamilies: {
    standard?: string;    // 标准字体
    fixed?: string;      // 等宽字体
    serif?: string;      // 衬线字体
    sansSerif?: string;  // 无衬线字体
    cursive?: string;    // 草书字体
    fantasy?: string;    // 装饰字体
    math?: string;       // 数学字体
  };
  forScripts?: array;    // 针对特定脚本的字体设置
}
```
- **返回**：无

#### Page.setFontSizes()
- **描述**：设置字体大小
- **参数**：
```typescript
{
  fontSizes: {
    standard?: number;    // 标准字体大小
    fixed?: number;      // 等宽字体大小
  }
}
```
- **返回**：无

## Events (事件)

### Page.loadEventFired
- **描述**：页面加载完成时触发
- **参数**：
```typescript
{
  timestamp: number;         // 事件时间戳
}
```

### Page.frameNavigated
- **描述**：框架导航完成时触发
- **参数**：
```typescript
{
  frame: {                   // 框架信息
    id: string;             // 框架ID
    parentId?: string;      // 父框架ID
    loaderId: string;       // 加载器ID
    name?: string;          // 框架名称
    url: string;            // 框架URL
    securityOrigin: string; // 安全源
  }
}
```

### Page.frameAttached
- **描述**：框架附加到目标时触发
- **参数**：
```typescript
{
  frameId: string;          // 框架ID
  parentFrameId?: string;   // 父框架ID
  stack?: Runtime.StackTrace; // 堆栈跟踪
}
```

### Page.frameDetached
- **描述**：框架从目标分离时触发
- **参数**：
```typescript
{
  frameId: string;          // 框架ID
  reason: string;           // 分离原因
}
```

### Page.frameStartedLoading
- **描述**：框架开始加载时触发
- **参数**：
```typescript
{
  frameId: string;          // 框架ID
}
```

### Page.frameStoppedLoading
- **描述**：框架停止加载时触发
- **参数**：
```typescript
{
  frameId: string;          // 框架ID
}
```

### Page.domContentEventFired
- **描述**：DOM内容加载事件触发时发送
- **参数**：
```typescript
{
  timestamp: number;        // 事件时间戳
}
```

### Page.javascriptDialogOpening
- **描述**：JavaScript对话框打开时触发
- **参数**：
```typescript
{
  url: string;              // 触发对话框的URL
  message: string;          // 对话框消息
  type: string;             // 对话框类型
  hasBrowserHandler: boolean; // 是否有浏览器处理程序
  defaultPrompt?: string;   // 默认提示文本
}
```

### Page.javascriptDialogClosed
- **描述**：JavaScript对话框关闭时触发
- **参数**：
```typescript
{
  result: boolean;          // 用户接受还是取消对话框
  userInput?: string;       // 用户输入的文本
}
```

### Page.screencastFrame
- **描述**：屏幕广播帧可用时触发
- **参数**：
```typescript
{
  data: string;             // Base64编码的帧数据
  metadata: {               // 帧元数据
    offsetTop: number;      // 帧顶部偏移
    pageScaleFactor: number; // 页面缩放因子
    deviceWidth: number;    // 设备宽度
    deviceHeight: number;   // 设备高度
    scrollOffsetX: number;  // 水平滚动偏移
    scrollOffsetY: number;  // 垂直滚动偏移
    timestamp?: number;     // 时间戳
  };
  sessionId: number;        // 会话ID
}
```

### Page.fileChooserOpened
- **描述**：文件选择器打开时触发
- **参数**：
```typescript
{
  frameId: string;          // 框架ID
  mode: string;             // 选择器模式
  backendNodeId?: DOM.BackendNodeId; // 后端节点ID
}
```

### Page.downloadWillBegin
- **描述**：下载即将开始时触发
- **参数**：
```typescript
{
  frameId: string;          // 框架ID
  guid: string;             // 下载唯一标识符
  url: string;              // 下载URL
  suggestedFilename: string; // 建议的文件名
}
```

### Page.downloadProgress
- **描述**：下载进度更新时触发
- **参数**：
```typescript
{
  guid: string;             // 下载唯一标识符
  totalBytes: number;       // 总字节数
  receivedBytes: number;    // 已接收字节数
  state: string;            // 下载状态
}
```

### Page.interstitialHidden
- **描述**：插页广告隐藏时触发
- **参数**：无

### Page.interstitialShown
- **描述**：插页广告显示时触发
- **参数**：无

### Page.lifecycleEvent
- **描述**：生命周期事件触发时发送
- **参数**：
```typescript
{
  frameId: string;          // 框架ID
  loaderId: string;         // 加载器ID
  name: string;             // 事件名称
  timestamp: number;        // 时间戳
}
```

### Page.windowOpen
- **描述**：窗口打开时触发
- **参数**：
```typescript
{
  url: string;              // 打开的URL
  windowName: string;       // 窗口名称
  windowFeatures: string[]; // 窗口特性列表
  userGesture: boolean;     // 是否由用户手势触发
}
```

## Types (类型定义)

### Page.FrameId
- **类型**：`string`
- **描述**：唯一标识框架的ID

### Page.ScriptIdentifier
- **类型**：`string`
- **描述**：唯一标识脚本的ID

### Page.TransitionType
- **类型**：`string`
- **可选值**：
  - `navigation`
  - `reload`
  - `backForward`
  - `formSubmit`
  - `formResubmit`
  - `other`

### Page.DialogType
- **类型**：`string`
- **可选值**：
  - `alert`
  - `confirm`
  - `prompt`
  - `beforeunload`

### Page.ClientNavigationReason
- **类型**：`string`
- **可选值**：
  - `formSubmissionGet`
  - `formSubmissionPost`
  - `httpHeaderRefresh`
  - `scriptInitiated`
  - `metaTagRefresh`
  - `pageBlockInterstitial`
  - `reload`
  - `anchorClick`

### Page.FrameNavigationReason
- **类型**：`string`
- **可选值**：
  - `formSubmissionGet`
  - `formSubmissionPost`
  - `httpHeaderRefresh`
  - `scriptInitiated`
  - `metaTagRefresh`
  - `pageBlockInterstitial`
  - `reload`
  - `anchorClick`

### Page.NavigationResponse
- **类型**：`string`
- **可选值**：
  - `Proceed`
  - `Cancel`
  - `CancelAndIgnore`

### Page.PrintOrientation
- **类型**：`string`
- **可选值**：
  - `portrait`
  - `landscape`

### Page.FileChooserOpenedMode
- **类型**：`string`
- **可选值**：
  - `selectSingle`
  - `selectMultiple`


## 注意事项

### 实验性功能
以下功能标记为实验性(Experimental)，可能在未来版本中发生变化：

#### Methods
- Page.addCompilationCache
- Page.captureSnapshot
- Page.clearCompilationCache
- Page.crash
- Page.generateTestReport
- Page.getAdScriptId
- Page.getAppId
- Page.getInstallabilityErrors
- Page.getOriginTrials
- Page.getPermissionsPolicyState
- Page.getResourceContent
- Page.getResourceTree
- Page.produceCompilationCache
- Page.screencastFrameAck
- Page.searchInResource
- Page.setAdBlockingEnabled
- Page.setFontFamilies
- Page.setFontSizes
- Page.setPrerenderingAllowed
- Page.setRPHRegistrationMode
- Page.setSPCTransactionMode
- Page.setWebLifecycleState
- Page.startScreencast
- Page.stopScreencast
- Page.waitForDebugger

#### Types
- Page.AdFrameExplanation
- Page.AdFrameStatus
- Page.AdFrameType
- Page.AdScriptId
- Page.AppManifestParsedProperties
- Page.AutoResponseMode
- Page.BackForwardCacheBlockingDetails
- Page.BackForwardCacheNotRestoredExplanation
- Page.BackForwardCacheNotRestoredExplanationTree
- Page.BackForwardCacheNotRestoredReason
- Page.BackForwardCacheNotRestoredReasonType
- Page.ClientNavigationDisposition
- Page.ClientNavigationReason
- Page.CompilationCacheParams
- Page.CrossOriginIsolatedContextType
- Page.FileFilter
- Page.FileHandler
- Page.FontFamilies
- Page.FontSizes
- Page.FrameResource
- Page.FrameResourceTree
- Page.GatedAPIFeatures
- Page.ImageResource
- Page.InstallabilityError
- Page.InstallabilityErrorArgument
- Page.LaunchHandler
- Page.NavigationType
- Page.OriginTrial
- Page.OriginTrialStatus
- Page.OriginTrialToken
- Page.OriginTrialTokenStatus
- Page.OriginTrialTokenWithStatus
- Page.OriginTrialUsageRestriction
- Page.PermissionsPolicyBlockLocator
- Page.PermissionsPolicyBlockReason
- Page.PermissionsPolicyFeature
- Page.PermissionsPolicyFeatureState
- Page.ProtocolHandler
- Page.ReferrerPolicy
- Page.RelatedApplication
- Page.ScopeExtension
- Page.ScreencastFrameMetadata
- Page.Screenshot
- Page.ScriptFontFamilies
- Page.SecureContextType
- Page.SecurityOriginDetails
- Page.ShareTarget
- Page.Shortcut
- Page.WebAppManifest

### 废弃功能
以下功能已被标记为废弃(Deprecated)，不建议在新代码中使用：

#### Methods
- Page.clearGeolocationOverride
- Page.setGeolocationOverride
- Page.addScriptToEvaluateOnLoad
- Page.clearDeviceMetricsOverride
- Page.clearDeviceOrientationOverride
- Page.deleteCookie
- Page.getManifestIcons
- Page.removeScriptToEvaluateOnLoad
- Page.setDeviceMetricsOverride
- Page.setDeviceOrientationOverride
- Page.setDownloadBehavior
- Page.setTouchEmulationEnabled

## 参考链接

- [Chrome DevTools Protocol Viewer - Page Domain](https://chromedevtools.github.io/devtools-protocol/tot/Page)
