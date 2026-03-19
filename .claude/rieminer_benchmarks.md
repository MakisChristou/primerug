# rieMiner Benchmark Configs Tested (2026-03-19)

Hardware: AMD Ryzen 9 5950X (32T), 62 GB RAM, RTX 3080
rieMiner version: 2507

## Configs Tested

All at 500-digit 8-tuple (difficulty 1661), pattern `0, 2, 4, 2, 4, 6, 2, 6`.

### 1. PTL=268M, SW=4 (60s)
```
Threads=32, SieveWorkers=4, PrimeTableLimit=268435399, SieveIterations=16, SieveBits=25
```
- c/s: 15,875 | ratio: 33.50 | blocks/day: 0.000864

### 2. PTL=536M, SW=4 (60s)
```
Threads=32, SieveWorkers=4, PrimeTableLimit=536870912, SieveIterations=16, SieveBits=25
```
- c/s: 14,758 | ratio: 32.33 | blocks/day: 0.001069

### 3. PTL=1B, SW=4 (60s)
```
Threads=32, SieveWorkers=4, PrimeTableLimit=1000000000, SieveIterations=16, SieveBits=25
```
- c/s: 14,062 | ratio: 31.28 | blocks/day: 0.001326

### 4. PTL=4B, SW=4 (60s)
```
Threads=32, SieveWorkers=4, PrimeTableLimit=4294967296, SieveIterations=16, SieveBits=25
```
- c/s: 11,821 | ratio: 29.26 | blocks/day: 0.001901

### 5. Fully auto-tuned (120s) — BEST RESULT
```
Threads=32, SieveWorkers=0, PrimeTableLimit=0, SieveIterations=0, SieveBits=0
```
rieMiner auto-selected: **SW=9, SieveBits=24, PTL=4B, SI=16, 92 primorial offsets**
- c/s: 12,392 | ratio: 29.27 | **blocks/day: 0.001987**
- ETA for 500-digit 8-tuple: **503 days (1.4 years)**
- Tuples found in 120s: 1,487,046 / 50,802 / 1,797 / 58 / 4 / 0 / 0 / 0

## What We Did NOT Test

- Manual SW sweep (we only tested SW=4 manually; auto picked SW=9)
- Different SieveBits values (auto picked 24; we used 25 in manual tests)
- Different SI values (always 16)
- Different thread counts (always 32)
- Different primorial numbers (rieMiner uses p185#, auto-selected)

The auto-tuned result (0.001987 bpd) is 4.5% better than our best manual config
(PTL=4B SW=4: 0.001901 bpd). The difference comes from SW=9 and SieveBits=24.

## Key Takeaway

rieMiner's auto-tuner is good but we haven't verified it's globally optimal.
A manual sweep of SW={4,6,8,9,10,12} × SieveBits={23,24,25} could find a better config.
