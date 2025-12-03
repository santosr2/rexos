/**
 * RexOS Emulator Bridge
 *
 * Low-level C interface for emulator integration, performance monitoring,
 * and input handling. Used by Rust services for emulator orchestration.
 */

#ifndef REXOS_EMULATOR_BRIDGE_H
#define REXOS_EMULATOR_BRIDGE_H

#include <stdbool.h>
#include <stdint.h>
#include <sys/types.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ============================================================================
 * Version and Constants
 * ========================================================================== */

#define REXOS_BRIDGE_VERSION_MAJOR 0
#define REXOS_BRIDGE_VERSION_MINOR 1
#define REXOS_BRIDGE_VERSION_PATCH 0

#define REXOS_MAX_PATH 4096
#define REXOS_MAX_ARGS 64
#define REXOS_MAX_ENV  128

/* ============================================================================
 * Error Codes
 * ========================================================================== */

typedef enum {
    REXOS_OK = 0,
    REXOS_ERR_INVALID_ARG = -1,
    REXOS_ERR_NOT_FOUND = -2,
    REXOS_ERR_PERMISSION = -3,
    REXOS_ERR_FORK_FAILED = -4,
    REXOS_ERR_EXEC_FAILED = -5,
    REXOS_ERR_TIMEOUT = -6,
    REXOS_ERR_MEMORY = -7,
    REXOS_ERR_IO = -8,
    REXOS_ERR_INTERNAL = -100
} rexos_error_t;

/* ============================================================================
 * Emulator Types
 * ========================================================================== */

/**
 * Emulator type enumeration
 */
typedef enum {
    REXOS_EMU_RETROARCH,  /* RetroArch with libretro core */
    REXOS_EMU_STANDALONE, /* Standalone emulator */
    REXOS_EMU_PPSSPP,     /* PPSSPP for PSP */
    REXOS_EMU_DRASTIC,    /* DraStic for DS */
    REXOS_EMU_CUSTOM      /* Custom emulator */
} rexos_emulator_type_t;

/**
 * Process state
 */
typedef enum {
    REXOS_PROC_UNKNOWN,
    REXOS_PROC_RUNNING,
    REXOS_PROC_SLEEPING,
    REXOS_PROC_STOPPED,
    REXOS_PROC_ZOMBIE,
    REXOS_PROC_DEAD
} rexos_proc_state_t;

/* ============================================================================
 * Launch Configuration
 * ========================================================================== */

/**
 * Environment variable
 */
typedef struct {
    char key[256];
    char value[1024];
} rexos_env_var_t;

/**
 * Launch configuration structure
 */
typedef struct {
    /* Emulator type */
    rexos_emulator_type_t type;

    /* Executable path */
    char executable[REXOS_MAX_PATH];

    /* ROM/game path */
    char rom_path[REXOS_MAX_PATH];

    /* Core path (for RetroArch) */
    char core_path[REXOS_MAX_PATH];

    /* Configuration file path */
    char config_path[REXOS_MAX_PATH];

    /* Additional arguments */
    char* args[REXOS_MAX_ARGS];
    int arg_count;

    /* Environment variables */
    rexos_env_var_t env[REXOS_MAX_ENV];
    int env_count;

    /* Options */
    bool fullscreen;
    bool verbose;
    bool use_32bit;
    int load_state_slot; /* -1 = don't load */

    /* Performance options */
    int cpu_affinity; /* -1 = no affinity */
    int nice_value;   /* Process priority */
    bool realtime_priority;

} rexos_launch_config_t;

/**
 * Initialize launch config with defaults
 */
void rexos_launch_config_init(rexos_launch_config_t* config);

/**
 * Add an argument to launch config
 */
int rexos_launch_config_add_arg(rexos_launch_config_t* config, const char* arg);

/**
 * Add an environment variable
 */
int rexos_launch_config_add_env(rexos_launch_config_t* config, const char* key, const char* value);

/* ============================================================================
 * Process Management
 * ========================================================================== */

/**
 * Process information
 */
typedef struct {
    pid_t pid;
    rexos_proc_state_t state;
    int exit_code;
    uint64_t start_time;  /* Timestamp in milliseconds */
    uint64_t cpu_time_ms; /* CPU time used */
    uint64_t memory_kb;   /* Memory usage in KB */
} rexos_process_info_t;

/**
 * Launch an emulator process
 *
 * @param config Launch configuration
 * @param pid Output: Process ID
 * @return REXOS_OK on success, error code otherwise
 */
rexos_error_t rexos_launch(const rexos_launch_config_t* config, pid_t* pid);

/**
 * Wait for process to exit
 *
 * @param pid Process ID
 * @param timeout_ms Timeout in milliseconds (-1 = infinite)
 * @param exit_code Output: Exit code
 * @return REXOS_OK on success, REXOS_ERR_TIMEOUT on timeout
 */
rexos_error_t rexos_wait(pid_t pid, int timeout_ms, int* exit_code);

/**
 * Get process information
 *
 * @param pid Process ID
 * @param info Output: Process information
 * @return REXOS_OK on success
 */
rexos_error_t rexos_get_process_info(pid_t pid, rexos_process_info_t* info);

/**
 * Send signal to process
 *
 * @param pid Process ID
 * @param sig Signal number
 * @return REXOS_OK on success
 */
rexos_error_t rexos_signal(pid_t pid, int sig);

/**
 * Gracefully stop emulator (SIGTERM)
 */
rexos_error_t rexos_stop(pid_t pid);

/**
 * Force kill emulator (SIGKILL)
 */
rexos_error_t rexos_kill(pid_t pid);

/* ============================================================================
 * Performance Monitoring
 * ========================================================================== */

/**
 * Performance statistics
 */
typedef struct {
    /* CPU */
    float cpu_usage_percent;
    int cpu_temperature;    /* Celsius */
    uint32_t cpu_frequency; /* MHz */

    /* Memory */
    uint64_t mem_total_kb;
    uint64_t mem_used_kb;
    uint64_t mem_free_kb;

    /* Battery */
    int battery_percent;
    bool battery_charging;
    int battery_temp;

    /* GPU (if available) */
    float gpu_usage_percent;
    int gpu_temperature;

    /* Frame timing */
    float fps;
    float frame_time_ms;

} rexos_perf_stats_t;

/**
 * Get current performance statistics
 */
rexos_error_t rexos_get_perf_stats(rexos_perf_stats_t* stats);

/**
 * Set CPU governor
 *
 * @param governor Governor name (powersave, schedutil, performance)
 */
rexos_error_t rexos_set_cpu_governor(const char* governor);

/**
 * Set CPU frequency limits
 *
 * @param min_freq Minimum frequency in kHz (0 = no limit)
 * @param max_freq Maximum frequency in kHz (0 = no limit)
 */
rexos_error_t rexos_set_cpu_freq(uint32_t min_freq, uint32_t max_freq);

/* ============================================================================
 * Input Remapping
 * ========================================================================== */

/**
 * Button codes (matching Linux input event codes)
 */
typedef enum {
    REXOS_BTN_A = 0x130,
    REXOS_BTN_B = 0x131,
    REXOS_BTN_X = 0x133,
    REXOS_BTN_Y = 0x134,
    REXOS_BTN_L1 = 0x136,
    REXOS_BTN_R1 = 0x137,
    REXOS_BTN_L2 = 0x138,
    REXOS_BTN_R2 = 0x139,
    REXOS_BTN_SELECT = 0x13A,
    REXOS_BTN_START = 0x13B,
    REXOS_BTN_L3 = 0x13D,
    REXOS_BTN_R3 = 0x13E,
    REXOS_BTN_DPAD_UP = 0x220,
    REXOS_BTN_DPAD_DOWN = 0x221,
    REXOS_BTN_DPAD_LEFT = 0x222,
    REXOS_BTN_DPAD_RIGHT = 0x223,
} rexos_button_t;

/**
 * Button mapping entry
 */
typedef struct {
    rexos_button_t from;
    rexos_button_t to;
} rexos_button_map_t;

/**
 * Apply button remapping for current session
 */
rexos_error_t rexos_apply_button_map(const rexos_button_map_t* map, int count);

/**
 * Get analog stick deadzone
 */
int rexos_get_deadzone(void);

/**
 * Set analog stick deadzone (0-32767)
 */
rexos_error_t rexos_set_deadzone(int deadzone);

/* ============================================================================
 * Audio Bridge
 * ========================================================================== */

/**
 * Get current audio volume (0-100)
 */
int rexos_get_volume(void);

/**
 * Set audio volume (0-100)
 */
rexos_error_t rexos_set_volume(int volume);

/**
 * Check if headphones are connected
 */
bool rexos_headphones_connected(void);

/* ============================================================================
 * Display Bridge
 * ========================================================================== */

/**
 * Get current brightness (0-255)
 */
int rexos_get_brightness(void);

/**
 * Set brightness (0-255)
 */
rexos_error_t rexos_set_brightness(int brightness);

/* ============================================================================
 * RetroArch Hooks
 * ========================================================================== */

/**
 * RetroArch callback for hotkey detection
 */
typedef void (*rexos_hotkey_callback_t)(int action, void* user_data);

/**
 * Register hotkey callback
 */
rexos_error_t rexos_register_hotkey_callback(rexos_hotkey_callback_t callback, void* user_data);

/**
 * Hotkey actions
 */
typedef enum {
    REXOS_HOTKEY_EXIT = 1,
    REXOS_HOTKEY_SAVE_STATE,
    REXOS_HOTKEY_LOAD_STATE,
    REXOS_HOTKEY_SCREENSHOT,
    REXOS_HOTKEY_FAST_FORWARD,
    REXOS_HOTKEY_REWIND,
    REXOS_HOTKEY_PAUSE,
    REXOS_HOTKEY_MENU,
    REXOS_HOTKEY_VOLUME_UP,
    REXOS_HOTKEY_VOLUME_DOWN,
    REXOS_HOTKEY_BRIGHTNESS_UP,
    REXOS_HOTKEY_BRIGHTNESS_DOWN,
} rexos_hotkey_action_t;

/**
 * Check if a hotkey combination is pressed
 */
bool rexos_check_hotkey(rexos_hotkey_action_t action);

/* ============================================================================
 * Utility Functions
 * ========================================================================== */

/**
 * Get error message for error code
 */
const char* rexos_strerror(rexos_error_t err);

/**
 * Get bridge version string
 */
const char* rexos_version(void);

/**
 * Initialize the bridge (call once at startup)
 */
rexos_error_t rexos_init(void);

/**
 * Cleanup the bridge (call at shutdown)
 */
void rexos_cleanup(void);

#ifdef __cplusplus
}
#endif

#endif /* REXOS_EMULATOR_BRIDGE_H */
