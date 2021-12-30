
# may-cdbc
```
docker run -it --net=host --rm williamyeh/wrk -t12 -c400 -d30s http://192.168.28.235:8000
Running 30s test @ http://192.168.28.235:8000
  12 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency    92.21ms   66.06ms 708.65ms   80.61%
    Req/Sec   386.64     48.25   570.00     72.07%
  138365 requests in 30.03s, 7.92MB read
Requests/sec:   4606.98
Transfer/sec:    269.94KB
```

# actix-web-sqlx-async-std
```
docker run -it --net=host --rm williamyeh/wrk -t12 -c400 -d30s http://192.168.28.235:8000
Running 30s test @ http://192.168.28.235:8000
  12 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency    86.35ms    2.30ms  95.54ms   89.08%
    Req/Sec   382.66     59.97   666.00     78.68%
  136953 requests in 30.03s, 15.28MB read
Requests/sec:   4560.69
Transfer/sec:    521.09KB
```

# axum-sqlx
```
docker run -it --network host --rm williamyeh/wrk -t12 -c400 -d30s http://192.168.28.235:8000
Running 30s test @ http://192.168.28.235:8000
  12 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency    97.85ms    3.36ms 110.02ms   88.77%
    Req/Sec   337.56     20.69   545.00     83.58%
  120854 requests in 30.03s, 13.48MB read
Requests/sec:   4024.23
Transfer/sec:    459.80KB
```


