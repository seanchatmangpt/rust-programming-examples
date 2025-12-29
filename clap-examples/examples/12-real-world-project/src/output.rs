//! Output formatting for cloudctl

use crate::cli::OutputFormat;
use serde::Serialize;

/// Output formatter.
pub struct Formatter {
    format: OutputFormat,
}

impl Formatter {
    /// Create a new formatter.
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    /// Format data for output.
    pub fn format<T: Serialize + TableDisplay>(&self, data: &T) -> String {
        match self.format {
            OutputFormat::Json => self.format_json(data),
            OutputFormat::Yaml => self.format_yaml(data),
            OutputFormat::Csv => self.format_csv(data),
            OutputFormat::Table => self.format_table(data),
            OutputFormat::Plain => self.format_plain(data),
        }
    }

    /// Format a list of items.
    pub fn format_list<T: Serialize + TableDisplay>(&self, items: &[T]) -> String {
        match self.format {
            OutputFormat::Json => {
                serde_json::to_string_pretty(items).unwrap_or_else(|_| "[]".to_string())
            }
            OutputFormat::Yaml => {
                // Simple YAML-like output
                items
                    .iter()
                    .map(|item| format!("- {}", self.format_plain(item)))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            OutputFormat::Csv => {
                if items.is_empty() {
                    return String::new();
                }
                let headers = items.first().map(|i| i.headers()).unwrap_or_default();
                let mut lines = vec![headers.join(",")];
                for item in items {
                    lines.push(item.row().join(","));
                }
                lines.join("\n")
            }
            OutputFormat::Table => self.format_table_list(items),
            OutputFormat::Plain => {
                items
                    .iter()
                    .map(|item| self.format_plain(item))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
    }

    fn format_json<T: Serialize>(&self, data: &T) -> String {
        serde_json::to_string_pretty(data).unwrap_or_else(|_| "{}".to_string())
    }

    fn format_yaml<T: Serialize + TableDisplay>(&self, data: &T) -> String {
        // Simple YAML-like formatting
        let headers = data.headers();
        let values = data.row();
        headers
            .iter()
            .zip(values.iter())
            .map(|(h, v)| format!("{}: {}", h, v))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_csv<T: TableDisplay>(&self, data: &T) -> String {
        let headers = data.headers();
        let values = data.row();
        format!("{}\n{}", headers.join(","), values.join(","))
    }

    fn format_table<T: TableDisplay>(&self, data: &T) -> String {
        let headers = data.headers();
        let values = data.row();

        // Calculate column widths
        let widths: Vec<usize> = headers
            .iter()
            .zip(values.iter())
            .map(|(h, v)| h.len().max(v.len()))
            .collect();

        // Format header
        let header_line: String = headers
            .iter()
            .zip(widths.iter())
            .map(|(h, &w)| format!("{:width$}", h.to_uppercase(), width = w))
            .collect::<Vec<_>>()
            .join("  ");

        // Format separator
        let separator: String = widths.iter().map(|&w| "-".repeat(w)).collect::<Vec<_>>().join("  ");

        // Format values
        let value_line: String = values
            .iter()
            .zip(widths.iter())
            .map(|(v, &w)| format!("{:width$}", v, width = w))
            .collect::<Vec<_>>()
            .join("  ");

        format!("{}\n{}\n{}", header_line, separator, value_line)
    }

    fn format_table_list<T: TableDisplay>(&self, items: &[T]) -> String {
        if items.is_empty() {
            return "No items found".to_string();
        }

        let headers = items.first().map(|i| i.headers()).unwrap_or_default();
        let rows: Vec<Vec<String>> = items.iter().map(|i| i.row()).collect();

        // Calculate column widths
        let widths: Vec<usize> = headers
            .iter()
            .enumerate()
            .map(|(i, h)| {
                let max_value = rows
                    .iter()
                    .map(|r| r.get(i).map(|v| v.len()).unwrap_or(0))
                    .max()
                    .unwrap_or(0);
                h.len().max(max_value)
            })
            .collect();

        // Format header
        let header_line: String = headers
            .iter()
            .zip(widths.iter())
            .map(|(h, &w)| format!("{:width$}", h.to_uppercase(), width = w))
            .collect::<Vec<_>>()
            .join("  ");

        // Format separator
        let separator: String = widths.iter().map(|&w| "-".repeat(w)).collect::<Vec<_>>().join("  ");

        // Format rows
        let data_lines: Vec<String> = rows
            .iter()
            .map(|row| {
                row.iter()
                    .zip(widths.iter())
                    .map(|(v, &w)| format!("{:width$}", v, width = w))
                    .collect::<Vec<_>>()
                    .join("  ")
            })
            .collect();

        format!("{}\n{}\n{}", header_line, separator, data_lines.join("\n"))
    }

    fn format_plain<T: TableDisplay>(&self, data: &T) -> String {
        data.row().join(" ")
    }
}

/// Trait for types that can be displayed in a table.
pub trait TableDisplay {
    /// Get column headers.
    fn headers(&self) -> Vec<String>;

    /// Get row values.
    fn row(&self) -> Vec<String>;
}

// =============================================================================
// RESOURCE TYPES
// =============================================================================

/// A compute instance.
#[derive(Debug, Serialize)]
pub struct Instance {
    pub name: String,
    pub status: String,
    pub zone: String,
    pub machine_type: String,
    pub internal_ip: String,
    pub external_ip: Option<String>,
}

impl TableDisplay for Instance {
    fn headers(&self) -> Vec<String> {
        vec![
            "NAME".to_string(),
            "STATUS".to_string(),
            "ZONE".to_string(),
            "TYPE".to_string(),
            "INTERNAL_IP".to_string(),
            "EXTERNAL_IP".to_string(),
        ]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.name.clone(),
            self.status.clone(),
            self.zone.clone(),
            self.machine_type.clone(),
            self.internal_ip.clone(),
            self.external_ip.clone().unwrap_or_else(|| "-".to_string()),
        ]
    }
}

/// A storage bucket.
#[derive(Debug, Serialize)]
pub struct Bucket {
    pub name: String,
    pub location: String,
    pub storage_class: String,
    pub created: String,
}

impl TableDisplay for Bucket {
    fn headers(&self) -> Vec<String> {
        vec![
            "NAME".to_string(),
            "LOCATION".to_string(),
            "STORAGE_CLASS".to_string(),
            "CREATED".to_string(),
        ]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.name.clone(),
            self.location.clone(),
            self.storage_class.clone(),
            self.created.clone(),
        ]
    }
}

/// An IAM user.
#[derive(Debug, Serialize)]
pub struct User {
    pub name: String,
    pub path: String,
    pub created: String,
}

impl TableDisplay for User {
    fn headers(&self) -> Vec<String> {
        vec![
            "NAME".to_string(),
            "PATH".to_string(),
            "CREATED".to_string(),
        ]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.name.clone(),
            self.path.clone(),
            self.created.clone(),
        ]
    }
}

/// A VPC.
#[derive(Debug, Serialize)]
pub struct Vpc {
    pub name: String,
    pub cidr: String,
    pub state: String,
}

impl TableDisplay for Vpc {
    fn headers(&self) -> Vec<String> {
        vec![
            "NAME".to_string(),
            "CIDR".to_string(),
            "STATE".to_string(),
        ]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.name.clone(),
            self.cidr.clone(),
            self.state.clone(),
        ]
    }
}
