# tinyboot-protocol

Wire protocol for tinyboot. Defines the frame format used between host and device over UART/RS-485.

## Frame format

A single `Frame` struct is used for both requests (host to device) and responses (device to host), so we keep code size tiny.

```
 0       1       2       3       4       5       6       6+len   6+len+2
 +-------+-------+-------+-------+-------+-------+-------+- - - -+-------+-------+
 | SYNC0 | SYNC1 |  CMD  |  LEN  | ADDR_LO ADDR_HI | STATUS | DATA... | CRC_LO  CRC_HI |
 | 0xAA  | 0x55  |       |       |                  |        |         |                 |
 +-------+-------+-------+-------+-------+-------+-------+- - - -+-------+-------+
 |<------------- header (7 bytes) ------------>|<- payload ->|<--- CRC --->|
```

| Field  | Size    | Description                                                   |
| ------ | ------- | ------------------------------------------------------------- |
| SYNC   | 2 bytes | Preamble `0xAA 0x55` for frame synchronization                |
| CMD    | 1 byte  | Command code                                                  |
| LEN    | 1 byte  | Data payload length (0..255)                                  |
| ADDR   | 2 bytes | Flash address (LE). Echoed in responses                       |
| STATUS | 1 byte  | `Request (0x05)` for requests, result status for responses    |
| DATA   | 0..255  | Payload bytes                                                 |
| CRC    | 2 bytes | CRC16-CCITT (LE) over SYNC + CMD + LEN + ADDR + STATUS + DATA |

## Commands

| Code | Name   | Direction      | Description                                  |
| ---- | ------ | -------------- | -------------------------------------------- |
| 0x01 | Info   | Host to Device | Query device geometry (write size, app size) |
| 0x02 | Erase  | Host to Device | Erase entire app region                      |
| 0x03 | Write  | Host to Device | Write data at address                        |
| 0x04 | Verify | Host to Device | Compute CRC16 over app region                |
| 0x05 | Reset  | Host to Device | Advance boot state and reset                 |

## Status codes

| Code | Name            | Description                         |
| ---- | --------------- | ----------------------------------- |
| 0x00 | Ok              | Success                             |
| 0x01 | Error           | Generic error                       |
| 0x02 | CrcMismatch     | CRC verification failed             |
| 0x03 | AddrOutOfBounds | Address or length out of range      |
| 0x04 | NotReady        | Device not ready                    |
| 0x05 | Request         | Frame is a request (not a response) |

## CRC

CRC16-CCITT with polynomial `0x1021` and initial value `0xFFFF`. Computed over the entire frame body (SYNC through DATA, excluding the CRC field itself). Bit-bang implementation with no lookup table for minimal flash footprint.

## Protocol flow

1. Host sends a request frame with `status = Request (0x05)`
2. Device reads the frame, processes the command
3. Device sends a response frame with `cmd` and `addr` echoed from the request, `status` set to the result

The same `Frame` struct is reused: after `read()`, the device modifies `status`, `len`, and `data`, then calls `send()`. The `cmd` and `addr` fields carry over automatically.

## Example: write sequence

```
Host  -> [0xAA 0x55] [0x03] [0x04] [0x00 0x01] [0x05] [DE AD BE EF] [CRC]
          sync        Write   len=4  addr=0x100   Req    data          crc

Device -> [0xAA 0x55] [0x03] [0x00] [0x00 0x01] [0x00] [CRC]
          sync        Write   len=0  addr=0x100   Ok     crc
```
