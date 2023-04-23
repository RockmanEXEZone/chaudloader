import os
import datetime
import zipfile


now = datetime.datetime.now()


def zfile(name, mode=0o644):
    zi = zipfile.ZipInfo(name, now.timetuple()[:6])
    zi.external_attr = (0o100000 | mode) << 16
    return zi


def zdir(name, mode=0o755):
    zi = zipfile.ZipInfo(name, now.timetuple()[:6])
    zi.compress_size = 0
    zi.file_size = 0
    zi.CRC = 0
    zi.external_attr = (0o40000 | mode) << 16
    zi.external_attr |= 0x10
    return zi


files = [
    (zfile("README.md"), "README.md"),
    (zfile("chaudloader.dll"), "target/release/chaudloader.dll"),
    (zfile("dxgi.dll"), "target/release/chaudloader.dll"),
    (zfile("lua54.dll"), "lua54.dll"),
    (zfile("install.exe"), "target/release/install.exe"),
    (
        zfile("install-linux", 0o755),
        "target/x86_64-unknown-linux-musl/release/install",
    ),
]

for root, _, filenames in os.walk("build"):
    files.append((zdir(f"{root}/"), None))
    for filename in filenames:
        files.append((zfile(f"{root}/{filename}"), f"{root}/{filename}"))


with zipfile.ZipFile("dist.zip", "w", zipfile.ZIP_DEFLATED) as zf:
    for zi, src in files:
        if src is not None:
            with open(src, "rb") as f:
                buf = f.read()
        else:
            buf = b""
        zf.writestr(zi, buf)
