use colored::Colorize;
use doctor::{CheckReport, CheckResult};

pub fn print_reports(reports: &[CheckReport]) {
    println!();

    for report in reports {
        match &report.result {
            CheckResult::Ok => {
                println!("  {}  {}", "✔".green(), report.name);
            }
            CheckResult::Warning { message } => {
                println!(
                    "  {}  {}   {}",
                    "⚠".yellow(),
                    report.name,
                    message.dimmed()
                );
            }
            CheckResult::Error { message, fix } => {
                print!(
                    "  {}  {}   {}",
                    "✘".red(),
                    report.name,
                    message.dimmed()
                );
                if let Some(fix) = fix {
                    print!("   {} {}", "→".dimmed(), fix.cyan());
                }
                println!();
            }
        }
    }

    let errors = reports
        .iter()
        .filter(|r| matches!(r.result, CheckResult::Error { .. }))
        .count();
    let warnings = reports
        .iter()
        .filter(|r| matches!(r.result, CheckResult::Warning { .. }))
        .count();

    println!();
    if errors == 0 && warnings == 0 {
        println!("  {}", "All checks passed.".green().bold());
    } else {
        println!(
            "  {} errors, {} warnings",
            errors.to_string().red().bold(),
            warnings.to_string().yellow().bold()
        );
    }
    println!();
}
