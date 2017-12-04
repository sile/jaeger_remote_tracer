jaeger_remote_tracer
====================

### HTTP API

- `GET /start_span?operation_name=${NAME}&span_id=${SID}`
- `GET /finish?span_id=${SID}`
- `GET /set_tag?span_id=${SID}`
- `GET /log?span_id=${SID}`
