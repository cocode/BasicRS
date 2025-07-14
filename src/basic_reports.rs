use std::collections::HashMap;
use std::fs;

use crate::basic_types::Program;

/// Coverage data: maps line numbers to sets of executed statement indices
pub type CoverageData = HashMap<usize, std::collections::HashSet<usize>>;

/// Print a text-based coverage report
pub fn print_coverage_report(coverage: &CoverageData, program: &Program, show_lines: bool) {
    let total_lines = program.lines.len();
    let mut total_stmts = 0;
    for line in &program.lines {
        total_stmts += line.statements.len();
    }

    if total_stmts == 0 {
        println!("Program is empty.");
        return;
    }

    let executed_lines = coverage.len();
    let mut executed_stmts = 0;
    for stmt_set in coverage.values() {
        executed_stmts += stmt_set.len();
    }

    let column = 20;
    println!("Code Coverage Report");
    println!("{:>width$} {:>width$} {:>width$}", "Total", "Executed", "Percent", width = column);
    println!("Lines.....: {:>width$} {:>width$} {:>width$.1}%", 
             total_lines, executed_lines, 
             100.0 * executed_lines as f64 / total_lines as f64,
             width = column);
    println!("Statements: {:>width$} {:>width$} {:>width$.1}%", 
             total_stmts, executed_stmts, 
             100.0 * executed_stmts as f64 / total_stmts as f64,
             width = column);

    if show_lines {
        println!("\nUncovered Lines:");
        for line in &program.lines {
            if !coverage.contains_key(&line.line_number) {
                println!("  Line {}: {}", line.line_number, line.source);
            } else {
                // Check for partially covered lines
                let stmt_count = line.statements.len();
                let covered_stmts = coverage.get(&line.line_number).unwrap();
                if covered_stmts.len() < stmt_count {
                    println!("  Line {} (partial): {}", line.line_number, line.source);
                }
            }
        }
    }
}

/// Generate a beautiful HTML coverage report
pub fn generate_html_coverage_report(coverage: &CoverageData, program: &Program, filename: &str) -> std::io::Result<()> {
    let total_lines = program.lines.len();
    let mut total_stmts = 0;
    for line in &program.lines {
        total_stmts += line.statements.len();
    }

    if total_stmts == 0 {
        println!("Program is empty.");
        return Ok(());
    }

    let executed_lines = coverage.len();
    let mut executed_stmts = 0;
    for stmt_set in coverage.values() {
        executed_stmts += stmt_set.len();
    }

    let line_coverage_percent = (executed_lines as f64 / total_lines as f64) * 100.0;
    let stmt_coverage_percent = (executed_stmts as f64 / total_stmts as f64) * 100.0;

    // Generate timestamp
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let timestamp = format!("{}", chrono::DateTime::from_timestamp(now as i64, 0).unwrap().format("%Y-%m-%d %H:%M:%S"));

    // Determine coverage quality classes
    let line_coverage_class = if line_coverage_percent >= 90.0 {
        "excellent"
    } else if line_coverage_percent >= 75.0 {
        "good"
    } else if line_coverage_percent >= 50.0 {
        "fair"
    } else {
        "poor"
    };

    let stmt_coverage_class = if stmt_coverage_percent >= 90.0 {
        "excellent"
    } else if stmt_coverage_percent >= 75.0 {
        "good"
    } else if stmt_coverage_percent >= 50.0 {
        "fair"
    } else {
        "poor"
    };

    // Generate program listing HTML
    let mut program_listing = String::new();
    for line in &program.lines {
        program_listing.push_str(&format!("<div class=\"program-line\">\n"));
        program_listing.push_str(&format!("    <div class=\"line-number\">{}</div>\n", line.line_number));
        program_listing.push_str(&format!("    <div class=\"line-code\">"));
        
        for (i, stmt) in line.statements.iter().enumerate() {
            let is_covered = coverage.get(&line.line_number)
                .map(|set| set.contains(&i))
                .unwrap_or(false);
            
            let class = if is_covered { "covered" } else { "uncovered" };
            program_listing.push_str(&format!("<span class=\"stmt-{}\">{}</span>", class, html_escape(&format!("{}", stmt))));
            
            if i < line.statements.len() - 1 {
                program_listing.push_str(" : ");
            }
        }
        
        program_listing.push_str("</div>\n");
        program_listing.push_str("</div>\n");
    }

    let html_content = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>BASIC Code Coverage Report</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 0;
            padding: 20px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            border-radius: 10px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.3);
            padding: 30px;
        }}
        .header {{
            text-align: center;
            margin-bottom: 30px;
            padding-bottom: 20px;
            border-bottom: 2px solid #e0e0e0;
        }}
        .header h1 {{
            color: #333;
            margin: 0;
            font-size: 2.5em;
        }}
        .header .timestamp {{
            color: #666;
            font-size: 0.9em;
            margin-top: 5px;
        }}
        .stats-overview {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }}
        .stat-card {{
            background: #f8f9fa;
            padding: 20px;
            border-radius: 8px;
            text-align: center;
            border-left: 4px solid #007bff;
        }}
        .stat-card h3 {{
            margin: 0;
            color: #333;
            font-size: 1.2em;
        }}
        .stat-card .number {{
            font-size: 2em;
            font-weight: bold;
            color: #007bff;
            margin: 10px 0;
        }}
        .stat-card .detail {{
            color: #666;
            font-size: 0.9em;
        }}
        .charts-container {{
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 30px;
            margin-bottom: 30px;
        }}
        .chart-card {{
            background: #f8f9fa;
            padding: 20px;
            border-radius: 8px;
            text-align: center;
        }}
        .chart-card h3 {{
            margin: 0 0 20px 0;
            color: #333;
        }}
        .chart-wrapper {{
            position: relative;
            height: 300px;
            margin: 0 auto;
        }}
        .uncovered-section {{
            margin-top: 30px;
        }}
        .uncovered-section h2 {{
            color: #333;
            border-bottom: 2px solid #e0e0e0;
            padding-bottom: 10px;
        }}
        .program-listing {{
            background: #f8f9fa;
            border: 1px solid #e0e0e0;
            border-radius: 6px;
            max-height: 600px;
            overflow-y: auto;
            margin: 20px 0;
            box-shadow: inset 0 2px 4px rgba(0,0,0,0.1);
        }}
        .program-line {{
            display: flex;
            align-items: center;
            padding: 4px 0;
            border-bottom: 1px solid #f0f0f0;
            font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
            font-size: 14px;
            transition: background-color 0.2s ease;
        }}
        .program-line:hover {{
            background-color: rgba(0,0,0,0.05);
        }}
        .stmt-covered {{
            background-color: #d4edda;
            padding: 2px 4px;
            border-radius: 3px;
            margin: 0 2px;
            border: 1px solid #28a745;
            display: inline-block;
        }}
        .stmt-uncovered {{
            background-color: #f8d7da;
            padding: 2px 4px;
            border-radius: 3px;
            margin: 0 2px;
            border: 1px solid #dc3545;
            display: inline-block;
        }}
        .line-number {{
            min-width: 60px;
            padding: 0 15px;
            text-align: right;
            color: #666;
            font-weight: bold;
            background-color: rgba(255,255,255,0.7);
            border-right: 1px solid #ddd;
        }}
        .line-code {{
            padding: 0 15px;
            flex: 1;
            white-space: pre-wrap;
            word-break: break-all;
        }}
        .coverage-excellent {{ border-left-color: #28a745; }}
        .coverage-good {{ border-left-color: #007bff; }}
        .coverage-fair {{ border-left-color: #ffc107; }}
        .coverage-poor {{ border-left-color: #dc3545; }}
        @media (max-width: 768px) {{
            .charts-container {{
                grid-template-columns: 1fr;
            }}
            .stats-overview {{
                grid-template-columns: 1fr;
            }}
            .program-listing {{
                max-height: 400px;
            }}
            .line-number {{
                min-width: 50px;
                padding: 0 10px;
            }}
            .line-code {{
                padding: 0 10px;
                font-size: 12px;
            }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🚀 BASIC Code Coverage Report</h1>
            <div class="timestamp">Generated on {timestamp}</div>
        </div>

        <div class="stats-overview">
            <div class="stat-card coverage-{line_coverage_class}">
                <h3>Line Coverage</h3>
                <div class="number">{line_coverage_percent:.1}%</div>
                <div class="detail">{executed_lines} of {total_lines} lines</div>
            </div>
            <div class="stat-card coverage-{stmt_coverage_class}">
                <h3>Statement Coverage</h3>
                <div class="number">{stmt_coverage_percent:.1}%</div>
                <div class="detail">{executed_stmts} of {total_stmts} statements</div>
            </div>
        </div>

        <div class="charts-container">
            <div class="chart-card">
                <h3>Line Coverage</h3>
                <div class="chart-wrapper">
                    <canvas id="lineChart"></canvas>
                </div>
            </div>
            <div class="chart-card">
                <h3>Statement Coverage</h3>
                <div class="chart-wrapper">
                    <canvas id="statementChart"></canvas>
                </div>
            </div>
        </div>

        <div class="uncovered-section">
            <h2>📋 Complete Program Listing</h2>
            <p>Full program with statement-level coverage visualization: 
               <span style="background-color: #d4edda; padding: 2px 4px; border-radius: 3px;">Covered Statement</span> | 
               <span style="background-color: #f8d7da; padding: 2px 4px; border-radius: 3px;">Uncovered Statement</span></p>

            <div class="program-listing">
                {program_listing}
            </div>
        </div>
    </div>

    <script>
        // Line Coverage Chart
        const lineCtx = document.getElementById('lineChart').getContext('2d');
        const lineChart = new Chart(lineCtx, {{
            type: 'doughnut',
            data: {{
                labels: ['Covered', 'Uncovered'],
                datasets: [{{
                    data: [{executed_lines}, {uncovered_lines}],
                    backgroundColor: ['#28a745', '#dc3545'],
                    borderWidth: 2,
                    borderColor: '#fff'
                }}]
            }},
            options: {{
                responsive: true,
                maintainAspectRatio: false,
                plugins: {{
                    legend: {{
                        position: 'bottom'
                    }},
                    tooltip: {{
                        callbacks: {{
                            label: function(context) {{
                                const label = context.label || '';
                                const value = context.parsed || 0;
                                const total = {total_lines};
                                const percentage = ((value / total) * 100).toFixed(1);
                                return `${{label}}: ${{value}} (${{percentage}}%)`;
                            }}
                        }}
                    }}
                }}
            }}
        }});

        // Statement Coverage Chart
        const stmtCtx = document.getElementById('statementChart').getContext('2d');
        const statementChart = new Chart(stmtCtx, {{
            type: 'doughnut',
            data: {{
                labels: ['Covered', 'Uncovered'],
                datasets: [{{
                    data: [{executed_stmts}, {uncovered_stmts}],
                    backgroundColor: ['#007bff', '#ffc107'],
                    borderWidth: 2,
                    borderColor: '#fff'
                }}]
            }},
            options: {{
                responsive: true,
                maintainAspectRatio: false,
                plugins: {{
                    legend: {{
                        position: 'bottom'
                    }},
                    tooltip: {{
                        callbacks: {{
                            label: function(context) {{
                                const label = context.label || '';
                                const value = context.parsed || 0;
                                const total = {total_stmts};
                                const percentage = ((value / total) * 100).toFixed(1);
                                return `${{label}}: ${{value}} (${{percentage}}%)`;
                            }}
                        }}
                    }}
                }}
            }}
        }});
    </script>
</body>
</html>"#,
        timestamp = timestamp,
        line_coverage_class = line_coverage_class,
        line_coverage_percent = line_coverage_percent,
        executed_lines = executed_lines,
        total_lines = total_lines,
        stmt_coverage_class = stmt_coverage_class,
        stmt_coverage_percent = stmt_coverage_percent,
        executed_stmts = executed_stmts,
        total_stmts = total_stmts,
        program_listing = program_listing,
        uncovered_lines = total_lines - executed_lines,
        uncovered_stmts = total_stmts - executed_stmts,
    );

    // Write HTML file
    fs::write(filename, html_content)?;

    println!();
    println!("HTML coverage report generated: {}", filename);
    println!("Line coverage: {:.1}% ({}/{})", line_coverage_percent, executed_lines, total_lines);
    println!("Statement coverage: {:.1}% ({}/{})", stmt_coverage_percent, executed_stmts, total_stmts);

    Ok(())
}

/// HTML escape utility function
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Save coverage data to a JSON file
pub fn save_coverage_to_file(coverage: &CoverageData, filename: &str) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(coverage)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fs::write(filename, json)
}

/// Load coverage data from a JSON file
pub fn load_coverage_from_file(filename: &str) -> std::io::Result<CoverageData> {
    let content = fs::read_to_string(filename)?;
    serde_json::from_str(&content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// Merge two coverage datasets, combining statement sets for each line
pub fn merge_coverage(mut existing: CoverageData, new: CoverageData) -> CoverageData {
    for (line_num, new_stmts) in new {
        existing.entry(line_num)
            .or_insert_with(std::collections::HashSet::new)
            .extend(new_stmts);
    }
    existing
} 