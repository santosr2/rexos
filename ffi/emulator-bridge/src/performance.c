/**
 * RexOS Emulator Bridge - Performance Monitoring
 */

#include "emulator_bridge.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>

/* Sysfs paths */
#define CPU_TEMP_PATH "/sys/class/thermal/thermal_zone0/temp"
#define CPU_FREQ_PATH "/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq"
#define CPU_GOVERNOR_PATH "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor"
#define CPU_MIN_FREQ_PATH "/sys/devices/system/cpu/cpu0/cpufreq/scaling_min_freq"
#define CPU_MAX_FREQ_PATH "/sys/devices/system/cpu/cpu0/cpufreq/scaling_max_freq"
#define BATTERY_CAPACITY_PATH "/sys/class/power_supply/battery/capacity"
#define BATTERY_STATUS_PATH "/sys/class/power_supply/battery/status"
#define BATTERY_TEMP_PATH "/sys/class/power_supply/battery/temp"

/* Static CPU usage tracking */
static unsigned long long prev_user = 0;
static unsigned long long prev_nice = 0;
static unsigned long long prev_system = 0;
static unsigned long long prev_idle = 0;
static unsigned long long prev_iowait = 0;
static unsigned long long prev_irq = 0;
static unsigned long long prev_softirq = 0;

/**
 * Read an integer from a sysfs file
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
 * Read a string from a sysfs file
 */
static int read_sysfs_string(const char* path, char* buf, size_t len)
{
    FILE* f = fopen(path, "r");
    if (!f) return -1;

    if (!fgets(buf, len, f)) {
        fclose(f);
        return -1;
    }

    /* Remove trailing newline */
    size_t slen = strlen(buf);
    if (slen > 0 && buf[slen - 1] == '\n') {
        buf[slen - 1] = '\0';
    }

    fclose(f);
    return 0;
}

/**
 * Write a string to a sysfs file
 */
static int write_sysfs_string(const char* path, const char* value)
{
    FILE* f = fopen(path, "w");
    if (!f) return -1;

    fprintf(f, "%s", value);
    fclose(f);
    return 0;
}

/**
 * Write an integer to a sysfs file
 */
static int write_sysfs_int(const char* path, int value)
{
    FILE* f = fopen(path, "w");
    if (!f) return -1;

    fprintf(f, "%d", value);
    fclose(f);
    return 0;
}

/**
 * Calculate CPU usage from /proc/stat
 */
static float calculate_cpu_usage(void)
{
    FILE* f = fopen("/proc/stat", "r");
    if (!f) return 0.0f;

    char line[256];
    if (!fgets(line, sizeof(line), f)) {
        fclose(f);
        return 0.0f;
    }
    fclose(f);

    unsigned long long user, nice, system, idle, iowait, irq, softirq;

    sscanf(line, "cpu %llu %llu %llu %llu %llu %llu %llu",
           &user, &nice, &system, &idle, &iowait, &irq, &softirq);

    /* Calculate deltas */
    unsigned long long d_user = user - prev_user;
    unsigned long long d_nice = nice - prev_nice;
    unsigned long long d_system = system - prev_system;
    unsigned long long d_idle = idle - prev_idle;
    unsigned long long d_iowait = iowait - prev_iowait;
    unsigned long long d_irq = irq - prev_irq;
    unsigned long long d_softirq = softirq - prev_softirq;

    unsigned long long total = d_user + d_nice + d_system + d_idle +
                               d_iowait + d_irq + d_softirq;
    unsigned long long busy = d_user + d_nice + d_system + d_irq + d_softirq;

    /* Save current values */
    prev_user = user;
    prev_nice = nice;
    prev_system = system;
    prev_idle = idle;
    prev_iowait = iowait;
    prev_irq = irq;
    prev_softirq = softirq;

    if (total == 0) return 0.0f;
    return (float)busy / (float)total * 100.0f;
}

/**
 * Get memory information from /proc/meminfo
 */
static void get_memory_info(uint64_t* total, uint64_t* free, uint64_t* available)
{
    *total = 0;
    *free = 0;
    *available = 0;

    FILE* f = fopen("/proc/meminfo", "r");
    if (!f) return;

    char line[256];
    while (fgets(line, sizeof(line), f)) {
        unsigned long long value;

        if (sscanf(line, "MemTotal: %llu kB", &value) == 1) {
            *total = value;
        } else if (sscanf(line, "MemFree: %llu kB", &value) == 1) {
            *free = value;
        } else if (sscanf(line, "MemAvailable: %llu kB", &value) == 1) {
            *available = value;
        }
    }

    fclose(f);
}

rexos_error_t rexos_get_perf_stats(rexos_perf_stats_t* stats)
{
    if (!stats) {
        return REXOS_ERR_INVALID_ARG;
    }

    memset(stats, 0, sizeof(*stats));

    /* CPU usage */
    stats->cpu_usage_percent = calculate_cpu_usage();

    /* CPU temperature (value is in millidegrees) */
    int temp = read_sysfs_int(CPU_TEMP_PATH);
    if (temp >= 0) {
        stats->cpu_temperature = temp / 1000;
    }

    /* CPU frequency (value is in kHz) */
    int freq = read_sysfs_int(CPU_FREQ_PATH);
    if (freq >= 0) {
        stats->cpu_frequency = freq / 1000; /* Convert to MHz */
    }

    /* Memory */
    uint64_t mem_total, mem_free, mem_available;
    get_memory_info(&mem_total, &mem_free, &mem_available);
    stats->mem_total_kb = mem_total;
    stats->mem_free_kb = mem_available > 0 ? mem_available : mem_free;
    stats->mem_used_kb = mem_total - stats->mem_free_kb;

    /* Battery */
    stats->battery_percent = read_sysfs_int(BATTERY_CAPACITY_PATH);
    if (stats->battery_percent < 0) stats->battery_percent = 100;

    char status[32];
    if (read_sysfs_string(BATTERY_STATUS_PATH, status, sizeof(status)) == 0) {
        stats->battery_charging = (strcmp(status, "Charging") == 0);
    }

    int bat_temp = read_sysfs_int(BATTERY_TEMP_PATH);
    if (bat_temp >= 0) {
        stats->battery_temp = bat_temp / 10; /* Value is in tenths of degrees */
    }

    /* GPU stats - try common paths for Mali/Adreno */
    const char* gpu_paths[] = {
        "/sys/class/devfreq/ffa30000.gpu/load",
        "/sys/kernel/gpu/gpu_busy",
        "/sys/class/kgsl/kgsl-3d0/gpu_busy_percentage",
        NULL
    };

    for (int i = 0; gpu_paths[i]; i++) {
        int gpu_load = read_sysfs_int(gpu_paths[i]);
        if (gpu_load >= 0) {
            stats->gpu_usage_percent = gpu_load;
            break;
        }
    }

    /* GPU temperature */
    const char* gpu_temp_paths[] = {
        "/sys/class/thermal/thermal_zone1/temp",
        "/sys/class/kgsl/kgsl-3d0/temp",
        NULL
    };

    for (int i = 0; gpu_temp_paths[i]; i++) {
        int gpu_temp = read_sysfs_int(gpu_temp_paths[i]);
        if (gpu_temp >= 0) {
            stats->gpu_temperature = gpu_temp / 1000;
            break;
        }
    }

    return REXOS_OK;
}

rexos_error_t rexos_set_cpu_governor(const char* governor)
{
    if (!governor) {
        return REXOS_ERR_INVALID_ARG;
    }

    /* Valid governors */
    const char* valid[] = {
        "performance", "powersave", "schedutil", "ondemand",
        "conservative", "userspace", NULL
    };

    int valid_governor = 0;
    for (int i = 0; valid[i]; i++) {
        if (strcmp(governor, valid[i]) == 0) {
            valid_governor = 1;
            break;
        }
    }

    if (!valid_governor) {
        return REXOS_ERR_INVALID_ARG;
    }

    /* Set governor for all CPUs */
    char path[128];
    for (int cpu = 0; cpu < 8; cpu++) {
        snprintf(path, sizeof(path),
                 "/sys/devices/system/cpu/cpu%d/cpufreq/scaling_governor", cpu);

        if (access(path, W_OK) != 0) continue;

        if (write_sysfs_string(path, governor) != 0) {
            /* First failure is acceptable, might just be less CPUs */
            if (cpu == 0) {
                return REXOS_ERR_PERMISSION;
            }
            break;
        }
    }

    return REXOS_OK;
}

rexos_error_t rexos_set_cpu_freq(uint32_t min_freq, uint32_t max_freq)
{
    char path[128];

    /* Set for all CPUs */
    for (int cpu = 0; cpu < 8; cpu++) {
        if (min_freq > 0) {
            snprintf(path, sizeof(path),
                     "/sys/devices/system/cpu/cpu%d/cpufreq/scaling_min_freq", cpu);
            if (access(path, W_OK) == 0) {
                write_sysfs_int(path, min_freq);
            }
        }

        if (max_freq > 0) {
            snprintf(path, sizeof(path),
                     "/sys/devices/system/cpu/cpu%d/cpufreq/scaling_max_freq", cpu);
            if (access(path, W_OK) == 0) {
                write_sysfs_int(path, max_freq);
            }
        }
    }

    return REXOS_OK;
}
