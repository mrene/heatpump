/** An IR PWM encoder/decoder with configurable pulse length */
use std::{collections::HashMap, hash::Hash, time::Duration};

use thiserror::Error;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Rule {
    pub duration: Duration,
    pub tolerance: Duration,
}

impl Rule {
    pub fn new(duration: Duration) -> Self {
        // Use a 20% tolerance by default
        Self {
            duration,
            tolerance: duration / 5,
        }
    }

    fn matches(&self, duration: Duration) -> bool {
        let diff = self
            .duration
            .checked_sub(duration)
            .unwrap_or_else(|| duration.checked_sub(self.duration).unwrap());
        diff <= self.tolerance
    }
}

#[derive(Error, Debug, Copy, Clone)]
pub enum CodecError<T: Copy + std::fmt::Debug> {
    #[error("invalid pulse length: {0:?}")]
    InvalidPulseLength(Duration),

    #[error("a pulse was missing from the rule set: {0:?}")]
    InvalidPulse(T),
}

pub struct Codec<TPulse> {
    rules: HashMap<TPulse, Rule>,
    sorted_rules: Vec<(TPulse, Rule)>,
}

impl<T: Copy + Eq + Hash + std::fmt::Debug> Codec<T> {
    pub fn new(rules: impl Iterator<Item = (T, Rule)>) -> Self {
        let mut sorted_rules: Vec<_> = rules.collect();
        sorted_rules.sort_by_key(|f| f.1.duration);
        
        let rules = sorted_rules.iter().copied().collect();

        Self { sorted_rules, rules }
    }

    pub fn decode(
        &self,
        pulses: impl Iterator<Item = Duration>,
    ) -> Result<Vec<(T, T)>, CodecError<T>> {
        let mut ret = Vec::new();
        let mut pending: Option<T> = None;

        let pulses = pulses.map(|d| self.decode_pulse(d));
        for pulse in pulses {
            match pending.take() {
                Some(p) => ret.push((p, pulse?)),
                None => {
                    pending.replace(pulse?);
                }
            };
        }

        return Ok(ret);
    }

    pub fn decode_pulse(&self, pulse: Duration) -> Result<T, CodecError<T>> {
        self.sorted_rules
            .iter()
            .find(|(_, r)| r.matches(pulse))
            .map(|(p, _)| *p)
            .ok_or(CodecError::InvalidPulseLength(pulse))
    }

    pub fn encode(
        &self,
        pulses: impl Iterator<Item = T>,
    ) -> Result<Vec<Duration>, CodecError<T>> {
        let mut ret = Vec::new();

        for p in pulses {
            ret.push(self.encode_pulse(p).ok_or(CodecError::InvalidPulse(p))?);
        }

        Ok(ret)
    }

    pub fn encode_pulse(&self, pulse: T) -> Option<Duration> {
        self.rules.get(&pulse).map(|r| r.duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
    enum Pulse {
        Short,
        Long,
    }

    fn get_codec() -> Codec<Pulse> {
        let rules = [
            (Pulse::Short, Rule::new(Duration::from_micros(100))),
            (Pulse::Long, Rule::new(Duration::from_micros(500))),
        ];
        Codec::new(rules.into_iter())
    }
    
    #[test]
    fn test_decode() {
        let pulses = [100, 500, 100, 500, 500, 500, 500, 100].map(|d| Duration::from_micros(d));
        let decoded = get_codec().decode(pulses.into_iter()).unwrap();
        assert_eq!(
            decoded,
            vec![
                (Pulse::Short, Pulse::Long),
                (Pulse::Short, Pulse::Long),
                (Pulse::Long, Pulse::Long),
                (Pulse::Long, Pulse::Short),
            ]
        );
    }

    #[test]
    fn test_encode() {
        let pulses = [
            Pulse::Short, Pulse::Long,
            Pulse::Long, Pulse::Long,
            Pulse::Long, Pulse::Short,
        ];

        let encoded = get_codec().encode(pulses.into_iter()).unwrap();
        assert_eq!(
            encoded,
            vec![
                Duration::from_micros(100),
                Duration::from_micros(500),
                Duration::from_micros(500),
                Duration::from_micros(500),
                Duration::from_micros(500),
                Duration::from_micros(100),
            ]
        );
    }
}
