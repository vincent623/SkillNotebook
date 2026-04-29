use std::cmp::max;

pub fn diff_text(previous: &str, current: &str) -> String {
    let previous_lines = previous.lines().collect::<Vec<_>>();
    let current_lines = current.lines().collect::<Vec<_>>();
    let previous_len = previous_lines.len();
    let current_len = current_lines.len();

    let mut lcs = vec![vec![0usize; current_len + 1]; previous_len + 1];
    for left in (0..previous_len).rev() {
        for right in (0..current_len).rev() {
            lcs[left][right] = if previous_lines[left] == current_lines[right] {
                lcs[left + 1][right + 1] + 1
            } else {
                max(lcs[left + 1][right], lcs[left][right + 1])
            };
        }
    }

    let mut left = 0usize;
    let mut right = 0usize;
    let mut output = Vec::new();

    while left < previous_len && right < current_len {
        if previous_lines[left] == current_lines[right] {
            output.push(format!("  {}", previous_lines[left]));
            left += 1;
            right += 1;
        } else if lcs[left + 1][right] >= lcs[left][right + 1] {
            output.push(format!("- {}", previous_lines[left]));
            left += 1;
        } else {
            output.push(format!("+ {}", current_lines[right]));
            right += 1;
        }
    }

    while left < previous_len {
        output.push(format!("- {}", previous_lines[left]));
        left += 1;
    }

    while right < current_len {
        output.push(format!("+ {}", current_lines[right]));
        right += 1;
    }

    if output.is_empty() {
        "  (no textual changes)".to_string()
    } else {
        output.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::diff_text;

    #[test]
    fn creates_a_basic_line_diff() {
        let diff = diff_text("alpha\nbeta\ngamma", "alpha\nbeta changed\ngamma\ndelta");
        assert!(diff.contains("- beta"));
        assert!(diff.contains("+ beta changed"));
        assert!(diff.contains("+ delta"));
    }
}
