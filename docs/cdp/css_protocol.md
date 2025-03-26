# Chrome DevTools Protocol - CSS Domain

CSS Domain提供了CSS读写操作的能力。所有CSS对象（样式表、规则和样式）都有一个关联的`id`，用于后续对相关对象的操作。每种对象类型都有特定的`id`结构，不同类型的对象之间不能互换。CSS对象可以使用`get*ForNode()`调用（接受DOM节点id）加载。客户端还可以通过`styleSheetAdded`/`styleSheetRemoved`事件跟踪样式表，并随后使用`getStyleSheet[Text]()`方法加载所需的样式表内容。

**注意**：整个CSS Domain都被标记为实验性(Experimental)。

## Methods (方法)

### 1. 基础操作

#### CSS.enable()
- **描述**：启用CSS域
- **参数**：无
- **返回**：无

#### CSS.disable()
- **描述**：禁用CSS域
- **参数**：无
- **返回**：无

### 2. 样式表操作

#### CSS.createStyleSheet
- **描述**：创建新的样式表
- **参数**：
```typescript
{
  frameId: string;           // 框架ID
}
```
- **返回**：
```typescript
{
  styleSheetId: StyleSheetId; // 创建的样式表ID
}
```

#### CSS.getStyleSheetText
- **描述**：获取样式表文本
- **参数**：
```typescript
{
  styleSheetId: StyleSheetId; // 样式表ID
}
```
- **返回**：
```typescript
{
  text: string;              // 样式表文本
}
```

#### CSS.setStyleSheetText
- **描述**：设置样式表文本
- **参数**：
```typescript
{
  styleSheetId: StyleSheetId; // 样式表ID
  text: string;               // 新的样式表文本
}
```
- **返回**：无

#### CSS.addRule
- **描述**：添加新的样式规则
- **参数**：
```typescript
{
  styleSheetId: StyleSheetId; // 样式表ID
  ruleText: string;          // 规则文本
  location: SourceRange;     // 插入位置
}
```
- **返回**：
```typescript
{
  rule: CSSRule;            // 创建的规则
}
```

#### CSS.collectClassNames
- **描述**：收集所有类名
- **参数**：
```typescript
{
  styleSheetId: StyleSheetId; // 样式表ID
}
```
- **返回**：
```typescript
{
  classNames: string[];     // 类名列表
}
```

#### CSS.getMediaQueries
- **描述**：获取所有媒体查询
- **参数**：无
- **返回**：
```typescript
{
  medias: CSSMedia[];       // 媒体查询列表
}
```

#### CSS.forcePseudoState
- **描述**：强制元素的伪类状态
- **参数**：
```typescript
{
  nodeId: DOM.NodeId;       // 节点ID
  forcedPseudoClasses: string[]; // 强制的伪类列表
}
```
- **返回**：无

#### CSS.setKeyframeKey
- **描述**：修改关键帧规则的关键帧
- **参数**：
```typescript
{
  styleSheetId: StyleSheetId; // 样式表ID
  range: SourceRange;        // 范围
  keyText: string;          // 关键帧文本
}
```
- **返回**：
```typescript
{
  keyText: Value;           // 修改后的关键帧
}
```

#### CSS.setMediaText
- **描述**：修改媒体查询
- **参数**：
```typescript
{
  styleSheetId: StyleSheetId; // 样式表ID
  range: SourceRange;        // 范围
  text: string;             // 新的媒体查询文本
}
```
- **返回**：
```typescript
{
  media: CSSMedia;          // 修改后的媒体查询
}
```

#### CSS.setRuleSelector
- **描述**：修改规则的选择器
- **参数**：
```typescript
{
  styleSheetId: StyleSheetId; // 样式表ID
  range: SourceRange;        // 范围
  selector: string;         // 新的选择器文本
}
```
- **返回**：
```typescript
{
  selectorList: SelectorList; // 修改后的选择器列表
}
```

#### CSS.setStyleTexts
- **描述**：批量修改样式文本
- **参数**：
```typescript
{
  edits: StyleDeclarationEdit[]; // 样式声明编辑列表
}
```
- **返回**：
```typescript
{
  styles: CSSStyle[];       // 修改后的样式列表
}
```

### 3. 样式查询

#### CSS.getComputedStyleForNode
- **描述**：获取节点的计算样式
- **参数**：
```typescript
{
  nodeId: DOM.NodeId;        // 节点ID
}
```
- **返回**：
```typescript
{
  computedStyle: CSSComputedStyleProperty[]; // 计算样式属性数组
}
```

#### CSS.getInlineStylesForNode
- **描述**：获取节点的内联样式
- **参数**：
```typescript
{
  nodeId: DOM.NodeId;        // 节点ID
}
```
- **返回**：
```typescript
{
  inlineStyle?: CSSStyle;    // 内联样式
  attributesStyle?: CSSStyle; // 属性样式
}
```

#### CSS.getMatchedStylesForNode
- **描述**：获取节点的匹配样式
- **参数**：
```typescript
{
  nodeId: DOM.NodeId;        // 节点ID
}
```
- **返回**：
```typescript
{
  inlineStyle?: CSSStyle;    // 内联样式
  attributesStyle?: CSSStyle; // 属性样式
  matchedCSSRules?: RuleMatch[]; // 匹配的CSS规则
  pseudoElements?: PseudoElementMatches[]; // 伪元素匹配
  inherited?: InheritedStyleEntry[]; // 继承的样式
}
```

### 4. 实验性方法

#### CSS.getAnimatedStylesForNode
- **描述**：获取节点的动画样式
- **实验性**

#### CSS.getLayersForNode
- **描述**：获取节点的层信息
- **实验性**

#### CSS.getLocationForSelector
- **描述**：获取选择器的位置信息
- **实验性**

#### CSS.getLonghandProperties
- **描述**：获取简写属性的所有展开属性
- **实验性**

#### CSS.setContainerQueryText
- **描述**：设置容器查询文本
- **实验性**

#### CSS.setLocalFontsEnabled
- **描述**：启用/禁用本地字体
- **实验性**

#### CSS.setScopeText
- **描述**：设置作用域文本
- **实验性**

#### CSS.setSupportsText
- **描述**：设置@supports规则文本
- **实验性**

#### CSS.takeComputedStyleUpdates
- **描述**：获取计算样式更新
- **实验性**

#### CSS.trackComputedStyleUpdates
- **描述**：跟踪计算样式更新
- **实验性**

#### CSS.trackComputedStyleUpdatesForNode
- **描述**：跟踪特定节点的计算样式更新
- **实验性**

### 5. 性能分析

#### CSS.startRuleUsageTracking
- **描述**：开始规则使用跟踪
- **参数**：无
- **返回**：无

#### CSS.stopRuleUsageTracking
- **描述**：停止规则使用跟踪
- **参数**：无
- **返回**：
```typescript
{
  ruleUsage: RuleUsage[];   // 规则使用情况
}
```

#### CSS.takeCoverageDelta
- **描述**：获取规则覆盖率变化
- **参数**：无
- **返回**：
```typescript
{
  coverage: RuleUsage[];    // 规则覆盖率信息
}
```

## Events (事件)

### CSS.fontsUpdated
- **描述**：字体更新时触发
- **参数**：
```typescript
{
  font?: FontFace;           // 更新的字体信息
}
```

### CSS.mediaQueryResultChanged
- **描述**：媒体查询结果改变时触发
- **参数**：无

### CSS.styleSheetAdded
- **描述**：添加样式表时触发
- **参数**：
```typescript
{
  header: CSSStyleSheetHeader; // 样式表头信息
}
```

### CSS.styleSheetChanged
- **描述**：样式表改变时触发
- **参数**：
```typescript
{
  styleSheetId: StyleSheetId; // 样式表ID
}
```

### CSS.styleSheetRemoved
- **描述**：移除样式表时触发
- **参数**：
```typescript
{
  styleSheetId: StyleSheetId; // 样式表ID
}
```

### CSS.computedStyleUpdated (实验性)
- **描述**：计算样式更新时触发
- **参数**：
```typescript
{
  nodeIds: DOM.NodeId[];     // 更新的节点ID列表
}
```

## Types (类型定义)

### 基础类型

#### StyleSheetId
- **类型**：string
- **描述**：样式表的唯一标识符

#### SourceRange
- **类型**：object
- **描述**：源代码中的位置范围
- **属性**：
```typescript
{
  startLine: number;         // 起始行
  startColumn: number;       // 起始列
  endLine: number;          // 结束行
  endColumn: number;        // 结束列
}
```

### 样式相关类型

#### CSSStyle
- **类型**：object
- **描述**：CSS样式声明
- **属性**：
```typescript
{
  styleSheetId?: StyleSheetId; // 样式表ID
  cssProperties: CSSProperty[]; // CSS属性数组
  shorthandEntries: ShorthandEntry[]; // 简写属性条目
  cssText?: string;          // CSS文本
  range?: SourceRange;       // 源码范围
}
```

#### CSSProperty
- **类型**：object
- **描述**：CSS属性
- **属性**：
```typescript
{
  name: string;              // 属性名
  value: string;             // 属性值
  important?: boolean;       // 是否important
  implicit?: boolean;        // 是否隐式
  text?: string;             // 文本
  parsedOk?: boolean;        // 是否解析成功
  disabled?: boolean;        // 是否禁用
  range?: SourceRange;       // 源码范围
}
```

#### CSSRule
- **类型**：object
- **描述**：CSS规则
- **属性**：
```typescript
{
  styleSheetId: StyleSheetId; // 样式表ID
  selectorList: SelectorList; // 选择器列表
  origin: StyleSheetOrigin;  // 规则来源
  style: CSSStyle;          // 样式信息
  media?: CSSMedia[];       // 媒体查询
}
```

#### CSSComputedStyleProperty
- **类型**：object
- **描述**：计算样式属性
- **属性**：
```typescript
{
  name: string;             // 属性名
  value: string;            // 属性值
}
```

#### RuleUsage
- **类型**：object
- **描述**：规则使用情况
- **属性**：
```typescript
{
  styleSheetId: StyleSheetId; // 样式表ID
  startOffset: number;       // 开始偏移
  endOffset: number;        // 结束偏移
  used: boolean;            // 是否使用
}
```

#### StyleDeclarationEdit
- **类型**：object
- **描述**：样式声明编辑
- **属性**：
```typescript
{
  styleSheetId: StyleSheetId; // 样式表ID
  range: SourceRange;        // 范围
  text: string;             // 新文本
}
```

### 实验性类型

#### CSSContainerQuery
- **描述**：CSS容器查询
- **实验性**

#### CSSLayer
- **描述**：CSS层
- **实验性**

#### CSSLayerData
- **描述**：CSS层数据
- **实验性**

#### CSSRuleType
- **描述**：CSS规则类型
- **实验性**

#### CSSScope
- **描述**：CSS作用域
- **实验性**

#### CSSStartingStyle
- **描述**：CSS起始样式
- **实验性**

#### CSSSupports
- **描述**：CSS @supports规则
- **实验性**

#### Specificity
- **描述**：选择器特异性
- **实验性**

## 注意事项

1. CSS Domain是实验性的，所有功能在未来版本中可能会发生变化。

2. 主要功能包括：
   - 样式表管理（创建、读取、修改样式表）
   - 样式查询（获取计算样式、内联样式、匹配样式）
   - 样式监控（通过事件监听样式变化）
   - 字体管理（监控字体更新）

3. 实验性功能主要集中在：
   - 动画样式查询
   - 层管理
   - 容器查询
   - 计算样式更新跟踪
   - 选择器位置查询

## 参考链接

- [Chrome DevTools Protocol Viewer - CSS Domain](https://chromedevtools.github.io/devtools-protocol/tot/CSS) 