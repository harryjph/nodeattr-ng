use pest::Span;

pub fn span_into_str(span: Span) -> &str {
    span.as_str()
}