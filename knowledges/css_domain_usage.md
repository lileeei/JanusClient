# CSS Domain Usage Guide

## Overview

Firefox DevTools Protocol (FDP) CSS Domain provides methods for inspecting and modifying CSS styles. This guide explains how to use the CSS domain in the FDP client to:

1. Retrieve computed styles for DOM nodes
2. Get matched styles for elements
3. Modify stylesheet content
4. Listen for stylesheet changes

## Prerequisites

Before using the CSS domain, you must:

1. Connect to a browser instance
2. Attach to a specific target (usually a browser tab)
3. Have a valid node ID (typically obtained from DOM.getDocument)

## Basic Usage Examples

### Getting Computed Styles

To retrieve computed styles for a DOM node:

```rust
// Connect to browser and attach to target first
let client = FdpClient::new("ws://localhost:9222/devtools/browser");
client.connect().await?;

// Attach to a target
let targets = client.get_targets().await?;
if let Some(target) = targets.first() {
    if let Some(target_id) = target.get("targetId").and_then(|v| v.as_str()) {
        let session_id = client.attach_to_target(target_id).await?;

        // Get document root (assume node ID 1 for this example)
        // In a real implementation, you would use DOM.getDocument
        let root_node_id = 1;

        // Get computed styles
        let computed_style = client.get_computed_style_for_node(root_node_id).await?;

        // Access style properties
        for property in computed_style.properties {
            println!("{}: {}", property.name, property.value);
        }
    }
}
```

### Getting Matched Styles

To get matched styles (including rules, selectors, etc.):

```rust
let matched_styles = client.get_matched_styles_for_node(node_id).await?;

// matched_styles is a JSON value containing:
// - rules: array of matched CSS rules
// - pseudoElements: styles for pseudo-elements
// - inherited: inherited styles
```

### Modifying Styles

To modify styles:

```rust
use JanusClient::domain::css::{StyleEdit, StyleSheetId};

// Create style edits
let edits = vec![
    StyleEdit {
        style_sheet_id: StyleSheetId("stylesheet-id-here".to_string()),
        style_text: "body { background-color: red; }".to_string(),
    }
];

// Apply edits
let result = client.set_style_texts(edits).await?;
```

## Event Listening

The CSS domain provides events for tracking stylesheet changes. Here's how to listen for them:

### Stylesheet Added Event

```rust
// Register for stylesheet added events
client.on("CSS.styleSheetAdded", |event| {
    println!("New stylesheet added: {:?}", event.params);
})?;
```

### Stylesheet Removed Event

```rust
// Register for stylesheet removed events
client.on("CSS.styleSheetRemoved", |event| {
    println!("Stylesheet removed: {:?}", event.params);
})?;
```

## Common Use Cases

### Theme Switching

```rust
async fn toggle_dark_mode(client: &FdpClient, stylesheet_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let dark_mode_css = "
        body {
            background-color: #121212;
            color: #e0e0e0;
        }
        a {
            color: #90caf9;
        }
    ";

    let edits = vec![
        StyleEdit {
            style_sheet_id: StyleSheetId(stylesheet_id.to_string()),
            style_text: dark_mode_css.to_string(),
        }
    ];

    client.set_style_texts(edits).await?;
    Ok(())
}
```

### Style Inspection

```rust
async fn inspect_element_styles(client: &FdpClient, node_id: i32) -> Result<(), Box<dyn std::error::Error>> {
    // Get computed styles
    let computed = client.get_computed_style_for_node(node_id).await?;

    // Get matched styles (including selectors and rules)
    let matched = client.get_matched_styles_for_node(node_id).await?;

    println!("Computed styles:");
    for prop in computed.properties {
        println!("  {}: {}", prop.name, prop.value);
    }

    println!("Matched rules:");
    if let Some(rules) = matched.get("matchedCSSRules").and_then(|v| v.as_array()) {
        for rule in rules {
            if let Some(selector_text) = rule.get("rule")
                .and_then(|r| r.get("selectorText"))
                .and_then(|s| s.as_str()) {
                println!("  Selector: {}", selector_text);
            }
        }
    }

    Ok(())
}
```

## Error Handling

Common errors when working with the CSS domain:

1. **Node not found**: The specified node ID doesn't exist
2. **Invalid stylesheet**: The stylesheet ID is invalid
3. **Syntax errors**: Invalid CSS syntax in style edits

Example error handling:

```rust
match client.get_computed_style_for_node(node_id).await {
    Ok(computed_style) => {
        // Process computed style
    },
    Err(e) => {
        match e {
            FdpError::MessageError { code, message } => {
                if code == -32000 {
                    println!("Node not found: {}", message);
                } else {
                    println!("Protocol error: {}", message);
                }
            },
            _ => println!("Other error: {}", e),
        }
    }
}
```

## Performance Considerations

When working with CSS styles:

1. Limit the frequency of style operations to avoid performance issues
2. Batch style modifications when possible using single `setStyleTexts` calls
3. Consider caching computed styles for frequently accessed nodes

## Version Information

This documentation applies to FDP Client version 0.1.0 and Firefox DevTools Protocol v1.0.
