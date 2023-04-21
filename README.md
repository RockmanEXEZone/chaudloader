# chaudloader

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

-   `init.lua`: **Optional.** The Lua script to run on mod load.

-   `init.dll`: **Optional.** The DLL to load on mod load. It should implement a suitable `DllMain` attach hook to detour the applicable functions in the executable.

### Asset modding

In `init.lua`, you may use the following functions:

```lua
-- Print a log line.
function print(...)

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

For instance, for a simple font mod, you can write the following script:

```lua
local font = bnlc_mod_loader.read_mod_contents("eng_mojiFont.fnt")
bnlc_mod_loader.write_exe_dat_contents("exe6.dat", "exe6/data/font/eng_mojiFont.fnt", font)
bnlc_mod_loader.write_exe_dat_contents("exe6f.dat", "exe6f/data/font/eng_mojiFont.fnt", font)
```

Mods are order dependent: the DAT contents written by a previous mod will be visible to a subsequent mod.

## For library developers

Build the binary with `cargo build --release`. You will need nightly Rust.
