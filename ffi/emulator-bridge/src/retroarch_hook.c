/**
 * RexOS Emulator Bridge - RetroArch Hooks
 */

#include "emulator_bridge.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <fcntl.h>
#include <unistd.h>
#include <linux/input.h>
#include <pthread.h>
#include <time.h>

/* Hotkey configuration */
#define HOTKEY_MODIFIER_MASK 0x01  /* Select button */
#define HOTKEY_TIMEOUT_MS 500

/* Hotkey state */
static pthread_mutex_t g_hotkey_mutex = PTHREAD_MUTEX_INITIALIZER;
static rexos_hotkey_callback_t g_hotkey_callback = NULL;
static void* g_hotkey_user_data = NULL;

/* Button state for hotkey detection */
static struct {
    bool modifier_pressed;
    uint64_t modifier_time;
    bool buttons[32];
} g_input_state = {0};

/* Input device file descriptor */
static int g_input_fd = -1;

/* Display brightness control */
#define BRIGHTNESS_PATH "/sys/class/backlight/backlight/brightness"
#define BRIGHTNESS_MAX_PATH "/sys/class/backlight/backlight/max_brightness"

/**
 * Get current time in milliseconds
 */
static uint64_t get_time_ms(void)
{
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint64_t)ts.tv_sec * 1000 + ts.tv_nsec / 1000000;
}

/**
 * Read integer from sysfs
 */
static int read_sysfs_int(const char* path)
{
    FILE* f = fopen(path, "r");
    if (!f) return -1;

    int value;
    if (fscanf(f, "%d", &value) != 1) {
        fclose(f);
        return -1;
    }

    fclose(f);
    return value;
}

/**
 * Write integer to sysfs
 */
static int write_sysfs_int(const char* path, int value)
{
    FILE* f = fopen(path, "w");
    if (!f) return -1;

    fprintf(f, "%d", value);
    fclose(f);
    return 0;
}

int rexos_get_brightness(void)
{
    return read_sysfs_int(BRIGHTNESS_PATH);
}

rexos_error_t rexos_set_brightness(int brightness)
{
    int max_brightness = read_sysfs_int(BRIGHTNESS_MAX_PATH);
    if (max_brightness < 0) max_brightness = 255;

    if (brightness < 0) brightness = 0;
    if (brightness > max_brightness) brightness = max_brightness;

    if (write_sysfs_int(BRIGHTNESS_PATH, brightness) != 0) {
        return REXOS_ERR_PERMISSION;
    }

    return REXOS_OK;
}

rexos_error_t rexos_register_hotkey_callback(rexos_hotkey_callback_t callback,
                                              void* user_data)
{
    pthread_mutex_lock(&g_hotkey_mutex);
    g_hotkey_callback = callback;
    g_hotkey_user_data = user_data;
    pthread_mutex_unlock(&g_hotkey_mutex);

    return REXOS_OK;
}

/**
 * Trigger hotkey callback
 */
static void trigger_hotkey(rexos_hotkey_action_t action)
{
    pthread_mutex_lock(&g_hotkey_mutex);

    if (g_hotkey_callback) {
        g_hotkey_callback((int)action, g_hotkey_user_data);
    }

    pthread_mutex_unlock(&g_hotkey_mutex);
}

/**
 * Check if a specific hotkey is pressed (Select + button)
 */
bool rexos_check_hotkey(rexos_hotkey_action_t action)
{
    if (!g_input_state.modifier_pressed) {
        return false;
    }

    /* Check for timeout */
    uint64_t now = get_time_ms();
    if (now - g_input_state.modifier_time > HOTKEY_TIMEOUT_MS) {
        return false;
    }

    /* Map action to button */
    int button_index = -1;

    switch (action) {
        case REXOS_HOTKEY_EXIT:
            button_index = BTN_START & 0x1F;
            break;
        case REXOS_HOTKEY_SAVE_STATE:
            button_index = BTN_TR & 0x1F;  /* R1 */
            break;
        case REXOS_HOTKEY_LOAD_STATE:
            button_index = BTN_TL & 0x1F;  /* L1 */
            break;
        case REXOS_HOTKEY_SCREENSHOT:
            button_index = BTN_TL2 & 0x1F;  /* L2 */
            break;
        case REXOS_HOTKEY_FAST_FORWARD:
            button_index = BTN_TR2 & 0x1F;  /* R2 */
            break;
        case REXOS_HOTKEY_MENU:
            button_index = BTN_X & 0x1F;
            break;
        case REXOS_HOTKEY_PAUSE:
            button_index = BTN_Y & 0x1F;
            break;
        default:
            return false;
    }

    if (button_index >= 0 && button_index < 32) {
        return g_input_state.buttons[button_index];
    }

    return false;
}

/**
 * Handle input event for hotkey detection
 */
static void handle_input_event(const struct input_event* ev)
{
    if (ev->type != EV_KEY) return;

    bool pressed = (ev->value != 0);
    int button_index = ev->code & 0x1F;

    /* Track modifier (Select button) */
    if (ev->code == BTN_SELECT) {
        g_input_state.modifier_pressed = pressed;
        if (pressed) {
            g_input_state.modifier_time = get_time_ms();
        }
        return;
    }

    /* Track other buttons */
    if (button_index < 32) {
        g_input_state.buttons[button_index] = pressed;
    }

    /* Check for hotkey combinations when button is pressed */
    if (pressed && g_input_state.modifier_pressed) {
        rexos_hotkey_action_t action = 0;

        switch (ev->code) {
            case BTN_START:
                action = REXOS_HOTKEY_EXIT;
                break;
            case BTN_TR:  /* R1 */
                action = REXOS_HOTKEY_SAVE_STATE;
                break;
            case BTN_TL:  /* L1 */
                action = REXOS_HOTKEY_LOAD_STATE;
                break;
            case BTN_TL2:  /* L2 */
                action = REXOS_HOTKEY_SCREENSHOT;
                break;
            case BTN_TR2:  /* R2 */
                action = REXOS_HOTKEY_FAST_FORWARD;
                break;
            case BTN_X:
                action = REXOS_HOTKEY_MENU;
                break;
            case BTN_Y:
                action = REXOS_HOTKEY_PAUSE;
                break;
            case BTN_DPAD_UP:
                action = REXOS_HOTKEY_VOLUME_UP;
                break;
            case BTN_DPAD_DOWN:
                action = REXOS_HOTKEY_VOLUME_DOWN;
                break;
            case BTN_DPAD_RIGHT:
                action = REXOS_HOTKEY_BRIGHTNESS_UP;
                break;
            case BTN_DPAD_LEFT:
                action = REXOS_HOTKEY_BRIGHTNESS_DOWN;
                break;
        }

        if (action != 0) {
            trigger_hotkey(action);
        }
    }
}

/**
 * Open input device for hotkey monitoring
 */
rexos_error_t rexos_open_input_device(const char* device_path)
{
    if (g_input_fd >= 0) {
        close(g_input_fd);
    }

    g_input_fd = open(device_path, O_RDONLY | O_NONBLOCK);
    if (g_input_fd < 0) {
        return REXOS_ERR_NOT_FOUND;
    }

    return REXOS_OK;
}

/**
 * Poll input device for hotkeys
 */
rexos_error_t rexos_poll_hotkeys(void)
{
    if (g_input_fd < 0) {
        return REXOS_ERR_IO;
    }

    struct input_event ev;

    while (read(g_input_fd, &ev, sizeof(ev)) == sizeof(ev)) {
        handle_input_event(&ev);
    }

    return REXOS_OK;
}

/**
 * Close input device
 */
void rexos_close_input_device(void)
{
    if (g_input_fd >= 0) {
        close(g_input_fd);
        g_input_fd = -1;
    }
}

/**
 * Generate RetroArch config snippet for hotkeys
 */
int rexos_generate_hotkey_config(char* buffer, size_t len)
{
    const char* config =
        "# RexOS Hotkey Configuration\n"
        "input_enable_hotkey_btn = 6\n"      /* Select */
        "input_exit_emulator_btn = 7\n"      /* Start */
        "input_save_state_btn = 5\n"         /* R1 */
        "input_load_state_btn = 4\n"         /* L1 */
        "input_screenshot_btn = 10\n"        /* L2 */
        "input_hold_fast_forward_btn = 11\n" /* R2 */
        "input_menu_toggle_btn = 3\n"        /* X */
        "input_pause_toggle_btn = 2\n"       /* Y */
        "input_state_slot_increase_btn = h0right\n"
        "input_state_slot_decrease_btn = h0left\n"
        "input_volume_up_btn = h0up\n"
        "input_volume_down_btn = h0down\n";

    size_t config_len = strlen(config);
    if (len <= config_len) {
        return -1;
    }

    strcpy(buffer, config);
    return config_len;
}

/**
 * Handle brightness hotkey
 */
void rexos_handle_brightness_hotkey(bool increase)
{
    int current = rexos_get_brightness();
    if (current < 0) return;

    int max_brightness = read_sysfs_int(BRIGHTNESS_MAX_PATH);
    if (max_brightness < 0) max_brightness = 255;

    int step = max_brightness / 10;  /* 10 steps */
    if (step < 1) step = 1;

    int new_brightness;
    if (increase) {
        new_brightness = current + step;
        if (new_brightness > max_brightness) {
            new_brightness = max_brightness;
        }
    } else {
        new_brightness = current - step;
        if (new_brightness < 0) {
            new_brightness = 0;
        }
    }

    rexos_set_brightness(new_brightness);
}

/**
 * Handle volume hotkey
 */
void rexos_handle_volume_hotkey(bool increase)
{
    int current = rexos_get_volume();
    if (current < 0) return;

    int step = 10;  /* 10% steps */

    int new_volume;
    if (increase) {
        new_volume = current + step;
        if (new_volume > 100) new_volume = 100;
    } else {
        new_volume = current - step;
        if (new_volume < 0) new_volume = 0;
    }

    rexos_set_volume(new_volume);
}
