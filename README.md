# chaudloader

![](loadchaud.png)

chaudloader is a mod loader for Mega Man Battle Network Legacy Collection.

## For users

1. Run `install.exe`.

2. Start the game. Mods in the `mods` folder will be activated in alphabetical order.

## For mod developers

Mods consists of the following files in a directory inside the `mods` folder:

-   `info.toml`: **Required.** Metadata about your mod. It should look something like this:

    ```toml
    title = "my cool mod"
    version = "0.0.1"
    authors = ["my cool name"]
    ```

-   `init.lua`: The Lua script to run on mod load.

### Asset modding

In `init.lua`, you may use the following functions:

```lua
--
-- exe/data .dat file functions
--

-- Opens an exe/data .dat file located in exe/data (e.g. `exe6.dat`).
function chaudloader.ExeDat(dat_filename: string): ExeDat

-- Reads the contents of a file out of the .dat file.
--
-- Previous calls to write_exe_dat_contents are visible to subsequent calls to read_exe_dat_contents.
function chaudloader.ExeDat:read_file(path: string): string

-- Writes the file data into the .dat file.
--
-- Note that this does not mutate the original .dat file on disk, but for all intents and purposes to both the game and the mod loader it does.
function chaudloader.ExeDat:write_file(path: string, contents: string): string

--
-- .map + .mpak file functions
--

-- Unmarshals an .map + .mpak file.
function chaudloader.Mpak(map_contents: string, mpak_contents): Mpak

-- Inserts an entry at the given ROM address into the mpak. Existing entries will be clobbered. If contents is nil, the entry will be deleted.
function chaudloader.Mpak:__newindex(rom_addr: number, contents: string)

-- Reads an entry at the given ROM address.
function chaudloader.Mpak:__index(rom_addr: number): number

-- Marshals an mpak back into .map + .mpak format.
function chaudloader.Mpak:to_raw(): (string, string)

--
-- Mod file functions
--

-- Reads the contents of a file from the mod folder.
function chaudloader.read_mod_file(path: string): string

-- Loads a library from the mod folder and call its ChaudLoaderInit function.
--
--     ChaudLoaderInit: unsafe extern "system" fn(userdata: *const u8, n: usize) -> bool
function chaudloader.init_mod_dll(path: string, userdata: string)

--
-- Utility functions
--

-- Print a log line.
function print(...)
```

For instance, for a simple font mod, you can write the following script:

```lua
local exe6_dat = chaudloader.ExeDat("exe6.dat")
local exe6f_dat = chaudloader.ExeDat("exe6f.dat")

local font = chaudloader.read_mod_file("eng_mojiFont.fnt")

exe6_dat.write_file("exe6/data/font/eng_mojiFont.fnt", font)
exe6f_dat.write_file("exe6f/data/font/eng_mojiFont.fnt", font)
```

Mods are order dependent: the DAT contents written by a previous mod will be visible to a subsequent mod.

<details>
<summary>Legacy bnlc_mod_loader API</summary>

```lua
-- Reads the contents of a file out of a .dat file located in exe/data (e.g. `exe6.dat`).
--
-- Previous calls to write_exe_dat_contents are visible to subsequent calls to read_exe_dat_contents.
function bnlc_mod_loader.read_exe_dat_contents(dat_filename: string, path: string): string

-- Writes the given data into a zip .dat file located in exe/data.
--
-- Note that this does not mutate the original .dat file on disk, but for all intents and purposes to both the game and the mod loader it does.
function bnlc_mod_loader.write_exe_dat_contents(dat_filename: string, path: string, contents: string)

-- Reads the contents of a file from the mod folder.
function bnlc_mod_loader.read_mod_contents(path: string): string
```

</details>

## For library developers

Build the binary with `cargo build --release`. You will need nightly Rust.
