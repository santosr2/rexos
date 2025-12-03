#include "launcher.h"

#include <errno.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/wait.h>
#include <unistd.h>

pid_t launch_emulator(const char* core_path, const char* rom_path, const char* config_path)
{
    if (!core_path || !rom_path) {
        fprintf(stderr, "RexOS: Invalid arguments to launch_emulator\n");
        return -1;
    }

    pid_t pid = fork();

    if (pid < 0) {
        // Fork failed
        fprintf(stderr, "RexOS: Failed to fork process: %s\n", strerror(errno));
        return -1;
    }

    if (pid == 0) {
        // Child process - execute emulator
        // TODO: Implement actual emulator launching
        // For now, this is a placeholder

        fprintf(stdout, "RexOS: Launching emulator\n");
        fprintf(stdout, "  Core: %s\n", core_path);
        fprintf(stdout, "  ROM: %s\n", rom_path);
        if (config_path) {
            fprintf(stdout, "  Config: %s\n", config_path);
        }

        // In actual implementation, this would exec the emulator
        // execl(core_path, core_path, rom_path, NULL);

        exit(0);
    }

    // Parent process - return child PID
    return pid;
}

int monitor_emulator(pid_t pid)
{
    if (pid <= 0) {
        return -1;
    }

    int status;
    if (waitpid(pid, &status, 0) == -1) {
        fprintf(stderr, "RexOS: Failed to wait for process %d: %s\n", pid, strerror(errno));
        return -1;
    }

    if (WIFEXITED(status)) {
        return WEXITSTATUS(status);
    }

    return -1;
}

int stop_emulator(pid_t pid)
{
    if (pid <= 0) {
        return -1;
    }

    // Send SIGTERM for graceful shutdown
    if (kill(pid, SIGTERM) == -1) {
        fprintf(stderr, "RexOS: Failed to send SIGTERM to process %d: %s\n", pid, strerror(errno));
        return -1;
    }

    return 0;
}

int kill_emulator(pid_t pid)
{
    if (pid <= 0) {
        return -1;
    }

    // Send SIGKILL to force termination
    if (kill(pid, SIGKILL) == -1) {
        fprintf(stderr, "RexOS: Failed to send SIGKILL to process %d: %s\n", pid, strerror(errno));
        return -1;
    }

    return 0;
}
