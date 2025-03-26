# Chrome DevTools Protocol (CDP) 文档

本目录包含了Chrome DevTools Protocol (CDP)的详细协议文档。CDP是一个用于与Chrome浏览器进行通信和控制的协议，它允许开发者通过程序化的方式来操作和监控Chrome浏览器。

## 文档结构

### 1. [browser_protocol.md](./browser_protocol.md)
浏览器域（Browser Domain）协议文档，包含：
- 基础浏览器操作（关闭、获取版本等）
- 权限管理（设置、授予、重置权限）
- 窗口管理（获取和设置窗口边界）
- 下载管理（设置下载行为、取消下载）
- 诊断和调试功能
- 相关事件和类型定义

### 2. [target_protocol.md](./target_protocol.md)
目标域（Target Domain）协议文档，包含：
- 目标管理（激活、附加、关闭目标）
- 浏览器上下文管理（创建和管理上下文）
- 会话管理（分离会话、自动附加设置）
- 相关事件（目标创建、销毁、崩溃等）
- 类型定义（SessionID、TargetID、TargetInfo等）

### 3. [page_protocol.md](./page_protocol.md)
页面域（Page Domain）协议文档，包含：
- 页面生命周期控制（启用、禁用、关闭）
- 页面导航管理（导航、重载、历史记录）
- 页面内容操作（截图、打印PDF）
- JavaScript对话框处理
- 框架树管理
- 相关事件（加载、导航、对话框等）
- 类型定义（FrameId、ScriptIdentifier等）

### 4. [css_protocol.md](./css_protocol.md)
CSS域（CSS Domain）协议文档，包含：
- 样式表操作（创建、修改、删除）
- 样式计算（获取计算样式、匹配样式）
- 样式规则管理（添加规则、修改规则）
- CSS动画和过渡
- 媒体查询和容器查询
- 相关事件（样式表变更、字体更新等）
- 类型定义（StyleSheetId、CSSStyle等）

### 5. [dom_protocol.md](./dom_protocol.md)
DOM域（DOM Domain）协议文档，包含：
- DOM节点操作（查询、修改、删除）
- 属性管理（获取、设置、删除属性）
- 节点遍历和查找
- Shadow DOM操作
- 事件监听（节点变更、属性修改等）
- 节点高亮和定位
- 类型定义（NodeId、Node、BoxModel等）

## 使用说明

这些文档详细描述了CDP的各个方法、事件和类型定义。每个方法都包含：
- 详细的描述
- 参数列表及其类型
- 返回值定义
- 可能的错误情况

## 文档更新

这些文档基于Chrome DevTools Protocol的最新规范。如需了解具体版本信息，请参考浏览器的`Browser.getVersion()`方法返回的协议版本。

## 相关资源

- [Chrome DevTools Protocol 官方文档](https://chromedevtools.github.io/devtools-protocol/)
- [Chrome DevTools Protocol Viewer](https://chromedevtools.github.io/devtools-protocol/tot/)
- [CDP GitHub 仓库](https://github.com/ChromeDevTools/devtools-protocol)

## 注意事项

1. 某些方法和事件可能标记为实验性或已废弃，使用时请注意查看相关说明
2. 部分功能可能需要特定的Chrome版本支持
3. 在使用这些API时，建议先测试目标Chrome版本是否支持相应的功能 