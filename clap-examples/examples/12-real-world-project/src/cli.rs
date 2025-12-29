//! CLI definition for cloudctl

use clap::{Parser, Subcommand, Args, ValueEnum};
use std::path::PathBuf;

/// Cloud infrastructure management CLI.
///
/// cloudctl provides a unified interface for managing cloud resources
/// across compute, storage, networking, and identity services.
#[derive(Parser, Debug)]
#[command(name = "cloudctl")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(arg_required_else_help = true)]
#[command(after_help = "EXAMPLES:\n    cloudctl compute instances list\n    cloudctl storage buckets create my-bucket\n    cloudctl --output json iam users list")]
pub struct Cli {
    // =========================================================================
    // GLOBAL OPTIONS
    // =========================================================================

    /// Configuration file path
    #[arg(long, global = true, env = "CLOUDCTL_CONFIG")]
    pub config: Option<PathBuf>,

    /// Profile to use
    #[arg(short = 'P', long, global = true, env = "CLOUDCTL_PROFILE")]
    pub profile: Option<String>,

    /// Output format
    #[arg(short, long, global = true, value_enum, default_value_t = OutputFormat::Table)]
    pub output: OutputFormat,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Enable debug output
    #[arg(long, global = true)]
    pub debug: bool,

    /// Dry run (show what would happen without making changes)
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Suppress all non-error output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(ValueEnum, Clone, Debug, Default)]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
    Yaml,
    Csv,
    Plain,
}

// =============================================================================
// TOP-LEVEL COMMANDS
// =============================================================================

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage compute resources (VMs, containers)
    #[command(subcommand)]
    Compute(ComputeCommands),

    /// Manage storage resources (buckets, volumes)
    #[command(subcommand)]
    Storage(StorageCommands),

    /// Manage networking resources (VPCs, subnets)
    #[command(subcommand)]
    Network(NetworkCommands),

    /// Manage identity and access (users, roles)
    #[command(subcommand)]
    Iam(IamCommands),

    /// Manage cloudctl configuration
    #[command(subcommand)]
    Config(ConfigCommands),

    /// Show version information
    Version,
}

// =============================================================================
// COMPUTE COMMANDS
// =============================================================================

#[derive(Subcommand, Debug)]
pub enum ComputeCommands {
    /// Manage VM instances
    #[command(subcommand)]
    Instances(InstanceCommands),

    /// Manage instance groups
    #[command(subcommand)]
    InstanceGroups(InstanceGroupCommands),

    /// List available machine types
    MachineTypes {
        /// Filter by zone
        #[arg(long)]
        zone: Option<String>,
    },

    /// List available images
    Images {
        /// Filter by project
        #[arg(long)]
        project: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum InstanceCommands {
    /// List instances
    List(InstanceListArgs),

    /// Create a new instance
    Create(InstanceCreateArgs),

    /// Delete an instance
    Delete {
        /// Instance name
        name: String,

        /// Skip confirmation
        #[arg(short, long)]
        force: bool,

        /// Delete associated resources
        #[arg(long)]
        delete_disks: bool,
    },

    /// Start an instance
    Start {
        /// Instance name(s)
        #[arg(required = true)]
        names: Vec<String>,
    },

    /// Stop an instance
    Stop {
        /// Instance name(s)
        #[arg(required = true)]
        names: Vec<String>,

        /// Force stop
        #[arg(short, long)]
        force: bool,
    },

    /// SSH into an instance
    Ssh {
        /// Instance name
        name: String,

        /// SSH command to execute
        #[arg(trailing_var_arg = true)]
        command: Vec<String>,
    },

    /// Describe an instance
    Describe {
        /// Instance name
        name: String,
    },
}

#[derive(Args, Debug)]
pub struct InstanceListArgs {
    /// Filter by zone
    #[arg(long)]
    pub zone: Option<String>,

    /// Filter by status
    #[arg(long)]
    pub status: Option<String>,

    /// Filter by label (key=value)
    #[arg(long)]
    pub label: Vec<String>,

    /// Maximum results
    #[arg(long, default_value_t = 100)]
    pub limit: u32,

    /// Page token for pagination
    #[arg(long)]
    pub page_token: Option<String>,
}

#[derive(Args, Debug)]
pub struct InstanceCreateArgs {
    /// Instance name
    pub name: String,

    /// Machine type
    #[arg(short = 't', long, default_value = "standard")]
    pub machine_type: String,

    /// Zone to create in
    #[arg(short, long)]
    pub zone: Option<String>,

    /// Boot disk image
    #[arg(long, default_value = "debian-11")]
    pub image: String,

    /// Boot disk size in GB
    #[arg(long, default_value_t = 20)]
    pub disk_size: u32,

    /// Network to attach to
    #[arg(long)]
    pub network: Option<String>,

    /// Subnet to attach to
    #[arg(long)]
    pub subnet: Option<String>,

    /// Labels (key=value pairs)
    #[arg(long)]
    pub labels: Vec<String>,

    /// Tags for firewall rules
    #[arg(long)]
    pub tags: Vec<String>,

    /// Startup script
    #[arg(long)]
    pub startup_script: Option<String>,

    /// Preemptible instance
    #[arg(long)]
    pub preemptible: bool,
}

#[derive(Subcommand, Debug)]
pub enum InstanceGroupCommands {
    /// List instance groups
    List {
        /// Filter by zone
        #[arg(long)]
        zone: Option<String>,
    },

    /// Create an instance group
    Create {
        /// Group name
        name: String,

        /// Size of the group
        #[arg(long)]
        size: u32,

        /// Template to use
        #[arg(long)]
        template: String,
    },
}

// =============================================================================
// STORAGE COMMANDS
// =============================================================================

#[derive(Subcommand, Debug)]
pub enum StorageCommands {
    /// Manage storage buckets
    #[command(subcommand)]
    Buckets(BucketCommands),

    /// Manage bucket objects
    #[command(subcommand)]
    Objects(ObjectCommands),
}

#[derive(Subcommand, Debug)]
pub enum BucketCommands {
    /// List buckets
    List {
        /// Project to list buckets from
        #[arg(long)]
        project: Option<String>,
    },

    /// Create a bucket
    Create {
        /// Bucket name
        name: String,

        /// Region
        #[arg(short, long)]
        region: Option<String>,

        /// Storage class
        #[arg(long, default_value = "standard")]
        storage_class: String,

        /// Enable versioning
        #[arg(long)]
        versioning: bool,
    },

    /// Delete a bucket
    Delete {
        /// Bucket name
        name: String,

        /// Force delete (delete all objects first)
        #[arg(short, long)]
        force: bool,
    },

    /// Describe a bucket
    Describe {
        /// Bucket name
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ObjectCommands {
    /// List objects in a bucket
    List {
        /// Bucket name
        bucket: String,

        /// Prefix filter
        #[arg(long)]
        prefix: Option<String>,

        /// Delimiter for hierarchy
        #[arg(long)]
        delimiter: Option<String>,
    },

    /// Upload an object
    Upload {
        /// Local file path
        source: PathBuf,

        /// Destination (bucket/path)
        destination: String,

        /// Content type
        #[arg(long)]
        content_type: Option<String>,
    },

    /// Download an object
    Download {
        /// Source (bucket/path)
        source: String,

        /// Local destination
        destination: PathBuf,
    },

    /// Delete an object
    Delete {
        /// Object path (bucket/path)
        path: String,

        /// Recursive delete
        #[arg(short, long)]
        recursive: bool,
    },
}

// =============================================================================
// NETWORK COMMANDS
// =============================================================================

#[derive(Subcommand, Debug)]
pub enum NetworkCommands {
    /// Manage VPCs
    #[command(subcommand)]
    Vpcs(VpcCommands),

    /// Manage subnets
    #[command(subcommand)]
    Subnets(SubnetCommands),

    /// Manage security groups
    #[command(subcommand)]
    SecurityGroups(SecurityGroupCommands),
}

#[derive(Subcommand, Debug)]
pub enum VpcCommands {
    /// List VPCs
    List,

    /// Create a VPC
    Create {
        /// VPC name
        name: String,

        /// CIDR block
        #[arg(long, default_value = "10.0.0.0/16")]
        cidr: String,
    },

    /// Delete a VPC
    Delete {
        /// VPC name
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum SubnetCommands {
    /// List subnets
    List {
        /// Filter by VPC
        #[arg(long)]
        vpc: Option<String>,
    },

    /// Create a subnet
    Create {
        /// Subnet name
        name: String,

        /// VPC to create in
        #[arg(long)]
        vpc: String,

        /// CIDR block
        #[arg(long)]
        cidr: String,

        /// Availability zone
        #[arg(long)]
        zone: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum SecurityGroupCommands {
    /// List security groups
    List {
        /// Filter by VPC
        #[arg(long)]
        vpc: Option<String>,
    },

    /// Create a security group
    Create {
        /// Security group name
        name: String,

        /// VPC
        #[arg(long)]
        vpc: String,

        /// Description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Add a rule to a security group
    AddRule {
        /// Security group name
        name: String,

        /// Protocol (tcp, udp, icmp, all)
        #[arg(long)]
        protocol: String,

        /// Port or port range (e.g., 80, 8000-9000)
        #[arg(long)]
        port: Option<String>,

        /// Source CIDR
        #[arg(long)]
        source: String,

        /// Rule direction (inbound, outbound)
        #[arg(long, default_value = "inbound")]
        direction: String,
    },
}

// =============================================================================
// IAM COMMANDS
// =============================================================================

#[derive(Subcommand, Debug)]
pub enum IamCommands {
    /// Manage users
    #[command(subcommand)]
    Users(UserCommands),

    /// Manage roles
    #[command(subcommand)]
    Roles(RoleCommands),

    /// Manage policies
    #[command(subcommand)]
    Policies(PolicyCommands),
}

#[derive(Subcommand, Debug)]
pub enum UserCommands {
    /// List users
    List {
        /// Path prefix filter
        #[arg(long)]
        path_prefix: Option<String>,
    },

    /// Create a user
    Create {
        /// Username
        name: String,

        /// User path
        #[arg(long, default_value = "/")]
        path: String,

        /// Tags
        #[arg(long)]
        tags: Vec<String>,
    },

    /// Delete a user
    Delete {
        /// Username
        name: String,
    },

    /// Get user details
    Get {
        /// Username
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum RoleCommands {
    /// List roles
    List {
        /// Path prefix filter
        #[arg(long)]
        path_prefix: Option<String>,
    },

    /// Get role details
    Get {
        /// Role name
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum PolicyCommands {
    /// List policies
    List {
        /// Scope (local, aws)
        #[arg(long, default_value = "all")]
        scope: String,
    },

    /// Attach a policy
    Attach {
        /// Policy ARN
        policy: String,

        /// User to attach to
        #[arg(long, conflicts_with = "role")]
        user: Option<String>,

        /// Role to attach to
        #[arg(long, conflicts_with = "user")]
        role: Option<String>,
    },
}

// =============================================================================
// CONFIG COMMANDS
// =============================================================================

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Get a configuration value
    Get {
        /// Configuration key
        key: String,
    },

    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,

        /// Configuration value
        value: String,
    },

    /// Unset a configuration value
    Unset {
        /// Configuration key
        key: String,
    },

    /// List all configuration
    List,

    /// Show current configuration file path
    Path,
}
