# Zephyr firmware research

Primary-source notes for the first Accessory firmware tree. 15 August 2026.

## Versions

- Zephyr latest docs list 4.4.0 as a current release line; v4.4.2 is the bugfix tag on GitHub ([Zephyr docs versions](https://docs.zephyrproject.org/latest/connectivity/bluetooth/api/gap.html), [v4.4.2](https://github.com/zephyrproject-rtos/zephyr/releases)).
- nRF Connect SDK 3.2.x is current for Nordic product work ([NCS 3.2.3 nRF52 guide](https://docs.nordicsemi.com/bundle/ncs-3.2.3/page/nrf/app_dev/device_guides/nrf52/index.html)).
- This repo uses **vanilla Zephyr 4.4.2**, not NCS. NCS is a superset (extra modules, nRF samples). The first Accessory only needs Zephyr BLE host + `hal_nordic`. We can add an NCS overlay later if we want Nordic-only helpers.

## Workspace topology

Zephyr defines a **workspace application**: the app is a west project that *imports* Zephyr ([Application development](https://docs.zephyrproject.org/latest/develop/application/index.html)). The official template is [zephyrproject-rtos/example-application](https://github.com/zephyrproject-rtos/example-application) (`west.yml` imports `zephyr` with a module allowlist).

For this monorepo:

- `firmware/west.yml` is the manifest.
- `west init -l firmware` from the git root makes the repo the west workspace.
- Zephyr clones to `./zephyr` (gitignored).
- App lives at `firmware/accessory`.

Do not put a full Zephyr clone in git. Do not start a second repo until the west tree is large.

## Board

Default target: **`nrf52840dk/nrf52840`** (PCA10056). Same string in Zephyr 4.2+ and NCS 3.2 ([nRF52840 DK](https://docs.zephyrproject.org/4.2.0/boards/nordic/nrf52840dk/doc/index.html), [NCS board names](https://docs.nordicsemi.com/bundle/ncs-3.2.2/page/nrf/app_dev/board_names.html)).

Retarget with `west build -b <board> firmware/accessory`.

## BLE Kconfig (screenless Accessory)

| Goal | Kconfig | Source |
| --- | --- | --- |
| Peripheral + SMP | `CONFIG_BT`, `CONFIG_BT_PERIPHERAL`, `CONFIG_BT_SMP` | [peripheral_sc_only prj.conf](https://github.com/zephyrproject-rtos/zephyr/blob/v4.4.2/samples/bluetooth/peripheral_sc_only/prj.conf) |
| LE SC, no legacy | `CONFIG_BT_SMP_SC_PAIR_ONLY` (default on since Zephyr 4.1) | [4.1 migration](https://docs.zephyrproject.org/4.2.0/releases/migration-guide-4.1.html) |
| Not authenticated-only | Do **not** set `CONFIG_BT_SMP_SC_ONLY` | That sample requires security level 4 ([SC-only README](https://docs.zephyrproject.org/latest/samples/bluetooth/peripheral_sc_only/README.html)). A `NoInputNoOutput` Accessory uses Just Works. |
| Bond Table size 8 | `CONFIG_BT_MAX_PAIRED=8` | Spec `N_SB_BONDS_REC` |
| Persist bonds | `CONFIG_SETTINGS`, `CONFIG_BT_SETTINGS`, flash + NVS | Zephyr BT settings |
| Privacy / RPA | `CONFIG_BT_PRIVACY` | LE Privacy; not a substitute for leaving Public Advertising |
| No silent eviction | leave `CONFIG_BT_KEYS_OVERWRITE_OLDEST` off | Spec: full table refuses |

## Advertising after bond

Nameless **undirected** advertising is still Public Advertising (spec §7.3). Filter Accept List + undirected ads can still blip a scan list.

After Stealth Commitment the firmware MUST `bt_le_adv_stop()` undirected ads, then either Directed Knock (`BT_LE_ADV_CONN_DIR_LOW_DUTY`) or stay silent. [GAP API](https://docs.zephyrproject.org/latest/connectivity/bluetooth/api/gap.html).

## Custom SMP opcodes

Draft Params PDUs `0xF0` / `0xF1` live in the Zephyr **host** SMP state machine (`subsys/bluetooth/host/smp.c`). An out-of-tree app cannot register new SMP opcodes. That needs a Zephyr fork or an upstream patch. This tree implements the **GAP half** (Enrollment Window, LE SC bond, commitment = leave Public Advertising) on stock Zephyr.

## CI

Official action: [zephyrproject-rtos/action-zephyr-setup@v1](https://github.com/zephyrproject-rtos/action-zephyr-setup). example-application [build.yml](https://github.com/zephyrproject-rtos/example-application/blob/main/.github/workflows/build.yml) checks out the app, runs the action, then `west build` / twister.

First CI bar: compile `firmware/accessory` for `nrf52840dk/nrf52840`. No board required.

## Sysbuild

Sysbuild is for multi-image (MCUboot + app). A single Accessory image uses classic `west build`. Add sysbuild when we add a bootloader.
