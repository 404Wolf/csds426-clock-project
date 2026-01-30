// From
// https://github.com/zmap/zmap/blob/main/src/probe_modules/module_icmp_echo_time.c

// probe module for performing ICMP timestamp request scans for clock
// synchronization uses ICMP type 13 (Timestamp Request) and expects type 14
// (Timestamp Reply)

#include <send.h>
#include <state.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/time.h>
#include <time.h>
#include <unistd.h>

#include "../../lib/includes.h"
#include "../fieldset.h"
#include "bits/types/struct_timeval.h"
#include "net/ethernet.h"
#include "netinet/in.h"
#include "netinet/ip.h"
#include "netinet/ip_icmp.h"
#include "packet.h"
#include "probe_modules.h"
#include "sys/types.h"
#include "types.h"
#include "validate.h"

#define ICMP_SMALLEST_SIZE 5
#define ICMP_TIMXCEED_UNREACH_HEADER_SIZE 8

probe_module_t module_icmp_timestamp;

static uint32_t get_timestamp_ms(void) {
  struct timeval tv;
  gettimeofday(&tv, NULL);
  // Convert to milliseconds since midnight UTC
  time_t now = tv.tv_sec;
  struct tm *tm_utc = gmtime(&now);
  uint32_t ms_since_midnight = (tm_utc->tm_hour * 3600000) +
                               (tm_utc->tm_min * 60000) +
                               (tm_utc->tm_sec * 1000) + (tv.tv_usec / 1000);
  return htonl(ms_since_midnight);
}

static int icmp_timestamp_prepare_packet(void *buf, macaddr_t *src,
                                         macaddr_t *gw, UNUSED void *arg_ptr) {
  memset(buf, 0, MAX_PACKET_SIZE);

  struct ether_header *eth_header = (struct ether_header *)buf;
  make_eth_header(eth_header, src, gw);

  struct ip *ip_header = (struct ip *)(&eth_header[1]);
  uint16_t len = htons(sizeof(struct ip) +
                       20); // 8 bytes ICMP header + 12 bytes timestamp data
  make_ip_header(ip_header, IPPROTO_ICMP, len);

  struct icmp *icmp_header = (struct icmp *)(&ip_header[1]);

  // This is all the same for each packet, so only set it up one time
  icmp_header->icmp_type = ICMP_TIMESTAMP; // Type 13

  return EXIT_SUCCESS;
}

static int icmp_timestamp_make_packet(void *buf, size_t *buf_len,
                                      ipaddr_n_t src_ip, ipaddr_n_t dst_ip,
                                      UNUSED port_n_t dport, uint8_t ttl,
                                      uint32_t *validation,
                                      UNUSED int probe_num, uint16_t ip_id,
                                      UNUSED void *arg) {
  struct ether_header *eth_header = (struct ether_header *)buf;
  struct ip *ip_header = (struct ip *)(&eth_header[1]);
  struct icmp *icmp = (struct icmp *)(&ip_header[1]);

  uint16_t icmp_idnum = validation[1] & 0xFFFF;
  uint16_t icmp_seqnum = validation[2] & 0xFFFF;

  ip_header->ip_src.s_addr = src_ip;
  ip_header->ip_dst.s_addr = dst_ip;
  ip_header->ip_ttl = ttl;

  icmp->icmp_id = icmp_idnum;
  icmp->icmp_seq = icmp_seqnum;

  // Set originate timestamp (milliseconds since midnight UTC)
  icmp->icmp_otime = get_timestamp_ms();
  // icmp_header->icmp_ttime = 0; these will be filled by responder, and are
  // already 0 icmp_header->icmp_ttime = 0;

  // Calculate checksum over ICMP header + timestamp payload (20 bytes total)
  icmp->icmp_cksum = 0;
  icmp->icmp_cksum = icmp_checksum((unsigned short *)icmp, 20);

  // Set IP length: IP header + ICMP header (8 bytes) + timestamp data (12
  // bytes)
  size_t ip_len = sizeof(struct ip) + 20;
  ip_header->ip_len = htons(ip_len);

  ip_header->ip_id = ip_id;
  ip_header->ip_sum = 0;
  ip_header->ip_sum = zmap_ip_checksum((unsigned short *)ip_header);

  *buf_len = ip_len + sizeof(struct ether_header);
  return EXIT_SUCCESS;
}

static void icmp_timestamp_print_packet(FILE *fp, void *packet) {
  struct ether_header *ethh = (struct ether_header *)packet;
  struct ip *iph = (struct ip *)&ethh[1];
  struct icmp *icmp = (struct icmp *)(&iph[1]);

  fprintf(fp,
          "icmp_timestamp { type: %u | code: %u "
          "| checksum: %#04X | id: %u | seq: %u | orig_ts: %u | recv_ts: %u | "
          "xmit_ts: %u }\n",
          icmp->icmp_type, icmp->icmp_code, ntohs(icmp->icmp_cksum),
          ntohs(icmp->icmp_id), ntohs(icmp->icmp_seq), ntohl(icmp->icmp_otime),
          ntohl(icmp->icmp_rtime), ntohl(icmp->icmp_ttime));
  fprintf_ip_header(fp, iph);
  fprintf_eth_header(fp, ethh);
  fprintf(fp, PRINT_PACKET_SEP);
}

static int icmp_validate_packet(const struct ip *ip_hdr, uint32_t len,
                                uint32_t *src_ip, uint32_t *validation,
                                UNUSED const struct port_conf *ports) {
  if (ip_hdr->ip_p != IPPROTO_ICMP) {
    return 0;
  }
  if (((uint32_t)4 * ip_hdr->ip_hl + ICMP_SMALLEST_SIZE) > len) {
    // buffer not large enough to contain expected icmp header
    return 0;
  }
  struct icmp *icmp_h = (struct icmp *)((char *)ip_hdr + 4 * ip_hdr->ip_hl);
  uint16_t icmp_idnum = icmp_h->icmp_id;
  uint16_t icmp_seqnum = icmp_h->icmp_seq;
  // ICMP validation is tricky: for some packet types, we must look inside
  // the payload
  if (icmp_h->icmp_type == ICMP_TIMXCEED || icmp_h->icmp_type == ICMP_UNREACH) {
    // Should have 16B TimeExceeded/Dest_Unreachable header +
    // original IP header + 1st 8B of original ICMP frame
    if ((4 * ip_hdr->ip_hl + ICMP_TIMXCEED_UNREACH_HEADER_SIZE +
         sizeof(struct ip)) > len) {
      return 0;
    }
    struct ip *ip_inner = (struct ip *)((char *)icmp_h + 8);
    if (((uint32_t)4 * ip_hdr->ip_hl + ICMP_TIMXCEED_UNREACH_HEADER_SIZE +
         4 * ip_inner->ip_hl + 8 /*1st 8 bytes of original*/) > len) {
      return 0;
    }
    struct icmp *icmp_inner =
        (struct icmp *)((char *)ip_inner + 4 * ip_hdr->ip_hl);
    // Regenerate validation and icmp id based off inner payload
    icmp_idnum = icmp_inner->icmp_id;
    icmp_seqnum = icmp_inner->icmp_seq;
    *src_ip = ip_inner->ip_dst.s_addr;
    validate_gen(ip_hdr->ip_dst.s_addr, ip_inner->ip_dst.s_addr, 0,
                 (uint8_t *)validation);
  }
  // validate icmp id and seqnum
  if (icmp_idnum != (validation[1] & 0xFFFF)) {
    return 0;
  }
  if (icmp_seqnum != (validation[2] & 0xFFFF)) {
    return 0;
  }

  return 1;
}

static void icmp_timestamp_process_packet(const u_char *packet,
                                          UNUSED uint32_t len, fieldset_t *fs,
                                          UNUSED uint32_t *validation,
                                          UNUSED struct timespec ts) {
  struct ip *ip_hdr = (struct ip *)&packet[sizeof(struct ether_header)];
  struct icmp *icmp = (struct icmp *)((char *)ip_hdr + 4 * ip_hdr->ip_hl);

  fs_add_uint64(fs, "type", icmp->icmp_type);
  fs_add_uint64(fs, "code", icmp->icmp_code);
  fs_add_uint64(fs, "icmp_id", ntohs(icmp->icmp_id));
  fs_add_uint64(fs, "seq", ntohs(icmp->icmp_seq));

  // Extract timestamp values (convert from network byte order)
  uint64_t otime = ntohl(icmp->icmp_otime);
  uint64_t rtime = ntohl(icmp->icmp_rtime);
  uint64_t ttime = ntohl(icmp->icmp_ttime);

  fs_add_uint64(fs, "otime", otime);
  fs_add_uint64(fs, "rtime", rtime);
  fs_add_uint64(fs, "ttime", ttime);

  // Calculate RTT and remote processing time
  uint64_t local_recv_ms = (ts.tv_sec % 86400) * 1000 + (ts.tv_nsec / 1000000);
  uint64_t rtt_ms = 0;
  uint64_t remote_proc_ms = 0;

  if (rtime > 0 && ttime > 0) {
    // RTT = (local_recv_time - originate_time) - (transmit_time - receive_time)
    uint64_t total_time = (local_recv_ms >= otime)
                              ? (local_recv_ms - otime)
                              : (86400000 - otime + local_recv_ms);
    remote_proc_ms =
        (ttime >= rtime) ? (ttime - rtime) : (86400000 - rtime + ttime);
    rtt_ms = (total_time >= remote_proc_ms) ? (total_time - remote_proc_ms) : 0;
  }

  fs_add_uint64(fs, "rtt_ms", rtt_ms);
  fs_add_uint64(fs, "remote_processing_ms", remote_proc_ms);

  switch (icmp->icmp_type) {
  case ICMP_ECHOREPLY:
    fs_add_string(fs, "classification", (char *)"echoreply", 0);
    fs_add_uint64(fs, "success", 0);
    break;
  case ICMP_UNREACH:
    fs_add_string(fs, "classification", (char *)"unreach", 0);
    fs_add_uint64(fs, "success", 0);
    break;
  case ICMP_SOURCEQUENCH:
    fs_add_string(fs, "classification", (char *)"sourcequench", 0);
    fs_add_uint64(fs, "success", 0);
    break;
  case ICMP_REDIRECT:
    fs_add_string(fs, "classification", (char *)"redirect", 0);
    fs_add_uint64(fs, "success", 0);
    break;
  case ICMP_TIMXCEED:
    fs_add_string(fs, "classification", (char *)"timxceed", 0);
    fs_add_uint64(fs, "success", 0);
    break;
  case ICMP_TIMESTAMP:
    fs_add_string(fs, "classification", (char *)"timestamp", 0);
    fs_add_uint64(fs, "success", 0);
    break;
  case ICMP_TIMESTAMPREPLY:
    fs_add_string(fs, "classification", (char *)"timestampreply", 0);
    fs_add_uint64(fs, "success", 1);
    break;
  default:
    fs_add_string(fs, "classification", (char *)"other", 0);
    fs_add_uint64(fs, "success", 0);
    break;
  }
}

static fielddef_t fields[] = {
    {.name = "type", .type = "int", .desc = "icmp message type"},
    {.name = "code", .type = "int", .desc = "icmp message sub type code"},
    {.name = "icmp_id", .type = "int", .desc = "icmp id number"},
    {.name = "seq", .type = "int", .desc = "icmp sequence number"},
    {.name = "otime",
     .type = "int",
     .desc = "originate timestamp from sender (ms since midnight UTC)"},
    {.name = "rtime",
     .type = "int",
     .desc = "receive timestamp from responder (ms since midnight UTC)"},
    {.name = "ttime",
     .type = "int",
     .desc = "transmit timestamp from responder (ms since midnight UTC)"},
    {.name = "rtt_ms",
     .type = "int",
     .desc = "round-trip time in milliseconds"},
    {.name = "remote_processing_ms",
     .type = "int",
     .desc = "remote processing time in milliseconds"},
    {.name = "classification",
     .type = "string",
     .desc = "probe module classification"},
    {.name = "success",
     .type = "int",
     .desc = "did probe module classify response as success"}};

probe_module_t module_icmp_timestamp = {
    .name = "icmp_timestamp",
    .max_packet_length = 62,
    .pcap_filter = "icmp",
    .pcap_snaplen = 96,
    .port_args = 0,
    .prepare_packet = &icmp_timestamp_prepare_packet,
    .make_packet = &icmp_timestamp_make_packet,
    .print_packet = &icmp_timestamp_print_packet,
    .process_packet = &icmp_timestamp_process_packet,
    .validate_packet = &icmp_validate_packet,
    .close = NULL,
    .output_type = OUTPUT_TYPE_STATIC,
    .fields = fields,
    .numfields = 11};
