# oze_canopen
Python bindings for [oze-canopen](https://github.com/ozontech/oze-canopen-rs) library for Rust. It is not implementing Object Dictionary and designed to work as sniffer or master node.

## Features
- Fully async using tokio and socketcan-rs
- All packets type parsing based on COB ID
- Node ID parsing
- Async mutex-protected SDO client for every node
- NMT & SYNC servers with possibility to send packages periodically
- Parallel receiver & transmitter to ensure non blocking access

# SDO example
```rust
import asyncio
import time

import oze_canopen

async def f():
    a = await oze_canopen.OzeCO.start("vcan0", 100000)
    res = await a.sdo_upload(4, 0x1800, 0)
    print(res)
    await a.sdo_download(4, 0x2110, 1, b"\xff\x12\x13\x11")
    res = await a.sdo_upload(4, 0x2110, 1)
    print(res)

asyncio.run(f())
```

More examples you can find in `examples/` directory.

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