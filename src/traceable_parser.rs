use crate::observer::{ParserObserver, RuleTracer};
use pest::error::Error;
use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;
use std::fmt::Debug;
use std::sync::Arc;
use tracing;

pub struct TraceableParser<P> {
    #[allow(dead_code)]
    inner: P,
    observer: Option<Arc<ParserObserver>>,
}

impl<P> TraceableParser<P> {
    pub fn new(parser: P) -> Self {
        Self {
            inner: parser,
            observer: None,
        }
    }

    pub fn with_observer(parser: P, observer: Arc<ParserObserver>) -> Self {
        Self {
            inner: parser,
            observer: Some(observer),
        }
    }

    pub fn traceable_parse<'a, R>(
        &self,
        rule: R,
        input: &'a str,
    ) -> Result<Pairs<'a, R>, Box<Error<R>>>
    where
        P: Parser<R>,
        R: pest::RuleType + Debug + Copy,
    {
        if let Some(observer) = &self.observer {
            let rule_name = format!("{:?}", rule);
            let input_pos = 0;

            tracing::debug!("Entering top-level rule: {}", rule_name);
            let tracer: RuleTracer = observer.enter_rule(&rule_name, input_pos);

            let result = P::parse(rule, input).map_err(Box::new);

            match &result {
                Ok(pairs) => {
                    tracing::debug!("Parse succeeded for rule: {}", rule_name);
                    let pairs_clone = pairs.clone();
                    let pairs_vec: Vec<_> = pairs_clone.collect();

                    if !pairs_vec.is_empty() {
                        tracing::debug!("Found {} top-level pairs", pairs_vec.len());
                        for pair in pairs_vec {
                            let span = pair.as_span();
                            tracing::debug!(
                                "Processing pair: {:?} at span {:?}-{:?}",
                                pair.as_rule(),
                                span.start(),
                                span.end()
                            );
                            self.trace_pair(pair, observer);
                        }
                    } else {
                        tracing::debug!("No pairs found in successful parse");
                    }
                    tracer.success(input.len());
                }
                Err(err) => {
                    tracing::debug!("Parse failed for rule {}: {:?}", rule_name, err);
                    // Tracer is dropped automatically on error
                }
            }
            result
        } else {
            P::parse(rule, input).map_err(Box::new)
        }
    }

    #[allow(clippy::only_used_in_recursion)] // Recursion is inherent here
    fn trace_pair<'a, R>(&self, pair: Pair<'a, R>, observer: &Arc<ParserObserver>)
    where
        R: pest::RuleType + Debug + Copy,
    {
        let rule_name = format!("{:?}", pair.as_rule());
        let span = pair.as_span();
        let start_pos = span.start();
        let end_pos = span.end();

        let inner_pairs: Vec<_> = pair.clone().into_inner().collect();
        let tracer = observer.enter_rule(&rule_name, start_pos);

        for inner_pair in inner_pairs {
            self.trace_pair(inner_pair, observer);
        }
        tracer.success(end_pos);
    }
}

impl<P, R> Parser<R> for TraceableParser<P>
where
    P: Parser<R>,
    R: pest::RuleType + Debug + Copy,
{
    fn parse(rule: R, input: &str) -> Result<Pairs<R>, Error<R>> {
        P::parse(rule, input)
    }
}
