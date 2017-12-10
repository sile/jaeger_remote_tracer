jaeger_remote_tracer
====================

### HTTP API

#### GET /start_span

QUERY:
- `span_id`:
  - type: u64
  - required
- `operation_name`:
  - type: string
  - required
- `child_of`:
  - type: u64
  - optional
- `follows_from`:
  - type: u64
  - optional
- `tags`:
  - type: string ("${KEY}:${VALUE},...")
  - optional
- `time`:
  - type: f64 (unixtimestamp)
  - optional

#### GET /finish

- `span_id`:
  - type: u64
  - required
- `time`:
  - type: f64 (unixtimestamp)
  - optional

### Example

```
$ docker run -d -p6831:6831/udp -p6832:6832/udp -p16686:16686 jaegertracing/all-in-one:latest

$ cargo run

$ curl 'http://localhost:8888/start_span?operation_name=foo&span_id=10&tags=foo:bar,111:222'
$ curl 'http://localhost:8888/start_span?operation_name=foo&span_id=20&child_of=10'
$ curl 'http://localhost:8888/finish?span_id=20'
$ curl 'http://localhost:8888/finish?span_id=10'

$ firefox http://localhost:16686/
```
