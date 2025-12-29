# Case Study: DevOps Tooling

> **Chapter 17** | Part 4: Real-World Systems | Estimated reading time: 24 minutes

DevOps tools like kubectl, terraform, docker, and ansible have unique CLI requirements that differ significantly from developer utilities. They manage infrastructure across multiple environments, handle sensitive credentials, operate in both interactive and automated contexts, and must provide robust error recovery. This chapter explores patterns for building deployment and infrastructure CLIs based on production experience with container orchestration and configuration management tools.

## Multi-Target Deployment CLIs

### Environment and Context Management

Infrastructure tools must manage multiple deployment targets. The challenge is providing flexibility while preventing accidental production deployments.

```rust
use clap::{Parser, Subcommand, Args, ValueEnum};
use std::path::PathBuf;

/// Infrastructure management CLI
#[derive(Parser)]
#[command(
    name = "infra",
    version,
    about = "Multi-environment infrastructure management",
    after_help = "Use 'infra <command> --help' for more information about a command."
)]
pub struct Cli {
    /// Target environment (overrides INFRA_ENV)
    #[arg(
        short, long,
        env = "INFRA_ENV",
        global = true,
        value_enum,
        default_value_t = Environment::Dev
    )]
    pub environment: Environment,

    /// Target cluster/region (environment-specific)
    #[arg(short = 'c', long, env = "INFRA_CLUSTER", global = true)]
    pub cluster: Option<String>,

    /// Path to kubeconfig or similar context file
    #[arg(long, env = "INFRA_CONFIG", global = true)]
    pub config: Option<PathBuf>,

    /// Override default namespace
    #[arg(short = 'n', long, env = "INFRA_NAMESPACE", global = true)]
    pub namespace: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Environment {
    Dev,
    Staging,
    #[value(alias = "prod")]
    Production,
}

impl Environment {
    /// Production requires explicit confirmation
    pub fn requires_confirmation(&self) -> bool {
        matches!(self, Environment::Production)
    }

    /// Default namespace per environment
    pub fn default_namespace(&self) -> &'static str {
        match self {
            Environment::Dev => "development",
            Environment::Staging => "staging",
            Environment::Production => "production",
        }
    }

    /// Environment-specific timeout multipliers
    pub fn timeout_multiplier(&self) -> f64 {
        match self {
            Environment::Dev => 1.0,
            Environment::Staging => 1.5,
            Environment::Production => 2.0, // More conservative in prod
        }
    }
}
```

### Resource Targeting with Type Safety

kubectl-style resource targeting benefits from strong typing to catch errors at parse time.

```rust
use std::str::FromStr;

/// Resource selector with validation
#[derive(Args)]
pub struct ResourceSelector {
    /// Resource type (deployment, service, pod, etc.)
    #[arg(short = 'k', long)]
    pub kind: ResourceKind,

    /// Resource name (supports wildcards: app-*)
    pub name: String,

    /// Label selector (key=value pairs)
    #[arg(short = 'l', long = "selector", value_parser = parse_label_selector)]
    pub labels: Vec<LabelSelector>,

    /// Field selector for filtering
    #[arg(long, value_parser = parse_field_selector)]
    pub field_selector: Option<FieldSelector>,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum ResourceKind {
    Deployment,
    Service,
    Pod,
    ConfigMap,
    Secret,
    Ingress,
    Job,
    CronJob,
    StatefulSet,
    DaemonSet,
}

#[derive(Clone, Debug)]
pub struct LabelSelector {
    pub key: String,
    pub operator: SelectorOperator,
    pub values: Vec<String>,
}

#[derive(Clone, Debug)]
pub enum SelectorOperator {
    Equals,
    NotEquals,
    In,
    NotIn,
    Exists,
    DoesNotExist,
}

fn parse_label_selector(s: &str) -> Result<LabelSelector, String> {
    // Parse kubernetes-style label selectors
    // Examples: "app=nginx", "env!=prod", "tier in (frontend,backend)"
    if let Some((key, value)) = s.split_once('=') {
        if key.ends_with('!') {
            Ok(LabelSelector {
                key: key.trim_end_matches('!').to_string(),
                operator: SelectorOperator::NotEquals,
                values: vec![value.to_string()],
            })
        } else {
            Ok(LabelSelector {
                key: key.to_string(),
                operator: SelectorOperator::Equals,
                values: vec![value.to_string()],
            })
        }
    } else if s.contains(" in ") {
        let parts: Vec<&str> = s.splitn(2, " in ").collect();
        if parts.len() != 2 {
            return Err("Invalid 'in' selector format".to_string());
        }
        let values = parts[1]
            .trim_start_matches('(')
            .trim_end_matches(')')
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        Ok(LabelSelector {
            key: parts[0].to_string(),
            operator: SelectorOperator::In,
            values,
        })
    } else {
        // Existence check
        Ok(LabelSelector {
            key: s.to_string(),
            operator: SelectorOperator::Exists,
            values: vec![],
        })
    }
}
```

### Multi-Cluster Operations

Production tools often need to operate across multiple clusters simultaneously.

```rust
#[derive(Args)]
pub struct MultiClusterArgs {
    /// Target clusters (comma-separated or 'all')
    #[arg(long, value_delimiter = ',', required = true)]
    pub clusters: Vec<String>,

    /// Continue on cluster failure
    #[arg(long)]
    pub continue_on_error: bool,

    /// Maximum parallel operations
    #[arg(long, default_value = "3")]
    pub parallelism: usize,

    /// Rollout strategy for multi-cluster
    #[arg(long, value_enum, default_value_t = RolloutStrategy::Sequential)]
    pub strategy: RolloutStrategy,
}

#[derive(Clone, ValueEnum)]
pub enum RolloutStrategy {
    /// Deploy to all clusters simultaneously
    Parallel,
    /// Deploy one cluster at a time
    Sequential,
    /// Deploy to canary cluster first, then others
    Canary,
    /// Blue-green: switch traffic after all ready
    BlueGreen,
}

impl MultiClusterArgs {
    pub fn resolve_clusters(&self, known_clusters: &[String]) -> Result<Vec<String>, Error> {
        if self.clusters.len() == 1 && self.clusters[0] == "all" {
            return Ok(known_clusters.to_vec());
        }

        // Validate all specified clusters exist
        for cluster in &self.clusters {
            if !known_clusters.contains(cluster) {
                return Err(Error::UnknownCluster {
                    name: cluster.clone(),
                    available: known_clusters.to_vec(),
                });
            }
        }

        Ok(self.clusters.clone())
    }
}
```

## Interactive vs Batch Modes

### Detecting and Adapting to Execution Context

DevOps tools run in terminals, CI pipelines, cron jobs, and scripts. Each context has different requirements.

```rust
use std::io::{IsTerminal, stdin, stdout};

#[derive(Args)]
pub struct InteractionArgs {
    /// Non-interactive mode (assume yes to prompts)
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// Force interactive prompts even in non-TTY
    #[arg(long)]
    pub force_interactive: bool,

    /// Output format for scripting
    #[arg(long, short = 'o', value_enum, default_value_t = OutputFormat::Auto)]
    pub output: OutputFormat,

    /// Disable progress indicators
    #[arg(long)]
    pub no_progress: bool,
}

#[derive(Clone, ValueEnum, Default)]
pub enum OutputFormat {
    #[default]
    Auto,
    Text,
    Json,
    Yaml,
    Table,
    #[value(alias = "go-template")]
    GoTemplate,
}

impl InteractionArgs {
    /// Determine if we're in an interactive context
    pub fn is_interactive(&self) -> bool {
        if self.yes {
            return false;
        }
        if self.force_interactive {
            return true;
        }
        // Check if both stdin and stdout are terminals
        stdin().is_terminal() && stdout().is_terminal()
    }

    /// Resolve output format based on context
    pub fn resolved_output(&self) -> OutputFormat {
        match self.output {
            OutputFormat::Auto => {
                if stdout().is_terminal() {
                    OutputFormat::Table
                } else {
                    OutputFormat::Json
                }
            }
            other => other,
        }
    }

    /// Should we show progress bars?
    pub fn show_progress(&self) -> bool {
        !self.no_progress && stdout().is_terminal()
    }
}
```

### Confirmation Prompts with Risk Assessment

Production environments need extra protection against accidental destructive operations.

```rust
use dialoguer::{Confirm, theme::ColorfulTheme};

pub struct ConfirmationManager {
    environment: Environment,
    interactive: bool,
    force: bool,
}

impl ConfirmationManager {
    pub fn new(env: Environment, args: &InteractionArgs) -> Self {
        Self {
            environment: env,
            interactive: args.is_interactive(),
            force: args.yes,
        }
    }

    /// Confirm a destructive action with environment-aware messaging
    pub fn confirm_destructive(
        &self,
        action: &str,
        resources: &[String],
    ) -> Result<bool, Error> {
        // In non-interactive with --yes, proceed
        if !self.interactive && self.force {
            return Ok(true);
        }

        // In non-interactive without --yes, fail safely
        if !self.interactive {
            return Err(Error::NonInteractiveDestructiveAction {
                action: action.to_string(),
                hint: "Use --yes to confirm, or run interactively".to_string(),
            });
        }

        // Build confirmation message
        let resource_list = if resources.len() <= 5 {
            resources.join(", ")
        } else {
            format!("{} and {} more", resources[..3].join(", "), resources.len() - 3)
        };

        let prompt = format!(
            "{} {} in {:?}?\nResources: {}",
            action, resources.len(), self.environment, resource_list
        );

        // Production gets extra confirmation
        if self.environment.requires_confirmation() {
            eprintln!("\n{}", "WARNING: You are targeting PRODUCTION".red().bold());
            eprintln!("This action will {} the following resources:\n", action);
            for resource in resources.iter().take(10) {
                eprintln!("  - {}", resource);
            }
            if resources.len() > 10 {
                eprintln!("  ... and {} more", resources.len() - 10);
            }
            eprintln!();

            // Require typing environment name
            let input: String = dialoguer::Input::new()
                .with_prompt("Type 'production' to confirm")
                .interact_text()?;

            if input.to_lowercase() != "production" {
                return Ok(false);
            }
        }

        Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .default(false)
            .interact()
            .map_err(Error::from)
    }
}
```

### Dry-Run and Plan Modes

Terraform-style plan/apply workflows prevent accidents and enable review.

```rust
#[derive(Args)]
pub struct ExecutionArgs {
    /// Show what would be done without making changes
    #[arg(long)]
    pub dry_run: bool,

    /// Generate detailed execution plan
    #[arg(long)]
    pub plan: bool,

    /// Apply changes from a saved plan file
    #[arg(long, conflicts_with_all = ["dry_run", "plan"])]
    pub plan_file: Option<PathBuf>,

    /// Auto-approve plan (skip confirmation)
    #[arg(long, requires = "plan")]
    pub auto_approve: bool,
}

#[derive(Debug)]
pub struct ExecutionPlan {
    pub changes: Vec<PlannedChange>,
    pub warnings: Vec<String>,
    pub requires_downtime: bool,
}

#[derive(Debug)]
pub struct PlannedChange {
    pub action: ChangeAction,
    pub resource: String,
    pub before: Option<serde_json::Value>,
    pub after: Option<serde_json::Value>,
}

#[derive(Debug)]
pub enum ChangeAction {
    Create,
    Update,
    Delete,
    Replace,
    NoOp,
}

impl ExecutionPlan {
    pub fn summarize(&self) -> String {
        let creates = self.changes.iter()
            .filter(|c| matches!(c.action, ChangeAction::Create))
            .count();
        let updates = self.changes.iter()
            .filter(|c| matches!(c.action, ChangeAction::Update))
            .count();
        let deletes = self.changes.iter()
            .filter(|c| matches!(c.action, ChangeAction::Delete))
            .count();

        format!(
            "Plan: {} to create, {} to update, {} to delete",
            creates, updates, deletes
        )
    }

    pub fn display(&self) {
        for change in &self.changes {
            let symbol = match change.action {
                ChangeAction::Create => "+".green(),
                ChangeAction::Update => "~".yellow(),
                ChangeAction::Delete => "-".red(),
                ChangeAction::Replace => "+-".cyan(),
                ChangeAction::NoOp => " ".normal(),
            };
            println!("{} {}", symbol, change.resource);
        }
    }
}
```

## Credential Management Patterns

### Multi-Source Credential Resolution

Production tools need flexible credential management with clear precedence.

```rust
use secrecy::{ExposeSecret, Secret};
use std::collections::HashMap;

#[derive(Args)]
pub struct AuthArgs {
    /// Authentication token (prefer INFRA_TOKEN env var)
    #[arg(long, env = "INFRA_TOKEN", hide_env_values = true)]
    pub token: Option<Secret<String>>,

    /// Path to credentials file
    #[arg(long, env = "INFRA_CREDENTIALS")]
    pub credentials_file: Option<PathBuf>,

    /// Use OS keychain for credentials
    #[arg(long)]
    pub use_keychain: bool,

    /// Credential profile name
    #[arg(long, default_value = "default")]
    pub profile: String,
}

pub struct CredentialResolver {
    sources: Vec<Box<dyn CredentialSource>>,
}

pub trait CredentialSource: Send + Sync {
    fn name(&self) -> &'static str;
    fn priority(&self) -> i32;
    fn resolve(&self, profile: &str) -> Result<Option<Credentials>, Error>;
}

impl CredentialResolver {
    pub fn new(args: &AuthArgs) -> Self {
        let mut sources: Vec<Box<dyn CredentialSource>> = vec![];

        // Highest priority: explicit token
        if args.token.is_some() {
            sources.push(Box::new(ExplicitTokenSource {
                token: args.token.clone(),
            }));
        }

        // Environment variables
        sources.push(Box::new(EnvironmentSource));

        // Credentials file
        if let Some(ref path) = args.credentials_file {
            sources.push(Box::new(FileSource {
                path: path.clone(),
            }));
        }

        // OS keychain
        if args.use_keychain {
            sources.push(Box::new(KeychainSource));
        }

        // Default credentials file
        sources.push(Box::new(DefaultFileSource));

        // Sort by priority (higher = tried first)
        sources.sort_by_key(|s| std::cmp::Reverse(s.priority()));

        Self { sources }
    }

    pub fn resolve(&self, profile: &str) -> Result<Credentials, Error> {
        for source in &self.sources {
            match source.resolve(profile) {
                Ok(Some(creds)) => {
                    tracing::debug!("Credentials found in {}", source.name());
                    return Ok(creds);
                }
                Ok(None) => continue,
                Err(e) => {
                    tracing::warn!("Error reading {}: {}", source.name(), e);
                    continue;
                }
            }
        }

        Err(Error::NoCredentialsFound {
            tried: self.sources.iter().map(|s| s.name().to_string()).collect(),
        })
    }
}

// Implementation example for keychain
struct KeychainSource;

impl CredentialSource for KeychainSource {
    fn name(&self) -> &'static str { "OS Keychain" }
    fn priority(&self) -> i32 { 50 }

    fn resolve(&self, profile: &str) -> Result<Option<Credentials>, Error> {
        let service = format!("infra-cli-{}", profile);

        #[cfg(target_os = "macos")]
        {
            use security_framework::passwords::get_generic_password;
            match get_generic_password(&service, "token") {
                Ok(password) => {
                    let token = String::from_utf8(password)?;
                    Ok(Some(Credentials {
                        token: Secret::new(token),
                        source: "keychain".to_string(),
                    }))
                }
                Err(_) => Ok(None),
            }
        }

        #[cfg(not(target_os = "macos"))]
        Ok(None)
    }
}
```

### Secure Token Handling

Never log or display credentials accidentally.

```rust
use secrecy::{ExposeSecret, Secret};
use zeroize::Zeroize;

pub struct Credentials {
    token: Secret<String>,
    source: String,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Credentials {
    /// Get token for API calls - explicit exposure
    pub fn token(&self) -> &str {
        self.token.expose_secret()
    }

    /// Check if credentials are still valid
    pub fn is_valid(&self) -> bool {
        match self.expires_at {
            Some(expires) => chrono::Utc::now() < expires,
            None => true,
        }
    }

    /// Redacted display for debugging
    pub fn redacted_summary(&self) -> String {
        let token = self.token.expose_secret();
        let redacted = if token.len() > 8 {
            format!("{}...{}", &token[..4], &token[token.len()-4..])
        } else {
            "****".to_string()
        };
        format!("token={} source={}", redacted, self.source)
    }
}

// Debug implementation never shows token
impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credentials")
            .field("token", &"[REDACTED]")
            .field("source", &self.source)
            .field("expires_at", &self.expires_at)
            .finish()
    }
}
```

## Progress Reporting and Error Recovery

### Progress Bars for Long Operations

Infrastructure operations can take minutes. Good progress feedback is essential.

```rust
use indicatif::{MultiProgress, ProgressBar, ProgressStyle, ProgressDrawTarget};

pub struct DeploymentProgress {
    multi: MultiProgress,
    overall: ProgressBar,
    current_phase: ProgressBar,
}

impl DeploymentProgress {
    pub fn new(total_resources: u64, show: bool) -> Self {
        let multi = MultiProgress::new();

        if !show {
            multi.set_draw_target(ProgressDrawTarget::hidden());
        }

        let overall = multi.add(ProgressBar::new(total_resources));
        overall.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("=>-"));

        let current_phase = multi.add(ProgressBar::new_spinner());
        current_phase.set_style(ProgressStyle::default_spinner()
            .template("  {spinner:.yellow} {msg}")
            .unwrap());

        Self { multi, overall, current_phase }
    }

    pub fn set_phase(&self, phase: &str) {
        self.current_phase.set_message(phase.to_string());
        self.current_phase.enable_steady_tick(std::time::Duration::from_millis(100));
    }

    pub fn increment(&self, resource: &str) {
        self.current_phase.set_message(format!("Deploying {}", resource));
        self.overall.inc(1);
    }

    pub fn warn(&self, msg: &str) {
        self.multi.suspend(|| {
            eprintln!("{} {}", "WARNING:".yellow(), msg);
        });
    }

    pub fn finish_success(&self) {
        self.overall.finish_with_message("Deployment complete");
        self.current_phase.finish_and_clear();
    }

    pub fn finish_error(&self, error: &str) {
        self.overall.abandon_with_message(format!("Failed: {}", error));
        self.current_phase.finish_and_clear();
    }
}
```

### Error Recovery and Rollback

Production deployments need graceful error handling and rollback capabilities.

```rust
pub struct DeploymentContext {
    applied: Vec<AppliedResource>,
    rollback_enabled: bool,
}

impl DeploymentContext {
    /// Execute deployment with automatic rollback on failure
    pub async fn deploy_with_rollback(
        &mut self,
        resources: Vec<Resource>,
        progress: &DeploymentProgress,
    ) -> Result<DeploymentResult, Error> {
        progress.set_phase("Validating resources");
        self.validate_all(&resources)?;

        progress.set_phase("Deploying resources");
        for resource in resources {
            match self.deploy_single(&resource).await {
                Ok(applied) => {
                    self.applied.push(applied);
                    progress.increment(&resource.name);
                }
                Err(e) => {
                    progress.warn(&format!("Failed to deploy {}: {}", resource.name, e));

                    if self.rollback_enabled {
                        progress.set_phase("Rolling back changes");
                        self.rollback().await?;
                    }

                    return Err(Error::DeploymentFailed {
                        resource: resource.name,
                        cause: Box::new(e),
                        rolled_back: self.rollback_enabled,
                    });
                }
            }
        }

        progress.finish_success();
        Ok(DeploymentResult {
            applied_count: self.applied.len(),
            resources: self.applied.clone(),
        })
    }

    async fn rollback(&mut self) -> Result<(), Error> {
        // Rollback in reverse order
        for resource in self.applied.iter().rev() {
            match resource.rollback_action {
                RollbackAction::Delete => {
                    self.delete_resource(&resource.name).await?;
                }
                RollbackAction::Restore(ref previous) => {
                    self.apply_resource(previous).await?;
                }
                RollbackAction::None => {}
            }
        }
        self.applied.clear();
        Ok(())
    }
}
```

## Lessons Learned

### What Worked Well

1. **Environment as Global Enum**: Making environment a `ValueEnum` with validation caught many configuration errors at parse time. Production protection through confirmation prompts saved us from several potential incidents.

2. **Credential Source Abstraction**: The layered credential resolution allowed seamless migration from file-based to keychain credentials without changing user workflows.

3. **Dry-Run First**: Requiring `--plan` before `--apply` for production deployments became a team policy that prevented countless mistakes.

### What We Would Do Differently

1. **Earlier Output Format Abstraction**: We added JSON output late, requiring extensive refactoring. Design output formatting from day one.

2. **Better Cluster Context Management**: We underestimated how often users switch between clusters. A `context` subcommand for managing saved configurations (like `kubectl config`) would have been valuable.

3. **Structured Logging from Start**: We mixed `println!` and `eprintln!` with structured logging. Use `tracing` consistently from the beginning.

### Performance Benchmarks

Our infrastructure CLI performance after optimization:

| Operation | Cold Start | Warm Cache | kubectl Reference |
|-----------|------------|------------|-------------------|
| `--version` | 12ms | 4ms | 8ms |
| `get pods` (10 pods) | 180ms | 95ms | 120ms |
| `apply` (dry-run, 50 resources) | 850ms | 420ms | 680ms |
| `deploy` (20 resources) | 12.4s | 11.8s | N/A |
| `plan` generation | 2.1s | 1.4s | N/A |

**Key findings**:
- Credential resolution adds 15-40ms depending on source
- API response caching provides 40-50% speedup for repeated queries
- Parallel resource deployment (3 concurrent) is 2.5x faster than sequential

## Summary

DevOps CLIs require careful attention to multi-environment targeting, interactive/batch mode detection, secure credential handling, and robust error recovery. The patterns in this chapter have been refined through production use managing thousands of deployments.

### Key Takeaways

1. **Environment as first-class concept** with appropriate safeguards for production
2. **Auto-detect interactive mode** but allow explicit override for CI/CD
3. **Layer credential sources** with clear precedence and secure handling
4. **Implement dry-run/plan modes** before apply for all destructive operations
5. **Progress feedback** is essential for long-running infrastructure operations
6. **Design for rollback** from the beginning

### Architecture Decisions Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Environment handling | Global `ValueEnum` with confirmation | Type safety + production protection |
| Credential resolution | Layered sources with priority | Flexibility for different contexts |
| Output format | Auto-detect with explicit override | Works in terminals and pipelines |
| Destructive actions | Require explicit confirmation | Prevent accidental production changes |
| Progress reporting | `indicatif` with suspend for messages | Clean output without interleaving |

---

*Next: [Case Study: Interactive CLIs](./18-case-study-interactive-clis.md)*
