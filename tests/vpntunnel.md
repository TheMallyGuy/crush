# Extra: The SwiftTunnel integration

If you already know or not, Roblox split their player base into 2 groups; One is global, Two is VNG. Recently, this so called VNG blocked some games in Vietnam, making people unable to join their favourite game and lose their money on gamepasses.

## The solution

I've been using a VPN called [SwiftTunnel](https://swifttunnel.net/), it offers free VPN with low latency (28 servers, 10 regions), it's open source, no telemetry and written in Tauri + React. So i integrated it in crush Extra using my forked SDK of SwiftTunnel.

The way it works is only `robloxplayerbeta.exe` gets tunneled, everything else goes through normally. There's also an optional Route Assist mode that tunnels Roblox's TCP API and bootstrap DNS traffic too, which helps Roblox place you in a server near the region you picked instead of near where you actually are.

## How did i do it

I compiled `swifttunnel.dll` as the SDK, added safe Rust FFI bindings for it in [`swifttunnel_sdk.rs`](../src-tauri/src/swifttunnel_sdk.rs) and exposed it to the frontend through Tauri commands in [`commands/swifttunnel.rs`](../src-tauri/src/commands/swifttunnel.rs). The build script links the DLL automatically and copies it into the output folder.

## Benchmarking

For me (Vietnam) this works very stable, I average about 40~60ms on ping, highest ever recorded is 100-150ms when gaming with SwiftTunnel!

## Other stuff

You can find my forked SDK [here](https://github.com/TheMallyGuy/Swifttunnel-sdk). Download rustup and compile it like this:

```
cargo build --release
```
The output will be at `src/target/release/swifttunnel.dll`, take that and see the [headers](https://github.com/TheMallyGuy/Swifttunnel-sdk/blob/main/include/swifttunnel.h) to get started on integrating this into your app.

This feature is exclusive to crush Extra. Its will not be added in normal crush.