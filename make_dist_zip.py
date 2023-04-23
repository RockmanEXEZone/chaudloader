import dataclasses
import os
import datetime
import zipfile


@dataclasses.dataclass
class File:
    path: str


@dataclasses.dataclass
class Directory:
    pass


now = datetime.datetime.now()


def zip_info(name, permissions=0o644):
    zi = zipfile.ZipInfo(name, now.timetuple()[:6])
    zi.external_attr = permissions << 16
    return zi


files = [
    (zip_info("README.md"), File("README.md")),
    (zip_info("chaudloader.dll"), File("target/release/chaudloader.dll")),
    (zip_info("dxgi.dll"), File("target/release/chaudloader.dll")),
    (zip_info("lua54.dll"), File("lua54.dll")),
    (zip_info("install.exe"), File("target/release/install.exe")),
    (
        zip_info("install-linux", 0o755),
        File("target/x86_64-unknown-linux-musl/release/install"),
    ),
]

for root, _, filenames in os.walk("build"):
    files.append((zip_info(root, 0o755), Directory()))
    for filename in filenames:
        files.append((zip_info(f"{root}/{filename}"), File(f"{root}/{filename}")))


with zipfile.ZipFile("dist.zip", "w", zipfile.ZIP_DEFLATED) as zf:
    for zi, src in files:
        if isinstance(src, Directory):
            zf.mkdir(zi)
        elif isinstance(src, File):
            with open(src.path, "rb") as f:
                buf = f.read()
            zf.writestr(zi, buf)
