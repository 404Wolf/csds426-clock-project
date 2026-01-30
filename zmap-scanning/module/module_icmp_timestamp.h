#ifndef MODULE_ICMP_TIMESTAMP_H
#define MODULE_ICMP_TIMESTAMP_H

#include "types.h"
#include <stdint.h>
#include <stdio.h>
#include <sys/types.h>
#include <time.h>

struct state_conf;
struct ip;
struct port_conf;
typedef struct fieldset fieldset_t;

int icmp_timestamp_global_initialize(struct state_conf *conf);
int icmp_timestamp_init_perthread(void **arg_ptr);
int icmp_timestamp_make_packet(void *buf, size_t *buf_len, ipaddr_n_t src_ip,
                               ipaddr_n_t dst_ip, port_n_t dst_port,
                               uint8_t ttl, uint32_t *validation, int probe_num,
                               uint16_t ip_id, void *arg);
void icmp_timestamp_print_packet(FILE *fp, void *packet);
int icmp_timestamp_validate_packet(const struct ip *ip_hdr, uint32_t len,
                                   uint32_t *src_ip, uint32_t *validation,
                                   const struct port_conf *ports);
void icmp_timestamp_process_packet(const u_char *packet, uint32_t len,
                                   fieldset_t *fs, uint32_t *validation,
                                   struct timespec ts);

#endif // MODULE_ICMP_TIMESTAMP_H
