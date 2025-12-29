# Diagram References

This section provides detailed visual references for understanding Clap's architecture and patterns. Each diagram includes explanations and usage context.

## Available Diagrams

### [Architecture Overview](./architecture-overview.md)
High-level view of Clap's internal architecture, showing how components interact during argument parsing.

### [Parsing Pipeline](./parsing-pipeline.md)
Step-by-step visualization of how arguments flow through Clap's parsing stages.

### [Error Recovery](./error-recovery.md)
Diagrams showing error handling flows and recovery patterns.

### [Command Lifecycle](./command-lifecycle.md)
The complete lifecycle of a command from definition to execution.

### [Configuration Precedence](./config-precedence.md)
Visual representation of how different configuration sources are prioritized.

### [Testing Strategy](./testing-strategy.md)
Testing pyramid and strategies for CLI applications.

## How to Use These Diagrams

These diagrams are designed to complement the main chapters. You can:

1. **Reference during reading** - Jump to relevant diagrams while studying chapters
2. **Use for presentations** - Each diagram is self-contained and explained
3. **Print for quick reference** - ASCII art renders well in documentation

## Diagram Conventions

Throughout these references, we use consistent visual conventions:

- `[Component]` - Major system components
- `-->` - Data flow direction
- `|` and `+` - Connection points
- Boxes with `=` borders - Key decision points
