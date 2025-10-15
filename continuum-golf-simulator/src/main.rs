// CLI entry point for Continuum Golf Simulator

use clap::{Parser, Subcommand};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use prettytable::{Table, Row, Cell, format};

use continuum_golf_simulator::{
    models::{hole::HOLE_CONFIGURATIONS, player::*},
    simulators::{player_session::*, venue::*, tournament::*},
    analytics::{metrics::*, export::*},
};

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
        #[arg(long)]
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

        /// Fixed hole ID (1-8) or random selection
        #[arg(long)]
        hole: Option<u8>,

        /// Enable developer mode (manual miss input)
        #[arg(long, default_value = "false")]
        developer_mode: bool,

        /// Export results to CSV file
        #[arg(long)]
        export: Option<String>,
    },

    /// Run venue economics simulation
    Venue {
        /// Number of hitting bays
        #[arg(long)]
        bays: usize,

        /// Operating hours
        #[arg(long)]
        hours: f64,

        /// Average shots per bay per hour
        #[arg(long, default_value = "100")]
        shots_per_hour: usize,

        /// Player archetype (uniform|bell|beginners|experts)
        #[arg(long, default_value = "uniform")]
        archetype: String,

        /// Wager range minimum
        #[arg(long, default_value = "5.0")]
        wager_min: f64,

        /// Wager range maximum
        #[arg(long, default_value = "10.0")]
        wager_max: f64,

        /// Export venue results to JSON
        #[arg(long)]
        export_json: Option<String>,

        /// Export heatmap to CSV
        #[arg(long)]
        export_heatmap: Option<String>,

        /// Show progress bar
        #[arg(long, default_value = "true")]
        progress: bool,
    },

    /// Run tournament simulation
    Tournament {
        /// Game mode (longest|ctp)
        #[arg(long, default_value = "ctp")]
        mode: String,

        /// Hole ID for CTP mode (1-8)
        #[arg(long, default_value = "4")]
        hole: u8,

        /// Number of players
        #[arg(long)]
        players: usize,

        /// Entry fee per player
        #[arg(long)]
        entry_fee: f64,

        /// House rake percentage (0-100)
        #[arg(long, default_value = "10.0")]
        rake: f64,

        /// Payout structure (winner|top2|top3)
        #[arg(long, default_value = "top3")]
        payout: String,

        /// Number of attempts per player
        #[arg(long, default_value = "3")]
        attempts: usize,
    },

    /// Run validation tests
    Validate {
        /// Test to run (all|rtp|fairness|convergence)
        #[arg(long, default_value = "all")]
        test: String,

        /// Show verbose output
        #[arg(short, long, default_value = "false")]
        verbose: bool,
    },
}

fn main() {
    print_logo();

    let cli = Cli::parse();

    match cli.command {
        Commands::Player {
            handicap,
            shots,
            wager_min,
            wager_max,
            hole,
            developer_mode,
            export,
        } => {
            run_player_command(handicap, shots, wager_min, wager_max, hole, developer_mode, export);
        }
        Commands::Venue {
            bays,
            hours,
            shots_per_hour,
            archetype,
            wager_min,
            wager_max,
            export_json,
            export_heatmap,
            progress,
        } => {
            run_venue_command(
                bays,
                hours,
                shots_per_hour,
                &archetype,
                wager_min,
                wager_max,
                export_json,
                export_heatmap,
                progress,
            );
        }
        Commands::Tournament {
            mode,
            hole,
            players,
            entry_fee,
            rake,
            payout,
            attempts,
        } => {
            run_tournament_command(&mode, hole, players, entry_fee, rake, &payout, attempts);
        }
        Commands::Validate { test, verbose } => {
            run_validate_command(&test, verbose);
        }
    }
}

fn print_logo() {
    println!("{}", "");
    println!("{}", "╔═══════════════════════════════════════════════════════════════╗".bright_cyan());
    println!("{}", "║                                                               ║".bright_cyan());
    println!("{}", "║      ██████╗ ██████╗ ███╗   ██╗████████╗██╗███╗   ██╗       ║".bright_cyan());
    println!("{}", "║     ██╔════╝██╔═══██╗████╗  ██║╚══██╔══╝██║████╗  ██║       ║".bright_cyan());
    println!("{}", "║     ██║     ██║   ██║██╔██╗ ██║   ██║   ██║██╔██╗ ██║       ║".bright_cyan());
    println!("{}", "║     ██║     ██║   ██║██║╚██╗██║   ██║   ██║██║╚██╗██║       ║".bright_cyan());
    println!("{}", "║     ╚██████╗╚██████╔╝██║ ╚████║   ██║   ██║██║ ╚████║       ║".bright_cyan());
    println!("{}", "║      ╚═════╝ ╚═════╝ ╚═╝  ╚═══╝   ╚═╝   ╚═╝╚═╝  ╚═══╝       ║".bright_cyan());
    println!("{}", "║                                                               ║".bright_cyan());
    println!("{}", "║              Golf Wagering Simulator v0.1.0                   ║".bright_white());
    println!("{}", "║         Fair • Dynamic • Profitable • Rust-Powered           ║".bright_green());
    println!("{}", "║                                                               ║".bright_cyan());
    println!("{}", "╚═══════════════════════════════════════════════════════════════╝".bright_cyan());
    println!();
}

fn run_player_command(
    handicap: u8,
    shots: usize,
    wager_min: f64,
    wager_max: f64,
    hole_id: Option<u8>,
    _developer_mode: bool,
    export_path: Option<String>,
) {
    println!("{}", "═══════════════════════════════════════".bright_yellow());
    println!("{}", "       PLAYER SESSION SIMULATOR".bright_yellow().bold());
    println!("{}", "═══════════════════════════════════════".bright_yellow());
    println!();

    // Validate inputs
    if handicap > 30 {
        eprintln!("{}", "Error: Handicap must be between 0 and 30".red().bold());
        return;
    }

    if wager_min <= 0.0 || wager_max < wager_min {
        eprintln!("{}", "Error: Invalid wager range".red().bold());
        return;
    }

    // Display configuration
    let mut config_table = Table::new();
    config_table.set_format(*format::consts::FORMAT_BOX_CHARS);
    config_table.add_row(Row::new(vec![
        Cell::new("Configuration").style_spec("Fb"),
        Cell::new("Value").style_spec("Fb"),
    ]));
    config_table.add_row(Row::new(vec![
        Cell::new("Handicap"),
        Cell::new(&format!("{}", handicap)),
    ]));
    config_table.add_row(Row::new(vec![
        Cell::new("Number of Shots"),
        Cell::new(&format!("{}", shots)),
    ]));
    config_table.add_row(Row::new(vec![
        Cell::new("Wager Range"),
        Cell::new(&format!("${:.2} - ${:.2}", wager_min, wager_max)),
    ]));
    let hole_str = if let Some(h) = hole_id {
        format!("Fixed (H{})", h)
    } else {
        "Random".to_string()
    };
    config_table.add_row(Row::new(vec![
        Cell::new("Hole Selection"),
        Cell::new(&hole_str),
    ]));
    config_table.printstd();
    println!();

    // Create player
    let player_id = format!("player_{}", handicap);
    let mut player = Player::new(player_id, handicap);

    // Configure session
    let hole_selection = if let Some(h) = hole_id {
        HoleSelection::Fixed(h)
    } else {
        HoleSelection::Random
    };

    let config = SessionConfig {
        num_shots: shots,
        wager_min,
        wager_max,
        hole_selection,
        developer_mode: None,
        fat_tail_prob: 0.02,
        fat_tail_mult: 3.0,
    };

    // Run simulation with progress bar
    println!("{}", "Running simulation...".bright_blue());
    let pb = ProgressBar::new(shots as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} shots ({percent}%)")
            .unwrap()
            .progress_chars("=>-"),
    );

    // Run the session
    let result = run_session(&mut player, config);
    pb.finish_with_message("Complete!");
    println!();

    // Display results
    print_session_results(&result);

    // Export if requested
    if let Some(path) = export_path {
        match export_session_csv(&result, &path) {
            Ok(_) => println!("{} {}", "✓".green(), format!("Results exported to: {}", path).bright_white()),
            Err(e) => eprintln!("{} {}", "✗".red(), format!("Failed to export: {}", e).red()),
        }
        println!();
    }
}

fn run_venue_command(
    bays: usize,
    hours: f64,
    shots_per_hour: usize,
    archetype: &str,
    wager_min: f64,
    wager_max: f64,
    export_json: Option<String>,
    export_heatmap: Option<String>,
    show_progress: bool,
) {
    println!("{}", "═══════════════════════════════════════".bright_yellow());
    println!("{}", "      VENUE ECONOMICS SIMULATOR".bright_yellow().bold());
    println!("{}", "═══════════════════════════════════════".bright_yellow());
    println!();

    // Parse archetype
    let player_archetype = match archetype {
        "uniform" => PlayerArchetype::Uniform,
        "bell" => PlayerArchetype::BellCurve { mean: 15, std_dev: 5.0 },
        "beginners" => PlayerArchetype::SkewedHigh,
        "experts" => PlayerArchetype::SkewedLow,
        _ => {
            eprintln!("{}", "Error: Invalid archetype. Use: uniform|bell|beginners|experts".red().bold());
            return;
        }
    };

    // Display configuration
    let mut config_table = Table::new();
    config_table.set_format(*format::consts::FORMAT_BOX_CHARS);
    config_table.add_row(Row::new(vec![
        Cell::new("Configuration").style_spec("Fb"),
        Cell::new("Value").style_spec("Fb"),
    ]));
    config_table.add_row(Row::new(vec![
        Cell::new("Number of Bays"),
        Cell::new(&format!("{}", bays)),
    ]));
    config_table.add_row(Row::new(vec![
        Cell::new("Operating Hours"),
        Cell::new(&format!("{:.1}", hours)),
    ]));
    config_table.add_row(Row::new(vec![
        Cell::new("Shots per Hour"),
        Cell::new(&format!("{}", shots_per_hour)),
    ]));
    config_table.add_row(Row::new(vec![
        Cell::new("Player Archetype"),
        Cell::new(archetype),
    ]));
    config_table.add_row(Row::new(vec![
        Cell::new("Wager Range"),
        Cell::new(&format!("${:.2} - ${:.2}", wager_min, wager_max)),
    ]));
    config_table.add_row(Row::new(vec![
        Cell::new("Total Shots"),
        Cell::new(&format!("{}", (bays as f64 * hours * shots_per_hour as f64) as usize)),
    ]));
    config_table.printstd();
    println!();

    // Configure venue
    let config = VenueConfig {
        num_bays: bays,
        hours,
        shots_per_hour,
        player_archetype,
        wager_range: (wager_min, wager_max),
    };

    // Run simulation
    if show_progress {
        println!("{}", "Running venue simulation...".bright_blue());
        let total_shots = (bays as f64 * hours * shots_per_hour as f64) as u64;
        let pb = ProgressBar::new(total_shots);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} shots ({percent}%)")
                .unwrap()
                .progress_chars("=>-"),
        );

        let result = run_venue_simulation(config);
        pb.finish_with_message("Complete!");
        println!();

        print_venue_results(&result);

        // Export if requested
        if let Some(path) = export_json {
            match export_venue_json(&result, &path) {
                Ok(_) => println!("{} {}", "✓".green(), format!("Venue results exported to: {}", path).bright_white()),
                Err(e) => eprintln!("{} {}", "✗".red(), format!("Failed to export JSON: {}", e).red()),
            }
        }

        if let Some(path) = export_heatmap {
            match export_heatmap_csv(&result.heatmap_data, &path) {
                Ok(_) => println!("{} {}", "✓".green(), format!("Heatmap exported to: {}", path).bright_white()),
                Err(e) => eprintln!("{} {}", "✗".red(), format!("Failed to export heatmap: {}", e).red()),
            }
        }
    } else {
        let result = run_venue_simulation(config);
        print_venue_results(&result);
    }
}

fn run_tournament_command(
    mode: &str,
    hole: u8,
    players: usize,
    entry_fee: f64,
    rake: f64,
    payout: &str,
    attempts: usize,
) {
    println!("{}", "═══════════════════════════════════════".bright_yellow());
    println!("{}", "       TOURNAMENT SIMULATOR".bright_yellow().bold());
    println!("{}", "═══════════════════════════════════════".bright_yellow());
    println!();

    // Validate hole
    if hole < 1 || hole > 8 {
        eprintln!("{}", "Error: Hole must be between 1 and 8".red().bold());
        return;
    }

    // Parse game mode
    let game_mode = match mode {
        "longest" => GameMode::LongestDrive,
        "ctp" => GameMode::ClosestToPin { hole_id: hole },
        _ => {
            eprintln!("{}", "Error: Invalid mode. Use: longest|ctp".red().bold());
            return;
        }
    };

    // Parse payout structure
    let payout_structure = match payout {
        "winner" => PayoutStructure::WinnerTakesAll,
        "top2" => PayoutStructure::Top2 { first: 0.70, second: 0.30 },
        "top3" => PayoutStructure::Top3 { first: 0.50, second: 0.30, third: 0.20 },
        _ => {
            eprintln!("{}", "Error: Invalid payout. Use: winner|top2|top3".red().bold());
            return;
        }
    };

    // Display configuration
    let mut config_table = Table::new();
    config_table.set_format(*format::consts::FORMAT_BOX_CHARS);
    config_table.add_row(Row::new(vec![
        Cell::new("Configuration").style_spec("Fb"),
        Cell::new("Value").style_spec("Fb"),
    ]));
    config_table.add_row(Row::new(vec![
        Cell::new("Game Mode"),
        Cell::new(mode),
    ]));
    if mode == "ctp" {
        config_table.add_row(Row::new(vec![
            Cell::new("Hole"),
            Cell::new(&format!("H{}", hole)),
        ]));
    }
    config_table.add_row(Row::new(vec![
        Cell::new("Number of Players"),
        Cell::new(&format!("{}", players)),
    ]));
    config_table.add_row(Row::new(vec![
        Cell::new("Entry Fee"),
        Cell::new(&format!("${:.2}", entry_fee)),
    ]));
    config_table.add_row(Row::new(vec![
        Cell::new("House Rake"),
        Cell::new(&format!("{:.1}%", rake)),
    ]));
    config_table.add_row(Row::new(vec![
        Cell::new("Payout Structure"),
        Cell::new(payout),
    ]));
    config_table.add_row(Row::new(vec![
        Cell::new("Attempts per Player"),
        Cell::new(&format!("{}", attempts)),
    ]));
    config_table.printstd();
    println!();

    // Configure tournament
    let config = TournamentConfig {
        game_mode,
        num_players: players,
        entry_fee,
        house_rake_percent: rake,
        payout_structure,
        attempts_per_player: attempts,
    };

    // Run simulation
    println!("{}", "Running tournament simulation...".bright_blue());
    let result = run_tournament(config);
    println!();

    print_tournament_results(&result);
}

fn run_validate_command(test: &str, verbose: bool) {
    println!("{}", "═══════════════════════════════════════".bright_yellow());
    println!("{}", "        VALIDATION TEST SUITE".bright_yellow().bold());
    println!("{}", "═══════════════════════════════════════".bright_yellow());
    println!();

    match test {
        "all" => {
            run_rtp_validation(verbose);
            println!();
            run_fairness_validation(verbose);
            println!();
            run_convergence_validation(verbose);
        }
        "rtp" => run_rtp_validation(verbose),
        "fairness" => run_fairness_validation(verbose),
        "convergence" => run_convergence_validation(verbose),
        _ => {
            eprintln!("{}", "Error: Invalid test. Use: all|rtp|fairness|convergence".red().bold());
        }
    }
}

fn run_rtp_validation(verbose: bool) {
    println!("{}", "RTP Validation Test".bright_cyan().bold());
    println!("{}", "───────────────────────────────────────".bright_cyan());

    let holes = &HOLE_CONFIGURATIONS;
    let mut all_passed = true;

    for hole in holes.iter() {
        let handicap_range: Vec<u8> = (0..=30).step_by(5).collect();
        let results = validate_rtp_across_skills(hole, handicap_range, 1000);

        let avg_rtp: f64 = results.iter().map(|r| r.actual_rtp).sum::<f64>() / results.len() as f64;
        let rtp_diff = (avg_rtp - hole.rtp).abs();
        let passed = rtp_diff < 0.02; // Within 2%

        all_passed = all_passed && passed;

        let status = if passed {
            "✓ PASS".green()
        } else {
            "✗ FAIL".red()
        };

        println!(
            "{} H{} ({}yds): Target={:.1}%, Actual={:.1}%, Diff={:.2}%",
            status,
            hole.id,
            hole.distance_yds,
            hole.rtp * 100.0,
            avg_rtp * 100.0,
            rtp_diff * 100.0
        );

        if verbose {
            for result in results.iter() {
                println!("    Handicap {}: RTP={:.2}%", result.handicap, result.actual_rtp * 100.0);
            }
        }
    }

    println!();
    if all_passed {
        println!("{}", "All RTP tests passed!".green().bold());
    } else {
        println!("{}", "Some RTP tests failed.".red().bold());
    }
}

fn run_fairness_validation(verbose: bool) {
    println!("{}", "Fairness Validation Test".bright_cyan().bold());
    println!("{}", "───────────────────────────────────────".bright_cyan());

    let holes = &HOLE_CONFIGURATIONS;
    let mut all_passed = true;

    for hole in holes.iter() {
        let handicaps: Vec<u8> = vec![0, 5, 10, 15, 20, 25, 30];
        let report = calculate_fairness_metric(hole, handicaps, 1000);

        let passed = report.max_ev_difference < 0.01; // Within 1%
        all_passed = all_passed && passed;

        let status = if passed {
            "✓ PASS".green()
        } else {
            "✗ FAIL".red()
        };

        println!(
            "{} H{} ({}yds): Max EV Diff={:.3}%",
            status,
            hole.id,
            hole.distance_yds,
            report.max_ev_difference * 100.0
        );

        if verbose {
            for comp in &report.comparisons {
                println!("    Handicap {}: EV={:.4}, P_max={:.2}", comp.handicap, comp.expected_value, comp.p_max);
            }
        }
    }

    println!();
    if all_passed {
        println!("{}", "All fairness tests passed!".green().bold());
    } else {
        println!("{}", "Some fairness tests failed.".red().bold());
    }
}

fn run_convergence_validation(verbose: bool) {
    println!("{}", "Kalman Convergence Test".bright_cyan().bold());
    println!("{}", "───────────────────────────────────────".bright_cyan());

    // Simulate a session and check convergence
    let player_id = "test_player".to_string();
    let mut player = Player::new(player_id, 15);
    let config = SessionConfig {
        num_shots: 100,
        wager_min: 5.0,
        wager_max: 10.0,
        hole_selection: HoleSelection::Fixed(4),
        developer_mode: None,
        fat_tail_prob: 0.02,
        fat_tail_mult: 3.0,
    };

    let result = run_session(&mut player, config);
    let reports = analyze_kalman_convergence(&result);

    // Get the first report (if any)
    let mut overall_passed = true;
    for (category, report) in reports.iter() {
        let passed = report.final_confidence > 70.0;
        overall_passed = overall_passed && passed;

        let status = if passed {
            "✓ PASS".green()
        } else {
            "✗ FAIL".red()
        };

        println!(
            "{} {} Final Confidence: {:.1}% (target: >70%)",
            status,
            category,
            report.final_confidence
        );

        if verbose {
            println!("    Initial Confidence: {:.1}%", report.initial_confidence);
            println!("    Shots to 80% Confidence: {:?}", report.shots_to_80_percent);
            println!("    Converged: {}", report.converged);
        }
    }

    println!();
    if overall_passed {
        println!("{}", "Convergence test passed!".green().bold());
    } else {
        println!("{}", "Convergence test failed.".red().bold());
    }
}

fn print_session_results(result: &SessionResult) {
    println!("{}", "═══════════════════════════════════════".bright_green());
    println!("{}", "          SESSION RESULTS".bright_green().bold());
    println!("{}", "═══════════════════════════════════════".bright_green());
    println!();

    // Financial summary
    let mut summary_table = Table::new();
    summary_table.set_format(*format::consts::FORMAT_BOX_CHARS);
    summary_table.add_row(Row::new(vec![
        Cell::new("Metric").style_spec("Fb"),
        Cell::new("Value").style_spec("Fb"),
    ]));
    summary_table.add_row(Row::new(vec![
        Cell::new("Total Wagered"),
        Cell::new(&format!("${:.2}", result.total_wagered)),
    ]));
    summary_table.add_row(Row::new(vec![
        Cell::new("Total Won"),
        Cell::new(&format!("${:.2}", result.total_won)),
    ]));

    let net_cell = if result.net_gain_loss >= 0.0 {
        Cell::new(&format!("+${:.2}", result.net_gain_loss)).style_spec("Fg")
    } else {
        Cell::new(&format!("-${:.2}", -result.net_gain_loss)).style_spec("Fr")
    };
    summary_table.add_row(Row::new(vec![Cell::new("Net Gain/Loss"), net_cell]));

    summary_table.add_row(Row::new(vec![
        Cell::new("Session House Edge"),
        Cell::new(&format!("{:.2}%", result.session_house_edge * 100.0)),
    ]));
    summary_table.printstd();
    println!();

    // Skill profiles (now just sigma values)
    println!("{}", "Final Skill Profiles:".bright_white().bold());
    let mut skill_table = Table::new();
    skill_table.set_format(*format::consts::FORMAT_BOX_CHARS);
    skill_table.add_row(Row::new(vec![
        Cell::new("Category").style_spec("Fb"),
        Cell::new("Dispersion (σ)").style_spec("Fb"),
    ]));

    for (category, sigma) in result.final_skill_profiles.iter() {
        skill_table.add_row(Row::new(vec![
            Cell::new(category),
            Cell::new(&format!("{:.1} ft", sigma)),
        ]));
    }
    skill_table.printstd();
    println!();
}

fn print_venue_results(result: &VenueResult) {
    println!("{}", "═══════════════════════════════════════".bright_green());
    println!("{}", "          VENUE RESULTS".bright_green().bold());
    println!("{}", "═══════════════════════════════════════".bright_green());
    println!();

    // Financial summary
    let mut summary_table = Table::new();
    summary_table.set_format(*format::consts::FORMAT_BOX_CHARS);
    summary_table.add_row(Row::new(vec![
        Cell::new("Metric").style_spec("Fb"),
        Cell::new("Value").style_spec("Fb"),
    ]));
    summary_table.add_row(Row::new(vec![
        Cell::new("Total Handle"),
        Cell::new(&format!("${:.2}", result.total_wagered)),
    ]));
    summary_table.add_row(Row::new(vec![
        Cell::new("Total Payouts"),
        Cell::new(&format!("${:.2}", result.total_payouts)),
    ]));
    summary_table.add_row(Row::new(vec![
        Cell::new("Net Profit"),
        Cell::new(&format!("${:.2}", result.net_profit)).style_spec("Fg"),
    ]));
    summary_table.add_row(Row::new(vec![
        Cell::new("Hold Percentage"),
        Cell::new(&format!("{:.2}%", result.hold_percentage * 100.0)),
    ]));

    // Calculate ARPU (Average Revenue Per User) - assuming each bay is one user session
    if !result.profit_over_time.is_empty() {
        let num_sessions = result.profit_over_time.len();
        let arpu = result.net_profit / num_sessions as f64;
        summary_table.add_row(Row::new(vec![
            Cell::new("Profit per Session"),
            Cell::new(&format!("${:.2}", arpu)),
        ]));
    }

    summary_table.printstd();
    println!();

    // Payout distribution
    println!("{}", "Payout Distribution:".bright_white().bold());
    let mut payout_table = Table::new();
    payout_table.set_format(*format::consts::FORMAT_BOX_CHARS);
    payout_table.add_row(Row::new(vec![
        Cell::new("Multiplier Range").style_spec("Fb"),
        Cell::new("Count").style_spec("Fb"),
    ]));

    for (i, count) in result.payout_distribution.iter().enumerate() {
        let range = if i < 10 {
            format!("{}x - {}x", i, i + 1)
        } else {
            "10x+".to_string()
        };
        payout_table.add_row(Row::new(vec![
            Cell::new(&range),
            Cell::new(&format!("{}", count)),
        ]));
    }
    payout_table.printstd();
    println!();
}

fn print_tournament_results(result: &TournamentResult) {
    println!("{}", "═══════════════════════════════════════".bright_green());
    println!("{}", "       TOURNAMENT RESULTS".bright_green().bold());
    println!("{}", "═══════════════════════════════════════".bright_green());
    println!();

    // Financial summary
    let mut summary_table = Table::new();
    summary_table.set_format(*format::consts::FORMAT_BOX_CHARS);
    summary_table.add_row(Row::new(vec![
        Cell::new("Financial Summary").style_spec("Fb"),
        Cell::new("Amount").style_spec("Fb"),
    ]));
    summary_table.add_row(Row::new(vec![
        Cell::new("Total Pool"),
        Cell::new(&format!("${:.2}", result.total_pool)),
    ]));
    summary_table.add_row(Row::new(vec![
        Cell::new("House Rake"),
        Cell::new(&format!("${:.2}", result.house_rake)),
    ]));
    summary_table.add_row(Row::new(vec![
        Cell::new("Prize Pool"),
        Cell::new(&format!("${:.2}", result.prize_pool)),
    ]));
    summary_table.printstd();
    println!();

    // Leaderboard (top 10)
    println!("{}", "Leaderboard (Top 10):".bright_white().bold());
    let mut leaderboard_table = Table::new();
    leaderboard_table.set_format(*format::consts::FORMAT_BOX_CHARS);
    leaderboard_table.add_row(Row::new(vec![
        Cell::new("Rank").style_spec("Fb"),
        Cell::new("Player").style_spec("Fb"),
        Cell::new("Score").style_spec("Fb"),
        Cell::new("Prize").style_spec("Fb"),
    ]));

    for (i, (player_id, score)) in result.leaderboard.iter().take(10).enumerate() {
        let prize = result
            .payouts
            .iter()
            .find(|(p, _)| p == player_id)
            .map(|(_, amount)| format!("${:.2}", amount))
            .unwrap_or_else(|| "-".to_string());

        let rank_cell = if i < 3 {
            Cell::new(&format!("#{}", i + 1)).style_spec("Fg")
        } else {
            Cell::new(&format!("#{}", i + 1))
        };

        leaderboard_table.add_row(Row::new(vec![
            rank_cell,
            Cell::new(player_id),
            Cell::new(&format!("{:.2} ft", score)),
            Cell::new(&prize),
        ]));
    }
    leaderboard_table.printstd();
    println!();
}
