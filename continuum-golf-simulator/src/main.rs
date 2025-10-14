// CLI entry point for Continuum Golf Simulator

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "continuum-golf-simulator")]
#[command(about = "Continuum Golf Wagering Simulator", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run player session simulation
    Player {
        /// Starting handicap (0-30)
        #[arg(short, long)]
        handicap: u8,

        /// Number of shots to simulate
        #[arg(short, long)]
        shots: usize,

        /// Minimum wager
        #[arg(long, default_value = "5.0")]
        wager_min: f64,

        /// Maximum wager
        #[arg(long, default_value = "10.0")]
        wager_max: f64,
    },

    /// Run venue economics simulation
    Venue {
        /// Number of hitting bays
        #[arg(short, long)]
        bays: usize,

        /// Operating hours
        #[arg(long)]
        hours: f64,

        /// Average shots per bay per hour
        #[arg(long, default_value = "100")]
        shots_per_hour: usize,
    },

    /// Run tournament simulation
    Tournament {
        /// Number of players
        #[arg(short, long)]
        players: usize,

        /// Entry fee per player
        #[arg(short, long)]
        entry_fee: f64,
    },

    /// Run validation tests
    Validate {
        /// Test to run (all|rtp|fairness|convergence)
        #[arg(short, long, default_value = "all")]
        test: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Player { handicap, shots, wager_min, wager_max } => {
            println!("Running player simulation:");
            println!("  Handicap: {}", handicap);
            println!("  Shots: {}", shots);
            println!("  Wager range: ${}-${}", wager_min, wager_max);
            println!("\n[Implementation pending - Phase 3]");
        }
        Commands::Venue { bays, hours, shots_per_hour } => {
            println!("Running venue simulation:");
            println!("  Bays: {}", bays);
            println!("  Hours: {}", hours);
            println!("  Shots/hour: {}", shots_per_hour);
            println!("\n[Implementation pending - Phase 3]");
        }
        Commands::Tournament { players, entry_fee } => {
            println!("Running tournament simulation:");
            println!("  Players: {}", players);
            println!("  Entry fee: ${}", entry_fee);
            println!("\n[Implementation pending - Phase 3]");
        }
        Commands::Validate { test } => {
            println!("Running validation tests: {}", test);
            println!("\n[Implementation pending - Phase 4]");
        }
    }
}
