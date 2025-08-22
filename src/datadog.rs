use std::io::{self, Write};
use std::time::SystemTime;

use reqwest::header;
use tokio::runtime::Handle;
use zstd::stream::raw::{Encoder, Operation};
use zstd::zstd_safe::{InBuffer, OutBuffer};

use crate::{AggregatedMetric, AggregationSink, Aggregations, MetricMeta, MetricType, MetricUnit};

/// An aggregator sink which pushes metrics to Datadog, using the Datadog API.
pub struct DatadogSink {
    runtime: Handle,
    client: reqwest::Client,
    api_key: String,
    site: String,

    metric_buf: Vec<u8>,
    tag_buf: String,

    next_flush_len: usize,
    bytes_written: usize,
    cctx: Encoder<'static>,
    compression_buffer: Vec<u8>,
}

impl AggregationSink for DatadogSink {
    fn emit(&mut self, metrics: Aggregations) {
        self.emit_metrics(metrics).unwrap();
    }
}

const THRESHOLD: usize = 1024;
const MAX_COMPRESSED: usize = 512000;
const MAX_UNCOMPRESSED: usize = 5242880;

const DD_SITE: &str = "https://api.datadoghq.com";
const DISTRIBUTION_ENDPOINT: &str = "/api/v1/distribution_points";
const METRICS_ENDPOINT: &str = "/api/v2/series";

impl DatadogSink {
    /// Creates a new Sink.
    ///
    /// It needs to be configured with a Datadog API key, and optional server.
    /// The sink also needs a tokio runtime handle on which it will spawn outgoing requests.
    pub fn new(runtime: Handle, api_key: &str, dd_server: Option<&str>) -> io::Result<Self> {
        Ok(Self {
            runtime,
            client: reqwest::ClientBuilder::new()
                .build()
                .map_err(io::Error::other)?,
            api_key: api_key.into(),
            site: dd_server.unwrap_or(DD_SITE).trim_end_matches('/').into(),

            metric_buf: Vec::with_capacity(MAX_COMPRESSED),
            tag_buf: String::new(),

            next_flush_len: MAX_COMPRESSED - THRESHOLD,
            bytes_written: 0,
            cctx: Encoder::new(0)?,
            compression_buffer: Vec::with_capacity(MAX_COMPRESSED),
        })
    }

    fn emit_metrics(&mut self, metrics: Aggregations) -> io::Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_secs();

        for (meta, value) in metrics.counters {
            self.push_metric(&meta, timestamp, value)?;
        }
        for (meta, value) in metrics.gauges {
            self.push_metric(&meta, timestamp, value.last)?;
        }
        self.flush(METRICS_ENDPOINT)?;

        for (meta, values) in metrics.distributions {
            self.push_distribution(&meta, timestamp, &values.values)?;
        }
        self.flush(DISTRIBUTION_ENDPOINT)?;

        Ok(())
    }

    fn flush(&mut self, endpoint: &str) -> io::Result<()> {
        if self.metric_buf.is_empty() && self.bytes_written == 0 {
            return Ok(());
        }
        self.metric_buf.extend_from_slice(br#"]}"#);

        self.flush_to_zstd()?;
        self.do_flush(endpoint)?;

        Ok(())
    }
    fn maybe_flush(&mut self, endpoint: &str) -> io::Result<()> {
        if self.metric_buf.len() >= self.next_flush_len {
            self.flush_to_zstd()?;
        }

        let compressed_left = MAX_COMPRESSED
            .checked_sub(self.compression_buffer.len())
            .ok_or(io::ErrorKind::QuotaExceeded)?;
        let uncompressed_left = MAX_UNCOMPRESSED
            .checked_sub(self.bytes_written)
            .ok_or(io::ErrorKind::QuotaExceeded)?;

        self.next_flush_len = compressed_left
            .min(uncompressed_left)
            .saturating_sub(THRESHOLD);
        if self.next_flush_len < THRESHOLD {
            self.metric_buf.extend_from_slice(br#"]}"#);
            self.flush_to_zstd()?;
            self.do_flush(endpoint)?;
        }

        Ok(())
    }
    fn flush_to_zstd(&mut self) -> io::Result<()> {
        let mut input = InBuffer::around(&self.metric_buf);
        let mut output = OutBuffer::around(&mut self.compression_buffer);

        self.cctx.run(&mut input, &mut output)?;

        self.bytes_written += self.metric_buf.len();
        self.metric_buf.clear();
        Ok(())
    }
    fn do_flush(&mut self, endpoint: &str) -> io::Result<()> {
        let mut output = OutBuffer::around(&mut self.compression_buffer);
        self.cctx.finish(&mut output, true)?;
        self.cctx.reinit()?;

        let request = self
            .client
            .post(format!("{}{endpoint}", self.site))
            .header("DD-API-KEY", &self.api_key)
            .header(header::ACCEPT, "application/json")
            .header(header::CONTENT_ENCODING, "zstd")
            .header(header::CONTENT_TYPE, "application/json")
            .body(self.compression_buffer.clone())
            .send();

        self.runtime.spawn(async move {
            let response = request.await;
            response.unwrap().error_for_status().unwrap();
        });

        self.bytes_written = 0;
        self.compression_buffer.clear();

        Ok(())
    }

    fn push_metric(
        &mut self,
        meta: &AggregatedMetric,
        timestamp: u64,
        value: f64,
    ) -> io::Result<()> {
        self.write_begin();
        self.write_meta(meta)?;
        self.write_type_and_unit(meta)?;

        self.metric_buf.write_fmt(format_args!(
            r#""points":[{{"timestamp":{timestamp},"value":{value}}}]}}"#
        ))?;

        self.maybe_flush(METRICS_ENDPOINT)
    }

    fn push_distribution(
        &mut self,
        meta: &AggregatedMetric,
        timestamp: u64,
        values: &[f64],
    ) -> io::Result<()> {
        self.write_begin();
        self.write_meta(meta)?;

        self.metric_buf
            .write_fmt(format_args!(r#""points":[[{timestamp},"#))?;
        serde_json::to_writer(&mut self.metric_buf, values).map_err(io::Error::other)?;
        self.metric_buf.extend_from_slice(br#"]]}"#);

        self.maybe_flush(DISTRIBUTION_ENDPOINT)
    }

    fn write_begin(&mut self) {
        if self.metric_buf.is_empty() {
            self.metric_buf.extend_from_slice(br#"{"series":["#);
        } else {
            self.metric_buf.push(b',');
        }
    }

    fn write_meta(&mut self, meta: &AggregatedMetric) -> io::Result<()> {
        self.metric_buf.extend_from_slice(br#"{"metric":"#);
        serde_json::to_writer(&mut self.metric_buf, meta.key()).map_err(io::Error::other)?;
        self.metric_buf.push(b',');

        let tags = meta.tags();
        if tags.len() > 0 {
            self.metric_buf.extend_from_slice(br#""tags":["#);
            for (i, tag) in tags.enumerate() {
                self.tag_buf.clear();
                self.tag_buf.push_str(tag.0);
                self.tag_buf.push(':');
                self.tag_buf.push_str(tag.1);

                if i > 0 {
                    self.metric_buf.push(b',');
                }
                serde_json::to_writer(&mut self.metric_buf, &self.tag_buf)
                    .map_err(io::Error::other)?;
            }
            self.metric_buf.extend_from_slice(br#"],"#);
        }

        Ok(())
    }

    fn write_type_and_unit(&mut self, meta: &MetricMeta) -> io::Result<()> {
        self.metric_buf.extend_from_slice(br#""type":"#);
        let ty = match meta.ty() {
            MetricType::Counter => b'1',
            MetricType::Gauge => b'3',
            _ => b'0',
        };
        self.metric_buf.push(ty);
        self.metric_buf.push(b',');
        if meta.unit() != MetricUnit::Unknown {
            self.metric_buf.extend_from_slice(br#""unit":"#);
            serde_json::to_writer(&mut self.metric_buf, &meta.unit()).map_err(io::Error::other)?;
            self.metric_buf.push(b',');
        }

        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::tags::record_tags;

//     use super::*;

//     #[test]
//     fn builds_metrics_request() {
//         let mut sink = DatadogSink::new().unwrap();

//         sink.push_metric(
//             &AggregatedMetric {
//                 meta: MetricMeta::new(MetricType::Counter, MetricUnit::Bytes, "some.bytes"),
//                 tag_values: Default::default(),
//             },
//             12345,
//             123.45,
//         )
//         .unwrap();

//         sink.push_metric(
//             &AggregatedMetric {
//                 meta: MetricMeta::new(MetricType::Gauge, MetricUnit::Unknown, "a.gauge")
//                     .with_tags(&["a_tag"])
//                     .meta,
//                 tag_values: record_tags(&[&"a_value"]),
//             },
//             12346,
//             1234.567,
//         )
//         .unwrap();

//         sink.flush(METRICS_ENDPOINT).unwrap();

//         sink.push_distribution(
//             &AggregatedMetric {
//                 meta: MetricMeta::new(MetricType::Distribution, MetricUnit::Seconds, "a.timer"),
//                 tag_values: Default::default(),
//             },
//             12346,
//             &[1., 2., 3., 4.],
//         )
//         .unwrap();

//         sink.flush(DISTRIBUTION_ENDPOINT).unwrap();
//     }
// }
