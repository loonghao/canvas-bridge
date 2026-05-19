# canvas-bridge

`canvas-bridge` is a small Rust service that gives intelligent canvas clients a
stable WebSocket protocol while reusing the existing `dcc-mcp-core` Gateway for
DCC discovery, execution, resources, jobs, traces, and audit data.

```text
canvas client
  -> ws://127.0.0.1:9876/ws
  -> canvas-bridge
  -> http://127.0.0.1:9765/v1/*
  -> dcc-mcp Gateway
  -> Maya / Blender / Houdini / Photoshop / custom DCC
```

## Run locally

```bash
cargo run -- --gateway http://127.0.0.1:9765 --bind 127.0.0.1:9876
```

## WebSocket commands

Search tools:

```json
{"type":"canvas.tools.search","query":"sphere","dcc_type":"maya","limit":10}
```

Describe a tool:

```json
{"type":"canvas.tool.describe","tool_slug":"maya.abcdef01.create_sphere"}
```

Run a canvas node:

```json
{
  "type": "canvas.node.run",
  "run_id": "run-001",
  "node_id": "create-sphere",
  "tool_slug": "maya.abcdef01.create_sphere",
  "arguments": {"radius": 2.0, "name": "Ball_A"}
}
```

The bridge maps these commands to the Gateway REST surface:

- `POST /v1/search`
- `POST /v1/describe`
- `POST /v1/call`

Future iterations will add job SSE fan-in, resource proxying, trace attachment,
and release packaging that can be distributed beside `dcc-mcp-server`.
