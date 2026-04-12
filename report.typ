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
  }
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
