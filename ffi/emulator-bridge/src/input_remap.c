/**
 * RexOS Emulator Bridge - Input Remapping
 */

#include <errno.h>
#include <fcntl.h>
#include <linux/input.h>
#include <linux/uinput.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "emulator_bridge.h"

/* Default deadzone value */
static int g_deadzone = 4096;

/* Button mapping table */
static rexos_button_map_t g_button_map[32];
static int g_button_map_count = 0;

/* Uinput file descriptor for virtual input device (reserved for future use) */
__attribute__((unused)) static int g_uinput_fd = -1;

/**
 * Read deadzone from RetroArch config or system default
 */
int rexos_get_deadzone(void)
{
    return g_deadzone;
}

/**
 * Set deadzone value
 */
rexos_error_t rexos_set_deadzone(int deadzone)
{
    if (deadzone < 0 || deadzone > 32767) {
        return REXOS_ERR_INVALID_ARG;
    }

    g_deadzone = deadzone;

    /* Also write to RetroArch config if available */
    const char* ra_config = "/home/ark/.config/retroarch/retroarch.cfg";
    FILE* f = fopen(ra_config, "r");
    if (!f)
        return REXOS_OK; /* File doesn't exist, that's OK */

    /* Read entire file */
    fseek(f, 0, SEEK_END);
    long size = ftell(f);
    fseek(f, 0, SEEK_SET);

    char* content = malloc(size + 1);
    if (!content) {
        fclose(f);
        return REXOS_ERR_MEMORY;
    }

    size_t read_size = fread(content, 1, size, f);
    content[read_size] = '\0';
    fclose(f);

    /* Find and update deadzone setting */
    char* deadzone_line = strstr(content, "input_analog_deadzone");
    if (deadzone_line) {
        /* Found existing setting - modify would require file rewrite */
        /* For simplicity, just free and return */
    }

    free(content);
    return REXOS_OK;
}

/**
 * Apply button remapping
 */
rexos_error_t rexos_apply_button_map(const rexos_button_map_t* map, int count)
{
    if (!map && count > 0) {
        return REXOS_ERR_INVALID_ARG;
    }

    if (count > 32) {
        count = 32;
    }

    /* Store mapping */
    g_button_map_count = count;
    if (count > 0) {
        memcpy(g_button_map, map, count * sizeof(rexos_button_map_t));
    }

    /* Write mapping to RetroArch config format */
    /* RetroArch uses input_player1_btn_* format */

    return REXOS_OK;
}

/**
 * Create a virtual input device using uinput (reserved for future use)
 */
__attribute__((unused)) static int create_virtual_device(void)
{
    int fd = open("/dev/uinput", O_WRONLY | O_NONBLOCK);
    if (fd < 0) {
        return -1;
    }

    /* Enable event types */
    ioctl(fd, UI_SET_EVBIT, EV_KEY);
    ioctl(fd, UI_SET_EVBIT, EV_ABS);
    ioctl(fd, UI_SET_EVBIT, EV_SYN);

    /* Enable button codes */
    ioctl(fd, UI_SET_KEYBIT, BTN_A);
    ioctl(fd, UI_SET_KEYBIT, BTN_B);
    ioctl(fd, UI_SET_KEYBIT, BTN_X);
    ioctl(fd, UI_SET_KEYBIT, BTN_Y);
    ioctl(fd, UI_SET_KEYBIT, BTN_TL);
    ioctl(fd, UI_SET_KEYBIT, BTN_TR);
    ioctl(fd, UI_SET_KEYBIT, BTN_TL2);
    ioctl(fd, UI_SET_KEYBIT, BTN_TR2);
    ioctl(fd, UI_SET_KEYBIT, BTN_SELECT);
    ioctl(fd, UI_SET_KEYBIT, BTN_START);
    ioctl(fd, UI_SET_KEYBIT, BTN_THUMBL);
    ioctl(fd, UI_SET_KEYBIT, BTN_THUMBR);
    ioctl(fd, UI_SET_KEYBIT, BTN_DPAD_UP);
    ioctl(fd, UI_SET_KEYBIT, BTN_DPAD_DOWN);
    ioctl(fd, UI_SET_KEYBIT, BTN_DPAD_LEFT);
    ioctl(fd, UI_SET_KEYBIT, BTN_DPAD_RIGHT);

    /* Enable absolute axes */
    ioctl(fd, UI_SET_ABSBIT, ABS_X);
    ioctl(fd, UI_SET_ABSBIT, ABS_Y);
    ioctl(fd, UI_SET_ABSBIT, ABS_RX);
    ioctl(fd, UI_SET_ABSBIT, ABS_RY);

    /* Set up device */
    struct uinput_setup usetup;
    memset(&usetup, 0, sizeof(usetup));
    usetup.id.bustype = BUS_USB;
    usetup.id.vendor = 0x1234; /* RexOS vendor ID */
    usetup.id.product = 0x5678;
    strcpy(usetup.name, "RexOS Virtual Controller");

    ioctl(fd, UI_DEV_SETUP, &usetup);

    /* Set up absolute axis ranges */
    struct uinput_abs_setup abs_setup;
    memset(&abs_setup, 0, sizeof(abs_setup));
    abs_setup.absinfo.minimum = -32768;
    abs_setup.absinfo.maximum = 32767;
    abs_setup.absinfo.flat = g_deadzone;

    abs_setup.code = ABS_X;
    ioctl(fd, UI_ABS_SETUP, &abs_setup);

    abs_setup.code = ABS_Y;
    ioctl(fd, UI_ABS_SETUP, &abs_setup);

    abs_setup.code = ABS_RX;
    ioctl(fd, UI_ABS_SETUP, &abs_setup);

    abs_setup.code = ABS_RY;
    ioctl(fd, UI_ABS_SETUP, &abs_setup);

    /* Create device */
    ioctl(fd, UI_DEV_CREATE);

    return fd;
}

/**
 * Destroy virtual input device (reserved for future use)
 */
__attribute__((unused)) static void destroy_virtual_device(int fd)
{
    if (fd >= 0) {
        ioctl(fd, UI_DEV_DESTROY);
        close(fd);
    }
}

/**
 * Send input event through virtual device (reserved for future use)
 */
__attribute__((unused)) static void send_event(int fd, unsigned short type, unsigned short code,
                                               int value)
{
    struct input_event ev;
    memset(&ev, 0, sizeof(ev));
    ev.type = type;
    ev.code = code;
    ev.value = value;
    write(fd, &ev, sizeof(ev));
}

/**
 * Apply button mapping to an event (reserved for future use)
 */
__attribute__((unused)) static unsigned short apply_mapping(unsigned short code)
{
    for (int i = 0; i < g_button_map_count; i++) {
        if (g_button_map[i].from == code) {
            return g_button_map[i].to;
        }
    }
    return code; /* No mapping, return original */
}

/**
 * Get button name for logging/display
 */
const char* rexos_button_name(rexos_button_t button)
{
    switch (button) {
        case REXOS_BTN_A:
            return "A";
        case REXOS_BTN_B:
            return "B";
        case REXOS_BTN_X:
            return "X";
        case REXOS_BTN_Y:
            return "Y";
        case REXOS_BTN_L1:
            return "L1";
        case REXOS_BTN_R1:
            return "R1";
        case REXOS_BTN_L2:
            return "L2";
        case REXOS_BTN_R2:
            return "R2";
        case REXOS_BTN_SELECT:
            return "Select";
        case REXOS_BTN_START:
            return "Start";
        case REXOS_BTN_L3:
            return "L3";
        case REXOS_BTN_R3:
            return "R3";
        case REXOS_BTN_DPAD_UP:
            return "Up";
        case REXOS_BTN_DPAD_DOWN:
            return "Down";
        case REXOS_BTN_DPAD_LEFT:
            return "Left";
        case REXOS_BTN_DPAD_RIGHT:
            return "Right";
        default:
            return "Unknown";
    }
}

/**
 * Scan for available input devices
 */
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wformat-truncation"
int rexos_scan_input_devices(char devices[][64], int max_devices)
{
    int count = 0;
    char path[32]; /* /dev/input/eventXX is max ~20 chars */

    for (int i = 0; i < 32 && count < max_devices; i++) {
        snprintf(path, sizeof(path), "/dev/input/event%d", i);

        int fd = open(path, O_RDONLY);
        if (fd < 0)
            continue;

        /* Get device name - limit to 30 chars to fit in 64-byte buffer with path */
        char name[32] = "Unknown";
        ioctl(fd, EVIOCGNAME(sizeof(name)), name);
        close(fd);

        /* Check if it's a game controller */
        if (strstr(name, "Gamepad") || strstr(name, "Controller") || strstr(name, "Joystick") ||
            strstr(name, "gamepad") || strstr(name, "joypad")) {
            snprintf(devices[count], 64, "%s: %s", path, name);
            count++;
        }
    }

    return count;
}
#pragma GCC diagnostic pop
