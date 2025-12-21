//! Steward CLI
//!
//! Command-line interface for stewardship contract evaluation.
//!
//! ## Usage
//!
//! ```bash
//! # Evaluate output against contract
//! steward evaluate --contract contract.yaml --output response.txt
//!
//! # Pipe from stdin
//! cat response.txt | steward evaluate --contract contract.yaml
//!
//! # JSON output
//! steward evaluate --contract contract.yaml --output response.txt --format json
//!
//! # Validate contract
//! steward contract validate contract.yaml
//! ```
//!
//! ## Exit Codes
//!
//! - 0: PROCEED
//! - 1: ESCALATE
//! - 2: BLOCKED
//! - 3: Error

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::ExitCode;

use steward_core::{Contract, Output, State};

/// Steward: Stewardship contracts for AI systems
#[derive(Parser)]
#[command(name = "steward")]
#[command(author = "Patrick Peña - Agenisea AI")]
#[command(version)]
#[command(about = "Evaluate AI outputs against stewardship contracts", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Evaluate an output against a contract
    Evaluate {
        /// Path to the stewardship contract (YAML or JSON)
        #[arg(short, long)]
        contract: PathBuf,

        /// Path to the output file to evaluate (reads from stdin if not provided)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output format
        #[arg(short, long, default_value = "text")]
        format: OutputFormat,

        /// Show detailed explanation
        #[arg(long)]
        explain: bool,

        /// Context to include (can be specified multiple times)
        #[arg(long)]
        context: Vec<String>,
    },

    /// Contract management commands
    Contract {
        #[command(subcommand)]
        action: ContractAction,
    },
}

#[derive(Subcommand)]
enum ContractAction {
    /// Validate a contract file
    Validate {
        /// Path to the contract file
        path: PathBuf,
    },

    /// Show contract details
    Show {
        /// Path to the contract file
        path: PathBuf,
    },

    /// List contracts in a directory
    List {
        /// Directory containing contracts
        #[arg(default_value = "./contracts")]
        path: PathBuf,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

fn main() -> ExitCode {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .init();

    match run() {
        Ok(exit_code) => exit_code,
        Err(e) => {
            eprintln!("Error: {:#}", e);
            ExitCode::from(3)
        }
    }
}

fn run() -> Result<ExitCode> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Evaluate {
            contract,
            output,
            format,
            explain,
            context,
        } => evaluate_command(contract, output, format, explain, context),

        Commands::Contract { action } => match action {
            ContractAction::Validate { path } => validate_contract(path),
            ContractAction::Show { path } => show_contract(path),
            ContractAction::List { path } => list_contracts(path),
        },
    }
}

fn evaluate_command(
    contract_path: PathBuf,
    output_path: Option<PathBuf>,
    format: OutputFormat,
    explain: bool,
    context: Vec<String>,
) -> Result<ExitCode> {
    // Load contract
    let contract = if contract_path.extension().map(|e| e == "json").unwrap_or(false) {
        Contract::from_json_file(&contract_path)
    } else {
        Contract::from_yaml_file(&contract_path)
    }
    .with_context(|| format!("Failed to load contract from {:?}", contract_path))?;

    // Load output
    let content = match output_path {
        Some(path) => {
            std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read output from {:?}", path))?
        }
        None => {
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .context("Failed to read from stdin")?;
            buffer
        }
    };

    let output = Output::text(content);

    // Get context if provided
    let context_ref: Option<&[String]> = if context.is_empty() {
        None
    } else {
        Some(&context)
    };

    // Evaluate
    let result = steward_core::evaluate_with_context(&contract, &output, context_ref, None)
        .context("Evaluation failed")?;

    // Output results
    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&result)?;
            println!("{}", json);
        }
        OutputFormat::Text => {
            print_text_result(&result, explain);
        }
    }

    // Return appropriate exit code
    Ok(match result.state {
        State::Proceed { .. } => ExitCode::from(0),
        State::Escalate { .. } => ExitCode::from(1),
        State::Blocked { .. } => ExitCode::from(2),
    })
}

fn print_text_result(result: &steward_core::EvaluationResult, explain: bool) {
    match &result.state {
        State::Proceed { summary } => {
            println!("PROCEED");
            println!();
            println!("{}", summary);
            println!();
            println!("Confidence: {:.0}%", result.confidence * 100.0);
        }
        State::Escalate {
            uncertainty,
            decision_point,
            options,
        } => {
            println!("ESCALATE");
            println!();
            println!("Trigger: {}", uncertainty);
            println!();
            println!("Decision: {}", decision_point);
            println!();
            println!("Options:");
            for (i, option) in options.iter().enumerate() {
                println!("  {}. {}", i + 1, option);
            }
            println!();
            println!("Confidence: {:.0}%", result.confidence * 100.0);
        }
        State::Blocked { violation } => {
            println!("BLOCKED");
            println!();
            println!("Violation: {} - {}", violation.rule_id, violation.rule_text);
            println!();
            println!("Lens: {:?}", violation.lens);
            println!();
            if !violation.evidence.is_empty() {
                println!("Evidence:");
                for e in &violation.evidence {
                    println!("  - {}: {} ({:?})", e.claim, e.pointer, e.source);
                }
                println!();
            }
            println!("Contact: {}", violation.accountable_human);
            println!();
            println!("Confidence: {:.0}%", result.confidence * 100.0);
        }
    }

    if explain {
        println!();
        println!("--- Lens Findings ---");
        println!();

        let findings = [
            ("Dignity & Inclusion", &result.lens_findings.dignity_inclusion),
            ("Boundaries & Safety", &result.lens_findings.boundaries_safety),
            ("Restraint & Privacy", &result.lens_findings.restraint_privacy),
            ("Transparency & Contestability", &result.lens_findings.transparency_contestability),
            ("Accountability & Ownership", &result.lens_findings.accountability_ownership),
        ];

        for (name, finding) in findings {
            let status = if finding.state.is_pass() {
                "PASS"
            } else if finding.state.is_escalate() {
                "ESCALATE"
            } else {
                "BLOCKED"
            };

            println!(
                "{}: {} ({:.0}% confidence)",
                name,
                status,
                finding.confidence * 100.0
            );

            if !finding.rules_evaluated.is_empty() {
                for rule in &finding.rules_evaluated {
                    println!("  - {}: {:?}", rule.rule_id, rule.result);
                }
            }
        }
    }
}

fn validate_contract(path: PathBuf) -> Result<ExitCode> {
    let contract = if path.extension().map(|e| e == "json").unwrap_or(false) {
        Contract::from_json_file(&path)
    } else {
        Contract::from_yaml_file(&path)
    };

    match contract {
        Ok(c) => {
            println!("Contract is valid: {}", c.name);
            println!();
            println!("Version: {}", c.contract_version);
            println!("Schema: {}", c.schema_version);
            println!("Purpose: {}", c.intent.purpose);
            println!("Accountable: {}", c.accountability.answerable_human);
            Ok(ExitCode::from(0))
        }
        Err(e) => {
            eprintln!("Contract validation failed: {}", e);
            Ok(ExitCode::from(1))
        }
    }
}

fn show_contract(path: PathBuf) -> Result<ExitCode> {
    let contract = if path.extension().map(|e| e == "json").unwrap_or(false) {
        Contract::from_json_file(&path)
    } else {
        Contract::from_yaml_file(&path)
    }
    .with_context(|| format!("Failed to load contract from {:?}", path))?;

    println!("Contract: {}", contract.name);
    println!("Version: {} (schema {})", contract.contract_version, contract.schema_version);
    println!();

    println!("Intent:");
    println!("  Purpose: {}", contract.intent.purpose);
    if !contract.intent.optimizing_for.is_empty() {
        println!("  Optimizing for:");
        for opt in &contract.intent.optimizing_for {
            println!("    - {}", opt);
        }
    }
    println!();

    println!("Boundaries:");
    println!("  May do autonomously: {} rules", contract.boundaries.may_do_autonomously.len());
    println!("  Must pause when: {} rules", contract.boundaries.must_pause_when.len());
    println!("  Must escalate when: {} rules", contract.boundaries.must_escalate_when.len());
    println!("  Invalidated by: {} rules", contract.boundaries.invalidated_by.len());
    println!();

    println!("Accountability:");
    println!("  Answerable human: {}", contract.accountability.answerable_human);
    if let Some(approved) = &contract.accountability.approved_by {
        println!("  Approved by: {}", approved);
    }
    println!("  Escalation path: {} levels", contract.accountability.escalation_path.len());
    println!();

    println!("Acceptance:");
    println!("  Fit criteria: {} rules", contract.acceptance.fit_criteria.len());
    println!("  Dignity checks: {} rules", contract.acceptance.dignity_check.len());

    Ok(ExitCode::from(0))
}

fn list_contracts(path: PathBuf) -> Result<ExitCode> {
    let entries = std::fs::read_dir(&path)
        .with_context(|| format!("Failed to read directory {:?}", path))?;

    let mut found = false;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map(|e| e == "yaml" || e == "yml" || e == "json").unwrap_or(false) {
            let contract = if path.extension().map(|e| e == "json").unwrap_or(false) {
                Contract::from_json_file(&path)
            } else {
                Contract::from_yaml_file(&path)
            };

            match contract {
                Ok(c) => {
                    println!(
                        "{}: {} (v{})",
                        path.file_name().unwrap().to_string_lossy(),
                        c.name,
                        c.contract_version
                    );
                    found = true;
                }
                Err(e) => {
                    eprintln!(
                        "{}: INVALID - {}",
                        path.file_name().unwrap().to_string_lossy(),
                        e
                    );
                }
            }
        }
    }

    if !found {
        println!("No contracts found in {:?}", path);
    }

    Ok(ExitCode::from(0))
}
