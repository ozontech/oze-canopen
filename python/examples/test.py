import asyncio
import time

import py_oze_canopen


async def f():
    a = await py_oze_canopen.OzeCO.start("vcan0", 100000)
    a.nmt_send(py_oze_canopen.NmtCmd.StartRemoteNode, 0)
    print(a)
    res = await a.sdo_upload(4, 0x1800, 0)
    print(res)
    await a.sdo_download(4, 0x2110, 1, b"\xff\x12\x13\x11")
    res = await a.sdo_upload(4, 0x2110, 1)
    print(res)
    res = a.recv_blocking()
    await a.send(0x129, b"\x01\x02\x03\x04\x05\x06\x07")
    res = a.recv_blocking()
    print(res.cob(), res.data())

    a.sync_set_period(100)
    a.nmt_set_period(1000, py_oze_canopen.NmtCmd.StopRemoteNode, 4)
    time.sleep(5)
    a.sync_set_period(None)
    a.nmt_set_period(1000, None)


asyncio.run(f())
