#set page(
  margin: 1in,
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
#set text(font: "Liberation Serif", size: 12pt)
#set par(justify: true, leading: 1.5em)
#set heading(numbering: none)

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
  These days, most clocks use NTP. But how accurate are they? And how do different kinds of hosts fair to NTP? The internet has a lot of old devices, and it's inevitable that not every server does proper clock syncing.
]

#v(1.5em)
#line(length: 100%)
#v(1em)

#columns(2, gutter: 0.25in)[

  = Introduction

  We investigate a variety of techniques to try to determine, in general, how in sync the internet's clocks are. To do this, we take advantage

  #lorem(120)

  = Background & Related Work

  == Network Time Protocol and Clock Synchronization

  #lorem(80)

  == ICMP Timestamp

  #lorem(70)

  == HTTP Date Header

  + I send an HTTP request and see what their date time is. If it is more than 5 seconds off, I do not run further measurements, and we just assume that their timestamp is not accurate enough that it is not worth it.
  + Then I spam 5 HTTP requests just around the second boundary attempting to capture a second boundary.
  + I send 5 evenly spaced requests around the second boundary. When my time is :50.5, :50.75, :51:00, and :51:25 I send requests. Each of these requests has a time we sent the request at, the time we received a HTTP response at, and the server's time.
  + For each of the requests I sent, the server is going to respond with its time with second floor resolution, and it will (hopefully) be different for one of my requests. #footnote[If the server's time is the same for all 5 requests, that means that their clock is "frozen" since it was the same for over a full second].
  + Then I

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

      [ICMP (via ICMP)],          [157,929,504], [58,223,197], [36.87%],
      table.hline(),
      [HTTP + ICMP (via ICMP)],   [7,882],       [1,894],      [24.03%],
      table.hline(),
      [HTTP + ICMP (via HTTP)],   [7,860],       [1,236],      [15.73%],
      table.hline(),
      [Tranco (via HTTP)],        [14,938],      [427],        [2.86%],
      table.hline(),
    ),
    caption: [Fraction of hosts with clocks more than 5 seconds off, broken down by scan dataset and measurement method. HTTP + ICMP is the subset of ICMP hosts that also returned an HTTP Date header, measured independently by both methods (7,882 had a valid ICMP offset; 7,860 had a valid HTTP offset — 22 hosts returned no HTTP response). Among the overlapping hosts, HTTP clocks are better maintained than ICMP clocks, and Tranco sites are far more accurate than the general HTTP population.],
  )

  === Caching

  One trait of some HTTP servers is that they will aggressively cache requests, including the date header. This was the case for Vanderbilt's server, where we can clearly see that, if you hit the same cloudfront domain many times

  #lorem(60)

  == Related Measurement Work

  #lorem(70)

  = Methodology

  == ICMP Timestamp Measurement

  #lorem(90)

  == HTTP Date Header Measurement

  #lorem(90)

  == Enrichment Pipeline

  #lorem(80)

  == Ethical Considerations

  #lorem(70)

  = Results & Analysis

  == Protocol Support

  #lorem(80)

  == Clock Offset Distributions

  #lorem(90)

  == ICMP vs. HTTP Comparison

  #lorem(80)

  == Geographic Analysis

  #lorem(80)

  = Discussion

  #lorem(130)

  = Conclusion

  #lorem(100)

  = References

  #set par(hanging-indent: 0.5in, first-line-indent: 0em)

  Cooper, D., Santesson, S., Farrell, S., Boeyen, S., Housley, R., & Polk, W.
  (2008). _Internet X.509 public key infrastructure certificate and certificate
  revocation list (CRL) profile_ (RFC 5280). Internet Engineering Task Force.
  #link("https://www.rfc-editor.org/rfc/rfc5280")

  Durumeric, Z., Wustrow, E., & Halderman, J. A. (2013). ZMap: Fast internet-wide
  scanning and its security implications. In _Proceedings of the 22nd USENIX
  Security Symposium_ (pp. 605–620). USENIX Association.

  Fielding, R., & Reschke, J. (2014). _Hypertext transfer protocol (HTTP/1.1):
  Semantics and content_ (RFC 7231). Internet Engineering Task Force.
  #link("https://www.rfc-editor.org/rfc/rfc7231")

  MaxMind. (2024). _GeoLite2 databases_. MaxMind, Inc.
  #link("https://www.maxmind.com/en/geoip-databases")

  Mills, D. L. (1991). Internet time synchronization: The network time protocol.
  _IEEE Transactions on Communications_, _39_(10), 1482–1493.
  #link("https://doi.org/10.1109/26.103043")

  Pochat, V. L., Van Goethem, T., Tajalizadehkhoob, S., Korczynski, M., & Joosen,
  W. (2019). Tranco: A research-oriented top sites ranking hardened against
  manipulation. In _Proceedings of the Network and Distributed System Security
  Symposium (NDSS)_.
  #link("https://doi.org/10.14722/ndss.2019.23386")

  Postel, J. (1981). _Internet control message protocol_ (RFC 792). Internet
  Engineering Task Force.
  #link("https://www.rfc-editor.org/rfc/rfc792")

  Schulman, A., & Spring, N. (2011). Pingin' in the rain. In _Proceedings of the
  2011 ACM SIGCOMM Internet Measurement Conference_ (pp. 19–28). ACM.
  #link("https://doi.org/10.1145/2068816.2068819")

  Shulman, H., & Waidner, M. (2016). One key to sign them all considered
  vulnerable: Evaluation of DNSSEC in the internet. In _Proceedings of the 13th
  USENIX Symposium on Networked Systems Design and Implementation_ (pp. 131–144).
  USENIX Association.

  Veitch, D., Barreto, S., & Ridoux, J. (2009). A foundation for the accurate
  measurement of infrastructure metrics. _IEEE/ACM Transactions on Networking_,
  _17_(5), 1368–1381.
  #link("https://doi.org/10.1109/TNET.2008.2007945")

]

= Appendix

#figure(
  table(
    columns: 9,
    align: (right, right, right, left, left, right, right, right, left),
    inset: 4pt,
    stroke: (x: 0.5pt, y: none),

    table.hline(),
    table.header([*round*], [*req*], [*offset (µs)*], [*send (:ss.µs)*], [*recv (:ss.µs)*], [*Δsend (µs)*], [*Δrecv (µs)*], [*rtt (µs)*], [*server (:ss)*]),
    table.hline(),

    [1], [1], [-1300000], [:43.843904], [:44.007666], [],        [],        [163762], text(fill: gray)[:44],
    [1], [2], [-780000],  [:43.843910], [:44.048999], [6],       [41333],   [205089], text(fill: gray)[:44],
    [1], [3], [-260000],  [:43.843920], [:44.050189], [10],      [1190],    [206269], text(fill: gray)[:44],
    [1], [4], [260000],   [:44.260000], [:44.401762], [416080],  [351573],  [141762], text(fill: gray)[:44],
    [1], [5], [780000],   [:44.780001], [:44.941582], [520001],  [539820],  [161581], text(fill: black)[:44],
    [1], [6], [1300000],  [:45.300001], [:45.466201], [520000],  [524619],  [166200], text(fill: black)[:45],
    table.hline(),

    [2], [1], [390000],   [:46.390001], [:46.538538], [],        [],        [148537], text(fill: gray)[:46],
    [2], [2], [650000],   [:46.650000], [:46.812593], [259999],  [274055],  [162593], text(fill: black)[:46],
    [2], [3], [910000],   [:46.910001], [:47.050561], [260001],  [237968],  [140560], text(fill: black)[:47],
    [2], [4], [1170000],  [:47.170000], [:47.334228], [259999],  [283667],  [164228], text(fill: gray)[:47],
    [2], [5], [1430000],  [:47.430000], [:47.568042], [260000],  [233814],  [138042], text(fill: gray)[:47],
    [2], [6], [1690000],  [:47.690000], [:47.833351], [260000],  [265309],  [143351], text(fill: gray)[:47],
    table.hline(),

    [3], [1], [455000],   [:48.455001], [:48.596396], [],        [],        [141395], text(fill: gray)[:48],
    [3], [2], [585000],   [:48.585000], [:48.752314], [129999],  [155918],  [167314], text(fill: gray)[:48],
    [3], [3], [715000],   [:48.715000], [:48.854435], [130000],  [102121],  [139435], text(fill: gray)[:48],
    [3], [4], [845000],   [:48.845000], [:48.986004], [130000],  [131569],  [141004], text(fill: black)[:48],
    [3], [5], [975000],   [:48.975000], [:49.114429], [130000],  [128425],  [139429], text(fill: black)[:49],
    [3], [6], [1105000],  [:49.105000], [:49.242758], [130000],  [128329],  [137758], text(fill: gray)[:49],
    table.hline(),

    [4], [1], [747500],   [:50.747500], [:50.890591], [],        [],        [143091], text(fill: gray)[:50],
    [4], [2], [812500],   [:50.812500], [:50.958616], [65000],   [68025],   [146116], text(fill: black)[:50],
    [4], [3], [877500],   [:50.877500], [:51.052624], [65000],   [94008],   [175124], text(fill: black)[:51],
    [4], [4], [942500],   [:50.942500], [:51.108654], [65000],   [56030],   [166154], text(fill: gray)[:51],
    [4], [5], [1007500],  [:51.007500], [:51.144334], [65000],   [35680],   [136834], text(fill: gray)[:51],
    [4], [6], [1072500],  [:51.072501], [:51.210660], [65001],   [66326],   [138159], text(fill: gray)[:51],
    table.hline(),

    [5], [1], [763750],   [:52.763750], [:52.906060], [],        [],        [142310], text(fill: gray)[:52],
    [5], [2], [796250],   [:52.796250], [:52.944026], [32500],   [37966],   [147776], text(fill: gray)[:52],
    [5], [3], [828750],   [:52.828751], [:52.971577], [32501],   [27551],   [142826], text(fill: black)[:52],
    [5], [4], [861250],   [:52.861250], [:53.037005], [32499],   [65428],   [175755], text(fill: black)[:53],
    [5], [5], [893750],   [:52.893750], [:53.034709], [32500],   [-2296],   [140959], text(fill: gray)[:53],
    [5], [6], [926250],   [:52.926250], [:53.132740], [32500],   [98031],   [206490], text(fill: gray)[:53],
    table.hline(),
  ),
  caption: [Raw probe data from 5 rounds of binary search against case.edu (all times UTC 16:18). Probes within each round are sorted by server-reported second; Δsend and Δrecv are the gap from the previous probe. The boundary pair (black) is where the server second ticks over.],
)
