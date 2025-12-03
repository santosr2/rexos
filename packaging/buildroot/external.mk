# RexOS Buildroot External Packages

include $(sort $(wildcard $(BR2_EXTERNAL_REXOS_PATH)/package/*/*.mk))
