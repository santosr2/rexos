/**
 * RexOS Emulator Bridge - Audio Bridge
 */

#include "emulator_bridge.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <fcntl.h>
#include <unistd.h>

/* ALSA mixer control paths - adjust for your device */
#define MIXER_CONTROL "Master"
#define HEADPHONE_DETECT_PATH "/sys/class/switch/h2w/state"

/**
 * Run amixer command and parse output
 */
static int run_amixer(const char* args, char* output, size_t output_len)
{
    char cmd[256];
    snprintf(cmd, sizeof(cmd), "amixer %s 2>/dev/null", args);

    FILE* fp = popen(cmd, "r");
    if (!fp) return -1;

    if (output && output_len > 0) {
        size_t total = 0;
        char buf[256];
        while (fgets(buf, sizeof(buf), fp) && total < output_len - 1) {
            size_t len = strlen(buf);
            if (total + len >= output_len) break;
            memcpy(output + total, buf, len);
            total += len;
        }
        output[total] = '\0';
    }

    return pclose(fp);
}

/**
 * Parse volume percentage from amixer output
 */
static int parse_volume(const char* output)
{
    /* Look for [XX%] pattern */
    const char* p = strstr(output, "[");
    if (!p) return -1;

    int volume;
    if (sscanf(p, "[%d%%]", &volume) != 1) {
        return -1;
    }

    return volume;
}

int rexos_get_volume(void)
{
    char output[1024];

    if (run_amixer("sget " MIXER_CONTROL, output, sizeof(output)) != 0) {
        /* Try alternative control names */
        if (run_amixer("sget Playback", output, sizeof(output)) != 0) {
            if (run_amixer("sget PCM", output, sizeof(output)) != 0) {
                return -1;
            }
        }
    }

    return parse_volume(output);
}

rexos_error_t rexos_set_volume(int volume)
{
    if (volume < 0) volume = 0;
    if (volume > 100) volume = 100;

    char args[64];
    snprintf(args, sizeof(args), "sset " MIXER_CONTROL " %d%%", volume);

    if (run_amixer(args, NULL, 0) != 0) {
        /* Try alternative control names */
        snprintf(args, sizeof(args), "sset Playback %d%%", volume);
        if (run_amixer(args, NULL, 0) != 0) {
            snprintf(args, sizeof(args), "sset PCM %d%%", volume);
            if (run_amixer(args, NULL, 0) != 0) {
                return REXOS_ERR_IO;
            }
        }
    }

    return REXOS_OK;
}

bool rexos_headphones_connected(void)
{
    /* Try switch subsystem first (common on Android-derived systems) */
    FILE* f = fopen(HEADPHONE_DETECT_PATH, "r");
    if (f) {
        int state;
        if (fscanf(f, "%d", &state) == 1) {
            fclose(f);
            return state != 0;
        }
        fclose(f);
    }

    /* Try extcon subsystem */
    f = fopen("/sys/class/extcon/extcon0/state", "r");
    if (f) {
        char line[256];
        while (fgets(line, sizeof(line), f)) {
            if (strstr(line, "HEADPHONE=1") || strstr(line, "JACK=1")) {
                fclose(f);
                return true;
            }
        }
        fclose(f);
    }

    /* Try gpio-based detection (device specific) */
    const char* gpio_paths[] = {
        "/sys/class/gpio/gpio12/value",  /* Common on RK3566 */
        "/sys/class/gpio/gpio84/value",
        NULL
    };

    for (int i = 0; gpio_paths[i]; i++) {
        f = fopen(gpio_paths[i], "r");
        if (f) {
            int val;
            if (fscanf(f, "%d", &val) == 1) {
                fclose(f);
                /* GPIO logic level depends on hardware */
                return val == 0;  /* Usually active low */
            }
            fclose(f);
        }
    }

    /* Check ALSA jack state */
    char output[1024];
    if (run_amixer("contents", output, sizeof(output)) == 0) {
        if (strstr(output, "Jack=on") || strstr(output, "Headphone=on")) {
            return true;
        }
    }

    return false;
}

/**
 * Mute/unmute audio
 */
rexos_error_t rexos_set_mute(bool mute)
{
    const char* state = mute ? "off" : "on";
    char args[64];
    snprintf(args, sizeof(args), "sset " MIXER_CONTROL " %s", state);

    if (run_amixer(args, NULL, 0) != 0) {
        return REXOS_ERR_IO;
    }

    return REXOS_OK;
}

/**
 * Check if audio is muted
 */
bool rexos_is_muted(void)
{
    char output[1024];

    if (run_amixer("sget " MIXER_CONTROL, output, sizeof(output)) != 0) {
        return false;
    }

    return strstr(output, "[off]") != NULL;
}

/**
 * Set audio output device
 */
rexos_error_t rexos_set_audio_output(const char* device)
{
    if (!device) {
        return REXOS_ERR_INVALID_ARG;
    }

    /* This is highly device-specific */
    /* For RK3566 devices with codec + HDMI: */

    if (strcmp(device, "speaker") == 0) {
        /* Route to internal speakers */
        run_amixer("sset 'Playback Path' 'SPK'", NULL, 0);
    } else if (strcmp(device, "headphones") == 0) {
        /* Route to headphone jack */
        run_amixer("sset 'Playback Path' 'HP'", NULL, 0);
    } else if (strcmp(device, "hdmi") == 0) {
        /* Route to HDMI */
        run_amixer("sset 'Playback Path' 'HDMI'", NULL, 0);
    } else {
        return REXOS_ERR_INVALID_ARG;
    }

    return REXOS_OK;
}
