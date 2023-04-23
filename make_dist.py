import dataclasses
import os
import tarfile
import typing
import zipfile


@dataclasses.dataclass
class Entry:
    dest: str
    src: str
    mode: int = None


def make_entries():
    entries = [
        Entry("README.md", "README.md"),
        Entry("chaudloader.dll", "target/release/chaudloader.dll"),
        Entry("dxgi.dll", "target/release/chaudloader.dll"),
        Entry("lua54.dll", "lua54.dll"),
    ]

    for root, _, filenames in os.walk("build"):
        entries.append(Entry(f"{root}/", f"{root}/"))
        for filename in filenames:
            entries.append(Entry(f"{root}/{filename}", f"{root}/{filename}"))

    return entries


def make_windows_entries():
    return [
        *make_entries(),
        Entry("install.exe", "target/release/install.exe"),
    ]


def make_linux_entries():
    return [
        *make_entries(),
        Entry("install", "target/x86_64-unknown-linux-musl/release/install", 0o755),
    ]


def make_zip(entries: typing.List[Entry]):
    with zipfile.ZipFile("dist.zip", "w", zipfile.ZIP_DEFLATED) as zf:
        for entry in entries:
            # Mode is ignored here.
            zf.write(entry.src, entry.dest)


def make_tar(entries: typing.List[Entry]):
    with tarfile.open("dist.tar.bz2", "w:bz2") as tf:
        for entry in entries:
            ti = tf.gettarinfo(entry.src)
            ti.name = entry.dest
            if entry.mode is not None:
                ti.mode = entry.mode
            if not ti.isdir():
                with open(entry.src, "rb") as f:
                    tf.addfile(ti, f)
            else:
                tf.addfile(ti)


make_zip(make_windows_entries())
make_tar(make_linux_entries())
