#[derive(Debug, Clone)]
pub enum SseEvent {
    Data(String),
    Done,
    Comment(String),
}

#[derive(Debug, Default)]
pub struct SseParser {
    buffer: String,
}

impl SseParser {
    pub fn new() -> Self {
        Self { buffer: String::new() }
    }

    pub fn feed_line(&mut self, line: &str) -> Option<SseEvent> {
        let trimmed = line.trim_end_matches('\r');

        if trimmed.is_empty() {
            if !self.buffer.is_empty() {
                let payload = std::mem::take(&mut self.buffer);
                return Some(SseEvent::Data(payload));
            }
            return None;
        }

        if let Some(data) = trimmed.strip_prefix("data: ") {
            if data == "[DONE]" {
                return Some(SseEvent::Done);
            }
            if self.buffer.is_empty() {
                self.buffer = data.to_string();
            } else {
                self.buffer.push('\n');
                self.buffer.push_str(data);
            }
            return None;
        }

        if trimmed.starts_with(':') {
            let comment = trimmed.trim_start_matches(':').trim();
            return Some(SseEvent::Comment(comment.to_string()));
        }

        None
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
    }
}

pub fn parse_sse_line(line: &str) -> Result<Option<String>, crate::ChatError> {
    let trimmed = line.trim();
    if trimmed.is_empty() { return Ok(None) }
    if let Some(data) = trimmed.strip_prefix("data: ") {
        if data == "[DONE]" { return Ok(None) }
        Ok(Some(data.to_string()))
    } else if trimmed.starts_with(':') {
        Ok(None)
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_data_line() {
        let mut parser = SseParser::new();
        assert!(parser.feed_line("data: {\"key\":\"value\"}").is_none());
        let event = parser.feed_line("").unwrap();
        match event {
            SseEvent::Data(d) => assert_eq!(d, "{\"key\":\"value\"}"),
            _ => panic!("expected Data"),
        }
    }

    #[test]
    fn test_done_signal() {
        let mut parser = SseParser::new();
        match parser.feed_line("data: [DONE]").unwrap() {
            SseEvent::Done => {},
            _ => panic!("expected Done"),
        }
    }

    #[test]
    fn test_multi_line_data() {
        let mut parser = SseParser::new();
        assert!(parser.feed_line("data: {\"key\":").is_none());
        assert!(parser.feed_line("data: \"value\"}").is_none());
        let event = parser.feed_line("").unwrap();
        match event {
            SseEvent::Data(d) => assert_eq!(d, "{\"key\":\n\"value\"}"),
            _ => panic!("expected Data"),
        }
    }

    #[test]
    fn test_comment_line() {
        let mut parser = SseParser::new();
        match parser.feed_line(": keepalive").unwrap() {
            SseEvent::Comment(c) => assert_eq!(c, "keepalive"),
            _ => panic!("expected Comment"),
        }
    }

    #[test]
    fn test_malformed_line_ignored() {
        let mut parser = SseParser::new();
        assert!(parser.feed_line("not sse format").is_none());
        assert!(parser.feed_line("event: custom").is_none());
    }

    #[test]
    fn test_consecutive_events() {
        let mut parser = SseParser::new();
        assert!(parser.feed_line("data: {\"a\":1}").is_none());
        let e1 = parser.feed_line("").unwrap();
        assert!(matches!(e1, SseEvent::Data(_)));

        assert!(parser.feed_line("data: {\"b\":2}").is_none());
        let e2 = parser.feed_line("").unwrap();
        assert!(matches!(e2, SseEvent::Data(_)));
    }

    #[test]
    fn test_parse_sse_line_basic() {
        let result = parse_sse_line("data: hello").unwrap();
        assert_eq!(result, Some("hello".to_string()));
    }

    #[test]
    fn test_parse_sse_line_done_returns_none() {
        let result = parse_sse_line("data: [DONE]").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_sse_line_empty_returns_none() {
        let result = parse_sse_line("").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_sse_line_comment_returns_none() {
        let result = parse_sse_line(": comment").unwrap();
        assert!(result.is_none());
    }
}
