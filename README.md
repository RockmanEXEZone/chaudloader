# bnlc-mod-loader

bnlc-mod-loader is a mod loader for Mega Man Battle Network Legacy Collection.

## For users

1. Copy the `bnlc_mod_loader.dll` and `dxgi.dll` files into the same folder as `MMBN_LC1.exe` and `MMBN_LC2.exe`.

2. Make a folder called `mods` and put your mods in the folder.

3. Start the game. Mods in the `mods` folder will be activated in alphabetical order.

## For mod developers

Mods consists of two required files in a directory inside the `mods` folder:

-   `info.toml`: Metadata about your mod. It should look something like this:

    ```toml
    title = "my cool mod"
    version = "0.0.1"
    authors = ["my cool name"]
    ```

-   `init.lua`: The entry point of your mod.

You may additionally include an `init.dll` to be loaded when the mod loads. It should implement a suitable `DllMain` attach hook to detour the applicable functions in the executable. `init.dll` will automatically be loaded when the mod is loaded.

### Asset modding

In `init.lua`, you may use the following functions:

```lua
-- Print a log line.
function print(...)

-- Reads the contents of a file out of a zip .dat file (e.g. `exe6.dat`).
--
-- Previous calls to write_exe_zipdat_contents are visible to subsequent calls to read_exe_zipdat_contents.
function bnlc_mod_loader.read_exe_zipdat_contents(zipdat_filename: string, path: string): string

-- Writes the given data into a zip .dat file.
--
-- Note that this does not mutate the original .dat file on disk, but for all intents and purposes to both the game and the mod loader it does.
function bnlc_mod_loader.write_exe_zipdat_contents(zipdat_filename: string, path: string, contents: string)

-- Reads the contents of a file from the mod folder.
function bnlc_mod_loader.read_mod_contents(path: string): string

-- Deprecated aliases.
bnlc_mod_loader.read_dat_contents = bnlc_mod_loader.read_exe_zipdat_contents
bnlc_mod_loader.write_dat_contents = bnlc_mod_loader.write_exe_zipdat_contents
```

For instance, for a simple font mod, you can write the following script:

```lua
local font = bnlc_mod_loader.read_mod_contents("eng_mojiFont.fnt")
bnlc_mod_loader.write_exe_zipdat_contents("exe6.dat", "exe6/data/font/eng_mojiFont.fnt", font)
bnlc_mod_loader.write_exe_zipdat_contents("exe6f.dat", "exe6f/data/font/eng_mojiFont.fnt", font)
```

Mods are order dependent: the DAT contents written by a previous mod will be visible to a subsequent mod.

## For library developers

Build the binary with `cargo build --release`. You will need nightly Rust.
