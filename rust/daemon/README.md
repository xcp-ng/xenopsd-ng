HTTP server listening on port 3030 for jsonrpc request for xenops-ng.

# Compile & run daemon

From within the `rust` foder:
```
cargo {build|run} -p xenopsd
```

# Examples

## List domains

```
> curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "host.domain-list", "id":123 }' <server_ip>:3030
{"jsonrpc":"2.0","result":[0,10],"id":123}
```
> The result is the list of existing domain ids.

## Pause a domain

```
> curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "vm.pause", "params": { "dom_id": 5 }, "id": 1}' <server_ip>:3030
{"jsonrpc":"2.0","result":"success","id":1}
```

```
> curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "vm.pause", "params": { "dom_id": 12 }, "id": 1}' <server_ip>:3030
{"jsonrpc":"2.0","error":{"code":0,"message":"-3: No such process (os error 3) ()"},"id":1}
```
> Here the dom_id is invalid.

## Unpause a domain

```
> curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "vm.unpause", "params": { "dom_id": 5 }, "id": 1}' <server_ip>:3030
{"jsonrpc":"2.0","result":"success","id":1}
```

```
> curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "vm.unpause", "params": { "dom_id": 12 }, "id": 1}' <server_ip>:3030
{"jsonrpc":"2.0","error":{"code":0,"message":"-3: No such process (os error 3) ()"},"id":1}
```
> Here the dom_id is invalid

## Shutdown a domain

```
> curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "vm.shutdown", "params": { "dom_id": 5 }, "id": 1}' <server_ip>:3030
{"jsonrpc":"2.0","result":"success","id":1}
```

TODO: ERROR
