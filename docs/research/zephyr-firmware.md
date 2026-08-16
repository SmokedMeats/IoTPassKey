# Zephyr firmware tree for a Stealth Bonding Accessory

| Field | Value |
| --- | --- |
| Date | 2026-08-16 |
| Status | Research note. Implementation followed ADR 0005 (vanilla Zephyr 4.4 in `firmware/`), not the NCS T2 sibling option below. |
| Scope | How IoT Passkey should add an in-repo firmware tree for an nRF-class BLE peripheral without forking `zephyr.git` |
| Versions checked | Zephyr 4.4.0 (latest stable, 2026-04-14); nRF Connect SDK v3.4.0 LTS (2026-07-01) and v3.3.4 (GitHub Latest, 2026-08-03) |

This note answers the workspace, SDK, board, Kconfig, advertising, SMP, CI, and first-app questions. It does not add firmware. Claims are cited to Zephyr, Nordic, or the corresponding sample/source.

Related project docs: [`stealth-bonding.md`](../spec/stealth-bonding.md), [ADR 0001](../adr/0001-protocol-extension-not-profile.md), [ADR 0002](../adr/0002-reuse-le-secure-connections.md), [ADR 0003](../adr/0003-no-public-advertising-after-bond.md), [`reference-implementation.md`](../reference-implementation.md).

---

## Versions (as of 2026-08-16)

| Tree | Current official pin | Notes |
| --- | --- | --- |
| Zephyr | **4.4.0** (2026-04-14; EOL 2027-04-12) | Latest stable. Next is 4.5 (October 2026). LTS remains 3.7.0 through 2029-07-27. [Zephyr releases](https://docs.zephyrproject.org/latest/releases/index.html), [4.4.0 notes](https://docs.zephyrproject.org/latest/releases/release-notes-4.4.html), [v4.4.0 tag](https://github.com/zephyrproject-rtos/zephyr/releases/tag/v4.4.0) |
| nRF Connect SDK | **v3.4.0 LTS** (2026-07-01) | Based on Zephyr 4.4. Five-year LTS. **Last NCS release that includes nRF52 Series support.** Toolchain is Zephyr SDK v1.0.1. [v3.4.0 release notes](https://github.com/nrfconnect/sdk-nrf/blob/v3.4.0/doc/nrf/releases_and_maturity/releases/release-notes-3.4.0.rst), [v3.4.0 tag](https://github.com/nrfconnect/sdk-nrf/releases/tag/v3.4.0) |
| nRF Connect SDK (patch line) | **v3.3.4** (2026-08-03) | Marked Latest on GitHub. Patch on 3.3.x; `sdk-zephyr` tracks upstream through `93b5f19f99`. Prefer 3.4.0 LTS for an nRF52840 product. [v3.3.4 notes](https://github.com/nrfconnect/sdk-nrf/blob/v3.3.4/doc/nrf/releases_and_maturity/releases/release-notes-3.3.4.rst), [Latest release](https://github.com/nrfconnect/sdk-nrf/releases/latest) |

Zephyr 4.4 is the first release that supports Zephyr SDK 1.0 and defaults to C17. [4.4.0 notes](https://docs.zephyrproject.org/latest/releases/release-notes-4.4.html)

NCS v3.4.0 and its patches are the last NCS releases that include nRF52. Support for nRF52 will be removed from samples and applications on the development branch. [v3.4.0 release notes](https://github.com/nrfconnect/sdk-nrf/blob/v3.4.0/doc/nrf/releases_and_maturity/releases/release-notes-3.4.0.rst)

---

## 1. Recommended workspace topology

**Recommendation for IoT Passkey: T2 workspace application, with this Git repo as the west manifest. Import nRF Connect SDK v3.4.0 LTS. Do not fork `zephyr.git`. Do not make the west topdir itself a Git repository.**

### Three Zephyr application types

Zephyr distinguishes applications by where the app directory lives. [Application Development](https://docs.zephyrproject.org/latest/develop/application/index.html)

| Type | Location | Use for this repo? |
| --- | --- | --- |
| Repository | Inside `zephyr/` (or, for NCS, inside `nrf/`) | No. Nordic says placing an app in the SDK tree can corrupt the install. [Creating an application](https://github.com/nrfconnect/sdk-nrf/blob/v3.4.0/doc/nrf/app_dev/create_application.rst) |
| Workspace | Inside a west workspace, outside `zephyr/` / `nrf/` | **Yes.** App carries its own `west.yml` and pins the SDK. |
| Freestanding | Outside any west workspace; `ZEPHYR_BASE` points at a shared SDK | Only for throwaway prototypes. SDK revision is not in this repo. |

### Three west topologies

[West workspaces](https://docs.zephyrproject.org/latest/develop/west/workspaces.html)

| Topology | Manifest repo | Use for this repo? |
| --- | --- | --- |
| T1 | `zephyr` itself | No. That is a Zephyr fork. |
| **T2** | The application repo | **Yes.** This repo already exists and is not `zephyr.git`. |
| T3 | A dedicated empty manifest repo | Only if firmware later splits into several Git remotes. |

T2 layout from Zephyr (names adapted):

```
west-workspace/                 # not a Git repo
├── .west/
├── iotpasskey/                 # this Git repo; never modified by west
│   ├── west.yml                # imports nrf (or zephyr) + modules
│   ├── firmware/accessory/     # CMakeLists.txt, prj.conf, src/
│   └── docs/
├── nrf/                        # sdk-nrf, if using NCS
├── zephyr/                     # sdk-zephyr or upstream zephyr
└── modules/
```

`west.yml` with `import: true` pulls Zephyr’s (or NCS’s) module list at the imported revision, so this repo does not copy-paste every HAL revision. [T2 example](https://docs.zephyrproject.org/latest/develop/west/workspaces.html#t2-star-topology-application-is-the-manifest-repository)

West does **not** officially support making the workspace topdir a Git repository. [Unsupported topology](https://docs.zephyrproject.org/latest/develop/west/workspaces.html#not-supported-workspace-topdir-as-git-repository)

### NCS workflows (Nordic’s names for the same idea)

Nordic’s recommended path for an app in another repo is **Workflow 4: workspace application repository**. [Adding your own code](https://github.com/nrfconnect/sdk-nrf/blob/v3.3.4/doc/nrf/dev_model_and_contributions/adding_code.rst), [Creating an application](https://github.com/nrfconnect/sdk-nrf/blob/v3.4.0/doc/nrf/app_dev/create_application.rst)

| NCS workflow | What it is | For IoT Passkey |
| --- | --- | --- |
| 1. Avoid Git/west | Vendor-managed copy of NCS | No |
| 2. Freestanding | App outside workspace; `ZEPHYR_BASE` | Prototype only. Cannot track NCS patches in this repo. |
| 3. Fork `sdk-nrf` | App lives in a fork of Nordic’s manifest | No. Same class of problem as forking Zephyr. |
| **4. Workspace app (recommended)** | This repo is the manifest; `import` `sdk-nrf` | **Yes.** |

Workspace-application reasons Nordic lists that match this project: west manages the repo set; you can later override `zephyr` / `sdk-zephyr` if the host SMP stack must be patched; CI can pin the SDK in `west.yml`. [Creating an application — workspace](https://github.com/nrfconnect/sdk-nrf/blob/v3.4.0/doc/nrf/app_dev/create_application.rst)

### Zephyr vs nRF Connect SDK

| | Vanilla Zephyr 4.4.0 T2 | NCS v3.4.0 LTS T2 |
| --- | --- | --- |
| Manifest import | `zephyrproject-rtos/zephyr` @ `v4.4.0` | `nrfconnect/sdk-nrf` @ `v3.4.0` (`import: true`) |
| Host stack | Upstream Zephyr host | Nordic’s `sdk-zephyr` (Zephyr 4.4 + NCS commits) |
| Controller on nRF52 | Zephyr SoftDevice Controller / LL split in-tree | Nordic SoftDevice Controller in NCS |
| DK buttons/LEDs | `samples/basic/button`, `samples/basic/blinky`, GPIO | `CONFIG_DK_LIBRARY` / `dk_buttons_and_leds` |
| IDE | west + editor | nRF Connect for VS Code is Nordic’s recommended IDE. [v3.4.0 notes](https://github.com/nrfconnect/sdk-nrf/blob/v3.4.0/doc/nrf/releases_and_maturity/releases/release-notes-3.4.0.rst) |
| CI | `zephyrproject-rtos/action-zephyr-setup` + Zephyr SDK | NCS docker image or `nrfutil toolchain-manager` |
| nRF52 future | Stays in upstream Zephyr | Last NCS line with nRF52. [v3.4.0 notes](https://github.com/nrfconnect/sdk-nrf/blob/v3.4.0/doc/nrf/releases_and_maturity/releases/release-notes-3.4.0.rst) |

**Use NCS v3.4.0 LTS** for a production nRF52840 accessory: SoftDevice Controller, DK library, five-year LTS, and the last Nordic SDK that still treats nRF52 as supported. Pin it from this repo’s `west.yml`; do not clone `sdk-nrf` as the product repo.

**Vanilla Zephyr 4.4.0** is a valid T2 alternative if the first milestone is “compile a GAP peripheral with upstream samples only” and CI should stay on the public Zephyr SDK action. The BLE host APIs below are the same family in both trees.

If a later milestone must patch `subsys/bluetooth/host/smp.c`, do it by adding a `zephyr` project override in `west.yml` that points at a **private fork of `sdk-zephyr` (or upstream zephyr)**, not by turning IoT Passkey into a Zephyr fork. Nordic documents this override pattern. [Adding your own code — Workflow 4](https://github.com/nrfconnect/sdk-nrf/blob/v3.3.4/doc/nrf/dev_model_and_contributions/adding_code.rst)

### How developers open the workspace

Do not run `west init` inside the existing Git working tree as if it were the topdir.

From a sibling directory, using a local checkout:

```
west init -l /path/to/IoTPassKey workspace
cd workspace
west update
```

Or from the hosted repo (same pattern as the official example apps):

```
west init -m https://github.com/<org>/IoTPassKey --mr <branch> workspace
cd workspace
west update
```

[example-application getting started](https://github.com/zephyrproject-rtos/example-application/blob/main/README.md), [ncs-example-application getting started](https://github.com/nrfconnect/ncs-example-application/blob/v3.4.0/README.md)

Official templates to copy structure from (not to vendor as the product):

- Zephyr: [zephyrproject-rtos/example-application](https://github.com/zephyrproject-rtos/example-application) — T2, custom board, GitHub Actions. [Application Development — reference workspace](https://docs.zephyrproject.org/latest/develop/application/index.html)
- NCS: [nrfconnect/ncs-example-application](https://github.com/nrfconnect/ncs-example-application) tagged `v3.4.0` to match the SDK. Nordic recommends “Use this template”. [Adding your own code](https://github.com/nrfconnect/sdk-nrf/blob/v3.3.4/doc/nrf/dev_model_and_contributions/adding_code.rst)

Suggested `west.yml` for this repo (NCS path). Revision is the LTS tag, not `main`:

```yaml
manifest:
  remotes:
    - name: ncs
      url-base: https://github.com/nrfconnect
  projects:
    - name: nrf
      repo-path: sdk-nrf
      remote: ncs
      revision: v3.4.0
      import: true
  self:
    path: iotpasskey
```

That is the official Workflow 4 import, with the revision updated to v3.4.0. [Adding your own code](https://github.com/nrfconnect/sdk-nrf/blob/v3.3.4/doc/nrf/dev_model_and_contributions/adding_code.rst), [ncs-example-application v3.4.0 west.yml](https://github.com/nrfconnect/ncs-example-application/blob/v3.4.0/west.yml)

Vanilla-Zephyr alternative: import `zephyr` @ `v4.4.0` with a `name-allowlist` of `cmsis_6` and `hal_nordic` only, as `example-application` does. [example-application west.yml](https://github.com/zephyrproject-rtos/example-application/blob/main/west.yml)

---

## 2. Board id for nRF52840 DK

**Exact board target: `nrf52840dk/nrf52840`.**

That is hardware-model v2 (`board name / SoC`). The old `nrf52840dk_nrf52840` string is obsolete as of Zephyr 3.7. [nRF52840 DK](https://docs.zephyrproject.org/latest/boards/nordic/nrf52840dk/doc/index.html), [Board Porting Guide — terminology](https://docs.zephyrproject.org/latest/hardware/porting/board_porting.html)

```
west build -b nrf52840dk/nrf52840 samples/hello_world
```

[nRF52840 DK — building](https://docs.zephyrproject.org/latest/boards/nordic/nrf52840dk/doc/index.html)

`boards/nordic/nrf52840dk/board.yml` in Zephyr 4.4.0 names the board `nrf52840dk` with SoC `nrf52840` (and an `nrf52811` emulation variant). [board.yml](https://github.com/zephyrproject-rtos/zephyr/blob/v4.4.0/boards/nordic/nrf52840dk/board.yml)

Hardware: PCA10056, nRF52840 Cortex-M4F. Buttons on the DK: SW1–SW4 = P0.11, P0.12, P0.24, P0.25. [nRF52840 DK](https://docs.zephyrproject.org/latest/boards/nordic/nrf52840dk/doc/index.html)

For a later custom accessory board, create `boards/<vendor>/<name>/` with `board.yml`, SoC qualifier, and a unique `west boards` name. Vendor prefix should match `dts/bindings/vendor-prefixes.txt` if the vendor is listed. [Board Porting Guide](https://docs.zephyrproject.org/latest/hardware/porting/board_porting.html)

Until a custom PCB exists, build the first app for `nrf52840dk/nrf52840`. Put board-specific tweaks in `app.overlay` or `app/boards/nrf52840dk_nrf52840.overlay`. [Application Development](https://docs.zephyrproject.org/latest/develop/application/index.html)

---

## 3. Required Kconfig (LE SC, bonding, settings, privacy, limited discoverable)

There is **no** `CONFIG_BT_LIMITED_ADVERTISING`. Limited discoverable is an advertising-data flag plus a timeout Kconfig.

### Stack and role

From `samples/bluetooth/peripheral/prj.conf` and `peripheral_sc_only/prj.conf` on Zephyr v4.4.0. [peripheral prj.conf](https://github.com/zephyrproject-rtos/zephyr/blob/v4.4.0/samples/bluetooth/peripheral/prj.conf), [peripheral_sc_only prj.conf](https://github.com/zephyrproject-rtos/zephyr/blob/v4.4.0/samples/bluetooth/peripheral_sc_only/prj.conf)

```
CONFIG_BT=y
CONFIG_BT_PERIPHERAL=y
CONFIG_BT_SMP=y
```

`CONFIG_BT_SMP` enables the Security Manager Protocol so the device can pair over LE. [host Kconfig](https://github.com/zephyrproject-rtos/zephyr/blob/main/subsys/bluetooth/host/Kconfig)

### LE Secure Connections only

```
CONFIG_BT_SMP_SC_ONLY=y
```

This selects `CONFIG_BT_SMP_SC_PAIR_ONLY` and requires Security Mode 1 Level 4 (authenticated LE SC) except for services that need no security. Legacy pairing is disabled. [host Kconfig — BT_SMP_SC_ONLY](https://github.com/zephyrproject-rtos/zephyr/blob/main/subsys/bluetooth/host/Kconfig)

`CONFIG_BT_SMP_SC_PAIR_ONLY` is default `y` in current host Kconfig (legacy pairing off even without SC-only). SC-only is the stricter “level 4 only” mode the `peripheral_sc_only` sample uses. [peripheral_sc_only README](https://docs.zephyrproject.org/latest/samples/bluetooth/peripheral_sc_only/README.html)

The sample also calls `bt_conn_set_security(conn, BT_SECURITY_L4)` on connect and disconnects on pairing failure. [peripheral_sc_only main.c](https://github.com/zephyrproject-rtos/zephyr/blob/v4.4.0/samples/bluetooth/peripheral_sc_only/src/main.c)

That matches ADR 0002 (reuse LE SC; forbid legacy when this mode is selected).

### Bonding

```
CONFIG_BT_BONDABLE=y
CONFIG_BT_BONDING_REQUIRED=y
```

`CONFIG_BT_BONDABLE` defaults to `y` and is the default bonding flag for ACL connections. `CONFIG_BT_BONDING_REQUIRED` rejects pairing requests that do not set the bondable flag. Per-connection override needs `CONFIG_BT_BONDABLE_PER_CONNECTION` and `bt_conn_set_bondable()`. [host Kconfig](https://github.com/zephyrproject-rtos/zephyr/blob/main/subsys/bluetooth/host/Kconfig), [bt_set_bondable](https://docs.zephyrproject.org/latest/doxygen/html/group__bt__conn.html)

Also set `CONFIG_BT_MAX_PAIRED` to the Bond Table size the spec needs (sample uses `2`). [peripheral_sc_only prj.conf](https://github.com/zephyrproject-rtos/zephyr/blob/v4.4.0/samples/bluetooth/peripheral_sc_only/prj.conf)

### Settings (persist bonds)

From the peripheral / accept-list / direct-adv samples:

```
CONFIG_BT_SETTINGS=y
CONFIG_SETTINGS=y
CONFIG_FLASH=y
CONFIG_FLASH_MAP=y
CONFIG_NVS=y
CONFIG_SYSTEM_WORKQUEUE_STACK_SIZE=2048
```

[peripheral prj.conf](https://github.com/zephyrproject-rtos/zephyr/blob/v4.4.0/samples/bluetooth/peripheral/prj.conf)

`CONFIG_BT_SETTINGS` stores pairing keys in flash. The application **must** call `settings_load()` after `bt_enable()`. The host expects a settings backend that loads items in handle order. [host Kconfig — BT_SETTINGS](https://github.com/zephyrproject-rtos/zephyr/blob/main/subsys/bluetooth/host/Kconfig)

The settings sample itself is `samples/subsys/settings` and lists `nrf52840dk/nrf52840` as a supported board. It shows handler registration, not Bluetooth keys. [Settings API sample](https://docs.zephyrproject.org/latest/samples/subsys/settings/README.html)

### Privacy

```
CONFIG_BT_PRIVACY=y
```

Uses Resolvable Private Addresses by default. Directed advertising to a privacy-enabled peer requires controller privacy and `BT_LE_ADV_OPT_DIR_ADDR_RPA`. [host Kconfig — BT_PRIVACY](https://github.com/zephyrproject-rtos/zephyr/blob/main/subsys/bluetooth/host/Kconfig)

LE Privacy alone does **not** satisfy Stealth Commitment: undirected ads with rotating addresses still appear as nameless devices. [ADR 0003](../adr/0003-no-public-advertising-after-bond.md)

### Limited discoverable

Set the Flags AD type to **limited**, not general:

```c
BT_DATA_BYTES(BT_DATA_FLAGS, (BT_LE_AD_LIMITED | BT_LE_AD_NO_BREDR))
```

`BT_LE_AD_LIMITED` is `0x01` (Limited Discoverable); `BT_LE_AD_GENERAL` is `0x02`. Official samples use general (`BT_LE_AD_GENERAL | BT_LE_AD_NO_BREDR`). The first accessory app should use limited for the enrollment window. [adv.c limited-flag check](https://github.com/zephyrproject-rtos/zephyr/blob/v4.4.0/subsys/bluetooth/host/adv.c)

Timeout:

```
CONFIG_BT_LIM_ADV_TIMEOUT=30
```

Range 1–180 seconds (Core Spec 5.2 Vol 3 Part C Appendix A). Default 30. When the Flags field has `BT_LE_AD_LIMITED`, the host terminates advertising at this timeout. [host Kconfig — BT_LIM_ADV_TIMEOUT](https://github.com/zephyrproject-rtos/zephyr/blob/main/subsys/bluetooth/host/Kconfig)

Use `BT_LE_ADV_CONN_FAST_1` for that window. The API documents it as the GAP recommendation for user-initiated / limited discoverable connectable advertising. [bluetooth.h — BT_LE_ADV_CONN_FAST_1](https://github.com/zephyrproject-rtos/zephyr/blob/v4.4.0/include/zephyr/bluetooth/bluetooth.h)

### Filter accept list (reconnect aid)

```
CONFIG_BT_FILTER_ACCEPT_LIST=y
```

Enables the FAL API. The list is global and cannot be modified while in use. [host Kconfig — BT_FILTER_ACCEPT_LIST](https://github.com/zephyrproject-rtos/zephyr/blob/main/subsys/bluetooth/host/Kconfig)

### DK buttons and LEDs (NCS)

```
CONFIG_DK_LIBRARY=y
```

Then `dk_buttons_init()` / `dk_leds_init()`. [DK Buttons and LEDs](https://docs.nordicsemi.com/bundle/ncs-3.2.1/page/nrf/libraries/others/dk_buttons_and_leds.html), [header in v3.4.0](https://github.com/nrfconnect/sdk-nrf/blob/v3.4.0/include/dk_buttons_and_leds.h)

On vanilla Zephyr, use `samples/basic/button` and `samples/basic/blinky` against the DK GPIO aliases. [nRF52840 DK — testing LEDs and buttons](https://docs.zephyrproject.org/latest/boards/nordic/nrf52840dk/doc/index.html)

### Recommended first `prj.conf` set

```
CONFIG_BT=y
CONFIG_BT_PERIPHERAL=y
CONFIG_BT_SMP=y
CONFIG_BT_SMP_SC_ONLY=y
CONFIG_BT_BONDABLE=y
CONFIG_BT_BONDING_REQUIRED=y
CONFIG_BT_PRIVACY=y
CONFIG_BT_SETTINGS=y
CONFIG_SETTINGS=y
CONFIG_FLASH=y
CONFIG_FLASH_MAP=y
CONFIG_NVS=y
CONFIG_BT_FILTER_ACCEPT_LIST=y
CONFIG_BT_LIM_ADV_TIMEOUT=30
CONFIG_BT_MAX_PAIRED=4
CONFIG_SYSTEM_WORKQUEUE_STACK_SIZE=2048
CONFIG_LOG=y
CONFIG_DK_LIBRARY=y
```

`CONFIG_DK_LIBRARY` is NCS-only. Drop it on vanilla Zephyr.

---

## 4. Stop undirected advertising after bond; directed or filter-accept-list reconnect

This is application GAP policy. It does not require an SMP opcode patch. Official samples already do the two reconnect modes the spec allows.

### Stop undirected / public advertising

`bt_le_adv_stop()` stops ongoing advertising. [bluetooth.h](https://github.com/zephyrproject-rtos/zephyr/blob/v4.4.0/include/zephyr/bluetooth/bluetooth.h)

Call it from `bt_conn_auth_info_cb.pairing_complete` (and do **not** restart undirected general advertising in `disconnected` / `recycled` once a bond exists). The `peripheral_sc_only` sample restarts undirected ads on `recycled`; a stealth accessory must not copy that after commitment. [peripheral_sc_only main.c](https://github.com/zephyrproject-rtos/zephyr/blob/v4.4.0/samples/bluetooth/peripheral_sc_only/src/main.c)

Load bonds with `settings_load()`, then `bt_foreach_bond()` to decide the advertising mode. Both official reconnect samples do this. [direct_adv main.c](https://github.com/zephyrproject-rtos/zephyr/blob/v4.4.0/samples/bluetooth/direct_adv/src/main.c), [peripheral_accept_list main.c](https://github.com/zephyrproject-rtos/zephyr/blob/v4.4.0/samples/bluetooth/peripheral_accept_list/src/main.c)

### Directed reconnect (spec Directed Knock)

Sample: `samples/bluetooth/direct_adv`. [Direct Advertising](https://docs.zephyrproject.org/latest/samples/bluetooth/direct_adv/README.html)

Behavior: no bond → undirected `BT_LE_ADV_CONN_FAST_1`. After bond (sample reboots; an app can switch in place):

```c
adv_param = *BT_LE_ADV_CONN_DIR_LOW_DUTY(&bond_addr);
adv_param.options |= BT_LE_ADV_OPT_DIR_ADDR_RPA;
bt_le_adv_start(&adv_param, NULL, 0, NULL, 0);
```

[direct_adv main.c](https://github.com/zephyrproject-rtos/zephyr/blob/v4.4.0/samples/bluetooth/direct_adv/src/main.c)

`BT_LE_ADV_CONN_DIR_LOW_DUTY` is connectable low-duty directed advertising. `BT_LE_ADV_OPT_DIR_ADDR_RPA` is required when the peer is privacy-enabled and can resolve the target address. The application must check Central Address Resolution. [bluetooth.h](https://github.com/zephyrproject-rtos/zephyr/blob/v4.4.0/include/zephyr/bluetooth/bluetooth.h)

High-duty directed (`BT_LE_ADV_CONN_DIR`) times out; the connected callback gets `BT_HCI_ERR_ADV_TIMEOUT` if the peer does not connect. [bt_le_adv_param](https://docs.zephyrproject.org/latest/doxygen/html/structbt__le__adv__param.html)

**iOS constraint (from the Zephyr sample, not a Zephyr API):** “direct advertising towards iOS based devices is not allowed.” See Apple Accessory Design Guidelines. [Direct Advertising README](https://docs.zephyrproject.org/latest/samples/bluetooth/direct_adv/README.html)

### Filter Accept List reconnect

Sample: `samples/bluetooth/peripheral_accept_list`. [Peripheral Accept List](https://docs.zephyrproject.org/latest/samples/bluetooth/peripheral_accept_list/README.html)

```c
bt_foreach_bond(BT_ID_DEFAULT, add_bonded_addr_to_filter_list, NULL);
/* in callback: bt_le_filter_accept_list_add(&info->addr); */
adv_param = *BT_LE_ADV_CONN_FAST_1;
adv_param.options |= BT_LE_ADV_OPT_FILTER_CONN | BT_LE_ADV_OPT_FILTER_SCAN_REQ;
bt_le_adv_start(&adv_param, ad, ARRAY_SIZE(ad), sd, ARRAY_SIZE(sd));
```

[peripheral_accept_list main.c](https://github.com/zephyrproject-rtos/zephyr/blob/v4.4.0/samples/bluetooth/peripheral_accept_list/src/main.c)

`BT_LE_ADV_OPT_FILTER_CONN` accepts connections only from the list. `BT_LE_ADV_OPT_FILTER_SCAN_REQ` withholds scan response (including the name) from unknown scanners. [bluetooth.h](https://github.com/zephyrproject-rtos/zephyr/blob/v4.4.0/include/zephyr/bluetooth/bluetooth.h)

The sample README is explicit: some centrals (Android phones) may not show the device at all if they are not on the list; unknown connection requests are rejected. [Peripheral Accept List](https://docs.zephyrproject.org/latest/samples/bluetooth/peripheral_accept_list/README.html)

FAL still uses **undirected** advertising PDUs. That can still appear on a public scan list. The spec treats FAL as an aid, not a substitute for leaving Public Advertising. [`stealth-bonding.md` §1.4](../spec/stealth-bonding.md)

### Recommended first-app state machine (no SMP patch)

1. Unprovisioned or pairing-button Enrollment Window: limited discoverable, `BT_LE_ADV_CONN_FAST_1`, `BT_LE_AD_LIMITED`, timeout `CONFIG_BT_LIM_ADV_TIMEOUT`.
2. On `pairing_complete` with `bonded == true`: `bt_le_adv_stop()`. Persist via settings (host does this when `CONFIG_BT_SETTINGS=y`).
3. Reconnect: Directed Knock (`BT_LE_ADV_CONN_DIR_LOW_DUTY` + RPA) **or** SilentUntilWake (no ads until the existing button; then directed for a short window). Use FAL only if directed is rejected by the target OS (iOS).
4. Never restart general undirected advertising while any Bond Passkey exists, except a new Enrollment Window opened by the pairing button.

---

## 5. Custom SMP opcodes without forking Zephyr — honest answer

**No. Draft Stealth Bonding SMP opcodes (`0xF0` / `0xF1`) cannot be added from application code. They require a patch to the Zephyr host SMP state machine.**

This is Bluetooth **Security Manager Protocol** (Vol 3 Part H), not MCUmgr “SMP”. The spec places `Stealth Bonding Params` after Pairing Request/Response and before Phase 2 public-key exchange. [`stealth-bonding.md` §9.3](../spec/stealth-bonding.md)

What the host actually does in Zephyr 4.4 / current `smp.c`:

- Incoming L2CAP SM PDUs are dispatched through a **static** `handlers[]` table indexed by opcode. Entries are the Core Spec commands `0x01`–`0x0E` (Pairing Req/Rsp, Confirm, Random, Failed, key distribution, Security Request, Public Key, DHKey Check, Keypress). [smp.c handlers](https://github.com/zephyrproject-rtos/zephyr/blob/v4.4.0/subsys/bluetooth/host/smp.c)
- If `hdr->code >= ARRAY_SIZE(handlers)`, the PDU is **ignored** as “reserved for future use” (Core Spec 5.2 Vol 3 Part H). [smp.c `bt_smp_recv`](https://github.com/zephyrproject-rtos/zephyr/blob/v4.4.0/subsys/bluetooth/host/smp.c)
- There is no public `bt_smp_register_handler()` (or equivalent) in the Bluetooth API.

`CONFIG_BT_SMP_APP_PAIRING_ACCEPT` only lets the app accept or reject a **standard** pairing initiative. It does not inject PDUs. [host Kconfig](https://github.com/zephyrproject-rtos/zephyr/blob/main/subsys/bluetooth/host/Kconfig)

Do not confuse this with MCUmgr: custom **MCUmgr** groups (IDs ≥ 64) *can* be registered from the app without forking Zephyr. That protocol is unrelated to pairing. [MCUmgr handlers](https://docs.zephyrproject.org/latest/services/device_mgmt/mcumgr_handlers.html)

### What that means for a first firmware tree

| Goal | Without host patch? |
| --- | --- |
| LE SC pairing, bonding, settings, privacy | Yes |
| Limited discoverable enrollment window | Yes |
| Stop undirected ads after bond | Yes |
| Directed or FAL reconnect | Yes |
| Pairing-button / LED UX | Yes |
| Optional GATT management service after a bond exists | Yes (ADR 0001) |
| Exchange `Stealth Bonding Params` (`0xF0`/`0xF1`) | **No** |
| Negotiate Stealth Policy / reconnect mode in SMP | **No** |
| Fail pairing if commitment cannot hide the accessory, inside the SM procedure | **No** (app can disconnect after the fact; that is not the spec’s Phase 1b) |

Later, when the simulator’s PDUs are stable, add a `zephyr` override in `west.yml` to a private `sdk-zephyr` (or upstream) branch that extends `handlers[]` and the pairing sequence. That is a module/project override, not turning this repo into `zephyr.git`. [West T2 overrides](https://docs.zephyrproject.org/latest/develop/west/workspaces.html#t2-star-topology-application-is-the-manifest-repository)

---

## 6. CI that compiles without hardware

**Pattern: GitHub Actions checks out this repo as the manifest, `west init -l`, `west update`, installs the ARM Zephyr toolchain, runs Twister `build_only` / `--integration` against `nrf52840dk/nrf52840`.**

No board is attached. Twister compiles; it does not flash.

### Vanilla Zephyr / Zephyr SDK path

Official example: [example-application `.github/workflows/build.yml`](https://github.com/zephyrproject-rtos/example-application/blob/main/.github/workflows/build.yml)

1. `actions/checkout` **into a subdirectory** (`path: iotpasskey`), so the runner workspace can become the west topdir.
2. `actions/setup-python` with Python 3.12 (Zephyr’s current minimum). [Getting Started](https://docs.zephyrproject.org/latest/develop/getting_started/index.html)
3. [`zephyrproject-rtos/action-zephyr-setup@v1`](https://github.com/zephyrproject-rtos/action-zephyr-setup) with `app-path: iotpasskey` and `toolchains: arm-zephyr-eabi`.
4. `west twister -T firmware/accessory -v --inline-logs --integration`

`west sdk install --toolchains arm-zephyr-eabi` is the non-interactive SDK install for automation. [west sdk](https://docs.zephyrproject.org/latest/develop/west/zephyr-cmds.html)

Twister `--integration` is the CI mode: only platforms listed under `integration_platforms` in the test yaml. [Twister](https://docs.zephyrproject.org/latest/develop/twister/index.html)

Mark the app `build_only: true` so CI never tries to execute on hardware. Official `app/tests.yaml`:

```yaml
sample:
  name: stealth-bonding-accessory
common:
  build_only: true
  integration_platforms:
    - nrf52840dk/nrf52840
tests:
  app.default: {}
```

[example-application tests.yaml](https://github.com/zephyrproject-rtos/example-application/blob/main/app/tests.yaml)

On Windows runners, pass `--short-build-path` (and optionally `-O/tmp/twister-out`) because of path-length limits. The official workflow does this. [example-application build.yml](https://github.com/zephyrproject-rtos/example-application/blob/main/.github/workflows/build.yml)

Zephyr’s own project CI is also GitHub Actions + Twister, including “Build various samples for different boards” without requiring the contributor to have hardware. [Contribution Guidelines — CI](https://docs.zephyrproject.org/latest/contribute/guidelines.html), [Development Environment and Tools](https://docs.zephyrproject.org/latest/project/dev_env_and_tools.html)

Host minimums for Zephyr 4.4: CMake 3.28.0, Python 3.12, DTC 1.4.6. [Getting Started](https://docs.zephyrproject.org/latest/develop/getting_started/index.html)

### NCS path (recommended if `west.yml` imports `sdk-nrf`)

Official example: [ncs-example-application CI](https://github.com/nrfconnect/ncs-example-application/tree/v3.4.0/.github/workflows)

Two documented jobs:

1. **Docker:** `west init -l example-application`, shallow `west update`, then build inside `ghcr.io/nrfconnect/sdk-nrf-toolchain:<checksum>` from `nrf/scripts/print_toolchain_checksum.sh`. [build-using-docker.yml](https://github.com/nrfconnect/ncs-example-application/blob/v3.4.0/.github/workflows/build-using-docker.yml)
2. **Toolchain bundle:** `nrfutil toolchain-manager` installs the NCS bundle, then `nrfutil toolchain-manager launch -- west twister ...`. [build-using-toolchain-bundle.yml](https://github.com/nrfconnect/ncs-example-application/blob/v3.4.0/.github/workflows/build-using-toolchain-bundle.yml)

NCS v3.4.0’s toolchain **is** Zephyr SDK v1.0.1. [v3.4.0 release notes](https://github.com/nrfconnect/sdk-nrf/blob/v3.4.0/doc/nrf/releases_and_maturity/releases/release-notes-3.4.0.rst)

Either NCS workflow compiles hex artifacts (`twister-out/**/zephyr/zephyr.hex`) without hardware.

### What not to run in the first CI

- BabbleSim / `nrf52_bsim` (Linux-only radio simulation; extra setup). [Bluetooth application development](https://docs.zephyrproject.org/latest/connectivity/bluetooth/bluetooth-dev.html)
- QEMU + external controller (needs a dongle). Same page.
- On-target Twister. Not needed to prove the tree builds.

---

## 7. Files a minimal production-shaped app needs

Zephyr’s documented simple app. [Application Development](https://docs.zephyrproject.org/latest/develop/application/index.html)

```
<app>
├── CMakeLists.txt
├── app.overlay
├── prj.conf
├── VERSION
└── src/main.c
```

Sysbuild adds `sysbuild.conf` (and optionally `sysbuild.cmake` / `Kconfig.sysbuild`). [Sysbuild](https://docs.zephyrproject.org/latest/build/sysbuild/index.html)

T2 / NCS workspace adds `west.yml` at the **manifest root** (this Git repo), not necessarily inside `<app>`.

### File-by-file

| File | Required? | Role |
| --- | --- | --- |
| `west.yml` | Yes (T2 / NCS workflow 4) | Manifest. Import `sdk-nrf` v3.4.0 or `zephyr` v4.4.0. |
| `firmware/accessory/CMakeLists.txt` | Yes | `cmake_minimum_required(VERSION 3.28.0)`, `find_package(Zephyr)`, `project(...)`, `target_sources(app PRIVATE src/main.c)`. [Creating an application by hand](https://docs.zephyrproject.org/latest/develop/application/index.html) |
| `firmware/accessory/prj.conf` | Yes | Kconfig in §3. Default fragment name. |
| `firmware/accessory/app.overlay` | Yes for a “pro” tree | Default overlay name. Empty is fine until pins/partitions change. |
| `firmware/accessory/sysbuild.conf` | Yes if using sysbuild | Sysbuild Kconfig (`SB_CONFIG_*`). Default filename in the app directory. Empty is valid for a single-image app. [Sysbuild Kconfig file](https://docs.zephyrproject.org/latest/build/sysbuild/index.html) |
| `firmware/accessory/VERSION` | Recommended | Application version for signed images / lifecycle. [Application Development](https://docs.zephyrproject.org/latest/develop/application/index.html) |
| `firmware/accessory/src/main.c` | Yes | `bt_enable`, `settings_load`, limited adv, pairing callbacks, stop/redirect advertising. |
| `firmware/accessory/tests.yaml` | Yes for CI | Twister `build_only` + `integration_platforms: nrf52840dk/nrf52840`. |
| `firmware/accessory/sysbuild.cmake` | Only for extra images | `ExternalZephyrProject_Add()` for MCUboot or a second core. [Sysbuild](https://docs.zephyrproject.org/latest/build/sysbuild/index.html) |
| `firmware/accessory/Kconfig.sysbuild` | Only with the above | Gate extra images with `SB_CONFIG_*`. |
| `app/boards/*.overlay` | Optional | Per-board overlays, as in example-application. |

NCS enables sysbuild by default for **repository** applications (samples inside `nrf/`). Workspace apps should pass `--sysbuild` or `west config build.sysbuild True` so the same `sysbuild.conf` is used locally and in CI. [Creating an application](https://github.com/nrfconnect/sdk-nrf/blob/v3.4.0/doc/nrf/app_dev/create_application.rst), [Sysbuild — west config](https://docs.zephyrproject.org/latest/build/sysbuild/index.html)

For a first single-image accessory, sysbuild is still the production default (MCUboot later is `SB_CONFIG_BOOTLOADER_MCUBOOT=y` in `sysbuild.conf`). [Sysbuild — MCUboot](https://docs.zephyrproject.org/latest/build/sysbuild/index.html)

NCS v3.4.0 is moving flash layout from Partition Manager to devicetree. New designs should use DTS overlays, not `pm_static.yml`. [v3.4.0 release notes](https://github.com/nrfconnect/sdk-nrf/blob/v3.4.0/doc/nrf/releases_and_maturity/releases/release-notes-3.4.0.rst)

### Suggested tree inside this repo

```
IoTPassKey/                          # existing Git repo = west manifest
├── west.yml
├── firmware/
│   └── accessory/
│       ├── CMakeLists.txt
│       ├── prj.conf
│       ├── app.overlay
│       ├── sysbuild.conf
│       ├── VERSION
│       ├── tests.yaml
│       └── src/
│           └── main.c
├── docs/
└── sim/                             # PDU simulator stays here
```

Build, from the west workspace:

```
west build -b nrf52840dk/nrf52840 --sysbuild iotpasskey/firmware/accessory
```

[example-application: `west build -b $BOARD app`](https://github.com/zephyrproject-rtos/example-application/blob/main/README.md)

Keep `sim/` and `docs/` in this repo. [`reference-implementation.md`](../reference-implementation.md)

---

## What the first app can do (no Zephyr host SMP patch)

A first `firmware/accessory` can prove the **GAP half** of Stealth Bonding on an nRF52840 DK:

1. Pairing button (NCS `dk_buttons_and_leds`, or Zephyr GPIO) opens a limited-discoverable enrollment window.
2. LED shows idle / window-open / bonded.
3. Pair with LE Secure Connections only; require bonding; persist keys with settings.
4. On bond: stop undirected advertising; switch to directed (or silent-until-button).
5. On boot with an existing bond: do not start general/limited undirected advertising.
6. Optional: a post-bond GATT management service (list/revoke). That is allowed by ADR 0001 and does not perform pairing.

It **cannot** speak `Stealth Bonding Params` / `0xF0`–`0xF1`, so it is not a spec-complete Accessory. The Rust PDU simulator remains the oracle for those opcodes until a `sdk-zephyr` override exists. [`reference-implementation.md`](../reference-implementation.md)

Copy behavior from these Zephyr v4.4.0 samples, then delete the parts that keep the device publicly discoverable:

| Sample | Take | Do not copy after bond |
| --- | --- | --- |
| [`peripheral`](https://docs.zephyrproject.org/latest/samples/bluetooth/peripheral/README.html) | `CONFIG_BT_*` + settings + privacy | General discoverable ads, extra GATT profiles |
| [`peripheral_sc_only`](https://docs.zephyrproject.org/latest/samples/bluetooth/peripheral_sc_only/README.html) | `CONFIG_BT_SMP_SC_ONLY`, `BT_SECURITY_L4` | Restart undirected ads on `recycled` |
| [`direct_adv`](https://docs.zephyrproject.org/latest/samples/bluetooth/direct_adv/README.html) | Directed reconnect + RPA | Forced reboot; iOS directed-adv limitation |
| [`peripheral_accept_list`](https://docs.zephyrproject.org/latest/samples/bluetooth/peripheral_accept_list/README.html) | FAL + filter options | Assuming FAL undirected ads are “hidden” |
| [`settings`](https://docs.zephyrproject.org/latest/samples/subsys/settings/README.html) | `settings_load` / handlers | N/A |
| DK buttons | NCS `CONFIG_DK_LIBRARY` or `samples/basic/button` + `blinky` | N/A |

---

## Decision summary

1. **Topology:** T2 workspace application. This Git repo is the manifest. West topdir is a sibling directory, not the repo root.
2. **SDK:** nRF Connect SDK **v3.4.0 LTS** (Zephyr 4.4, last nRF52 NCS). Vanilla Zephyr 4.4.0 is acceptable only for an upstream-only prototype.
3. **Board:** `nrf52840dk/nrf52840`.
4. **Kconfig:** `BT_SMP_SC_ONLY`, bonding + settings, privacy, `BT_LIM_ADV_TIMEOUT`, FAL if used; limited discoverable via `BT_LE_AD_LIMITED`, not a Kconfig switch.
5. **After bond:** `bt_le_adv_stop()`, then directed low-duty + RPA, or silent-until-wake. FAL is a fallback, not stealth by itself.
6. **Custom SMP opcodes:** cannot be added without patching `subsys/bluetooth/host/smp.c`. First app is GAP-only.
7. **CI:** GitHub Actions + west + Twister `build_only` on `nrf52840dk/nrf52840`. Zephyr SDK action, or NCS docker / `nrfutil` toolchain.
8. **Files:** `west.yml`, `CMakeLists.txt`, `prj.conf`, `app.overlay`, `sysbuild.conf`, `VERSION`, `src/main.c`, `tests.yaml`.

---

## Primary sources

- [Zephyr Application Development](https://docs.zephyrproject.org/latest/develop/application/index.html)
- [West workspaces (T1/T2/T3)](https://docs.zephyrproject.org/latest/develop/west/workspaces.html)
- [Sysbuild](https://docs.zephyrproject.org/latest/build/sysbuild/index.html)
- [Getting Started (west, Zephyr SDK, Python 3.12, CMake 3.28)](https://docs.zephyrproject.org/latest/develop/getting_started/index.html)
- [Bluetooth application development](https://docs.zephyrproject.org/latest/connectivity/bluetooth/bluetooth-dev.html)
- [Board Porting Guide / hardware model v2](https://docs.zephyrproject.org/latest/hardware/porting/board_porting.html)
- [nRF52840 DK](https://docs.zephyrproject.org/latest/boards/nordic/nrf52840dk/doc/index.html)
- [Zephyr releases](https://docs.zephyrproject.org/latest/releases/index.html)
- [NCS adding your own code (Workflow 4)](https://github.com/nrfconnect/sdk-nrf/blob/v3.3.4/doc/nrf/dev_model_and_contributions/adding_code.rst)
- [NCS creating an application](https://github.com/nrfconnect/sdk-nrf/blob/v3.4.0/doc/nrf/app_dev/create_application.rst)
- [NCS v3.4.0 release notes](https://github.com/nrfconnect/sdk-nrf/blob/v3.4.0/doc/nrf/releases_and_maturity/releases/release-notes-3.4.0.rst)
- [ncs-example-application v3.4.0](https://github.com/nrfconnect/ncs-example-application/tree/v3.4.0)
- [example-application](https://github.com/zephyrproject-rtos/example-application)
