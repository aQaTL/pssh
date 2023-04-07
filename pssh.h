#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct List List;

typedef struct OptionsMap OptionsMap;

typedef struct SshConfig SshConfig;

/**
 * Strings are UTF-8, without null terminator. Not owned.
 */
typedef struct Host {
  const int8_t *name;
  uintptr_t name_len;
  const int8_t *host_name;
  uintptr_t host_name_len;
  const int8_t *user;
  uintptr_t user_len;
  const OptionsMap *other;
} Host;

typedef struct ListEntry {
  const void *data;
  uintptr_t len;
} ListEntry;

void config_add_host(struct SshConfig *config, const struct Host *host);

bool config_remove_host(struct SshConfig *config, uintptr_t idx);

uintptr_t config_hosts_len(struct SshConfig *config);

bool config_get_host(struct SshConfig *config, uintptr_t idx, struct Host *out_host);

OptionsMap *create_settings_list(void);

void free_settings_list(OptionsMap *options_map);

struct List *list_create(void);

void list_free(struct List *l);

void list_push(struct List *l, struct ListEntry entry);
