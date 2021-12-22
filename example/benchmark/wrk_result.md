
# may-cdbc
```
docker run -it --net=host --rm williamyeh/wrk -t12 -c400 -d30s http://192.168.28.235:8000
Running 30s test @ http://192.168.28.235:8000
  12 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency   100.56ms   73.30ms 809.01ms   78.81%
    Req/Sec   355.39     45.43   515.00     71.38%
  127240 requests in 30.03s, 10.80MB read
Requests/sec:   4237.31
Transfer/sec:    368.28KB
```

# actix-web-sqlx-async-std
```
docker run -it --net=host --rm williamyeh/wrk -t12 -c400 -d30s http://192.168.28.235:8000
Running 30s test @ http://192.168.28.235:8000
  12 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency   694.86ms  117.63ms 946.45ms   79.52%
    Req/Sec    74.56     60.21   320.00     64.56%
  16792 requests in 30.04s, 1.22MB read
Requests/sec:    559.00
Transfer/sec:     41.49KB
```

# axum-sqlx
```
docker run -it --net=host --rm williamyeh/wrk -t12 -c400 -d30s http://192.168.28.235:8000
Running 30s test @ http://192.168.28.235:8000
  12 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency    95.96ms    2.89ms 105.49ms   92.63%
    Req/Sec   344.23     23.56   464.00     81.91%
  123265 requests in 30.03s, 13.75MB read
Requests/sec:   4104.39
Transfer/sec:    468.96KB
```


