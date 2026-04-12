
ev/stdin /dev/stdout --method HEAD | awk -F, 'NR==2{print "http_clock_offset_ms=" $6 "ms"}'
   Compiling clocks v0.1.0 (/home/wolf/Projects/csds426-clock-project/http-syncing)
    Finished `release` profile [optimized] target(s) in 13.60s
     Running `target/release/enrich-http /dev/stdin /dev/stdout --method HEAD`
[2026-04-12T04:38:26Z TRACE clocks] HEAD http://404wolf.com recv 04:38:26.745452 server 04:38:26
[2026-04-12T04:38:26Z TRACE clocks] HEAD http://404wolf.com send 04:38:26.745535
[2026-04-12T04:38:26Z TRACE clocks] HEAD http://404wolf.com recv 04:38:26.772170 server 04:38:26
[2026-04-12T04:38:29Z TRACE clocks] HEAD http://404wolf.com send 04:38:29.000000
[2026-04-12T04:38:29Z TRACE clocks] HEAD http://404wolf.com recv 04:38:29.026376 server 04:38:29
[2026-04-12T04:38:29Z INFO  clocks] round 1: HIT boundary at 0µs (±2000ms window)
[2026-04-12T04:38:29Z TRACE clocks] HEAD http://404wolf.com send 04:38:29.026445
[2026-04-12T04:38:29Z TRACE clocks] HEAD http://404wolf.com recv 04:38:29.050526 server 04:38:29
[2026-04-12T04:38:31Z TRACE clocks] HEAD http://404wolf.com send 04:38:31.000000
[2026-04-12T04:38:31Z TRACE clocks] HEAD http://404wolf.com recv 04:38:31.026898 server 04:38:31
[2026-04-12T04:38:31Z INFO  clocks] round 2: HIT boundary at 0µs (±1000ms window)
[2026-04-12T04:38:31Z TRACE clocks] HEAD http://404wolf.com send 04:38:31.500001
[2026-04-12T04:38:31Z TRACE clocks] HEAD http://404wolf.com recv 04:38:31.519957 server 04:38:31
[2026-04-12T04:38:32Z TRACE clocks] HEAD http://404wolf.com send 04:38:32.500000
[2026-04-12T04:38:32Z TRACE clocks] HEAD http://404wolf.com recv 04:38:32.520688 server 04:38:32
[2026-04-12T04:38:32Z INFO  clocks] round 3: HIT boundary at 0µs (±500ms window)
[2026-04-12T04:38:32Z TRACE clocks] HEAD http://404wolf.com send 04:38:32.750001
[2026-04-12T04:38:32Z TRACE clocks] HEAD http://404wolf.com recv 04:38:32.773667 server 04:38:32
[2026-04-12T04:38:33Z TRACE clocks] HEAD http://404wolf.com send 04:38:33.250000
[2026-04-12T04:38:33Z TRACE clocks] HEAD http://404wolf.com recv 04:38:33.272491 server 04:38:33
[2026-04-12T04:38:33Z INFO  clocks] round 4: HIT boundary at 0µs (±250ms window)
[2026-04-12T04:38:33Z TRACE clocks] HEAD http://404wolf.com send 04:38:33.875000
[2026-04-12T04:38:33Z TRACE clocks] HEAD http://404wolf.com recv 04:38:33.897546 server 04:38:33
[2026-04-12T04:38:34Z TRACE clocks] HEAD http://404wolf.com send 04:38:34.125000
[2026-04-12T04:38:34Z TRACE clocks] HEAD http://404wolf.com recv 04:38:34.147356 server 04:38:34
[2026-04-12T04:38:34Z INFO  clocks] round 5: HIT boundary at 0µs (±125ms window)
[2026-04-12T04:38:34Z TRACE clocks] HEAD http://404wolf.com send 04:38:34.937500
[2026-04-12T04:38:34Z TRACE clocks] HEAD http://404wolf.com recv 04:38:34.964086 server 04:38:34
[2026-04-12T04:38:35Z TRACE clocks] HEAD http://404wolf.com send 04:38:35.062501
[2026-04-12T04:38:35Z TRACE clocks] HEAD http://404wolf.com recv 04:38:35.084770 server 04:38:35
[2026-04-12T04:38:35Z INFO  clocks] round 6: HIT boundary at 0µs (±62ms window)
[2026-04-12T04:38:35Z TRACE clocks] HEAD http://404wolf.com send 04:38:35.968750
[2026-04-12T04:38:35Z TRACE clocks] HEAD http://404wolf.com recv 04:38:35.991518 server 04:38:35
[2026-04-12T04:38:36Z TRACE clocks] HEAD http://404wolf.com send 04:38:36.031251
[2026-04-12T04:38:36Z TRACE clocks] HEAD http://404wolf.com recv 04:38:36.054496 server 04:38:36
[2026-04-12T04:38:36Z INFO  clocks] round 7: HIT boundary at 0µs (±31ms window)
[2026-04-12T04:38:36Z TRACE clocks] HEAD http://404wolf.com send 04:38:36.984375
[2026-04-12T04:38:37Z TRACE clocks] HEAD http://404wolf.com recv 04:38:37.006522 server 04:38:37
[2026-04-12T04:38:37Z TRACE clocks] HEAD http://404wolf.com send 04:38:37.015625
[2026-04-12T04:38:37Z TRACE clocks] HEAD http://404wolf.com recv 04:38:37.037076 server 04:38:37
[2026-04-12T04:38:37Z INFO  clocks] round 8: miss (±15ms window)
[2026-04-12T04:38:37Z TRACE clocks] HEAD http://404wolf.com send 04:38:37.992188
[2026-04-12T04:38:38Z TRACE clocks] HEAD http://404wolf.com send 04:38:38.007813
[2026-04-12T04:38:38Z TRACE clocks] HEAD http://404wolf.com recv 04:38:38.016554 server 04:38:38
[2026-04-12T04:38:38Z TRACE clocks] HEAD http://404wolf.com recv 04:38:38.052761 server 04:38:38
[2026-04-12T04:38:38Z INFO  clocks] round 9: miss (±7ms window)
[2026-04-12T04:38:38Z TRACE clocks] HEAD http://404wolf.com send 04:38:38.996094
[2026-04-12T04:38:39Z TRACE clocks] HEAD http://404wolf.com send 04:38:39.003906
[2026-04-12T04:38:39Z TRACE clocks] HEAD http://404wolf.com recv 04:38:39.017933 server 04:38:39
[2026-04-12T04:38:39Z TRACE clocks] HEAD http://404wolf.com recv 04:38:39.027263 server 04:38:39
[2026-04-12T04:38:39Z INFO  clocks] round 10: miss (±3ms window)
[2026-04-12T04:38:39Z TRACE clocks] HEAD http://404wolf.com send 04:38:39.998047
[2026-04-12T04:38:40Z TRACE clocks] HEAD http://404wolf.com send 04:38:40.001953
[2026-04-12T04:38:40Z TRACE clocks] HEAD http://404wolf.com recv 04:38:40.020529 server 04:38:40
[2026-04-12T04:38:40Z TRACE clocks] HEAD http://404wolf.com recv 04:38:40.025377 server 04:38:40
[2026-04-12T04:38:40Z INFO  clocks] round 11: miss (±1ms window)
[2026-04-12T04:38:40Z TRACE clocks] HEAD http://404wolf.com send 04:38:40.999024
[2026-04-12T04:38:41Z TRACE clocks] HEAD http://404wolf.com send 04:38:41.000976
[2026-04-12T04:38:41Z TRACE clocks] HEAD http://404wolf.com recv 04:38:41.022878 server 04:38:41
[2026-04-12T04:38:41Z TRACE clocks] HEAD http://404wolf.com recv 04:38:41.027953 server 04:38:41
[2026-04-12T04:38:41Z INFO  clocks] round 12: miss (±0ms window)
[2026-04-12T04:38:41Z TRACE clocks] HEAD http://404wolf.com send 04:38:41.999512
[2026-04-12T04:38:42Z TRACE clocks] HEAD http://404wolf.com send 04:38:42.000488
[2026-04-12T04:38:42Z TRACE clocks] HEAD http://404wolf.com recv 04:38:42.022135 server 04:38:42
[2026-04-12T04:38:42Z TRACE clocks] HEAD http://404wolf.com recv 04:38:42.022916 server 04:38:42
[2026-04-12T04:38:42Z INFO  clocks] round 13: miss (±0ms window)
[2026-04-12T04:38:42Z TRACE clocks] HEAD http://404wolf.com send 04:38:42.999756
[2026-04-12T04:38:43Z TRACE clocks] HEAD http://404wolf.com send 04:38:43.000244
[2026-04-12T04:38:43Z TRACE clocks] HEAD http://404wolf.com recv 04:38:43.022893 server 04:38:43
[2026-04-12T04:38:43Z TRACE clocks] HEAD http://404wolf.com recv 04:38:43.026348 server 04:38:43
[2026-04-12T04:38:43Z INFO  clocks] round 14: miss (±0ms window)
[2026-04-12T04:38:43Z TRACE clocks] HEAD http://404wolf.com send 04:38:43.999879
[2026-04-12T04:38:44Z TRACE clocks] HEAD http://404wolf.com send 04:38:44.000123
[2026-04-12T04:38:44Z TRACE clocks] HEAD http://404wolf.com recv 04:38:44.021881 server 04:38:44
[2026-04-12T04:38:44Z TRACE clocks] HEAD http://404wolf.com recv 04:38:44.022200 server 04:38:44
[2026-04-12T04:38:44Z INFO  clocks] round 15: miss (±0ms window)
[2026-04-12T04:38:44Z TRACE clocks] HEAD http://404wolf.com send 04:38:44.999939
[2026-04-12T04:38:45Z TRACE clocks] HEAD http://404wolf.com send 04:38:45.000061
[2026-04-12T04:38:45Z TRACE clocks] HEAD http://404wolf.com recv 04:38:45.022412 server 04:38:45
[2026-04-12T04:38:45Z TRACE clocks] HEAD http://404wolf.com recv 04:38:45.023103 server 04:38:45
[2026-04-12T04:38:45Z INFO  clocks] round 16: miss (±0ms window)
[2026-04-12T04:38:45Z TRACE clocks] HEAD http://404wolf.com send 04:38:45.999970
[2026-04-12T04:38:46Z TRACE clocks] HEAD http://404wolf.com send 04:38:46.000031
[2026-04-12T04:38:46Z TRACE clocks] HEAD http://404wolf.com recv 04:38:46.022921 server 04:38:46
[2026-04-12T04:38:46Z TRACE clocks] HEAD http://404wolf.com recv 04:38:46.022940 server 04:38:46
[2026-04-12T04:38:46Z INFO  clocks] round 17: miss (±0ms window)
[2026-04-12T04:38:46Z TRACE clocks] HEAD http://404wolf.com send 04:38:46.999986
[2026-04-12T04:38:47Z TRACE clocks] HEAD http://404wolf.com send 04:38:47.000015
[2026-04-12T04:38:47Z TRACE clocks] HEAD http://404wolf.com recv 04:38:47.022887 server 04:38:47
[2026-04-12T04:38:47Z TRACE clocks] HEAD http://404wolf.com recv 04:38:47.022907 server 04:38:47
[2026-04-12T04:38:47Z INFO  clocks] round 18: miss (±0ms window)
[2026-04-12T04:38:47Z TRACE clocks] HEAD http://404wolf.com send 04:38:47.999994
[2026-04-12T04:38:48Z TRACE clocks] HEAD http://404wolf.com send 04:38:48.000007
[2026-04-12T04:38:48Z TRACE clocks] HEAD http://404wolf.com recv 04:38:48.022754 server 04:38:48
[2026-04-12T04:38:48Z TRACE clocks] HEAD http://404wolf.com recv 04:38:48.022757 server 04:38:48
[2026-04-12T04:38:48Z INFO  clocks] round 19: miss (±0ms window)
[2026-04-12T04:38:48Z TRACE clocks] HEAD http://404wolf.com send 04:38:48.999997
[2026-04-12T04:38:49Z TRACE clocks] HEAD http://404wolf.com send 04:38:49.000004
[2026-04-12T04:38:49Z TRACE clocks] HEAD http://404wolf.com recv 04:38:49.022140 server 04:38:49
[2026-04-12T04:38:49Z TRACE clocks] HEAD http://404wolf.com recv 04:38:49.022505 server 04:38:49
[2026-04-12T04:38:49Z INFO  clocks] round 20: miss (±0ms window)
http_clock_offset_ms=-42ms
╭─wolf at server in ~/Projects/csds426-clock-project on main✘✘✘
╰─±
