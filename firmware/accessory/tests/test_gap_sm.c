/*
 * Copyright (c) 2026 IoT Passkey contributors
 * SPDX-License-Identifier: Apache-2.0
 *
 * Host-side checks for gap_sm (no Zephyr). From firmware/accessory:
 *   gcc -I src tests/test_gap_sm.c src/gap_sm.c -o /tmp/test_gap_sm && /tmp/test_gap_sm
 */

#include "../src/gap_sm.h"

#include <stdio.h>
#include <stdlib.h>

#define EXPECT(cond)                                                                           \
	do {                                                                                   \
		if (!(cond)) {                                                                 \
			fprintf(stderr, "fail %s:%d: %s\n", __FILE__, __LINE__, #cond);        \
			exit(1);                                                               \
		}                                                                              \
	} while (0)

int main(void)
{
	struct gap_sm sm;

	gap_sm_init(&sm, 8);
	EXPECT(sm.state == GAP_STATE_UNPROVISIONED);
	EXPECT(gap_sm_public_advertising(sm.state));

	EXPECT(gap_sm_apply(&sm, GAP_EVT_OPEN_ENROLLMENT) == GAP_STATE_ENROLLMENT);
	EXPECT(gap_sm_public_advertising(sm.state));

	EXPECT(gap_sm_apply(&sm, GAP_EVT_COMMITMENT_OK) == GAP_STATE_BONDED_KNOCK);
	EXPECT(sm.bond_count == 1);
	EXPECT(!gap_sm_public_advertising(sm.state));

	EXPECT(gap_sm_apply(&sm, GAP_EVT_OPEN_ENROLLMENT) == GAP_STATE_ENROLLMENT);
	EXPECT(gap_sm_apply(&sm, GAP_EVT_ENROLLMENT_TIMEOUT) == GAP_STATE_BONDED_KNOCK);
	EXPECT(!gap_sm_public_advertising(sm.state));

	EXPECT(gap_sm_apply(&sm, GAP_EVT_FACTORY_RESET) == GAP_STATE_UNPROVISIONED);
	EXPECT(sm.bond_count == 0);
	EXPECT(gap_sm_public_advertising(sm.state));

	gap_sm_init(&sm, 2);
	gap_sm_apply(&sm, GAP_EVT_OPEN_ENROLLMENT);
	gap_sm_apply(&sm, GAP_EVT_COMMITMENT_OK);
	gap_sm_apply(&sm, GAP_EVT_OPEN_ENROLLMENT);
	gap_sm_apply(&sm, GAP_EVT_COMMITMENT_OK);
	EXPECT(sm.bond_count == 2);
	EXPECT(gap_sm_apply(&sm, GAP_EVT_OPEN_ENROLLMENT) == GAP_STATE_BONDED_KNOCK);

	puts("ok");
	return 0;
}
