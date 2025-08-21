use std::io;

use crate::{AggregatedMetric, AggregationSink, Aggregations, MetricMeta, MetricType, MetricUnit};

pub struct DatadogSink {
    runtime: tokio::runtime::Handle,
    client: reqwest::Client,
}

impl AggregationSink for DatadogSink {
    fn emit(&self, metrics: Aggregations) {
        self.runtime.spawn(async {});
    }
}

const MAX_COMPRESSED: usize = 512000;
const MAX_UNCOMPRESSED: usize = 5242880;

static MSG_PREFIX: &[u8] = br#"{"series":["#;
static MSG_SUFFIX: &[u8] = br#"]}"#;
static METRIC_PREFIX: &[u8] = br#"{"metric": "#;

struct AggregationMessages {
    metrics: Aggregations,
    timestamp: u64,
    metric_buf: Vec<u8>,
    tag_buf: String,
}

impl Iterator for AggregationMessages {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.metrics.counters.is_empty()
            && self.metrics.gauges.is_empty()
            && self.metrics.distributions.is_empty()
        {
            return None;
        }

        return None;
    }
}

impl AggregationMessages {
    fn metric_prefix(&mut self, meta: &AggregatedMetric) -> io::Result<()> {
        self.metric_buf.extend_from_slice(br#"{"metric":"#);
        serde_json::to_writer(&mut *self.metric_buf, meta.key()).map_err(io::Error::other)?;

        let tags = meta.tags();
        if tags.len() > 0 {
            self.metric_buf.extend_from_slice(br#","tags":["#);
            for (i, tag) in tags.enumerate() {
                self.tag_buf.clear();
                self.tag_buf.push_str(tag.0);
                self.tag_buf.push(':');
                self.tag_buf.push_str(tag.1);

                if i > 0 {
                    self.metric_buf.push(b',');
                }
                serde_json::to_writer(&mut *self.metric_buf, &self.tag_buf)
                    .map_err(io::Error::other)?;
            }
            self.metric_buf.extend_from_slice(br#"],"#);
        }

        Ok(())
    }

    fn add_type_and_unit(&mut self, meta: &MetricMeta) -> io::Result<()> {
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
            serde_json::to_writer(&mut *self.metric_buf, &meta.unit()).map_err(io::Error::other)?;
            self.metric_buf.push(b',');
        }

        Ok(())
    }
}

/*
"unit": "bytes", etc…
"tags": ["foo:bar", "baz:qux"]
{
  "series": [
    {
      "metric": "system.load.1",
      "type": 0, // 1 => count 3 => gauge
      "points": [
        {
          "timestamp": 1636629071,
          "value": 0.7
        }
      ]
    }
  ]
}
*/
