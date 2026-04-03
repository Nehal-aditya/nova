// NOVA Code Formatter Implementation
// Opinionated formatter that enforces consistent style

use std::fmt;

/// Formatting options (currently minimal - nova-fmt is opinionated)
#[derive(Debug, Clone)]
pub struct FormatOptions {
    /// Tab width (default: 4)
    pub tab_width: usize,
    /// Use tabs instead of spaces (default: false)
    pub use_tabs: bool,
    /// Maximum line length before wrapping (default: 100)
    pub max_width: usize,
}

impl Default for FormatOptions {
    fn default() -> Self {
        FormatOptions {
            tab_width: 4,
            use_tabs: false,
            max_width: 100,
        }
    }
}

/// Result of formatting operation
#[derive(Debug)]
pub struct FormatResult {
    pub formatted: String,
    pub changed: bool,
}

/// The NOVA source code formatter
pub struct Formatter {
    options: FormatOptions,
    indent_level: usize,
    output: String,
}

impl Formatter {
    pub fn new(options: FormatOptions) -> Self {
        Formatter {
            options,
            indent_level: 0,
            output: String::new(),
        }
    }

    /// Format NOVA source code
    pub fn format(&mut self, source: &str) -> Result<String, String> {
        self.output.clear();
        self.indent_level = 0;
        
        // For now, implement basic formatting rules:
        // 1. Normalize whitespace
        // 2. Ensure consistent indentation
        // 3. Handle braces and blocks
        // 4. Normalize arrows (-> to →)
        
        let normalized = self.normalize_whitespace(source);
        let formatted = self.format_tokens(&normalized);
        
        Ok(formatted)
    }

    fn normalize_whitespace(&self, source: &str) -> String {
        let mut result = String::new();
        let mut prev_char = ' ';
        
        for ch in source.chars() {
            // Skip multiple consecutive spaces
            if ch == ' ' && prev_char == ' ' {
                continue;
            }
            
            // Normalize -> to → (except in comments)
            if ch == '-' && prev_char == ' ' {
                // Will be handled in format_tokens
            }
            
            result.push(ch);
            prev_char = ch;
        }
        
        result
    }

    fn format_tokens(&mut self, source: &str) -> String {
        let mut result = String::new();
        let mut lines = source.lines().peekable();
        
        while let Some(line) = lines.next() {
            let trimmed = line.trim();
            
            // Skip empty lines at the beginning
            if trimmed.is_empty() && result.is_empty() {
                continue;
            }
            
            // Normalize arrow syntax: -> becomes →
            let formatted_line = trimmed.replace("->", "→");
            
            // Add indentation based on brace tracking
            let indent = self.get_indent_string();
            
            if !formatted_line.is_empty() {
                result.push_str(&indent);
                result.push_str(&formatted_line);
            }
            
            result.push('\n');
            
            // Track indentation changes
            self.update_indent_level(trimmed);
        }
        
        // Remove trailing newline
        if result.ends_with('\n') {
            result.pop();
        }
        
        result
    }

    fn get_indent_string(&self) -> String {
        if self.options.use_tabs {
            "\t".repeat(self.indent_level)
        } else {
            " ".repeat(self.indent_level * self.options.tab_width)
        }
    }

    fn update_indent_level(&mut self, line: &str) {
        // Count opening and closing braces
        let open_braces = line.matches('{').count();
        let close_braces = line.matches('}').count();
        
        // Decrease indent for closing braces on their own line
        if line.trim() == "}" {
            if self.indent_level > 0 {
                self.indent_level -= 1;
            }
        }
        
        // Increase indent for lines ending with {
        if line.ends_with('{') {
            self.indent_level += 1;
        }
        
        // Handle single-line braces like "} else {"
        if open_braces > close_braces {
            self.indent_level += open_braces - close_braces;
        }
    }
}

impl fmt::Display for Formatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Formatter(indent={}, tab_width={})", 
               self.indent_level, self.options.tab_width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arrow_normalization() {
        let mut formatter = Formatter::new(FormatOptions::default());
        let input = "mission main() -> Void { }";
        let result = formatter.format(input).unwrap();
        assert!(result.contains("→"));
    }

    #[test]
    fn test_indentation() {
        let mut formatter = Formatter::new(FormatOptions::default());
        let input = "mission main() -> Void {\ntransmit(\"hi\")\n}";
        let result = formatter.format(input).unwrap();
        // Should have proper indentation
        assert!(result.contains("    transmit"));
    }
}