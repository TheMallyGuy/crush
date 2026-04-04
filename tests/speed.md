# Speed Test: Bloxstrap vs. Crush
_Last updated : 04/04/2026. Things might change in the future._

> **Note:** Keep in mind that download speeds are highly subjective\! Your results will depend on your region and your local ISP, so think of this as a baseline.

I wanted to see how **Crush** stacks up against **Bloxstrap** in a real-world download test. Even with my "average" home internet, the data showed a massive difference in performance.

### My Setup

  * **Avg Download:** 38.73 Mbps
  * **Avg Upload:** 21.22 Mbps

<img src="./assets/speed compare.webp" alt="Speed Comparison Graph" width="600"/>

As you can see, **Crush finished in just 53 seconds**, while Bloxstrap trailed behind at a whopping **3 minutes and 20 seconds**.

## Why is Crush pulling ahead?

While the **Rust** backend gives Crush a high-performance foundation, the real secret weapon is the **"Best Region Tester"** logic I've integrated.

### How the Region Tester works

Roblox hosts its files across several different CDNs (Content Delivery Networks). Most standard installers default to a single URL, but that path isn't always the fastest route for your specific location.

Crush takes a more active approach. Before the download even starts, it pings multiple endpoints to see which one offers the lowest latency:

  * `https://setup-aws.rbxcdn.com` (AWS)
  * `https://setup-ak.rbxcdn.com` (Akamai)
  * `https://setup-cfly.rbxcdn.com` (Cloudflare / Cachefly)

## Why?
In regions like **Asia**, the default AWS servers (which are often US-based) can become a major bottleneck. While Bloxstrap generally sticks to that AWS default, Crush identifies the most responsive provider like Akamai or Cloudflare before it starts pulling data.

By picking the "closest" server first, Crush skips the initial lag and maintains a much more consistent download speed throughout the entire process 🎉.