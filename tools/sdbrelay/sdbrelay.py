#!/usr/bin/env python3
"""Record the game's settings backup traffic (the "save data bank") so Kawari can learn its protocol.

The client's System → Server Backup ("Upload/Download Client Settings", "Character Configuration Data") doesn't go
over the lobby, zone or chat connections Chronofoil records: it opens its own TCP connection to `SaveDataBankHost`
(default `config-dl.ffxiv.com`) on `SaveDataBankPort` (default 55000), two of the client's network settings
(NetworkModule +0x628 / +0x620 in FFXIVClientStructs; the defaults are in the 7.57 client's config table). Pointing
those two at this relay (launch arguments `SaveDataBankHost=127.0.0.1 SaveDataBankPort=55000`, or the launcher's
`savedata_bank_server` / `savedata_bank_port`) lets the backup work as usual while every byte both ways is written to
a file.

    python3 sdbrelay.py [--listen 127.0.0.1:55000] [--upstream config-dl.ffxiv.com:55000] [--out sdb-<time>.jsonl.gz]

Each line of the output is one chunk as it arrived: `{"t": unix seconds, "conn": n, "dir": "up"|"down", "hex": ...}`,
plus `"frame"` when the chunk starts with an FFXIV packet header (size, connection type, segment count,
compression). The file holds your own settings (macros, keybinds, gear set names): keep it out of git, like a
Chronofoil capture; send it to Kawari's diagnostics the same way.
"""

import argparse
import asyncio
import gzip
import json
import struct
import time

FRAME_HEADER = 40


def parse_frame(data: bytes):
    """The FFXIV packet header at the start of `data`, if it looks like one (the size field fits, the segment count
    is small), else None."""
    if len(data) < FRAME_HEADER:
        return None
    size, connection_type, count = struct.unpack_from("<IHH", data, 24)
    compression = data[33]
    if not (FRAME_HEADER <= size <= 0x100000) or count > 4096:
        return None
    return {"size": size, "connection_type": connection_type, "segments": count, "compression": compression}


class Recorder:
    def __init__(self, path: str):
        self.file = gzip.open(path, "at")
        self.connections = 0

    def write(self, conn: int, direction: str, data: bytes):
        line = {"t": round(time.time(), 3), "conn": conn, "dir": direction, "hex": data.hex()}
        frame = parse_frame(data)
        if frame:
            line["frame"] = frame
        self.file.write(json.dumps(line) + "\n")
        self.file.flush()


async def pump(reader, writer, recorder: Recorder, conn: int, direction: str):
    try:
        while data := await reader.read(65536):
            recorder.write(conn, direction, data)
            writer.write(data)
            await writer.drain()
    except (ConnectionError, asyncio.IncompleteReadError):
        pass
    finally:
        writer.close()


async def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--listen", default="127.0.0.1:55000")
    parser.add_argument("--upstream", default="config-dl.ffxiv.com:55000")
    parser.add_argument("--out", default=time.strftime("sdb-%Y%m%d-%H%M%S.jsonl.gz"))
    args = parser.parse_args()
    listen_host, listen_port = args.listen.rsplit(":", 1)
    up_host, up_port = args.upstream.rsplit(":", 1)
    recorder = Recorder(args.out)

    async def client(reader, writer):
        recorder.connections += 1
        conn = recorder.connections
        print(f"connection {conn} from {writer.get_extra_info('peername')}")
        up_reader, up_writer = await asyncio.open_connection(up_host, int(up_port))
        await asyncio.gather(
            pump(reader, up_writer, recorder, conn, "up"),
            pump(up_reader, writer, recorder, conn, "down"),
        )
        print(f"connection {conn} closed")

    server = await asyncio.start_server(client, listen_host, int(listen_port))
    print(f"relaying {args.listen} -> {args.upstream}, writing {args.out}")
    async with server:
        await server.serve_forever()


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        pass
