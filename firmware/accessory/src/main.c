/*
 * Copyright (c) 2026 IoT Passkey contributors
 * SPDX-License-Identifier: Apache-2.0
 *
 * Accessory adapter: Enrollment Window, LE Secure Connections, Stealth Commitment.
 * Draft SMP Params (0xF0/0xF1) are not implemented on stock Zephyr.
 */

#include "gap_sm.h"

#include <zephyr/bluetooth/addr.h>
#include <zephyr/bluetooth/bluetooth.h>
#include <zephyr/bluetooth/conn.h>
#include <zephyr/bluetooth/gap.h>
#include <zephyr/bluetooth/hci.h>
#include <zephyr/drivers/gpio.h>
#include <zephyr/input/input.h>
#include <zephyr/kernel.h>
#include <zephyr/logging/log.h>
#include <zephyr/settings/settings.h>

LOG_MODULE_REGISTER(sb_accessory, LOG_LEVEL_INF);

#define T_SB_ENROLL_MS 30000
#define LED0_NODE DT_ALIAS(led0)

static const struct gpio_dt_spec status_led = GPIO_DT_SPEC_GET(LED0_NODE, gpios);

static struct gap_sm sm;
static struct k_work_delayable enroll_timeout;
static bt_addr_le_t last_bonded;

static const struct bt_data ad_enroll[] = {
	BT_DATA_BYTES(BT_DATA_FLAGS, (BT_LE_AD_LIMITED | BT_LE_AD_NO_BREDR)),
	BT_DATA(BT_DATA_NAME_COMPLETE, CONFIG_BT_DEVICE_NAME,
		sizeof(CONFIG_BT_DEVICE_NAME) - 1),
};

static void apply_radio(enum gap_state state);

static void count_bond(const struct bt_bond_info *info, void *user_data)
{
	uint8_t *count = user_data;

	ARG_UNUSED(info);
	(*count)++;
}

static uint8_t bond_count(void)
{
	uint8_t count = 0;

	bt_foreach_bond(BT_ID_DEFAULT, count_bond, &count);
	return count;
}

static void remember_bond(const struct bt_bond_info *info, void *user_data)
{
	ARG_UNUSED(user_data);
	bt_addr_le_copy(&last_bonded, &info->addr);
}

static int start_enrollment_adv(void)
{
	int err;

	bt_le_adv_stop();
	err = bt_le_adv_start(BT_LE_ADV_CONN_FAST_2, ad_enroll, ARRAY_SIZE(ad_enroll), NULL, 0);
	if (err) {
		LOG_ERR("Enrollment advertising failed (%d)", err);
	}
	return err;
}

static int start_directed_knock(const bt_addr_le_t *peer)
{
	int err;

	if (bt_addr_le_eq(peer, BT_ADDR_LE_ANY)) {
		return -EINVAL;
	}

	bt_le_adv_stop();
	err = bt_le_adv_start(BT_LE_ADV_CONN_DIR_LOW_DUTY(peer), NULL, 0, NULL, 0);
	if (err) {
		LOG_ERR("Directed Knock failed (%d)", err);
	}
	return err;
}

static void apply_radio(enum gap_state state)
{
	sm.bond_count = bond_count();

	switch (state) {
	case GAP_STATE_UNPROVISIONED:
	case GAP_STATE_ENROLLMENT:
	case GAP_STATE_PUBLIC_BONDED:
		(void)start_enrollment_adv();
		break;
	case GAP_STATE_BONDED_KNOCK:
		bt_foreach_bond(BT_ID_DEFAULT, remember_bond, NULL);
		if (start_directed_knock(&last_bonded) != 0) {
			LOG_WRN("Directed Knock failed; radio silent");
			bt_le_adv_stop();
		}
		break;
	case GAP_STATE_BONDED_SILENT:
		bt_le_adv_stop();
		break;
	}

	gpio_pin_set_dt(&status_led, state == GAP_STATE_ENROLLMENT ? 1 : 0);
	LOG_INF("GAP state %d public_adv=%d bonds=%u", (int)state,
		(int)gap_sm_public_advertising(state), sm.bond_count);
}

static void enroll_timeout_fn(struct k_work *work)
{
	enum gap_state next;

	ARG_UNUSED(work);
	next = gap_sm_apply(&sm, GAP_EVT_ENROLLMENT_TIMEOUT);
	apply_radio(next);
}

static void open_enrollment(void)
{
	enum gap_state next = gap_sm_apply(&sm, GAP_EVT_OPEN_ENROLLMENT);

	if (next != GAP_STATE_ENROLLMENT) {
		LOG_WRN("Enrollment Window refused (Bond Table full)");
		return;
	}

	k_work_reschedule(&enroll_timeout, K_MSEC(T_SB_ENROLL_MS));
	apply_radio(next);
}

static void on_button(struct input_event *evt, void *user_data)
{
	ARG_UNUSED(user_data);

	if (evt->value && evt->type == INPUT_EV_KEY && evt->code == INPUT_KEY_0) {
		LOG_INF("Pairing gesture: open Enrollment Window");
		open_enrollment();
	}
}

INPUT_CALLBACK_DEFINE(NULL, on_button, NULL);

static void pairing_complete(struct bt_conn *conn, bool bonded)
{
	enum gap_state next;
	char addr[BT_ADDR_LE_STR_LEN];
	bt_addr_le_t peer;
	int err;

	bt_addr_le_to_str(bt_conn_get_dst(conn), addr, sizeof(addr));

	if (!bonded) {
		LOG_WRN("Pairing with %s did not bond", addr);
		next = gap_sm_apply(&sm, GAP_EVT_COMMITMENT_FAIL);
		apply_radio(next);
		return;
	}

	bt_addr_le_copy(&peer, bt_conn_get_dst(conn));
	bt_le_adv_stop();
	err = start_directed_knock(&peer);
	if (err) {
		LOG_ERR("Stealth Commitment failed; discarding bond");
		(void)bt_unpair(BT_ID_DEFAULT, &peer);
		next = gap_sm_apply(&sm, GAP_EVT_COMMITMENT_FAIL);
		apply_radio(next);
		return;
	}

	k_work_cancel_delayable(&enroll_timeout);
	bt_addr_le_copy(&last_bonded, &peer);
	next = gap_sm_apply(&sm, GAP_EVT_COMMITMENT_OK);
	LOG_INF("Stealth Commitment ok with %s", addr);
	sm.state = next;
	sm.bond_count = bond_count();
	gpio_pin_set_dt(&status_led, 0);
}

static void pairing_failed(struct bt_conn *conn, enum bt_security_err reason)
{
	ARG_UNUSED(conn);
	LOG_WRN("Pairing failed (%d)", (int)reason);
	apply_radio(gap_sm_apply(&sm, GAP_EVT_COMMITMENT_FAIL));
}

static void bond_deleted(uint8_t id, const bt_addr_le_t *peer)
{
	ARG_UNUSED(id);
	ARG_UNUSED(peer);

	sm.bond_count = bond_count();
	if (sm.bond_count == 0U) {
		apply_radio(gap_sm_apply(&sm, GAP_EVT_LAST_BOND_GONE));
	}
}

static struct bt_conn_auth_info_cb auth_info_cb = {
	.pairing_complete = pairing_complete,
	.pairing_failed = pairing_failed,
	.bond_deleted = bond_deleted,
};

static void connected(struct bt_conn *conn, uint8_t err)
{
	char addr[BT_ADDR_LE_STR_LEN];

	bt_addr_le_to_str(bt_conn_get_dst(conn), addr, sizeof(addr));
	if (err) {
		LOG_WRN("Connect failed %s (%u)", addr, err);
		return;
	}
	LOG_INF("Connected %s", addr);
}

static void disconnected(struct bt_conn *conn, uint8_t reason)
{
	ARG_UNUSED(conn);
	LOG_INF("Disconnected (%u)", reason);
	apply_radio(sm.state);
}

BT_CONN_CB_DEFINE(conn_cb) = {
	.connected = connected,
	.disconnected = disconnected,
};

int main(void)
{
	int err;

	if (!gpio_is_ready_dt(&status_led)) {
		LOG_ERR("LED0 not ready");
		return 0;
	}
	(void)gpio_pin_configure_dt(&status_led, GPIO_OUTPUT_INACTIVE);

	gap_sm_init(&sm, CONFIG_BT_MAX_PAIRED);
	k_work_init_delayable(&enroll_timeout, enroll_timeout_fn);
	bt_addr_le_copy(&last_bonded, BT_ADDR_LE_ANY);

	err = settings_subsys_init();
	if (err) {
		LOG_ERR("settings_subsys_init failed (%d)", err);
		return 0;
	}

	err = bt_enable(NULL);
	if (err) {
		LOG_ERR("bt_enable failed (%d)", err);
		return 0;
	}

	settings_load();
	(void)bt_conn_auth_info_cb_register(&auth_info_cb);

	sm.bond_count = bond_count();
	if (sm.bond_count == 0U) {
		open_enrollment();
	} else {
		sm.state = GAP_STATE_BONDED_KNOCK;
		apply_radio(sm.state);
	}

	LOG_INF("Stealth Bonding Accessory");
	return 0;
}
