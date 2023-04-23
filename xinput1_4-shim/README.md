# xinput1_4-shim

xinput1_4-shim is a `xinput1_4.dll` that proxies enough of `xinput1_4.dll` to the system DLL, but will also load any library named `chaudloader.dll` on attach.

Note that loading another library on attach is illegal according to Microsoft, but we seem to be able to get away with it.
