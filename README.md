# bnlc-mod-loader

bnlc-mod-loader is a mod loader for Mega Man Battle Network Legacy Collection.

## For users

1. Copy the `bnlc_mod_loader.dll` and `dxgi.dll` files into the same folder as `MMBN_LC1.exe` and `MMBN_LC2.exe`.

2. Start the game. This will generate a config file named `bnlc_mod_loader.toml` and a mods directory named `mods`.

3. Put your mods in the `mods` file. To activate them, edit `bnlc_mod_loader.toml` like so:

    ```toml
    [[mods]]
    name = "name-of-your-mod"
    trusted = true  # set this if the mod uses a DLL and you really trust the author
    ```

## For mod developers

Mods consists of two required files:

-   `info.toml`: Metadata about your mod. It should look something like this:

    ```toml
    title = "my cool mod"
    authors = ["my cool name"]
    ```

-   `init.lua`: The entry point of your mod.

You may additionally include an `init.dll` to be loaded when the mod loads. It should implement a suitable `DllMain` attach hook to detour the applicable functions in the executable.

### Asset modding

In `init.lua`, you may use the following functions:

```lua
-- Print a log line.
function print(...)

-- Reads the contents of a file out of a .dat file (e.g. `exe6.dat`).
--
-- Previous calls to write_dat_contents are visible to read_dat_contents.
function bnlc_mod_loader.read_dat_contents(dat_filename: string, path: string): string

-- Writes the given data into a .dat file.
function bnlc_mod_loader.write_dat_contents(dat_filename: string, path: string, contents: string)

-- Reads the contents of a file from the mod folder.
function bnlc_mod_loader.read_mod_contents(path: string): string
```

For instance, for a simple font mod, you can write the following script:

```lua
local font = bnlc_mod_loader.read_mod_contents("eng_mojiFont.fnt")
bnlc_mod_loader.write_dat_contents("exe6.dat", "exe6/data/font/eng_mojiFont.fnt", font)
bnlc_mod_loader.write_dat_contents("exe6f.dat", "exe6f/data/font/eng_mojiFont.fnt", font)
```

## For library developers

Build the binary with `cargo build --release`. You will need nightly Rust.
