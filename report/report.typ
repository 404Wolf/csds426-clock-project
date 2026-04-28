#set page(
  margin: 0.7in,
  header: context {
    let page-num = counter(page).get().first()
    if page-num > 1 [
      #text(size: 10pt)[
        Internet scale clockdiff
        #h(1fr)
        #counter(page).display()
      ]
    ]
  },
)

#set text(font: "Liberation Serif", size: 11pt)

#set par(justify: true, leading: 0.58em)

#set heading(numbering: "1.")

#show heading.where(level: 1): it => [
  #v(0.8em)
  #text(weight: "bold", size: 12pt)[#it.body]
  #v(0.3em)
]

#show heading.where(level: 2): it => [
  #v(0.5em)
  #text(weight: "bold", style: "italic", size: 12pt)[#it.body]
  #v(0.2em)
]

#align(center)[
  #v(2em)
  #text(size: 16pt, weight: "bold")[Internet Scale Clockdiff]

  #v(0.5em)
  Wolf Mermelstein

  CSDS 426: Network Measurement and Analysis

  Case Western Reserve University

  Spring 2026
  #v(1.5em)
]

#align(center)[#text(weight: "bold")[Abstract]]

#par(first-line-indent: 0em)[
  Most internet-connected servers synchronize their clocks via NTP, yet the true distribution of clock offsets across the public internet remains largely unmeasured. We present two complementary techniques for estimating the clock difference of arbitrary hosts at scale: ICMP timestamp messages, surveyed across the entire IPv4 address space via a custom Zmap module, and a novel HTTP binary-search approach that infers clock offsets by probing the second boundary of the RFC 9110 `Date` response header. We optimize the HTTP method's parameters through an Optuna-based search against synthetic offsets. Applying both methods to over 150 million hosts and 15,000 Tranco-ranked sites, we find that the majority of clocks are accurate to within a second, but that some hosts are off by days, months, or years. Geographic distribution of offsets is surprisingly uniform, and we find no correlation between a site's popularity and the accuracy of its clock.
]

#v(1.5em)
#line(length: 100%)
#v(1em)

#set page(columns: 2)

= Introduction

In this paper we investigate a variety of techniques to try to determine how in sync the internet's clocks are. We take advantage of the ICMP timestamp feature, where you ask a server for its clock delta via ICMP messages, and also present a novel approach to try to infer the clock delta of HTTP servers by using the `Date` header in their responses. We run these approaches against a variety of hosts on the internet, and find that, while most hosts are pretty in sync, there are a non-trivial number of hosts that are very out of sync. We also present interesting findings like that geographic distribution of clock offsets is fairly uniform, and that there is no correlation between how "popular" a site is and how well maintained its clock is.

We structure our investigation around the following research questions:

- *RQ1:* How widely do IPv4 hosts support ICMP timestamp messages, and how does prevalence compare to ICMP echo (ping) support?
- *RQ2:* How accurate are internet clocks at scale, and what fraction of hosts exhibit large offsets?
- *RQ3:* Does geographic region predict clock accuracy?
- *RQ4:* Does a site's popularity (Tranco rank) correlate with clock accuracy?
- *RQ5:* Can the HTTP `Date` header be exploited to measure clock offsets with sub-second precision, and how does it compare to ICMP timestamp?

= Background & Related Work

== Getting in Sync

These days, most clocks on the internet synchronize their time using NTP (the Network Time Protocol) @mills1991ntp. NTP works by having clients connect to well-known servers with accurate times, using a simple clock diffing algorithm to compute a difference, which they use to adjust their time accordingly. NTP works great, but only really lets the host know how off its clock is. To measure how in sync the internet's clocks are, we need a way to get an approximation of the clock difference between an arbitrary client and server.

== ICMP Timestamp

#figure(image("figures/icmp_clockdiff.svg"))

ICMP is a subprotocol of IP used primarily for network diagnostics @rfc792; one of the diagnostic messages that the ICMP protocol supports is the ICMP timestamp message.

The ICMP timestamp message is a kind of ICMP message designed to allow a host to approximate the clock difference between itself and another host @rfc792. ICMP timestamp messages are very simple -- you send a request that includes your timestamp from when you sent the message, $T_1$, and then the server responds with a message that stamps on the timestamp it received the request, $T_2$, and the timestamp it retransmitted its response, $T_3$. Finally, you record the time you received the reply, $T_4$. You can then find the RTT (excluding server processing time), and use that to compute the clock difference:

$
  "rtt" & = (T_4 - T_1) - (T_3 - T_2) \
  Delta & = T_2 - (T_1 + "rtt" / 2)
$

The RFC also specifies that "if the time is not available in milliseconds or cannot be provided with respect to midnight UT then any time can be inserted in a timestamp provided the high order bit of the timestamp is also set to indicate this non-standard value." In my measurements I find that 7.6% of all hosts flip this bit. I take care to exclude entries that flip this bit in analyzing my results for clock synchronization purposes.

== HTTP Date Header

The HTTP clockdiff extraction algorithm described in @sec:methodology relies on RFC 9110 (@rfc9110). This RFC requires any HTTP server with a clock to include a `Date` header in all 2xx, 3xx, and 4xx responses, which an approximation of the server's date. The exact format that the `Date` header comes in is always of the form `Date: Tue, 15 Nov 1994 08:12:31 GMT` -- with second level precision and no fractional seconds.

The RFC permits servers to timestamp the response at any point before it is sent. I considered, to reduce this slack, sending a deliberately malformed request, reasoning that less server-side processing would tighten the timestamp; however, the RFC permits servers to omit the `Date` header on 500 responses. A separate caveat is that proxies are required to inject a `Date` header if one is absent -- this is a risk I accept #footnote[An investigation of popular frameworks confirms that this injection behavior is widespread: Express.js, Uvicorn, and reverse proxies such as Nginx all append a Date header if the application has not already done so. Further work should be done to repeat my measurements and properly profile for proxy usage in requests, and to determine how often application frameworks timestamp responses.].

The issue with this header it is only a low quality approximation; I seek a very precise timestamp since I hypothesize most servers are not off by over a second. To "infer" more accuracy using this header, I pay careful attention to the bleeding edge of the second. I have found that in many modern HTTP implementations they consistently floor the time to produce the timestamp, rounding down the fractional seconds, which is important for @sec:methodology.

= Methodology <sec:methodology>

== ICMP Timestamp Measurement

#figure(
  image("figures/icmp-echo-vs-clockdiff.png"),
  caption: [Distribution of ICMP echo and timestamp support across responsive IPv4 hosts. Percentages are relative to all hosts that responded to either probe (~375 million total). The majority (58.9%) respond to ping only and do not support ICMP timestamps. I find it highly interesting that there exists hosts (which I have manually verified) that *only* support ICMP timestamp messages and do *not* support ICMP echos.],
) <fig:icmp-echo-vs-clockdiff>

In most cases, your distro will disable ICMP timestamp messages by default, because it can be used for clock fingerprinting, and is generally not very useful. The main reason you would want to compute the delta of two clocks is for when you are syncing your clock, but for this you are more likely to just use NTP. This paper seeks to find good approximations of the clock difference between a client and server, and it is of value to know how, today, well supported this feature of ICMP is.

To this end, I created a Zmap module that attempts to do ICMP clockdiff measurements to every IPV4 host, which I produced by taking the ICMP echo module and modifying it to send ICMP timestamp messages instead; I also run the ICMP echo Zmap module to build a frame of reference @durumeric2013zmap. I was surprised by how many hosts on the internet publicly support this antiquated ICMP feature. I ran these Zmap scans on a Amsterdam based ClientVPS server. My Zmap module collects all the data that ICMP timestamp messages offer, so that I can compute rtts and clock deltas. I also notate hosts that flip the high bit of the timestamp, indicating that the timestamp is not actually a timestamp.

// TODO: insert results here

== HTTP Boundary Measurement

To get around the second-granularity limitation of the HTTP date header, I introduce a binary search based algorithm around the bleeding edge of the second. By sending many HTTP requests just around the transition point, finding the first request that sees the next second, combining what we learn about the round trip time, the true time that we know when we send a request, and the time that they report when they receive the request, we can come to a very good approximation of our clock difference. I spell out this approach in more detail below.

// TODO: I should try other HTTP methods
// For all of my requests, I hit the root with a `HEAD https://case.edu/?q=16122764704206209229`, . I observed that some hosts (for example, vanderbilt.edu), will cache the date header for a few seconds, leading to 20-30 second gaps where the date is frozen. This violates the RFC since the `Date` header is not a good approximation of the real clock of the server, but it still occurs for many hosts in the wild, and it seems that a random query string is enough to avoid cache hits. I use the `HEAD` HTTP method since I do not care about/require an actual HTTP payload. I considered requesting with phony methods like `FOOBAR` to trigger a almost guarantee error response across hosts, but a `Date` header is only required by the RFC to be included on 200/300/400 series responses, and such a request would trigger a 501 (or, at the least, likely a 500 series response).

My approach requires a lot of requests in a very small time frame, so it is worth taking some steps to try to not unnecessarily overwhelm our targets. I begin with a single HTTP request to get the second resolution reading and rtt. If it is more than 5 seconds off I do not run further measurements, and assume their timestamp is not accurate enough to be worth measuring more accurately. We record the time difference that we got for this sanity check as the "clock difference," with the asterisk that it is in second level granularity.

I then sleep until the next precise second boundary, and then send many evenly spaced requests up until some duration past the second boundary. We have control over how tightly spaced these initial requests are by virtue of how far around the second boundary we stride #footnote[For example, you could do 5 requests spaced 1 second apart, where we would send at :21, :22, *:23*, :24, and :25, where :23 is the second boundary we are concerned with. Or, if you make the parameter tighter, we may send requests at :50.50, :50.75, *:51:00*, :51:25, and :51:50].

// We can always, with enough requests, get a good and accurate estimate of the server's second boundary, unless there is too much network jitter; however, there's a tradeoff here. In order to observe a clock being off by $n$ seconds, we have to have a half boundary of at least $n/2$. We could use a range like 100 seconds, and just send a very high sustained number of requests, but this would both not be respectful to the host, and also not be necessary for the majority of hosts, since most hosts are not that off. This is where the binary search comes in.

// TODO: analysis of what % is frozen

For each of the requests I sent, the server is going to respond with its time with second floor resolution #footnote[Analysis of many common libraries and reverse proxies show that this is flooring is basically always the rounding approach, but there might be exception.], and it will (hopefully) be different for one of my requests. #footnote[If the server's time is the same for all 5 requests, that means that their clock is "frozen" since it was the same for over a full second, assuming that I covered a large enough range with my requests]. Then I can analyze the response, and found the boundary pair. Let $t_"send"^a, t_"recv"^a, t_"srv"^a$ be the send, receive, and server timestamps from the "after" probe, and $t_"send"^b, t_"recv"^b$ the corresponding timestamps from the "before" probe. My approach for computing the clock difference is basically identical to that of ICMP's clockdiff #footnote[I caution that I am assuming a uniform round trip, where the amount of time it takes for my request to reach the server is the same as the amount of time it takes for their response to reach me. This might not be true, but this is an assumption that we have to make, and is also an assumption that both ICMP clockdiff and NTP also make.]:

$
  "rtt" & = ((t_"recv"^a - t_"send"^a) + (t_"recv"^b - t_"send"^b)) / 2 \
  Delta & = t_"srv"^a - (t_"send"^a + "rtt" / 2)
$

This is not enough: if I send requests over the course of 1 second, and I send 10 requests, then that means I'm only getting `100ms` of accuracy. To make our measurements more accurate, I proceed with a binary search. I apply the approach as defined above, where the "center" is *my* second boundary, and scan an even range around this boundary. But now, I find the transition point, which, depending on how off their clock is, may not actually be the second boundary. I define the new center as this transition point. I now repeat the same approach, but change my radius around the center to be some fraction of what it was previously (so, instead of scanning a 0.5 second window around the boundary, I scan a 0.25 second window around a slightly different, but probably more accurate boundary of when their transition point is). For a clear illustration of this process I encourage the reader to consult @fig:binary-search.

#figure(
  placement: top,
  scope: "parent",
  image("figures/measurement_experiment_offset.png"),
  caption: [How much "error" we experience at different synthetic offsets locally in the Eastern US and when diffing against a server in Sydney using our "golden" parameters laid out in @fig:ideal-params. Impressively, we observe an error p90 as low as 25.2ms for local hosts. Most hosts are equally good in Sydney, but there are many more outliers.],
) <fig:measurement-experiment-offset>

My implementation of this algorithm takes the form of a simple Rust binary, since I wanted to preserve control of request timing #footnote[All code and analysis scripts are available at #link("https://github.com/404Wolf/csds426-clock-project").]. It sets up a thread pool where the threads spin up until slight offsets very close to the second boundary, and then share their results. We sort the results, find the boundary pair, and then repeat the process. I also implement a "best-of" approach, where I run multiple rounds of this binary search, and then take the best (smallest clock difference) result, which helps to mitigate the effects of network jitter.

Before running on the general internet, I set up a controlled environment to run tests. I designed a simple multithreaded "fake" Rust HTTP server that responds to requests with a synthetic `Date` header. When you request `http://myhost/0.1` the response includes a `Date` header that has the system time with an artificial `100ms` (based on the path) delay. Then I wrote a basic Python script that uses Optuna @akiba2019optuna (an optimization library) to attempt to find the ideal parameters for my algorithm -- how many rounds of binary search to run, how many probes to send per round, how long the half span should be for the first round, how much to shrink the half span by every round, and what the initial minimum half span should be. I spun up a server in Ohio, Atlanta, Chicago, and Sydney (a location with high network latency from the east coast). The Ohio server ran my Optuna optimizer script, and the other servers hosted the fake HTTP servers. For every round of my optimizer, I ran with the artificial offsets of -1s, -0.1s, -0.01s, 0s, 0.01s, 0.1s, and 1s, and, since I knew what the algorithm should return, computed to be the error to be the divergence from that value. These servers were all synced via NTP with Chrony so that their "real" times were all accurate. I ran this optimization against my fake HTTP server, and, after >1000 trials found that the parameters in @fig:ideal-params worked best. I also provide @fig:measurement-experiment-offset, which shows how the "synthetic offset," as expected, does not have a substantial effect on the error rate, and also demonstrates that the algorithm is generally robust to large round trip times, using our new "golden" parameters.

#figure(
  placement: top,
  table(
    columns: 2,
    align: (left, right),
    inset: 4pt,
    stroke: (x: 0.5pt, y: none),

    table.hline(),
    table.header([*Parameter*], [*Value*]),
    table.hline(),

    [Rounds], [17],
    table.hline(),
    [Probes per round], [18],
    table.hline(),
    [Initial half-span ($mu$s)], [1,750,000],
    table.hline(),
    [Minimum step ($mu$s)], [4,300],
    table.hline(),
    [Shrink factor], [5],
    table.hline(),
    [Best-of], [2],
    table.hline(),
  ),
  caption: [Ideal HTTP clockdiff binary search parameters found using an Optuna search.],
) <fig:ideal-params>

=== Caching

#figure(
  placement: top,
  table(
    columns: 3,
    align: (right, right, right),
    inset: 4pt,
    stroke: (x: 0.5pt, y: none),

    table.hline(),
    table.header([*offset (µs)*], [*server (:mm:ss)*], [*send\_at (:mm:ss.µs)*]),
    table.hline(),

    [-16,000], [:16:41], [:18:05.984000],
    [⋮], [⋮], [⋮],
    [1,000], [:16:41], [:18:06.001001],
    table.hline(),
    [-215,000], [:18:05], [:18:05.785001],
    [-207,000], [:18:05], [:18:05.793001],
    table.hline(),
  ),
  caption: [Probe data from `vanderbilt.edu` showing that the server reports `23:16:41` for every request until the cache expires, then correctly reports `23:18:05`. The apparent offset of ~84 seconds is entirely due to caching, not a clock error.],
) <tbl:vanderbilt>

// TODO: caching of the date header research

I observed in my data collection that some HTTP servers aggressively cache requests, including the date header. This was the case for Vanderbilt's server, where we can clearly see that, if you hit the same Cloudfront domain many times, the `Date` header is frozen. @tbl:vanderbilt shows a real probe session -- every request between `23:18:05` and `23:18:06` receives a `Date` of `23:16:41` -- roughly 84 seconds in the past -- before suddenly snapping to the correct second. In this specific case, we see the `x-cache: RefreshHit from cloudfront` header, but if we hit `vanderbilt.edu?q=757113` we get `x-cache: Miss from cloudfront`, which corroborates this. To avoid the effects of caching, I include a random query string in my requests. I note that the caching of the date header is a violation of the RFC and interesting network behavior that could be studied in the future.

= Data collection

Now that I have developed two approaches for accumulating clock delta data, there were a few different sets of data I sought to collect.

// TODO: figure showing rank vs clock diff

I began by running a Zmap of the IPv4 space to find all hosts supporting ICMP timestamp messages; I ran this scan twice and observed high overlap between the two runs. I then iterated over all of those hosts to see which of them also supported HTTP and returned a `Date` header. I then ran my binary search approach against all of the hosts that supported both ICMP timestamp messages and HTTP with a `Date` header, to get a more accurate estimate of their clock difference, and to better judge my algorithm. Finally, I ran my binary search approach against an evenly distributed list of roughly 15,000 hosts on the Tranco list to get a sense of how well maintained the clocks of popular sites are, which felt like a good target since it is a list of hosts that we know ahead of time all speak HTTP. I ran this scan twice — first against a 1,000-host sample using the golden parameters, and then against the full 15,000-host sample — and the two runs produced consistent. I expected there to be a correlation where hosts with higher ranks would have more accurate clocks, but I found no such correlation. At the end, I took all of my collected data and enriched every IP with its associated geographic location using MaxMind's database.

== Ethical Considerations

There are three components to consider the ethics of: the internet-wide Zmap scan for ICMP, my subsequent HTTP scanning, and my internal tuning. I ran my Zmap at full speed, but naturally took advantage of Zmap's automatic randomization to evenly distribute the load I put onto the internet, so the actual effect should not have been very significant to any given host. My HTTP scan was much more intensive -- I would do 17 back to back rounds of bursts of 18 probes. However, these requests were `HEAD` requests, and because these bursts came with 1 second pauses between them, I think the load is much more reasonable; I also feel 18 requests is a typical load for a web server. My individual parameter tuning has no ethical concerns, since it was done on datacenter networks with an overall a small request load and I was only hitting my own hosts.

= Results & Analysis

=== General algorithm accuracy

#figure(
  placement: top,
  scope: "parent",
  image("figures/clock-accuracy-method-comparison.png"),
) <fig:clock-accuracy-comparison>

=== Is the world in sync?

#figure(
  placement: top,
  scope: "parent",
  table(
    columns: 4,
    align: (left, right, right, right),
    inset: 4pt,
    stroke: (x: 0.5pt, y: none),

    table.hline(),
    table.header([*Scan*], [*Hosts*], [*>5s off*], [*%*]),
    table.hline(),

    [ICMP (via ICMP)], [157,929,504], [58,223,197], [36.87%],
    table.hline(),
    [HTTP + ICMP (via ICMP)], [7,882], [1,894], [24.03%],
    table.hline(),
    [HTTP + ICMP (via HTTP)], [7,860], [1,236], [15.73%],
    table.hline(),
    [Tranco (via HTTP)], [14,938], [427], [2.86%],
    table.hline(),
  ),
  caption: [Fraction of hosts with clocks more than 5 seconds off, broken down by scan dataset and measurement method. HTTP + ICMP is the subset of ICMP hosts that also returned an HTTP Date header, measured independently by both methods (7,882 had a valid ICMP offset; 7,860 had a valid HTTP offset — 22 hosts returned no HTTP response). Among the overlapping hosts, HTTP clocks are better maintained than ICMP clocks, and Tranco sites are far more accurate than the general HTTP population. Many of the worst offenders cluster near +4,224,000,000 ms (~48.9 days), just under the 32-bit millisecond rollover point of $2^{32}$ ms ≈ 49.7 days. For example, `89.86.248.14` (France, +48.9 days) and `211.116.45.165` (South Korea, +48.9 days) are likely returning a raw uptime counter as their ICMP timestamp. In the HTTP data, `41.70.48.34` (Malawi) reports +44 years via HTTP yet only ~3 minutes via ICMP, suggesting its HTTP stack is emitting a garbage timestamp rather than the clock truly being wrong. On the Tranco list, `ais.de` stands out at +78 days off — another likely overflow artifact.],
)

#figure(image("figures/geo_plot.png"), caption: [A geographic heat map of how accurate clocks are.]) <fig:geo-plot>

#figure(
  placement: bottom,
  image("figures/internet_clock_desync.png"),
  caption: [Box plots indicating how generally in sync clocks are for all IPv4 hosts on the internet that support ICMP timestamp messages (left), all hosts on the Tranco list that emit a http `Date` header (middle), and all ICMP timestamp supporting hosts that also happen to host a web server that includes a `Date` header. We note that theoretically the ICMP timestamp supporting servers should agree on the clock desync for HTTP and ICMP, but that in practice the HTTP results are much noisier for these hosts than even those on the Tranco list.],
) <fig:internet-clock-desync>

My overall findings were that, all things considered, most clocks are relatively in sync. One can get a sense by consulting @fig:geo-plot that the distribution of clock offsets across the world is fairly uniform, with America having slightly more consistently accurate clocks. Additionally, @fig:internet-clock-desync shows that the majority of hosts have a clock offset within a second, and that the P90 is 9-15ms #footnote[Except for when testing ICMP hosts via HTTP, in which case it's higher. I reason that it is because these servers are generally older and slower, and do not handle my HTTP burst technique as well.]. However, there are a non-trivial number of hosts with very large offsets -- on the order of days, months, or even years -- which is surprising to me.

To gauge how effective my HTTP approach is, I compare the HTTP clock difference to the ICMP clock difference for the hosts that support both. Looking at @fig:clock-accuracy-comparison, you can compare the distribution of offsets that I found for the two techniques. They both concentrate around 0 -- where the clock of my server is in sync -- but my HTTP algorithm diverges more, indicating that it is not quite as accurate as the ICMP clockdiff approach. Note that these are the same hosts, so theoretically both approaches should entirely overlap. #footnote[Anecdotally, this difference demolishes the more trials you run of my HTTP algorithm. I also have noticed that the algorithm did particularly poorly for the ICMP-timestamp-supporting hosts.]

=== Tranco list correlation

#figure(
  image("figures/tranco_vs_http_clock_offset.png"),
  caption: [A scatter plot showing the relationship between a site's rank on the Tranco list and how accurate its clock is using my HTTP clockdiff algorithm. These top 1M websites have relatively accurate clocks -- older ICMP timestamp supporting hosts have worse clock accuracy.],
) <fig:tranco-list-correlation>

Something that I had hypothesized going in to this project was that "more active" hosts on the internet would probably have more accurate clocks. One idea that I had to test this, armed with my new HTTP clock diffing algorithm, was to run my algorithm against an even distribution of the Tranco 1M list (roughly 15,000 hosts), and see if there is any correlation between your spot on the list and your clock offset. I ran this experiment, and, as evidenced by @fig:tranco-list-correlation, I did not observe any such correlation. In general everyone's clock seems to be relatively in sync, or off by a relatively fixed amount (within the P90 of my algorithm).

After conducting all of my experiments, I came to various interesting conclusions:

// TODO: insert country findings
- Most hosts fall into two buckets. Either their clock is very in sync (within a second), or they are very out of sync (are days, months, or more out of sync).
- The geographic distribution of ICMP clockdiff based time offsets is fairly uniform. I observe that no region stands out as dramatically better or worse synchronized than any other, though North American hosts trend slightly more accurate -- likely a reflection of higher NTP adoption and more reliable network infrastructure rather than a fundamental geographic factor.
- A non-trivial fraction of hosts with large offsets cluster near +48.9 days, just under the 32-bit millisecond rollover point ($2^32$ ms $approx$ 49.7 days). Among the 11.7 million hosts that set the nonstandard timestamp bit, 10.3% fall in this window -- strongly suggesting they return a raw uptime counter rather than a wall-clock time.

= Next steps

There's various different ideas and additional measurements I would like to experiment with

- NTP servers make a good de facto source of truth for time. I would like to run a Zmap scan to attempt to find all NTP servers that also HTTP and ICMP clockdiff servers; allowing me to run my analysis on them and compare accuracy against a more definative source of truth.
- Likewise, I would be interested in hosting many NTP servers and collecting data about how out-of-sync people who connect to my servers are. This would make an interesting passive approach to testing how desynced the internet's clocks are.
- Through my experiments I learned that some hosts cache the HTTP date header. I feel it would be interesting to do further analysis to determine how common this is, and what other forms of non-RFC compliant caching go on.
- My experiments are only measuring the absolute clock difference between servers, but it could also be interesting to measure clock skew; for example, by repeating my HTTP measurements over a longer period of time.
- We can further investigate what *kinds* of hosts are prone to having less accurate clocks. It would be interesting to attempt to do OS profiling on hosts to measure what types of hosts have bad clocks.

= Conclusion

We have presented two complementary methods for estimating clock offsets at internet scale: an ICMP timestamp scan of the full IPv4 space and a novel HTTP binary search algorithm that exploits second-boundary transitions in the HTTP `Date` header. Across more than 150 million ICMP-responsive hosts and roughly 15,000 Tranco-ranked sites, we find that the majority of internet clocks are accurate to within a second. However, a meaningful minority -- especially among the general IPv4 population -- exhibit extreme offsets. Geographic distribution and site popularity are both poor predictors of clock accuracy. Our HTTP method, while less precise than ICMP, makes clock measurement on any HTTP-speaking host possible, opening the door to testing clock accuracy on a much wider variety of hosts.

#set page(columns: 1)

#bibliography("sources.bib", full: true)

= Appendix

#figure(
  table(
    columns: 9,
    align: (right, right, right, left, left, right, right, right, left),
    inset: 4pt,
    stroke: (x: 0.5pt, y: none),

    table.hline(),
    table.header(
      [*round*],
      [*req*],
      [*offset (µs)*],
      [*send (:ss.µs)*],
      [*recv (:ss.µs)*],
      [*Δsend (µs)*],
      [*Δrecv (µs)*],
      [*rtt (µs)*],
      [*server (:ss)*],
    ),
    table.hline(),

    [1], [1], [-1300000], [:43.84390], [:44.00766], [], [], [163762], text(fill: gray)[:44],
    [1], [2], [-780000], [:43.84391], [:44.04899], [6], [41333], [205089], text(fill: gray)[:44],
    [1], [3], [-260000], [:43.84392], [:44.05018], [10], [1190], [206269], text(fill: gray)[:44],
    [1], [4], [260000], [:44.26000], [:44.40176], [416080], [351573], [141762], text(fill: gray)[:44],
    [1], [5], [780000], [:44.78000], [:44.94158], [520001], [539820], [161581], text(fill: black)[:44],
    [1], [6], [1300000], [:45.30000], [:45.46620], [520000], [524619], [166200], text(fill: black)[:45],
    table.hline(),

    [2], [1], [390000], [:46.39000], [:46.53853], [], [], [148537], text(fill: gray)[:46],
    [2], [2], [650000], [:46.65000], [:46.81259], [259999], [274055], [162593], text(fill: black)[:46],
    [2], [3], [910000], [:46.91000], [:47.05056], [260001], [237968], [140560], text(fill: black)[:47],
    [2], [4], [1170000], [:47.17000], [:47.33422], [259999], [283667], [164228], text(fill: gray)[:47],
    [2], [5], [1430000], [:47.43000], [:47.56804], [260000], [233814], [138042], text(fill: gray)[:47],
    [2], [6], [1690000], [:47.69000], [:47.83335], [260000], [265309], [143351], text(fill: gray)[:47],
    table.hline(),

    [3], [1], [455000], [:48.45500], [:48.59639], [], [], [141395], text(fill: gray)[:48],
    [3], [2], [585000], [:48.58500], [:48.75231], [129999], [155918], [167314], text(fill: gray)[:48],
    [3], [3], [715000], [:48.71500], [:48.85443], [130000], [102121], [139435], text(fill: gray)[:48],
    [3], [4], [845000], [:48.84500], [:48.98600], [130000], [131569], [141004], text(fill: black)[:48],
    [3], [5], [975000], [:48.97500], [:49.11442], [130000], [128425], [139429], text(fill: black)[:49],
    [3], [6], [1105000], [:49.10500], [:49.24275], [130000], [128329], [137758], text(fill: gray)[:49],
    table.hline(),

    [4], [1], [747500], [:50.74750], [:50.89059], [], [], [143091], text(fill: gray)[:50],
    [4], [2], [812500], [:50.81250], [:50.95861], [65000], [68025], [146116], text(fill: black)[:50],
    [4], [3], [877500], [:50.87750], [:51.05262], [65000], [94008], [175124], text(fill: black)[:51],
    [4], [4], [942500], [:50.94250], [:51.10865], [65000], [56030], [166154], text(fill: gray)[:51],
    [4], [5], [1007500], [:51.00750], [:51.14433], [65000], [35680], [136834], text(fill: gray)[:51],
    [4], [6], [1072500], [:51.07250], [:51.21066], [65001], [66326], [138159], text(fill: gray)[:51],
    table.hline(),

    [5], [1], [763750], [:52.76375], [:52.90606], [], [], [142310], text(fill: gray)[:52],
    [5], [2], [796250], [:52.79625], [:52.94402], [32500], [37966], [147776], text(fill: gray)[:52],
    [5], [3], [828750], [:52.82875], [:52.97157], [32501], [27551], [142826], text(fill: black)[:52],
    [5], [4], [861250], [:52.86125], [:53.03700], [32499], [65428], [175755], text(fill: black)[:53],
    [5], [5], [893750], [:52.89375], [:53.03470], [32500], [-2296], [140959], text(fill: gray)[:53],
    [5], [6], [926250], [:52.92625], [:53.13274], [32500], [98031], [206490], text(fill: gray)[:53],
    table.hline(),
  ),
  caption: [Raw probe data from 5 rounds of binary search against case.edu (all times UTC 16:18). Probes within each round are sorted by server-reported second; Δsend and Δrecv are the gap from the previous probe. The boundary pair (black) is where the server second ticks over.],
) <fig:binary-search>
