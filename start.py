import os
import sys

FS = "datadisk.img"
FS_SOURCE = "dsk"
ALWAYS_FETCH_TOOLS = False
ALWAYS_RECOMPILE_TOOLS = False
ALWAYS_RECOMPILE = False

if sys.version_info.major < 3:
    print("Trying to call Python 3...")
    os.system("python3 " + __file__)
    exit(0)

import shutil
import platform


def get_latest_modified_time(directory):
    latest_time = 0

    for root, dirs, files in os.walk(directory):
        for file in files:
            file_path = os.path.join(root, file)
            modified_time = os.path.getmtime(file_path)
            if modified_time > latest_time:
                latest_time = modified_time

    return latest_time


PWD = os.path.dirname(os.path.abspath(__file__))
EXE_PREFIX = "" if platform.system() == "Windows" else "./"
EXE_SUFFIX = ".exe" if platform.system() == "Windows" else ""

argv = sys.argv
if len(argv) < 2:
    print("Please give me an additional argument: boot image's position.")
    exit(1)

BOOT_IMAGE = argv[1]

print("Checking need for compile FS Helper...")
FS_COMPLIER = EXE_PREFIX + "fs-compiler" + EXE_SUFFIX
if ALWAYS_FETCH_TOOLS:
    os.system("git submodule update --remote")
if ALWAYS_RECOMPILE_TOOLS or not os.path.exists(FS_COMPLIER):
    os.makedirs("../tmp-compiling", exist_ok=True)
    TOOL_DIR = "../tmp-compiling"

    if os.path.exists("../tmp-compiling"):
        shutil.rmtree("../tmp-compiling", ignore_errors=True)

    if not os.path.exists("src/tools/fs-compiler"):
        print("Cloning fs-compiler...")
        os.system("git clone git@github.com:Cinea4678/TJU-cinea-os-tools.git ../tmp-compiling")
    else:
        shutil.copytree("src/tools/fs-compiler", "../tmp-compiling/fs-compiler", symlinks=True)

    os.chdir(TOOL_DIR + "/fs-compiler")
    os.system("cargo build --release")
    os.chdir(PWD)
    shutil.copy(TOOL_DIR + "/fs-compiler/target/release/" + FS_COMPLIER, FS_COMPLIER)
    shutil.rmtree("../tmp-compiling", ignore_errors=True)

print("Checking need for recompile the file system...")
re_compile = False
if not os.path.exists(FS):
    print("File System not exist yet...")
    re_compile = True
elif get_latest_modified_time(FS_SOURCE) > os.path.getmtime(FS):
    print("File System is older than source dir...")
    re_compile = True
if ALWAYS_RECOMPILE or re_compile:
    print("Recompiling the file system...")
    os.system(FS_COMPLIER + " " + FS_SOURCE + " " + FS)
else:
    print("File System is already newest.")

print("Starting QEMU...", flush=True)
os.system(f"qemu-system-x86_64 -drive format=raw,file={BOOT_IMAGE} -serial " +
          "stdio -m 1G -monitor telnet:localhost:4444,server,nowait -drive format=raw,file=datadisk.img")
