/*
 * Copyright (c) 2026 IoT Passkey contributors
 * SPDX-License-Identifier: Apache-2.0
 */

#include "gap_sm.h"

void gap_sm_init(struct gap_sm *sm, uint8_t bond_capacity)
{
	sm->state = GAP_STATE_UNPROVISIONED;
	sm->bond_count = 0;
	sm->bond_capacity = bond_capacity < 2U ? 2U : bond_capacity;
}

enum gap_state gap_sm_apply(struct gap_sm *sm, enum gap_event event)
{
	switch (event) {
	case GAP_EVT_OPEN_ENROLLMENT:
		if (sm->bond_count < sm->bond_capacity) {
			sm->state = GAP_STATE_ENROLLMENT;
		}
		break;
	case GAP_EVT_ENROLLMENT_TIMEOUT:
		if (sm->state == GAP_STATE_ENROLLMENT) {
			sm->state = sm->bond_count == 0U ? GAP_STATE_UNPROVISIONED
							: GAP_STATE_BONDED_KNOCK;
		}
		break;
	case GAP_EVT_COMMITMENT_OK:
		if (sm->state == GAP_STATE_ENROLLMENT && sm->bond_count < sm->bond_capacity) {
			sm->bond_count++;
			sm->state = GAP_STATE_BONDED_KNOCK;
		}
		break;
	case GAP_EVT_COMMITMENT_FAIL:
		if (sm->state == GAP_STATE_ENROLLMENT) {
			sm->state = sm->bond_count == 0U ? GAP_STATE_UNPROVISIONED
							: GAP_STATE_BONDED_KNOCK;
		}
		break;
	case GAP_EVT_LAST_BOND_GONE:
		sm->bond_count = 0;
		sm->state = GAP_STATE_UNPROVISIONED;
		break;
	case GAP_EVT_FACTORY_RESET:
		sm->bond_count = 0;
		sm->state = GAP_STATE_UNPROVISIONED;
		break;
	}

	return sm->state;
}

bool gap_sm_public_advertising(enum gap_state state)
{
	switch (state) {
	case GAP_STATE_UNPROVISIONED:
	case GAP_STATE_ENROLLMENT:
	case GAP_STATE_PUBLIC_BONDED:
		return true;
	case GAP_STATE_BONDED_KNOCK:
	case GAP_STATE_BONDED_SILENT:
		return false;
	default:
		return true;
	}
}
