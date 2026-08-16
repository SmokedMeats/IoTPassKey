# nRF52840 Development Kit: buy list and setup

How to get the first Accessory firmware onto a real board and prove hide-after-pair. Written for **Windows** (this project’s usual machine). Official Zephyr tabs for Ubuntu and macOS: [Getting Started Guide](https://docs.zephyrproject.org/latest/develop/getting_started/index.html).

This is not the protocol specification. Terms match [`primer.md`](primer.md) and [`CONTEXT.md`](../CONTEXT.md).

**What you will have when this is done:** the board appears as **SB Accessory** for 30 seconds, a laptop can pair, a second phone’s scan list no longer shows it.

You do **not** need nRF Connect Software Development Kit (NCS). This tree is vanilla Zephyr 4.4. You do **not** need the Security Manager Protocol `0xF0` / `0xF1` host patch for this demo.

---

## 1. Buy list (quantities)

Buy these. One of each is enough.

| Qty | Item | Why | Where |
| --- | ---: | --- | --- |
| **1** | **nRF52840 Development Kit (DK)** Nordic part **NRF52840-DK**, board name PCA10056 | The board this firmware targets (`nrf52840dk/nrf52840`) | Product: [nordicsemi.com — nRF52840 DK](https://www.nordicsemi.com/Products/Development-hardware/nRF52840-DK). Buy: [Digi-Key NRF52840-DK](https://www.digikey.com/en/products/detail/nordic-semiconductor-asa/NRF52840-DK/8593726) (~$49 USD) or search [Mouser NRF52840-DK](https://www.mouser.com/c/?q=NRF52840-DK) |
| **1** | **Micro-USB 2.0 cable that carries data** (not charge-only) | The kit is **micro-USB**, not USB-C. Flashing uses jack **J2** (debugger), not J3 | Any reputable cable. If a drive named **JLINK** never appears in Windows Explorer, replace the cable first |
| **1** | **USB-C to USB-A adapter** — only if the computer has no USB-A port | So the micro-USB cable can plug in | Any |

Use devices you already have (do not buy extras):

| Qty | Item | Why |
| --- | ---: | --- |
| **1** | Windows personal computer with Bluetooth Low Energy | Builds, flashes, and acts as the Host (pairs with the Accessory) |
| **1** | Second phone (Android or iPhone) | Scanner: confirm the Accessory leaves the public scan list after pairing |

The kit box usually includes the board and a Near Field Communication antenna. You do not need the antenna, a coin cell, or a second development kit.

**Do not buy for this step:** a Bluetooth sniffer, nRF Connect Software Development Kit, extra buttons or modules, a second nRF52840 DK.

---

## 2. Download and install (Windows)

Install in this order. Close and reopen the terminal after the `winget` step so `PATH` updates.

Official Zephyr host-tool versions (minimums): CMake 3.28, Python 3.12, Devicetree compiler 1.4.6. Python **3.12** is the version Zephyr recommends; newer Python can break Windows package installs. Source: [Zephyr Getting Started](https://docs.zephyrproject.org/latest/develop/getting_started/index.html).

### 2.1 Host tools (one command)

In PowerShell or Command Prompt:

```powershell
winget install Kitware.CMake Ninja-build.Ninja oss-winget.gperf Python.Python.3.12 Git.Git oss-winget.dtc wget 7zip.7zip
```

That is Zephyr’s Windows list. `winget` is built into current Windows 10/11. If `winget` is missing: [Install winget](https://learn.microsoft.com/en-us/windows/package-manager/winget/).

Manual downloads if you prefer not to use `winget`:

| Qty | Tool | Download |
| --- | --- | --- |
| 1 | Python 3.12 (64-bit Windows installer). Check **Add python.exe to PATH** | [python.org — Python 3.12 releases](https://www.python.org/downloads/release/python-31210/) |
| 1 | Git for Windows | [git-scm.com/download/win](https://git-scm.com/download/win) |
| 1 | CMake (Windows x64 installer; add CMake to PATH) | [cmake.org/download](https://cmake.org/download/) |
| 1 | Ninja | [github.com/ninja-build/ninja/releases](https://github.com/ninja-build/ninja/releases) |

### 2.2 Debugger and flash tools (required to program the board)

| Qty | Tool | What to download | After install |
| --- | --- | --- | --- |
| 1 | **SEGGER J-Link Software and Documentation Pack** (Windows 64-bit). Free for owners of a J-Link, including the debugger on this kit | [segger.com/downloads/jlink](https://www.segger.com/downloads/jlink/) — accept the license, run the installer, include USB drivers | Plug the board in later; Windows should show a **JLINK** drive |
| 1 | **nRF Util** (`nrfutil.exe`) | [nordicsemi.com — nRF Util](https://www.nordicsemi.com/Products/Development-tools/nRF-Util) — Download tab, Windows binary | Move `nrfutil.exe` to a folder on your `PATH` (example: `C:\nordic_tools\`). Then run: `nrfutil install device` |

`west flash` talks to nRF Util (or, as a fallback, `west flash --runner jlink` after J-Link is installed).

### 2.3 Phone app (scanner)

| Qty | App | Link |
| --- | --- | --- |
| 1 | **nRF Connect for Mobile** on the second phone | Nordic page: [nRF Connect for Mobile](https://www.nordicsemi.com/Products/Development-tools/nRF-Connect-for-mobile). [Google Play](https://play.google.com/store/apps/details?id=no.nordicsemi.android.mcp). [Apple App Store](https://apps.apple.com/app/nrf-connect-for-mobile/id1054362403) |

### 2.4 Optional (not required)

| Qty | Tool | Link |
| --- | --- | --- |
| 1 | nRF Connect for Desktop (graphical Programmer / serial terminal) | [nordicsemi.com — nRF Connect for Desktop](https://www.nordicsemi.com/Products/Development-tools/nRF-Connect-for-Desktop) |
| 1 | Serial terminal for logs (PuTTY or the Desktop terminal) | [putty.org](https://www.putty.org/) |

Do **not** install nRF Connect Software Development Kit for this repo.

---

## 3. One-time Zephyr workspace (this repository)

First download of Zephyr plus the Software Development Kit is several gigabytes and can take a long time.

Open a **new** PowerShell after section 2. From the git root (`IoTPassKey`):

```powershell
python --version
cmake --version
git --version
```

Expect Python 3.12.x and CMake ≥ 3.28.

Create a virtual environment so Zephyr’s Python packages stay isolated:

```powershell
python -m venv .venv
.\.venv\Scripts\Activate.ps1
python -m pip install --upgrade pip
pip install west
```

If PowerShell refuses to run `Activate.ps1`, either use Command Prompt (`.\.venv\Scripts\activate.bat`) or, for this user only: `Set-ExecutionPolicy -Scope CurrentUser RemoteSigned`.

Initialize the west workspace **from the git root** (this repo is the manifest; do not `west init` a second Zephyr tree elsewhere):

```powershell
west init -l firmware
west update
west packages pip --install
west zephyr-export
```

`west update` clones Zephyr v4.4.2 and the modules listed in `firmware/west.yml` into `zephyr/` and `modules/` (gitignored).

Install the Zephyr Software Development Kit (compiler). Arm only is enough for this board:

```powershell
west sdk install --toolchains arm-zephyr-eabi
```

Full install (every architecture) also works and is larger: `west sdk install`.

Activate `.venv` in every new terminal before `west` commands.

---

## 4. First power-on of the board

1. Slide **SW9** (nRF power source) to **VDD**.
2. Slide **SW8** (power) to **ON**.
3. Connect the micro-USB cable to **J2** (next to the debugger, labeled for the interface microcontroller), not J3.
4. Connect the other end to the computer.
5. Windows Explorer should show a removable drive named **JLINK**.

If there is no JLINK drive: try another cable (must be data-capable), another USB port, and reinstall J-Link USB drivers from the SEGGER install folder.

Nordic’s own first-plug steps: [nRF52840 DK — Get Started](https://www.nordicsemi.com/Products/Development-hardware/nRF52840-DK/GetStarted). Zephyr board page: [nrf52840dk](https://docs.zephyrproject.org/latest/boards/nordic/nrf52840dk/doc/index.html).

---

## 5. Build and flash this firmware

From the git root, with `.venv` active and the board plugged into J2:

```powershell
west build -p always -b nrf52840dk/nrf52840 firmware/accessory
west flash
```

If `west flash` says `nrfutil` was not found, confirm `nrfutil` is on `PATH` and you ran `nrfutil install device`. Fallback:

```powershell
west flash --runner jlink
```

The board resets and runs. On a serial terminal (115200 baud, J-Link virtual COM port) you should see a log line like `Stealth Bonding Accessory`.

---

## 6. Prove it on the air

Device name on the air: **SB Accessory**.

| Control | What it does |
| --- | --- |
| **Button 1** (SW1, labeled on the silk) | Opens a 30 second Enrollment Window |
| **LED 0** (LED1 on some silk; Zephyr alias `led0`) | On during the Enrollment Window |

1. If the Bond Table is empty, enrollment starts on boot. Otherwise press **Button 1**. LED on = findable.
2. On the **laptop**: Settings → Bluetooth → look for **SB Accessory** → pair. Accept Just Works (no PIN). This is ordinary Low Energy Secure Connections, not draft `0xF0` / `0xF1`.
3. On the **second phone**: open nRF Connect for Mobile → Scanner. After a successful pair, **SB Accessory** must not appear as a named or nameless undirected device.
4. The laptop may reconnect via Directed Knock. Some phones (especially iPhone) are picky about directed advertising. The first proof that matters is step 3.

To add another Host later: press Button 1 again during a new 30 second window (Bond Table holds up to 8).

To start over: recover/erase the chip, then flash again:

```powershell
west flash --erase
```

Or in nRF Util: `nrfutil device recover` then `west flash`.

---

## 7. Checklist

- [ ] 1× nRF52840 DK
- [ ] 1× data-capable micro-USB cable on **J2**
- [ ] USB-C adapter if the computer needs it
- [ ] winget host tools (or manual Python 3.12, Git, CMake, Ninja)
- [ ] J-Link pack
- [ ] nRF Util + `nrfutil install device`
- [ ] `west init -l firmware` / `west update` / `west sdk install --toolchains arm-zephyr-eabi`
- [ ] `west build` / `west flash`
- [ ] Laptop paired with **SB Accessory**
- [ ] Second phone scan list empty after the pair

---

## 8. What this setup does not give you

- Draft Security Manager Protocol Params (`0xF0` / `0xF1`) on the air — needs a Zephyr host-stack patch and a second patched peer. See [`primer.md`](primer.md).
- Stealth Bonding on an arbitrary speaker — this is one development kit running our Accessory app.
- A SIG-qualified product.

Firmware tree notes: [`firmware/README.md`](../firmware/README.md).
