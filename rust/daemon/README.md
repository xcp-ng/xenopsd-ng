HTTP server listening on port 3030 for jsonrpc request for xenops-ng.

# Compile & run daemon

From within the `rust` folder:
```
cargo {build|run} -p xenopsd
```

# Examples

## List domains

```
# curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "host.domain-list", "id":123 }' <server_ip>:3030

{"jsonrpc":"2.0","result":[{"dom_id":0,"name":"Domain-0"},{"dom_id":1,"name":"DNS Hole"},{"dom_id":3,"name":"Ubiquiti"},{"dom_id":4,"name":"FreeNAS"},{"dom_id":5,"name":"XOA"}],"id":1}
```
> The result is the list of existing domain ids.

You can use it with `jq` to get a prettier output:

```
# curl -s -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "host.domain-list", "id":1 }' 192.168.1.17:3030 | jq
```

This will display:
```
{
  "jsonrpc": "2.0",
  "result": [
    {
      "dom_id": 0,
      "name": "Domain-0"
    },
    {
      "dom_id": 1,
      "name": "DNS Hole"
    },
    {
      "dom_id": 3,
      "name": "Ubiquiti"
    },
    {
      "dom_id": 4,
      "name": "FreeNAS"
    },
    {
      "dom_id": 5,
      "name": "XOA"
    }
  ],
  "id": 1
}
```

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
> Here the dom_id is invalid.

## Shutdown a domain

```
> curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "vm.shutdown", "params": { "dom_id": 5 }, "id": 1}' <server_ip>:3030
{"jsonrpc":"2.0","result":"success","id":1}
```

TODO: ERROR
