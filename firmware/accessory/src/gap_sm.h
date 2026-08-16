/*
 * Copyright (c) 2026 IoT Passkey contributors
 * SPDX-License-Identifier: Apache-2.0
 *
 * GAP-side Accessory states. Same names as docs/spec/stealth-bonding.md §7.
 * Custom SMP Params PDUs are not handled here.
 */

#ifndef GAP_SM_H_
#define GAP_SM_H_

#include <stdbool.h>
#include <stdint.h>

enum gap_state {
	GAP_STATE_UNPROVISIONED = 0,
	GAP_STATE_ENROLLMENT,
	GAP_STATE_BONDED_KNOCK,
	GAP_STATE_BONDED_SILENT,
	GAP_STATE_PUBLIC_BONDED,
};

enum gap_event {
	GAP_EVT_OPEN_ENROLLMENT = 0,
	GAP_EVT_ENROLLMENT_TIMEOUT,
	GAP_EVT_COMMITMENT_OK,
	GAP_EVT_COMMITMENT_FAIL,
	GAP_EVT_LAST_BOND_GONE,
	GAP_EVT_FACTORY_RESET,
};

struct gap_sm {
	enum gap_state state;
	uint8_t bond_count;
	uint8_t bond_capacity;
};

void gap_sm_init(struct gap_sm *sm, uint8_t bond_capacity);

enum gap_state gap_sm_apply(struct gap_sm *sm, enum gap_event event);

bool gap_sm_public_advertising(enum gap_state state);

#endif /* GAP_SM_H_ */
