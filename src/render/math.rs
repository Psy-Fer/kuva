//! Math notation for `$...$` regions in labels.
//!
//! [`to_unicode`] rewrites a label's `$...$` regions (LaTeX-ish syntax) to
//! inline Unicode text: Greek letters (`\sigma`→σ), operators (`\leq`→≤),
//! super/subscripts (`x^2`→x², all-or-nothing with a clean `x^(2q)` fallback),
//! `\frac{a}{b}`→`a/b`, `\sqrt{x}`→`√x`. It never emits a stray `\` or `$`.
//!
//! This is a pure lookup/flattening pass — always compiled, zero dependencies,
//! deliberately **inline only** (no stacked fractions or other 2-D layout, so
//! it works everywhere a plain string does, including the terminal backend's
//! character grid and markdown body text). Literal dollars are written `\$`.

// ─────────────────────────── detection ─────────────────────────────────────

/// One segment of a label string: literal text or a math region (the body of
/// a `$...$`, without the dollar signs).
pub(crate) enum Segment<'a> {
    Text(&'a str),
    Math(&'a str),
}

/// Does the label need rewriting before display? True when it contains a
/// `$...$` math region or an escaped `\$` (which must render as a literal
/// `$`). Backends gate on this to skip the rewrite cost for plain labels.
pub fn needs_rewrite(s: &str) -> bool {
    contains_math(s) || s.contains("\\$")
}

/// Cheap pre-check: does the string contain at least one `$...$` region?
/// Requires two unescaped `$`. Avoids the segment-split cost for plain labels.
pub(crate) fn contains_math(s: &str) -> bool {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut count = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() && bytes[i + 1] == b'$' {
            i += 2;
            continue;
        }
        if bytes[i] == b'$' {
            count += 1;
            if count >= 2 {
                return true;
            }
        }
        i += 1;
    }
    false
}

/// Split a label on `$...$` regions, honoring `\$` as a literal dollar.
/// An unclosed `$` makes the remainder a literal text segment.
pub(crate) fn split_segments(s: &str) -> Vec<Segment<'_>> {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut cursor = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() && bytes[i + 1] == b'$' {
            i += 2;
            continue;
        }
        if bytes[i] == b'$' {
            if cursor < i {
                out.push(Segment::Text(&s[cursor..i]));
            }
            let math_start = i + 1;
            let mut j = math_start;
            while j < bytes.len() {
                if bytes[j] == b'\\' && j + 1 < bytes.len() {
                    j += 2;
                    continue;
                }
                if bytes[j] == b'$' {
                    break;
                }
                j += 1;
            }
            if j < bytes.len() {
                out.push(Segment::Math(&s[math_start..j]));
                i = j + 1;
                cursor = i;
            } else {
                out.push(Segment::Text(&s[i..]));
                cursor = bytes.len();
                break;
            }
        } else {
            i += 1;
        }
    }
    if cursor < bytes.len() {
        out.push(Segment::Text(&s[cursor..]));
    }
    out
}

/// Map a LaTeX-style command name (no leading `\`) to a single Unicode symbol.
/// For multi-character operator names (`\log`, `\sin`, …) see [`command_to_str`].
pub(crate) fn command_to_unicode(name: &str) -> Option<char> {
    Some(match name {
        // Greek lowercase
        "alpha" => 'α',
        "beta" => 'β',
        "gamma" => 'γ',
        "delta" => 'δ',
        "epsilon" | "varepsilon" => 'ε',
        "zeta" => 'ζ',
        "eta" => 'η',
        "theta" => 'θ',
        "iota" => 'ι',
        "kappa" => 'κ',
        "lambda" => 'λ',
        "mu" => 'μ',
        "nu" => 'ν',
        "xi" => 'ξ',
        "pi" => 'π',
        "rho" => 'ρ',
        "sigma" => 'σ',
        "tau" => 'τ',
        "upsilon" => 'υ',
        "phi" | "varphi" => 'φ',
        "chi" => 'χ',
        "psi" => 'ψ',
        "omega" => 'ω',
        // Greek uppercase
        "Gamma" => 'Γ',
        "Delta" => 'Δ',
        "Theta" => 'Θ',
        "Lambda" => 'Λ',
        "Xi" => 'Ξ',
        "Pi" => 'Π',
        "Sigma" => 'Σ',
        "Phi" => 'Φ',
        "Psi" => 'Ψ',
        "Omega" => 'Ω',
        // Operators / relations
        "cdot" => '·',
        "circ" => '∘',
        "times" => '×',
        "div" => '÷',
        "pm" => '±',
        "mp" => '∓',
        "leq" | "le" => '≤',
        "geq" | "ge" => '≥',
        "neq" | "ne" => '≠',
        "approx" => '≈',
        "equiv" => '≡',
        "sim" => '∼',
        "propto" => '∝',
        "ll" => '≪',
        "gg" => '≫',
        // Symbols
        "infty" => '∞',
        "partial" => '∂',
        "nabla" => '∇',
        "degree" => '°',
        "angle" => '∠',
        "forall" => '∀',
        "exists" => '∃',
        "in" => '∈',
        "notin" => '∉',
        "subset" => '⊂',
        "cup" => '∪',
        "cap" => '∩',
        "ldots" => '…',
        "cdots" => '⋯',
        // Large operators
        "sum" => '∑',
        "prod" => '∏',
        "int" => '∫',
        // Arrows
        "to" | "rightarrow" => '→',
        "leftarrow" => '←',
        "Rightarrow" => '⇒',
        "Leftarrow" => '⇐',
        "leftrightarrow" => '↔',
        _ => return None,
    })
}

/// Map a LaTeX-style operator name to its plain-text form.
/// These are the standard LaTeX "operator names" that render as upright roman
/// text. Returning a `&str` lets us handle multi-character names (`log`, `sin`)
/// without needing a separate Unicode character.
pub(crate) fn command_to_str(name: &str) -> Option<&'static str> {
    Some(match name {
        // Trigonometric
        "sin" => "sin",
        "cos" => "cos",
        "tan" => "tan",
        "cot" => "cot",
        "sec" => "sec",
        "csc" => "csc",
        "arcsin" => "arcsin",
        "arccos" => "arccos",
        "arctan" => "arctan",
        // Exponential / logarithm
        "exp" => "exp",
        "log" => "log",
        "ln" => "ln",
        "lg" => "lg",
        // Limits / extrema
        "lim" => "lim",
        "limsup" => "lim sup",
        "liminf" => "lim inf",
        "min" => "min",
        "max" => "max",
        "sup" => "sup",
        "inf" => "inf",
        // Algebra / analysis
        "arg" => "arg",
        "det" => "det",
        "dim" => "dim",
        "ker" => "ker",
        "gcd" => "gcd",
        "lcm" => "lcm",
        "Pr" => "Pr",
        "hom" => "hom",
        "deg" => "deg",
        _ => return None,
    })
}

// ─────────────────────────── unicode lowering ──────────────────────────────

/// Rewrite a label's `$...$` math regions to inline Unicode text, leaving
/// surrounding text untouched. The result is plain text every backend can
/// render directly. See the module docs for the supported set.
///
/// Guarantees: the output never contains a `\` introduced by a math command
/// or a `$` math delimiter.
pub fn to_unicode(label: &str) -> String {
    let mut out = String::with_capacity(label.len());
    for seg in split_segments(label) {
        match seg {
            // `\$` in text renders as a literal `$` — drop the escape.
            Segment::Text(t) => out.push_str(&t.replace("\\$", "$")),
            Segment::Math(body) => clean_math(body, &mut out),
        }
    }
    out
}

/// Lower a single `$...$` body to inline Unicode, appending to `out`.
fn clean_math(body: &str, out: &mut String) {
    let bytes = body.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c == '\\' {
            let (name, next) = read_command(body, i + 1);
            match name {
                "frac" => {
                    if let Some(((n, d), end)) = take_two_groups(body, next) {
                        push_frac_inline(n, d, out);
                        i = end;
                        continue;
                    }
                }
                "sqrt" => {
                    let (index, after_idx) = read_optional_bracket(body, next);
                    if let Some((arg, end)) = take_group(body, after_idx) {
                        if let Some(idx) = index {
                            // Index as superscript before the radical, if it
                            // maps cleanly; else fall back to inline form.
                            // Clean it first so a command index (`\sqrt[\alpha]`)
                            // can't leak a backslash through the fallback.
                            let mut cleaned = String::new();
                            clean_math(idx, &mut cleaned);
                            if let Some(sup) = all_super(&cleaned) {
                                out.push_str(&sup);
                            } else {
                                out.push_str(&cleaned);
                            }
                        }
                        out.push('√');
                        let mut inner = String::new();
                        clean_math(arg, &mut inner);
                        wrap_inline(&inner, out);
                        i = end;
                        continue;
                    }
                }
                _ => {
                    if let Some(u) = command_to_unicode(name) {
                        out.push(u);
                        i = next;
                        continue;
                    }
                    if let Some(s) = command_to_str(name) {
                        out.push_str(s);
                        i = next;
                        continue;
                    }
                    // Unknown command: drop it (its `{arg}` is emitted by the
                    // brace rules below as cleaned content).
                    if !name.is_empty() {
                        i = next;
                        continue;
                    }
                }
            }
            // Stray backslash (e.g. before a non-letter) — drop it.
            i += 1;
            continue;
        }
        if c == '^' || c == '_' {
            if let Some((grp, end)) = read_script_group(body, i + 1) {
                // Recurse so structure inside the group lowers correctly:
                // `x^{\frac{1}{2}}` must become `x^(1/2)` via the fallback,
                // not a silently corrupted x¹².
                let sub = {
                    let mut s = String::new();
                    clean_math(grp, &mut s);
                    s
                };
                let mapped = if c == '^' {
                    all_super(&sub)
                } else {
                    all_sub(&sub)
                };
                match mapped {
                    Some(uni) => out.push_str(&uni),
                    None => {
                        // All-or-nothing: keep a clean caret/underscore form.
                        out.push(c);
                        out.push('(');
                        out.push_str(&sub);
                        out.push(')');
                    }
                }
                i = end;
                continue;
            }
        }
        if c == '{' || c == '}' {
            // Stray grouping braces — strip.
            i += 1;
            continue;
        }
        out.push(c);
        i += 1;
    }
}

/// `\frac{a}{b}` → `a/b`, parenthesising multi-character parts.
fn push_frac_inline(num: &str, den: &str, out: &mut String) {
    let n = {
        let mut s = String::new();
        clean_math(num, &mut s);
        s
    };
    let d = {
        let mut s = String::new();
        clean_math(den, &mut s);
        s
    };
    wrap_inline(&n, out);
    out.push('/');
    wrap_inline(&d, out);
}

/// Append `s`, wrapping in parens when it's more than one grapheme so inline
/// fractions/radicals stay unambiguous (`1/2` but `(a+b)/c`, `√(x+y)`).
fn wrap_inline(s: &str, out: &mut String) {
    if s.chars().count() <= 1 {
        out.push_str(s);
    } else {
        out.push('(');
        out.push_str(s);
        out.push(')');
    }
}

/// Map every char of `s` to its Unicode superscript, or `None` if any lacks
/// one (all-or-nothing).
fn all_super(s: &str) -> Option<String> {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        out.push(super_char(c)?);
    }
    Some(out)
}

fn all_sub(s: &str) -> Option<String> {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        out.push(sub_char(c)?);
    }
    Some(out)
}

fn super_char(c: char) -> Option<char> {
    Some(match c {
        '0' => '⁰',
        '1' => '¹',
        '2' => '²',
        '3' => '³',
        '4' => '⁴',
        '5' => '⁵',
        '6' => '⁶',
        '7' => '⁷',
        '8' => '⁸',
        '9' => '⁹',
        '+' => '⁺',
        '-' => '⁻',
        '=' => '⁼',
        '(' => '⁽',
        ')' => '⁾',
        'a' => 'ᵃ',
        'b' => 'ᵇ',
        'c' => 'ᶜ',
        'd' => 'ᵈ',
        'e' => 'ᵉ',
        'f' => 'ᶠ',
        'g' => 'ᵍ',
        'h' => 'ʰ',
        'i' => 'ⁱ',
        'j' => 'ʲ',
        'k' => 'ᵏ',
        'l' => 'ˡ',
        'm' => 'ᵐ',
        'n' => 'ⁿ',
        'o' => 'ᵒ',
        'p' => 'ᵖ',
        'r' => 'ʳ',
        's' => 'ˢ',
        't' => 'ᵗ',
        'u' => 'ᵘ',
        'v' => 'ᵛ',
        'w' => 'ʷ',
        'x' => 'ˣ',
        'y' => 'ʸ',
        'z' => 'ᶻ',
        _ => return None,
    })
}

fn sub_char(c: char) -> Option<char> {
    Some(match c {
        '0' => '₀',
        '1' => '₁',
        '2' => '₂',
        '3' => '₃',
        '4' => '₄',
        '5' => '₅',
        '6' => '₆',
        '7' => '₇',
        '8' => '₈',
        '9' => '₉',
        '+' => '₊',
        '-' => '₋',
        '=' => '₌',
        '(' => '₍',
        ')' => '₎',
        'a' => 'ₐ',
        'e' => 'ₑ',
        'h' => 'ₕ',
        'i' => 'ᵢ',
        'j' => 'ⱼ',
        'k' => 'ₖ',
        'l' => 'ₗ',
        'm' => 'ₘ',
        'n' => 'ₙ',
        'o' => 'ₒ',
        'p' => 'ₚ',
        'r' => 'ᵣ',
        's' => 'ₛ',
        't' => 'ₜ',
        'u' => 'ᵤ',
        'v' => 'ᵥ',
        'x' => 'ₓ',
        _ => return None,
    })
}

// ── small parsing helpers ──

/// Read an alphabetic command name starting at `start`; returns (name, index
/// just past it). Empty name if `start` isn't a letter.
fn read_command(s: &str, start: usize) -> (&str, usize) {
    let bytes = s.as_bytes();
    let mut end = start;
    while end < bytes.len() && (bytes[end] as char).is_ascii_alphabetic() {
        end += 1;
    }
    (&s[start..end], end)
}

/// If `pos` is at `{`, return (inner, index past `}`).
fn take_group(s: &str, pos: usize) -> Option<(&str, usize)> {
    let bytes = s.as_bytes();
    if pos >= bytes.len() || bytes[pos] != b'{' {
        return None;
    }
    let mut depth = 1;
    let mut i = pos + 1;
    let inner_start = i;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some((&s[inner_start..i], i + 1));
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Read two consecutive `{..}{..}` groups.
fn take_two_groups(s: &str, pos: usize) -> Option<((&str, &str), usize)> {
    let (a, after_a) = take_group(s, pos)?;
    let (b, after_b) = take_group(s, after_a)?;
    Some(((a, b), after_b))
}

/// If `pos` is at `[`, return (inner, index past `]`).
fn read_optional_bracket(s: &str, pos: usize) -> (Option<&str>, usize) {
    let bytes = s.as_bytes();
    if pos < bytes.len() && bytes[pos] == b'[' {
        if let Some(close) = s[pos + 1..].find(']') {
            return (Some(&s[pos + 1..pos + 1 + close]), pos + 1 + close + 1);
        }
    }
    (None, pos)
}

/// Read a `^`/`_` operand: a `{group}`, a braceless `\command`, or a single
/// following char.
fn read_script_group(s: &str, pos: usize) -> Option<(&str, usize)> {
    let bytes = s.as_bytes();
    if pos >= bytes.len() {
        return None;
    }
    if bytes[pos] == b'{' {
        return take_group(s, pos);
    }
    // Braceless command operand, e.g. `x^\alpha` — grab the whole `\name` so it
    // isn't truncated to a lone `\` (which would leave `alpha` as literal text).
    if bytes[pos] == b'\\' {
        let (name, end) = read_command(s, pos + 1);
        if !name.is_empty() {
            return Some((&s[pos..end], end));
        }
    }
    // Single char (one UTF-8 scalar).
    let ch_len = s[pos..].chars().next()?.len_utf8();
    Some((&s[pos..pos + ch_len], pos + ch_len))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_math() {
        assert!(contains_math("a $x$ b"));
        assert!(!contains_math("a $ b"));
        assert!(!contains_math("price \\$5"));
    }

    #[test]
    fn greek_and_operators() {
        assert_eq!(to_unicode("$\\sigma$"), "σ");
        assert_eq!(to_unicode("$\\alpha + \\beta$"), "α + β");
        assert_eq!(to_unicode("$a \\leq b \\cdot c$"), "a ≤ b · c");
        assert_eq!(to_unicode("$\\Omega$"), "Ω");
        assert_eq!(to_unicode("$\\infty$"), "∞");
    }

    #[test]
    fn superscripts_and_subscripts() {
        assert_eq!(to_unicode("$x^2$"), "x²");
        assert_eq!(to_unicode("$x_i$"), "xᵢ");
        assert_eq!(to_unicode("$x^{2n}$"), "x²ⁿ");
        assert_eq!(to_unicode("$x_{i+1}$"), "xᵢ₊₁");
    }

    #[test]
    fn superscript_all_or_nothing_fallback() {
        // 'q' has no Unicode superscript, so the whole group falls back.
        assert_eq!(to_unicode("$x^{2q}$"), "x^(2q)");
        // Uppercase generally has none either.
        assert_eq!(to_unicode("$x^{A}$"), "x^(A)");
    }

    #[test]
    fn fractions_inline() {
        assert_eq!(to_unicode("$\\frac{1}{2}$"), "1/2");
        assert_eq!(to_unicode("$\\frac{a+b}{c}$"), "(a+b)/c");
        assert_eq!(to_unicode("$\\frac{\\sqrt{a}}{b}$"), "(√a)/b");
    }

    #[test]
    fn sqrt_inline() {
        assert_eq!(to_unicode("$\\sqrt{x}$"), "√x");
        assert_eq!(to_unicode("$\\sqrt{x+y}$"), "√(x+y)");
        assert_eq!(to_unicode("$\\sqrt{x^2+y^2}$"), "√(x²+y²)");
    }

    #[test]
    fn nested_chain_stays_linear() {
        assert_eq!(
            to_unicode("$\\frac{\\sqrt{a}}{b^2} + \\sum_{i=1}^{n} x_i$"),
            "(√a)/(b²) + ∑ᵢ₌₁ⁿ xᵢ"
        );
    }

    #[test]
    fn no_backslash_or_dollar_in_output() {
        let out = to_unicode("$\\frac{\\unknown{a}}{\\sqrt{b}}$");
        assert!(!out.contains('\\'), "got {out}");
        assert!(!out.contains('$'), "got {out}");
    }

    #[test]
    fn text_around_math_preserved() {
        assert_eq!(
            to_unicode("Variance, $\\sigma^2$ (units)"),
            "Variance, σ² (units)"
        );
    }

    #[test]
    fn escaped_dollar_is_literal() {
        // `\$` marks a literal dollar; the escape is dropped on output.
        assert_eq!(to_unicode("price \\$5 each"), "price $5 each");
        assert!(!contains_math("price \\$5 each"));
        assert!(needs_rewrite("price \\$5 each"));
        assert!(!needs_rewrite("no dollars at all"));
    }

    // ── detection / segmentation edge cases ──

    #[test]
    fn detection_edges() {
        assert!(contains_math("$$")); // two dollars, even if empty
        assert!(contains_math("a $x$ b $y$ c"));
        assert!(!contains_math("")); // empty
        assert!(!contains_math("no math here"));
        assert!(!contains_math("\\$5 and \\$6")); // both escaped
    }

    #[test]
    fn empty_and_unclosed_math() {
        assert_eq!(to_unicode("$$"), ""); // empty region
        assert_eq!(to_unicode("a $ b"), "a $ b"); // unclosed → literal
        assert_eq!(to_unicode("$\\alpha"), "$\\alpha"); // unclosed → literal
    }

    #[test]
    fn multiple_regions() {
        assert_eq!(to_unicode("$\\alpha$ and $\\beta$"), "α and β");
        assert_eq!(to_unicode("$x^2$ vs $y_1$"), "x² vs y₁");
    }

    #[test]
    fn full_greek_sample() {
        assert_eq!(to_unicode("$\\theta$"), "θ");
        assert_eq!(to_unicode("$\\lambda$"), "λ");
        assert_eq!(to_unicode("$\\mu$"), "μ");
        assert_eq!(to_unicode("$\\varphi$"), "φ");
        assert_eq!(to_unicode("$\\Delta$"), "Δ");
        assert_eq!(to_unicode("$\\Sigma$"), "Σ");
        assert_eq!(to_unicode("$\\Psi$"), "Ψ");
    }

    #[test]
    fn operator_sample() {
        assert_eq!(to_unicode("$a \\times b$"), "a × b");
        assert_eq!(to_unicode("$a \\div b$"), "a ÷ b");
        assert_eq!(to_unicode("$x \\neq y$"), "x ≠ y");
        assert_eq!(to_unicode("$x \\approx y$"), "x ≈ y");
        assert_eq!(to_unicode("$x \\to \\infty$"), "x → ∞");
        assert_eq!(to_unicode("$\\partial f$"), "∂ f");
        assert_eq!(to_unicode("$x \\in S$"), "x ∈ S");
        assert_eq!(to_unicode("$90\\degree$"), "90°");
    }

    #[test]
    fn unknown_command_and_stray_braces() {
        assert_eq!(to_unicode("$\\foo{x}$"), "x"); // drop cmd, keep cleaned arg
        assert_eq!(to_unicode("$\\foo$"), ""); // bare unknown dropped
        assert_eq!(to_unicode("${x}$"), "x"); // stray braces stripped
    }

    #[test]
    fn subscript_then_superscript() {
        assert_eq!(to_unicode("$x_i^2$"), "xᵢ²");
    }

    #[test]
    fn braceless_command_as_script_operand() {
        // `x^\alpha` must grab the whole `\alpha`, not a lone `\` (which left
        // `alpha` as literal text). α has no Unicode superscript → clean fallback.
        assert_eq!(to_unicode("$x^\\alpha$"), "x^(α)");
        assert_eq!(to_unicode("$x_\\beta$"), "x_(β)");
    }

    #[test]
    fn subscript_all_or_nothing_fallback() {
        // 'b' has no Unicode subscript, so the group falls back.
        assert_eq!(to_unicode("$x_{bc}$"), "x_(bc)");
    }

    #[test]
    fn sqrt_with_index() {
        assert_eq!(to_unicode("$\\sqrt[3]{x}$"), "³√x"); // cube root index
        assert_eq!(to_unicode("$\\sqrt[n]{x}$"), "ⁿ√x"); // n has a superscript
        // A command index must be lowered, never leak its backslash.
        assert_eq!(to_unicode("$\\sqrt[\\alpha]{x}$"), "α√x");
    }

    #[test]
    fn structural_command_in_script_falls_back_cleanly() {
        // `\frac` inside a script group has no inline-superscript form; the
        // whole group must fall back to `^(1/2)` — never silently corrupt to
        // x¹² by dropping the fraction structure.
        assert_eq!(to_unicode("$x^{\\frac{1}{2}}$"), "x^(1/2)");
        assert_eq!(to_unicode("$x_{\\frac{a}{b}}$"), "x_(a/b)");
    }

    #[test]
    fn quadratic_formula_full_chain() {
        assert_eq!(
            to_unicode("$\\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}$"),
            "(-b ± √(b² - 4ac))/(2a)"
        );
    }

    #[test]
    fn circ_composition_operator() {
        assert_eq!(to_unicode("$f \\circ g$"), "f ∘ g");
    }

    #[test]
    fn operator_names_preserved() {
        assert_eq!(to_unicode("$\\log x$"), "log x");
        assert_eq!(to_unicode("$\\ln x$"), "ln x");
        assert_eq!(to_unicode("$\\sin(\\theta)$"), "sin(θ)");
        assert_eq!(to_unicode("$\\cos(\\phi)$"), "cos(φ)");
        assert_eq!(to_unicode("$\\exp(-x^2)$"), "exp(-x²)");
        assert_eq!(to_unicode("$\\min(a, b)$"), "min(a, b)");
        assert_eq!(to_unicode("$\\max(a, b)$"), "max(a, b)");
        assert_eq!(to_unicode("$\\lim_{x \\to 0}$"), "lim_(x → 0)"); // space has no sub → fallback
    }

    #[test]
    fn operator_name_with_subscript() {
        // Common case in bioscience: log₁₀ p-value axis
        assert_eq!(to_unicode("$-\\log_{10}(p)$"), "-log₁₀(p)");
        assert_eq!(to_unicode("$\\log_2 n$"), "log₂ n");
    }
}
