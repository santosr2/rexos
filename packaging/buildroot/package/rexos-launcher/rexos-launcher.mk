################################################################################
#
# rexos-launcher
#
################################################################################

# This package is built as part of rexos-core
# This .mk file exists for dependency management

REXOS_LAUNCHER_VERSION = 0.1.0
REXOS_LAUNCHER_DEPENDENCIES = rexos-core

$(eval $(generic-package))
