#ifndef REXOS_EMULATOR_LAUNCHER_H
#define REXOS_EMULATOR_LAUNCHER_H

#include <stdint.h>
#include <sys/types.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Launch an emulator with the specified core and ROM
 *
 * @param core_path Path to the emulator core/binary
 * @param rom_path Path to the ROM file
 * @param config_path Optional configuration file path (can be NULL)
 * @return Process ID on success, -1 on error
 */
pid_t launch_emulator(const char* core_path, const char* rom_path, const char* config_path);

/**
 * Monitor an emulator process
 *
 * @param pid Process ID to monitor
 * @return Exit code of the process, -1 on error
 */
int monitor_emulator(pid_t pid);

/**
 * Stop an emulator process gracefully
 *
 * @param pid Process ID to stop
 * @return 0 on success, -1 on error
 */
int stop_emulator(pid_t pid);

/**
 * Force kill an emulator process
 *
 * @param pid Process ID to kill
 * @return 0 on success, -1 on error
 */
int kill_emulator(pid_t pid);

#ifdef __cplusplus
}
#endif

#endif  // REXOS_EMULATOR_LAUNCHER_H
