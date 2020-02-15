# Git Series

`gitseries` will clone and scan your Git repository for all commits, fetching metrics, and export them to InfluxDB's line protocol.

Unfortunately, the repo is currently hardcoded. This will be the first thing I change on Monday, promise 😂

```shell
gitseries | influxdb write
```
