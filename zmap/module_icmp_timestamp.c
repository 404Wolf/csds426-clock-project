// Minimal passthrough module for testing

#include <state.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include "netinet/ip.h"
#include "packet.h"
#include "probe_modules.h"
#include "sys/types.h"
#include "types.h"

probe_module_t module_icmp_timestamp;

int icmp_timestamp_global_initialize(struct state_conf *conf) {
  printf("[TEST] Module initialized\n");
  (void)conf; // unused
  return EXIT_SUCCESS;
}

int icmp_timestamp_init_perthread(void **arg_ptr) {
  printf("[TEST] Thread initialized\n");
  (void)arg_ptr; // unused
  return EXIT_SUCCESS;
}

int icmp_timestamp_make_packet(void *buf, size_t *buf_len, ipaddr_n_t src_ip,
                               ipaddr_n_t dst_ip, port_n_t dst_port,
                               uint8_t ttl, uint32_t *validation, int probe_num,
                               uint16_t ip_id, void *arg) {
  printf("[TEST] Packet made\n");
  (void)buf;
  (void)src_ip;
  (void)dst_ip;
  (void)dst_port;
  (void)ttl;
  (void)validation;
  (void)probe_num;
  (void)ip_id;
  (void)arg;
  *buf_len = 64;
  return EXIT_SUCCESS;
}

void icmp_timestamp_print_packet(FILE *fp, void *packet) {
  fprintf(fp, "[TEST] Print packet\n");
  (void)packet;
}

int icmp_timestamp_validate_packet(const struct ip *ip_hdr, uint32_t len,
                                   uint32_t *src_ip, uint32_t *validation,
                                   const struct port_conf *ports) {
  printf("[TEST] Validate\n");
  (void)ip_hdr;
  (void)len;
  (void)src_ip;
  (void)validation;
  (void)ports;
  return 1;
}

void icmp_timestamp_process_packet(const u_char *packet, uint32_t len,
                                   fieldset_t *fs, uint32_t *validation,
                                   struct timespec ts) {
  printf("[TEST] Process\n");
  (void)packet;
  (void)len;
  (void)validation;
  (void)ts;
  fs_add_string(fs, "test", (char *)"pass", 0);
}

static fielddef_t fields[] = {
    {.name = "test", .type = "string", .desc = "test field"}};

probe_module_t module_icmp_timestamp = {
    .name = "icmp_timestamp",
    .max_packet_length = 64,
    .pcap_filter = "icmp",
    .pcap_snaplen = 96,
    .port_args = 0,
    .global_initialize = &icmp_timestamp_global_initialize,
    .thread_initialize = &icmp_timestamp_init_perthread,
    .make_packet = &icmp_timestamp_make_packet,
    .print_packet = &icmp_timestamp_print_packet,
    .validate_packet = &icmp_timestamp_validate_packet,
    .process_packet = &icmp_timestamp_process_packet,
    .close = NULL,
    .output_type = OUTPUT_TYPE_STATIC,
    .fields = fields,
    .numfields = 1};
