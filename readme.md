# oze_canopen
CanOpen library for Rust. It is not implementing Object Dictionary and designed to work as sniffer or master node.

## Features
- Fully async using tokio and socketcan-rs
- All packets type parsing based on COB ID
- Node ID parsing
- Async mutex-protected SDO client for every node
- NMT & SYNC servers with possibility to send packages periodically
- Parallel receiver & transmitter to ensure non blocking access

# Roadmap
- Implement CAN FD support
- nostd for using on MCU

# SDO example
```rust
// init and start receiver, transmitter and interface monitor
let (interface, mut handles) = canopen::start(String::from("vcan0"), Some(100000));
 // wait for connection
sleep(Duration::from_secs(1)).await;
// get SDO client for nodeID=4
let sdo_client = interface.get_sdo_client(4).unwrap();
// because only single SDO request is supported then we guard acces using mutex and now we wait to aquire access
let sdo_client = sdo_client.lock().await;
// Send SDO request upload request and wait for result
let result = sdo_client.upload(0x2110, 0x01).await.unwrap();
// print result
println!("{result:?}");
// Gracefully close canopen stack and join all spawned futures
handles.close_and_join().await;
```

More examples you can find in `examples/` directory.

# Testing
Unit tests are in `src/` and has no external requirements, you can run it using `cargo test`.

Integration tests are in `tests/`. To run it successfully you need:
1. Create virtual CAN interface:
- `sudo modprobe vcan`
- `sudo ip link add dev vcan0 type vcan`
- `sudo ip link set up vcan0`
2. Clone and build CanOpenNode demo from here: https://github.com/CANopenNode/CANopenDemo/tree/b2a6b53c2c9d7b74e6ab4c4dcefc2be43533392e/demo
3. Run `./demoLinuxDevice vcan0`
4. Run `cargo test -- --test-threads 1`

After that you can check code coverage using `cargo tarpaulin -j 1 --engine llvm`

# Rust Docs
To generate rustdocs you need to execute
```
cargo doc
```

# License
```
   Copyright © 2024 LLC "Ozon Technologies"

   Licensed under the Apache License, Version 2.0 (the "LICENSE");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
```