# chaudloader

![](loadchaud.png)

chaudloader is a mod loader for Mega Man Battle Network Legacy Collection.

## For users

1. If you are on Windows, run `install.exe`. If you are on Steam Deck, you will need to run `install.desktop` instead from Desktop Mode.

2. If you are on Steam Deck, go to the game's Properties in Steam, go to Compatibility and set it to use `Proton 7.0`.

3. Start the game. Mods in the `mods` folder will be activated in alphabetical order.

## For modders

Mods consists of the following files in a directory inside the `mods` folder:

-   `info.toml`: Metadata about your mod. It should look something like this:

    ```toml
    title = "my cool mod"
    version = "0.0.1"
    authors = ["my cool name"]
    unsafe = false  # set to true if you want to use scary unsafe functions
    url = "https://mycoolmod.com"
    requires_loader_version = "*"  # or any semver requirement string
    requires_exe_crc32 = [0x11111111, 0x22222222]  # list of CRC32s to match against, can be unset if not required
    requires_game = ["Vol1", "Vol2"]  # list of game volumes this mod applies to
    ```

-   `init.lua`: The Lua script to run on mod load. Please consult [API.md](API.md) for the API documentation.

### Developer mode

chaudloader has some development options which can be enabled to aid with mod development. These options have to be manually set in `chaudloader.toml`.

**These options are purely for development purposes. For users it is strongly recommended not to enable them.**

-   `developer_mode` (type: boolean, default: `false`): Enables developer mode. Required to be `true` in order to use any of the other development options.
-   `enable_hook_guards` (type: boolean, default: `false`): Enables hook guards.

## For developers

### First time

1. Install Rust from https://rustup.rs/

2. Install Visual Studio 2022 with the Desktop development with C++ workload.

3. Build Lua 5.4 using `powershell .\download_and_build_lua.ps1` from a VS x64 command prompt. You only need to do this one time, and it will produce a dynamically linkable Lua library in `build\lua54`, as well as headers in `build\lua54\include`.

4. Copy `build\lua54\lua54.dll` into your BNLC `exe` folder.

### Every time

1. Build the binary with `cargo build --release`.

2. Copy `dxgi.dll` and `chaudloader.dll` from `target\release` into your BNLC `exe` folder.
