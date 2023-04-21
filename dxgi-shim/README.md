# dxgi-shim

dxgi-shim is a `dxgi.dll` that proxies enough of `dxgi.dll` to the system DLL, but will also load any library named `bnlc_mod_loader.dll` on attach.

Note that loading another library on attach is illegal according to Microsoft, but we seem to be able to get away with it.
