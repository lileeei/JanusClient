# Chrome DevTools Protocol - DOM Domain

DOM Domain提供了DOM读写操作的能力。每个DOM节点都有一个`id`作为其镜像对象的标识。这个`id`可以用于获取节点的额外信息，将其解析为JavaScript对象包装器等。重要的是客户端只接收已知节点的DOM事件。后端会跟踪已发送给客户端的节点，并且永远不会两次发送相同的节点。客户端负责收集已发送节点的信息。注意，`iframe`所有者元素将返回相应的文档元素作为其子节点。

## Methods (方法)

### 1. 基础操作

#### DOM.enable()
- **描述**：启用DOM域
- **参数**：无
- **返回**：无

#### DOM.disable()
- **描述**：禁用DOM域
- **参数**：无
- **返回**：无

### 2. 节点操作

#### DOM.getDocument
- **描述**：返回根文档节点
- **参数**：无
- **返回**：
```typescript
{
  root: Node;               // 根节点
}
```

#### DOM.requestChildNodes
- **描述**：请求节点的子节点
- **参数**：
```typescript
{
  nodeId: NodeId;           // 节点ID
  depth?: integer;          // 遍历深度（可选）
  pierce?: boolean;         // 是否穿透Shadow DOM（可选）
}
```
- **返回**：无

#### DOM.querySelector
- **描述**：查询与选择器匹配的节点
- **参数**：
```typescript
{
  nodeId: NodeId;           // 节点ID
  selector: string;         // CSS选择器
}
```
- **返回**：
```typescript
{
  nodeId: NodeId;           // 匹配节点的ID
}
```

#### DOM.querySelectorAll
- **描述**：查询与选择器匹配的所有节点
- **参数**：
```typescript
{
  nodeId: NodeId;           // 节点ID
  selector: string;         // CSS选择器
}
```
- **返回**：
```typescript
{
  nodeIds: NodeId[];        // 匹配节点ID数组
}
```

### 3. 节点属性操作

#### DOM.getAttributes
- **描述**：获取节点的所有属性
- **参数**：
```typescript
{
  nodeId: NodeId;           // 节点ID
}
```
- **返回**：
```typescript
{
  attributes: string[];     // 属性名值对数组
}
```

#### DOM.setAttributeValue
- **描述**：设置节点属性值
- **参数**：
```typescript
{
  nodeId: NodeId;           // 节点ID
  name: string;            // 属性名
  value: string;           // 属性值
}
```
- **返回**：无

#### DOM.setAttributesAsText
- **描述**：以文本形式设置节点属性
- **参数**：
```typescript
{
  nodeId: NodeId;           // 节点ID
  text: string;            // 属性文本
  name?: string;           // 要修改的属性名（可选）
}
```
- **返回**：无

#### DOM.removeAttribute
- **描述**：移除节点属性
- **参数**：
```typescript
{
  nodeId: NodeId;           // 节点ID
  name: string;            // 属性名
}
```
- **返回**：无

### 4. 节点内容操作

#### DOM.getOuterHTML
- **描述**：获取节点的外部HTML
- **参数**：
```typescript
{
  nodeId: NodeId;           // 节点ID
}
```
- **返回**：
```typescript
{
  outerHTML: string;        // 外部HTML
}
```

#### DOM.setOuterHTML
- **描述**：设置节点的外部HTML
- **参数**：
```typescript
{
  nodeId: NodeId;           // 节点ID
  outerHTML: string;       // 新的外部HTML
}
```
- **返回**：无

### 5. 节点高亮和定位

#### DOM.highlightNode
- **描述**：高亮显示节点
- **参数**：无
- **返回**：无

#### DOM.hideHighlight
- **描述**：隐藏节点高亮
- **参数**：无
- **返回**：无

#### DOM.scrollIntoViewIfNeeded
- **描述**：如果需要则将节点滚动到可见区域
- **参数**：
```typescript
{
  nodeId: NodeId;           // 节点ID
}
```
- **返回**：无

### 6. 实验性方法

#### DOM.collectClassNamesFromSubtree
- **描述**：从子树中收集类名
- **实验性**

#### DOM.copyTo
- **描述**：复制节点
- **实验性**

#### DOM.getContainerForNode
- **描述**：获取节点的容器
- **实验性**

#### DOM.getContentQuads
- **描述**：获取内容四边形
- **实验性**

#### DOM.getNodeStackTraces
- **描述**：获取节点堆栈跟踪
- **实验性**

### 7. 废弃方法

#### DOM.getFlattenedDocument
- **描述**：获取扁平化的文档
- **废弃**

## Events (事件)

### 1. 节点变更事件

#### DOM.documentUpdated
- **描述**：文档更新时触发
- **参数**：无

#### DOM.setChildNodes
- **描述**：设置子节点时触发
- **参数**：
```typescript
{
  parentId: NodeId;         // 父节点ID
  nodes: Node[];           // 子节点数组
}
```

#### DOM.attributeModified
- **描述**：属性修改时触发
- **参数**：
```typescript
{
  nodeId: NodeId;           // 节点ID
  name: string;            // 属性名
  value: string;           // 属性值
}
```

#### DOM.attributeRemoved
- **描述**：属性移除时触发
- **参数**：
```typescript
{
  nodeId: NodeId;           // 节点ID
  name: string;            // 属性名
}
```

#### DOM.characterDataModified
- **描述**：字符数据修改时触发
- **参数**：
```typescript
{
  nodeId: NodeId;           // 节点ID
  characterData: string;    // 新的字符数据
}
```

#### DOM.childNodeCountUpdated
- **描述**：子节点数量更新时触发
- **参数**：
```typescript
{
  nodeId: NodeId;           // 节点ID
  childNodeCount: integer;  // 新的子节点数量
}
```

#### DOM.childNodeInserted
- **描述**：插入子节点时触发
- **参数**：
```typescript
{
  parentNodeId: NodeId;     // 父节点ID
  previousNodeId: NodeId;   // 前一个节点ID
  node: Node;              // 插入的节点
}
```

#### DOM.childNodeRemoved
- **描述**：移除子节点时触发
- **参数**：
```typescript
{
  parentNodeId: NodeId;     // 父节点ID
  nodeId: NodeId;          // 被移除的节点ID
}
```

### 2. 实验性事件

#### DOM.distributedNodesUpdated
- **描述**：分布式节点更新时触发
- **实验性**

#### DOM.inlineStyleInvalidated
- **描述**：内联样式失效时触发
- **实验性**

#### DOM.pseudoElementAdded
- **描述**：添加伪元素时触发
- **实验性**

#### DOM.pseudoElementRemoved
- **描述**：移除伪元素时触发
- **实验性**

#### DOM.shadowRootPushed
- **描述**：Shadow Root推入时触发
- **实验性**

#### DOM.shadowRootPopped
- **描述**：Shadow Root弹出时触发
- **实验性**

## Types (类型定义)

### 1. 基础类型

#### DOM.NodeId
- **类型**：integer
- **描述**：DOM节点的唯一标识符

#### DOM.BackendNodeId
- **类型**：integer
- **描述**：后端节点的唯一标识符

#### DOM.BackendNode
- **类型**：object
- **描述**：后端节点的最小信息
- **属性**：
```typescript
{
  nodeType: integer;        // 节点类型
  nodeName: string;        // 节点名称
  backendNodeId: BackendNodeId; // 后端节点ID
}
```

### 2. 节点相关类型

#### DOM.Node
- **类型**：object
- **描述**：DOM节点
- **属性**：
```typescript
{
  nodeId: NodeId;           // 节点ID
  parentId?: NodeId;        // 父节点ID
  backendNodeId: BackendNodeId; // 后端节点ID
  nodeType: integer;        // 节点类型
  nodeName: string;        // 节点名称
  localName: string;       // 本地名称
  nodeValue: string;       // 节点值
  childNodeCount?: integer; // 子节点数量
  children?: Node[];       // 子节点数组
  attributes?: string[];   // 属性数组
  documentURL?: string;    // 文档URL
  baseURL?: string;        // 基础URL
  publicId?: string;       // 公共ID
  systemId?: string;       // 系统ID
  internalSubset?: string; // 内部子集
  xmlVersion?: string;     // XML版本
  name?: string;          // 名称
  value?: string;         // 值
  pseudoType?: PseudoType; // 伪类型
  shadowRootType?: ShadowRootType; // Shadow Root类型
  frameId?: string;       // 框架ID
  contentDocument?: Node;  // 内容文档
  shadowRoots?: Node[];   // Shadow Roots
  templateContent?: Node;  // 模板内容
  pseudoElements?: Node[]; // 伪元素
  importedDocument?: Node; // 导入的文档
  distributedNodes?: BackendNode[]; // 分布式节点
  isSVG?: boolean;        // 是否SVG
  compatibilityMode?: CompatibilityMode; // 兼容性模式
  assignedSlot?: BackendNode; // 分配的插槽
  isScrollable?: boolean;  // 是否可滚动（实验性）
}
```

### 3. 布局相关类型

#### DOM.BoxModel
- **类型**：object
- **描述**：盒模型
- **属性**：
```typescript
{
  content: Quad;           // 内容区域
  padding: Quad;          // 内边距区域
  border: Quad;           // 边框区域
  margin: Quad;           // 外边距区域
  width: integer;         // 节点宽度
  height: integer;        // 节点高度
  shapeOutside?: ShapeOutsideInfo; // 形状外部信息
}
```

#### DOM.Quad
- **类型**：array
- **描述**：四边形顶点数组，每个点的x坐标紧跟y坐标，顺时针排列

#### DOM.Rect
- **类型**：object
- **描述**：矩形
- **属性**：
```typescript
{
  x: number;              // X坐标
  y: number;              // Y坐标
  width: number;          // 宽度
  height: number;         // 高度
}
```

### 4. 其他类型

#### DOM.RGBA
- **类型**：object
- **描述**：RGBA颜色
- **属性**：
```typescript
{
  r: integer;             // 红色分量 [0-255]
  g: integer;             // 绿色分量 [0-255]
  b: integer;             // 蓝色分量 [0-255]
  a: number;              // 透明度 [0-1]
}
```

#### DOM.CompatibilityMode
- **类型**：string
- **允许值**：`"QuirksMode"`, `"LimitedQuirksMode"`, `"NoQuirksMode"`
- **描述**：文档兼容性模式

#### DOM.PseudoType
- **类型**：string
- **允许值**：`"first-line"`, `"first-letter"`, `"before"`, `"after"`, `"marker"`, `"backdrop"`, `"selection"`, `"target-text"`, `"spelling-error"`, `"grammar-error"`, `"highlight"`, `"first-line-inherited"`, `"scrollbar"`, `"scrollbar-thumb"`, `"scrollbar-button"`, `"scrollbar-track"`, `"scrollbar-track-piece"`, `"scrollbar-corner"`, `"resizer"`, `"input-list-button"`, `"view-transition"`, `"view-transition-group"`, `"view-transition-image-pair"`, `"view-transition-old"`, `"view-transition-new"`
- **描述**：伪元素类型

#### DOM.ShadowRootType
- **类型**：string
- **允许值**：`"user-agent"`, `"open"`, `"closed"`
- **描述**：Shadow Root类型

#### DOM.ShapeOutsideInfo
- **类型**：object
- **描述**：CSS Shape Outside信息
- **属性**：
```typescript
{
  bounds: Quad;            // 边界
  shape: any[];           // 形状坐标
  marginShape: any[];     // 边距形状
}
```

## 注意事项

1. DOM Domain提供了完整的DOM操作能力，包括：
   - 节点查询和遍历
   - 属性操作
   - 内容修改
   - 事件监听
   - 节点高亮和定位

2. 实验性功能主要集中在：
   - Shadow DOM操作
   - 性能相关的节点跟踪
   - 布局计算
   - 高级选择器功能

3. 废弃的功能：
   - `getFlattenedDocument`方法已被废弃
   - HTML Imports API相关功能已被移除

## 参考链接

- [Chrome DevTools Protocol Viewer - DOM Domain](https://chromedevtools.github.io/devtools-protocol/tot/DOM) 