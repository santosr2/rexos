/**
 * RexOS Emulator Bridge - Core Implementation
 */

#include "emulator_bridge.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <signal.h>
#include <errno.h>
#include <sys/wait.h>
#include <sys/stat.h>
#include <sys/resource.h>
#include <sched.h>
#include <time.h>

/* Global state */
static bool g_initialized = false;

/* ============================================================================
 * Utility Functions
 * ========================================================================== */

const char* rexos_strerror(rexos_error_t err)
{
    switch (err) {
        case REXOS_OK:               return "Success";
        case REXOS_ERR_INVALID_ARG:  return "Invalid argument";
        case REXOS_ERR_NOT_FOUND:    return "Not found";
        case REXOS_ERR_PERMISSION:   return "Permission denied";
        case REXOS_ERR_FORK_FAILED:  return "Fork failed";
        case REXOS_ERR_EXEC_FAILED:  return "Exec failed";
        case REXOS_ERR_TIMEOUT:      return "Timeout";
        case REXOS_ERR_MEMORY:       return "Memory allocation failed";
        case REXOS_ERR_IO:           return "I/O error";
        case REXOS_ERR_INTERNAL:     return "Internal error";
        default:                      return "Unknown error";
    }
}

const char* rexos_version(void)
{
    static char version[32];
    snprintf(version, sizeof(version), "%d.%d.%d",
             REXOS_BRIDGE_VERSION_MAJOR,
             REXOS_BRIDGE_VERSION_MINOR,
             REXOS_BRIDGE_VERSION_PATCH);
    return version;
}

rexos_error_t rexos_init(void)
{
    if (g_initialized) {
        return REXOS_OK;
    }

    /* Initialize signal handlers for child processes */
    struct sigaction sa;
    sa.sa_handler = SIG_IGN;
    sigemptyset(&sa.sa_mask);
    sa.sa_flags = SA_NOCLDWAIT;
    sigaction(SIGCHLD, &sa, NULL);

    g_initialized = true;
    return REXOS_OK;
}

void rexos_cleanup(void)
{
    g_initialized = false;
}

/* ============================================================================
 * Launch Configuration
 * ========================================================================== */

void rexos_launch_config_init(rexos_launch_config_t* config)
{
    if (!config) return;

    memset(config, 0, sizeof(*config));
    config->type = REXOS_EMU_RETROARCH;
    config->fullscreen = true;
    config->verbose = false;
    config->use_32bit = false;
    config->load_state_slot = -1;
    config->cpu_affinity = -1;
    config->nice_value = 0;
    config->realtime_priority = false;
}

int rexos_launch_config_add_arg(rexos_launch_config_t* config, const char* arg)
{
    if (!config || !arg) return -1;
    if (config->arg_count >= REXOS_MAX_ARGS - 1) return -1;

    config->args[config->arg_count] = strdup(arg);
    if (!config->args[config->arg_count]) return -1;

    config->arg_count++;
    return 0;
}

int rexos_launch_config_add_env(rexos_launch_config_t* config,
                                 const char* key, const char* value)
{
    if (!config || !key || !value) return -1;
    if (config->env_count >= REXOS_MAX_ENV) return -1;

    strncpy(config->env[config->env_count].key, key,
            sizeof(config->env[config->env_count].key) - 1);
    strncpy(config->env[config->env_count].value, value,
            sizeof(config->env[config->env_count].value) - 1);

    config->env_count++;
    return 0;
}

/* ============================================================================
 * Process Management
 * ========================================================================== */

static void setup_child_process(const rexos_launch_config_t* config)
{
    /* Set nice value */
    if (config->nice_value != 0) {
        setpriority(PRIO_PROCESS, 0, config->nice_value);
    }

    /* Set CPU affinity */
    if (config->cpu_affinity >= 0) {
        cpu_set_t cpuset;
        CPU_ZERO(&cpuset);
        CPU_SET(config->cpu_affinity, &cpuset);
        sched_setaffinity(0, sizeof(cpuset), &cpuset);
    }

    /* Set realtime priority */
    if (config->realtime_priority) {
        struct sched_param sp;
        sp.sched_priority = sched_get_priority_max(SCHED_FIFO);
        sched_setscheduler(0, SCHED_FIFO, &sp);
    }

    /* Set environment variables */
    for (int i = 0; i < config->env_count; i++) {
        setenv(config->env[i].key, config->env[i].value, 1);
    }

    /* Create new session */
    setsid();
}

static char** build_argv(const rexos_launch_config_t* config)
{
    int argc = 0;
    int max_args = 32 + config->arg_count;
    char** argv = calloc(max_args, sizeof(char*));
    if (!argv) return NULL;

    /* Executable */
    argv[argc++] = strdup(config->executable);

    /* RetroArch specific arguments */
    if (config->type == REXOS_EMU_RETROARCH) {
        /* Core */
        if (config->core_path[0]) {
            argv[argc++] = strdup("-L");
            argv[argc++] = strdup(config->core_path);
        }

        /* Config */
        if (config->config_path[0]) {
            argv[argc++] = strdup("--config");
            argv[argc++] = strdup(config->config_path);
        }

        /* Fullscreen */
        if (config->fullscreen) {
            argv[argc++] = strdup("--fullscreen");
        }

        /* Verbose */
        if (config->verbose) {
            argv[argc++] = strdup("-v");
        }

        /* Load state */
        if (config->load_state_slot >= 0) {
            argv[argc++] = strdup("-e");
            char slot_str[8];
            snprintf(slot_str, sizeof(slot_str), "%d", config->load_state_slot);
            argv[argc++] = strdup(slot_str);
        }
    }

    /* Custom arguments */
    for (int i = 0; i < config->arg_count && argc < max_args - 2; i++) {
        argv[argc++] = strdup(config->args[i]);
    }

    /* ROM path (usually last) */
    if (config->rom_path[0]) {
        argv[argc++] = strdup(config->rom_path);
    }

    argv[argc] = NULL;
    return argv;
}

static void free_argv(char** argv)
{
    if (!argv) return;
    for (int i = 0; argv[i]; i++) {
        free(argv[i]);
    }
    free(argv);
}

rexos_error_t rexos_launch(const rexos_launch_config_t* config, pid_t* pid)
{
    if (!config || !pid) {
        return REXOS_ERR_INVALID_ARG;
    }

    if (!config->executable[0]) {
        return REXOS_ERR_INVALID_ARG;
    }

    /* Check executable exists */
    if (access(config->executable, X_OK) != 0) {
        return REXOS_ERR_NOT_FOUND;
    }

    /* Build argument list */
    char** argv = build_argv(config);
    if (!argv) {
        return REXOS_ERR_MEMORY;
    }

    /* Fork process */
    pid_t child_pid = fork();

    if (child_pid < 0) {
        free_argv(argv);
        return REXOS_ERR_FORK_FAILED;
    }

    if (child_pid == 0) {
        /* Child process */
        setup_child_process(config);

        /* Close stdin */
        close(STDIN_FILENO);
        open("/dev/null", O_RDONLY);

        /* Execute */
        execvp(argv[0], argv);

        /* If we get here, exec failed */
        fprintf(stderr, "RexOS: exec failed: %s\n", strerror(errno));
        _exit(127);
    }

    /* Parent process */
    free_argv(argv);
    *pid = child_pid;

    return REXOS_OK;
}

rexos_error_t rexos_wait(pid_t pid, int timeout_ms, int* exit_code)
{
    if (pid <= 0) {
        return REXOS_ERR_INVALID_ARG;
    }

    int status;
    int options = 0;

    if (timeout_ms == 0) {
        options = WNOHANG;
    } else if (timeout_ms > 0) {
        /* Implement timeout with polling */
        int elapsed = 0;
        while (elapsed < timeout_ms) {
            pid_t result = waitpid(pid, &status, WNOHANG);

            if (result == pid) {
                if (exit_code && WIFEXITED(status)) {
                    *exit_code = WEXITSTATUS(status);
                }
                return REXOS_OK;
            }

            if (result < 0) {
                return REXOS_ERR_IO;
            }

            usleep(10000); /* 10ms */
            elapsed += 10;
        }
        return REXOS_ERR_TIMEOUT;
    }

    /* Infinite wait */
    pid_t result = waitpid(pid, &status, options);

    if (result == pid) {
        if (exit_code && WIFEXITED(status)) {
            *exit_code = WEXITSTATUS(status);
        }
        return REXOS_OK;
    }

    if (result == 0 && (options & WNOHANG)) {
        return REXOS_ERR_TIMEOUT;
    }

    return REXOS_ERR_IO;
}

rexos_error_t rexos_get_process_info(pid_t pid, rexos_process_info_t* info)
{
    if (pid <= 0 || !info) {
        return REXOS_ERR_INVALID_ARG;
    }

    memset(info, 0, sizeof(*info));
    info->pid = pid;

    /* Read from /proc/[pid]/stat */
    char path[64];
    snprintf(path, sizeof(path), "/proc/%d/stat", pid);

    FILE* f = fopen(path, "r");
    if (!f) {
        info->state = REXOS_PROC_DEAD;
        return REXOS_OK;
    }

    char state;
    unsigned long utime, stime, vsize;
    long rss;

    fscanf(f, "%*d %*s %c %*d %*d %*d %*d %*d %*u %*u %*u %*u %*u "
           "%lu %lu %*d %*d %*d %*d %*d %*d %*u %lu %ld",
           &state, &utime, &stime, &vsize, &rss);
    fclose(f);

    /* Parse state */
    switch (state) {
        case 'R': info->state = REXOS_PROC_RUNNING; break;
        case 'S': info->state = REXOS_PROC_SLEEPING; break;
        case 'T': info->state = REXOS_PROC_STOPPED; break;
        case 'Z': info->state = REXOS_PROC_ZOMBIE; break;
        default:  info->state = REXOS_PROC_UNKNOWN; break;
    }

    /* Calculate CPU time (in milliseconds) */
    long ticks_per_sec = sysconf(_SC_CLK_TCK);
    info->cpu_time_ms = ((utime + stime) * 1000) / ticks_per_sec;

    /* Memory in KB (rss is in pages) */
    long page_size = sysconf(_SC_PAGESIZE);
    info->memory_kb = (rss * page_size) / 1024;

    return REXOS_OK;
}

rexos_error_t rexos_signal(pid_t pid, int sig)
{
    if (pid <= 0) {
        return REXOS_ERR_INVALID_ARG;
    }

    if (kill(pid, sig) == 0) {
        return REXOS_OK;
    }

    if (errno == ESRCH) {
        return REXOS_ERR_NOT_FOUND;
    }
    if (errno == EPERM) {
        return REXOS_ERR_PERMISSION;
    }

    return REXOS_ERR_IO;
}

rexos_error_t rexos_stop(pid_t pid)
{
    return rexos_signal(pid, SIGTERM);
}

rexos_error_t rexos_kill(pid_t pid)
{
    return rexos_signal(pid, SIGKILL);
}

/* ============================================================================
 * File I/O Helpers
 * ========================================================================== */

static int read_int_from_file(const char* path)
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

static rexos_error_t write_int_to_file(const char* path, int value)
{
    FILE* f = fopen(path, "w");
    if (!f) {
        return errno == EACCES ? REXOS_ERR_PERMISSION : REXOS_ERR_IO;
    }

    fprintf(f, "%d", value);
    fclose(f);
    return REXOS_OK;
}

static rexos_error_t write_string_to_file(const char* path, const char* value)
{
    FILE* f = fopen(path, "w");
    if (!f) {
        return errno == EACCES ? REXOS_ERR_PERMISSION : REXOS_ERR_IO;
    }

    fprintf(f, "%s", value);
    fclose(f);
    return REXOS_OK;
}
