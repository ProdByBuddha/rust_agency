//! Truncation Utilities
//! 
//! Provides robust text truncation that preserves prefix/suffix and 
//! respects UTF-8 boundaries. Derived from codex-rs patterns.

const APPROX_BYTES_PER_TOKEN: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TruncationPolicy {
    Bytes(usize),
    Tokens(usize),
}

impl TruncationPolicy {
    pub fn byte_budget(&self) -> usize {
        match self {
            TruncationPolicy::Bytes(b) => *b,
            TruncationPolicy::Tokens(t) => t.saturating_mul(APPROX_BYTES_PER_TOKEN),
        }
    }

    pub fn token_budget(&self) -> usize {
        match self {
            TruncationPolicy::Tokens(t) => *t,
            TruncationPolicy::Bytes(b) => b.saturating_add(APPROX_BYTES_PER_TOKEN - 1) / APPROX_BYTES_PER_TOKEN,
        }
    }
}

pub fn truncate_text(content: &str, policy: TruncationPolicy) -> String {
    let max_bytes = policy.byte_budget();
    
    if content.len() <= max_bytes {
        return content.to_string();
    }

    if max_bytes == 0 {
        return format!("... [{} chars truncated] ...", content.len());
    }

    let half = max_bytes / 2;
    let mut prefix_end = 0;
    let mut suffix_start = content.len();

    // Find prefix end on UTF-8 boundary
    for (idx, c) in content.char_indices() {
        let char_end = idx + c.len_utf8();
        if char_end <= half {
            prefix_end = char_end;
        } else {
            break;
        }
    }

    // Find suffix start on UTF-8 boundary
    let suffix_target = content.len().saturating_sub(half);
    for (idx, _) in content.char_indices().rev() {
        if idx >= suffix_target {
            suffix_start = idx;
        } else {
            break;
        }
    }

    if suffix_start < prefix_end {
        suffix_start = prefix_end;
    }

    let prefix = &content[..prefix_end];
    let suffix = &content[suffix_start..];
    let truncated_count = content.len() - (prefix.len() + suffix.len());

    format!("{} ... [{} chars truncated] ... {}", prefix, truncated_count, suffix)
}

pub fn approx_token_count(text: &str) -> usize {
    text.len().saturating_add(APPROX_BYTES_PER_TOKEN - 1) / APPROX_BYTES_PER_TOKEN
}
