//! Quote-aware bounded delimiter sampling; structural validation remains the parser's job.

/// Inspect at most sixteen nonblank logical records with fixed-size count storage.
/// Candidates must occur on every sampled record; ties prefer comma, then semicolon, then tab.
pub(super) fn delimiter(bytes: &[u8]) -> u8 {
    let mut samples = [[0usize; 3]; 16];
    let mut sample = 0;
    let mut counts = [0; 3];
    let mut quoted = false;
    let mut nonblank = false;
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        // Doubled quotes do not leave a quoted field or expose its embedded delimiters.
        if byte == b'"' {
            nonblank = true;
            if quoted && bytes.get(index + 1) == Some(&b'"') {
                index += 1;
            } else {
                quoted = !quoted;
            }
        } else if !quoted && matches!(byte, b'\n' | b'\r') {
            if nonblank {
                samples[sample] = counts;
                sample += 1;
                if sample == samples.len() {
                    break;
                }
            }
            counts = [0; 3];
            nonblank = false;
        } else {
            if !quoted {
                for (candidate, delimiter) in [b',', b';', b'\t'].into_iter().enumerate() {
                    if byte == delimiter {
                        counts[candidate] += 1;
                    }
                }
            }
            nonblank |= !byte.is_ascii_whitespace();
        }
        index += 1;
    }
    if sample < samples.len() && nonblank {
        samples[sample] = counts;
        sample += 1;
    }
    let samples = &samples[..sample];
    let mut best = None;
    for (candidate, delimiter) in [b',', b';', b'\t'].into_iter().enumerate() {
        let Some(score) = consistency(samples, candidate) else {
            continue;
        };
        if best.as_ref().is_none_or(|(previous, _)| score > *previous) {
            best = Some((score, delimiter));
        }
    }
    best.map_or(b',', |(_, delimiter)| delimiter)
}

/// Require presence on every sampled record, then rank modal agreement and count spread.
fn consistency(
    samples: &[[usize; 3]],
    candidate: usize,
) -> Option<(usize, std::cmp::Reverse<usize>)> {
    if samples.iter().any(|counts| counts[candidate] == 0) {
        return None;
    }
    // At most sixteen records keep this modal comparison bounded to 256 comparisons.
    let agreement = samples
        .iter()
        .map(|counts| {
            samples
                .iter()
                .filter(|other| other[candidate] == counts[candidate])
                .count()
        })
        .max()?;
    let min = samples.iter().map(|counts| counts[candidate]).min()?;
    let max = samples.iter().map(|counts| counts[candidate]).max()?;
    Some((agreement, std::cmp::Reverse(max - min)))
}
