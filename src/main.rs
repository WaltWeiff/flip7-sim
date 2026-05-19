use clap::Parser;
use debug_print::debug_println;
use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet, VecDeque};
use std::vec;

const EXPECTED_NUM_CARDS: usize = 94;

#[derive(Eq, Hash, PartialEq, Clone)]
enum ActionType {
    Freeze,
    Flip3,
    SecondChance,
}

#[derive(Eq, Hash, PartialEq, Clone)]
enum Card {
    Number(i32),
    Modifier { is_mult: bool, value: i32 },
    Action(ActionType),
}

impl Card {
    pub fn to_string(&self) -> String {
        match self {
            Card::Number(value) => String::from(format!("Number: {value}")),
            Card::Modifier { is_mult, value } => {
                if *is_mult {
                    String::from(format!("Modifier: x{value}"))
                } else {
                    String::from(format!("Modifier: +{value}"))
                }
            }
            Card::Action(t) => match t {
                ActionType::Flip3 => String::from("Action: Flip Three"),
                ActionType::Freeze => String::from("Action: Freeze"),
                ActionType::SecondChance => String::from("Action: Second Chance"),
            },
        }
    }
}

struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Deck {
        let cards: Vec<Card> = Default::default();
        let mut deck: Deck = Deck { cards };
        debug_println!("Adding number cards...");
        deck.add_numbers();
        debug_println!("Adding modifier cards...");
        deck.add_modifiers();
        debug_println!("Adding action cards...");
        deck.add_actions();
        debug_println!("Created deck of {0} cards", deck.cards.len());
        assert!(deck.cards.len() == EXPECTED_NUM_CARDS);
        deck.print();
        deck.shuffle();
        deck
    }

    fn add_numbers(&mut self) {
        let initial_len = self.cards.len();

        debug_println!("Add 1 number card with value 0");
        self.cards.push(Card::Number(0));
        for i in 1..=12 {
            debug_println!("Add {i} number cards with value {i}");
            for _j in 0..i {
                self.cards.push(Card::Number(i));
            }
        }

        let cards_added = self.cards.len() - initial_len;
        debug_println!(
            "Deck has {0} cards, added {1}",
            self.cards.len(),
            cards_added
        );
        assert!(cards_added == 79);
    }

    fn add_modifiers(&mut self) {
        let initial_len = self.cards.len();

        for i in (2..=10).step_by(2) {
            debug_println!("Add 1 +{i} modifier card");
            self.cards.push(Card::Modifier {
                is_mult: false,
                value: i,
            });
        }
        debug_println!("Add 1 x2 modifier card");
        self.cards.push(Card::Modifier {
            is_mult: true,
            value: 2,
        });

        let cards_added = self.cards.len() - initial_len;
        debug_println!(
            "Deck has {0} cards, added {1}",
            self.cards.len(),
            cards_added
        );
        assert!(cards_added == 6)
    }

    fn add_actions(&mut self) {
        let initial_len = self.cards.len();

        debug_println!("Add 3 Freeze actions");
        for _i in 0..3 {
            self.cards.push(Card::Action(ActionType::Freeze));
        }
        debug_println!("Add 3 Flip Three actions");
        for _i in 0..3 {
            self.cards.push(Card::Action(ActionType::Flip3));
        }
        debug_println!("Add 3 Second Chance actions");
        for _i in 0..3 {
            self.cards.push(Card::Action(ActionType::SecondChance));
        }

        let cards_added = self.cards.len() - initial_len;
        debug_println!(
            "Deck has {0} cards, added {1}",
            self.cards.len(),
            cards_added
        );
        assert!(cards_added == 9);
    }

    pub fn print(&self) {
        println!("Deck:");
        let mut card_count: HashMap<&Card, i32> = HashMap::new();
        for card in &self.cards {
            card_count
                .entry(card)
                .and_modify(|value| *value += 1)
                .or_insert(1);
        }
        for (card, count) in card_count {
            println!("\t{count}x:\t{0}", card.to_string());
        }
    }

    pub fn shuffle(&mut self) {
        debug_println!("Shuffling the deck!");
        self.cards.shuffle(&mut rand::rng())
    }

    pub fn deal(&mut self) -> Option<Card> {
        let card = self.cards.pop();
        match card {
            None => {
                debug_println!("Failed to deal a card from the deck!");
            }
            Some(ref card) => {
                debug_println!("Dealing \"{0}\" from the deck", card.to_string());
            }
        };
        card
    }

    pub fn reset_discard(&mut self, discard: &mut Vec<Card>) {
        debug_println!("Attempting to refill the deck with the discard pile");
        assert!(self.cards.is_empty());
        self.cards = std::mem::take(discard);
        self.shuffle();
    }
}

struct Player {
    name: String,
    hand: Vec<Card>,
    stay_value: i32,
    score: i32,
    second_chance: bool,
    out: bool,
    flip7: bool,
    stay: bool,
    strategies: Vec<Strategy>,
}

impl Player {
    pub fn new(name: String, stay_value: i32) -> Player {
        debug_println!("Create a new player named {name}");
        Player {
            name,
            hand: vec![],
            stay_value,
            score: 0,
            second_chance: false,
            out: false,
            flip7: false,
            stay: false,
            strategies: vec![Strategy::RandomFlipThree],
        }
    }

    pub fn print_scoreboard(&self) {
        println!(
            "\t{} {}: {} points, {} live points (+{})",
            self.name,
            if self.out {
                String::from("(out)")
            } else if self.stay {
                String::from("(stayed)")
            } else {
                String::from("")
            },
            self.score,
            self.get_live_score(),
            self.get_points_scored()
        );
    }

    pub fn draw(&mut self, card: Card) -> Option<Card> {
        match card {
            Card::Number(_) => {
                if self.hand.contains(&card) {
                    // if the hand already has this card, don't score it
                    println!(
                        "{0}'s hand already has \"{1}\" in it!",
                        self.name,
                        card.to_string()
                    );
                    Some(card)
                } else {
                    self.hand.push(card);
                    None
                }
            }
            Card::Modifier {
                is_mult: _,
                value: _,
            } => {
                // add modifier card to hand
                self.hand.push(card);
                None
            }
            Card::Action(_) => Some(card),
        }
    }

    pub fn discard_hand(&mut self) -> Vec<Card> {
        debug_println!("{0} discards their hand!", self.name);
        std::mem::take(&mut self.hand)
    }

    pub fn get_live_score(&self) -> i32 {
        self.score + self.get_points_scored()
    }

    pub fn get_points_scored(&self) -> i32 {
        let mut points_scored = 0;
        if !self.out {
            let mut multipliers: VecDeque<&Card> = VecDeque::new();
            let mut modifier_value = 0;
            for ref card in &self.hand {
                match card {
                    Card::Number(value) => points_scored += value,
                    Card::Modifier { is_mult, value } => {
                        if *is_mult {
                            multipliers.push_back(card);
                        } else {
                            modifier_value += value;
                        }
                    }
                    Card::Action(_) => {}
                }
            }
            // multipliers are scored before addition modifiers
            for card in multipliers {
                match card {
                    Card::Modifier { is_mult: _, value } => points_scored *= value,
                    _ => eprintln!(
                        "Error: should not have a non-modifier card in the multiplier card list!"
                    ),
                }
            }
            points_scored += modifier_value;
            // flip 7 bonus scored last
            if self.flip7 {
                points_scored += 15;
            }
        }
        points_scored
    }

    pub fn take_turn(&mut self, card: Card) -> Option<Card> {
        println!("{0} has drawn \"{1}\"", self.name, card.to_string());
        // Add the card to hand, if able
        let handle_card = self.draw(card);

        match handle_card {
            Some(ref card) => {
                match card {
                    Card::Number(_) => {
                        // bust
                        println!("{0} busts!", self.name);
                        if self.second_chance {
                            println!("But they had a second chance. Phew!");
                            self.second_chance = false;
                        } else {
                            println!("{0} is out of the round!", self.name);
                            self.out = true;
                        }
                        handle_card
                    }
                    Card::Modifier {
                        is_mult: _,
                        value: _,
                    } => {
                        eprintln!("Error: Impossible state. Failed to draw a modifier card.");
                        std::process::exit(-1);
                    }
                    Card::Action(_) => handle_card,
                }
            }
            None => None,
        }
    }

    pub fn should_stay(&self) -> bool {
        // todo: this can be smarter
        self.get_live_score() - self.score >= self.stay_value
    }

    pub fn end_round(&mut self) -> Vec<Card> {
        println!(
            "{0} scored {1} points this round!",
            self.name,
            self.get_points_scored()
        );
        println!("{}'s cards in hand:", self.name);
        for card in &self.hand {
            println!("\t\t\"{}\"", card.to_string())
        }
        self.score = self.get_live_score();
        let hand = self.discard_hand();
        self.out = false;
        self.flip7 = false;
        self.second_chance = false;
        self.stay = false;
        hand
    }

    pub fn to_string(&self) -> String {
        format!(
            "Player Summary:\n\
            \tName: {}\n\
            \tScore: {}\n\
            \tLive Score: {}\n\
            \tHand Size: {}\n\
            \tNumber Cards: {}\n\
            \tModifier Cards: {}\n\
            \tAction Cards: {}\n\
            \tSecond Chance: {}\n\
            \tOut: {}\n\
            \tStayed: {}",
            self.name,
            self.score,
            self.get_live_score(),
            self.hand.len(),
            self.hand
                .iter()
                .filter(|card| match card {
                    Card::Number(_) => true,
                    Card::Modifier {
                        is_mult: _,
                        value: _,
                    } => false,
                    Card::Action(_) => false,
                })
                .count(),
            self.hand
                .iter()
                .filter(|card| match card {
                    Card::Number(_) => false,
                    Card::Modifier {
                        is_mult: _,
                        value: _,
                    } => true,
                    Card::Action(_) => false,
                })
                .count(),
            self.hand
                .iter()
                .filter(|card| match card {
                    Card::Number(_) => false,
                    Card::Modifier {
                        is_mult: _,
                        value: _,
                    } => false,
                    Card::Action(_) => true,
                })
                .count(),
            self.second_chance,
            self.out,
            self.stay,
        )
    }
}

struct Game {
    players: VecDeque<Player>,
    deck: Deck,
    discard: Vec<Card>,
    round_discard: Vec<Card>,
    target_score: i32,
}

impl Game {
    pub fn new(num_players: i32, target_score: i32, stay_value: i32) -> Game {
        debug_println!("Create a new game with {num_players} players staying at {stay_value}");
        let mut game: Game = Game {
            players: VecDeque::new(),
            deck: Deck::new(),
            discard: vec![],
            round_discard: vec![],
            target_score,
        };
        let mut player_set = HashSet::new();
        for i in 0..num_players {
            let player = Player::new(format!("Player {i}"), stay_value);
            assert!(player_set.insert(player.name.clone()));
            game.players.push_back(player);
        }
        game
    }

    pub fn player_turn(&mut self, player: &mut Player, must_hit: bool) -> Option<Card> {
        // out or not
        if !player.out && !player.stay {
            // hit or stay
            if !must_hit && player.should_stay() {
                println!("{0} chooses to stay!", player.name);
                player.stay = true;
            } else {
                // take turn
                if must_hit {
                    println!("{0} must hit!", player.name);
                } else {
                    println!("{0} chooses to hit!", player.name);
                }
                let card = self.deck.deal().unwrap_or_else(|| {
                    self.deck.reset_discard(&mut self.discard);
                    self.deck
                        .deal()
                        .expect("Error: Deck could not be refilled with discard pile")
                });
                let handle_card = player.take_turn(card);

                match handle_card {
                    Some(card) => {
                        match card {
                            Card::Number(_) => {
                                // bust
                                // or used second chance
                                // either way, discard the number card
                                debug_println!("Add \"{0}\" to round discard", card.to_string());
                                self.round_discard.push(card);
                            }
                            Card::Modifier {
                                is_mult: _,
                                value: _,
                            } => {
                                eprintln!(
                                    "Error: Impossible state. Failed to take a turn with a modifier card."
                                );
                                std::process::exit(-1);
                            }
                            Card::Action(ref action_type) => {
                                match action_type {
                                    ActionType::Freeze => {
                                        println!("{} Drew a Freeze!", player.name);
                                        // Freeze Strategy: target the player with the highest live score
                                        // This gets weird when the highest score is > target score
                                        // In that case, a human player might play kingmaker or make a social decision
                                        // beep boop I'm a computer and cannot have context for that
                                        let target = self
                                            .players
                                            .iter_mut()
                                            .filter(|p| !p.out && !p.stay)
                                            .max_by(|a, b| {
                                                a.get_live_score().cmp(&b.get_live_score())
                                            })
                                            .unwrap_or_else(|| player);
                                        println!(
                                            "Targeting {} because they have the highest score {} (or were the only valid target)",
                                            target.name,
                                            target.get_live_score()
                                        );

                                        target.hand.push(card);
                                        target.stay = true;
                                        println!("{0} is forced to stay!", target.name);
                                    }
                                    ActionType::Flip3 => {
                                        println!("{0} drew Flip Three", player.name);
                                        return Some(card);
                                    }
                                    ActionType::SecondChance => {
                                        println!("{0} drew a second chance!", player.name);
                                        if player.second_chance {
                                            println!("But they already had one!");
                                            // This is such an edge case but whatever
                                            // Target the player with the lowest score
                                            let target = self
                                                .players
                                                .iter_mut()
                                                // filter to valid targets: in the game and does not have a second chance
                                                .filter(|p| !p.out && !p.stay && !p.second_chance)
                                                .min_by(|a, b| {
                                                    a.get_live_score().cmp(&b.get_live_score())
                                                })
                                                .unwrap_or_else(|| player);
                                            println!(
                                                "Targeting {} because they have the lowest score {} (or were the only valid target)",
                                                target.name,
                                                target.get_live_score()
                                            );
                                            // only possible if target is current player
                                            if target.second_chance {
                                                self.round_discard.push(card);
                                            } else {
                                                target.hand.push(card);
                                                target.second_chance = true;
                                            }
                                        } else {
                                            player.second_chance = true;
                                            player.hand.push(card);
                                        }
                                    }
                                }
                            }
                        };
                    }
                    None => (),
                }
                let player_number_count = player
                    .hand
                    .iter()
                    .filter(|card| match card {
                        Card::Number(_) => true,
                        Card::Modifier {
                            is_mult: _,
                            value: _,
                        } => false,
                        Card::Action(_) => false,
                    })
                    .count();
                if player_number_count == 7 {
                    println!(
                        "{0} Flipped 7! Congratulations! +15 points, and the round is over",
                        player.name
                    );
                    player.flip7 = true;
                    return Some(Card::Number(-1));
                } else if player_number_count > 7 {
                    eprintln!(
                        "Error: illegal state. Cannot have more than 7 number cards in hand!"
                    );
                    std::process::exit(-1);
                }
            }
        } else if player.out && !player.stay {
            println!("{0} is out of the round!", player.name);
        } else if !player.out && player.stay {
            println!("{0} has stayed!", player.name);
        } else {
            eprintln!("How did we get here?");
            std::process::exit(-1);
        }
        None
    }

    pub fn print_scoreboard(&self) {
        println!("Scoreboard:");
        for player in &self.players {
            player.print_scoreboard();
        }
    }

    pub fn turn(&mut self, first_round: bool) -> bool {
        println!("\nPlay a turn of Flip 7!");
        let n_players = self.players.len();
        for i in 0..n_players {
            let mut current_player = self
                .players
                .pop_front()
                .expect("Error: How can we have a game with no players?");
            println!("\n{0} is taking their turn", current_player.name);

            let handle_game_action = self.player_turn(&mut current_player, first_round);

            println!(
                "{0}: {1} points, {2} live points (+{3})",
                current_player.name,
                current_player.score,
                current_player.get_live_score(),
                current_player.get_points_scored()
            );

            match handle_game_action {
                Some(card) => {
                    match card {
                        Card::Number(value) => {
                            if value == -1 {
                                // flipped 7
                                self.players.push_back(current_player);
                                self.print_scoreboard();
                                for _ in i..n_players {
                                    let current_player = self
                                        .players
                                        .pop_front()
                                        .expect("Error: How can we have a game with no players?");
                                    self.players.push_back(current_player);
                                }
                                return true;
                            } else {
                                eprintln!(
                                    "Error: Encountered a number card not handled during a turn"
                                );
                                std::process::exit(-1);
                            }
                        }
                        Card::Action(ref t) => match t {
                            ActionType::Flip3 => {
                                println!("{} choosing Flip Three target..", current_player.name);
                                if current_player.strategies.contains(&Strategy::SelfFlipThree) {
                                    println!("Targeting self ({})", current_player.name);
                                    for _ in 0..3 {
                                        self.player_turn(&mut current_player, true);
                                    }
                                } else {
                                    let idx = rand::random_range(0..self.players.len() + 1);
                                    if idx == self.players.len() {
                                        println!("Targeting {}", current_player.name);
                                        for _ in 0..3 {
                                            self.player_turn(&mut current_player, true);
                                        }
                                    } else {
                                        let mut target = self
                                            .players
                                            .remove(idx)
                                            .expect("Error: Could not get player by index");
                                        println!("Targeting {}", target.name);
                                        for _ in 0..3 {
                                            self.player_turn(&mut target, true);
                                        }
                                        self.players.insert(idx, target);
                                    };
                                }
                                debug_println!("Add \"{}\" to round discard", card.to_string());
                                self.round_discard.push(card);
                            }
                            _ => {
                                eprintln!(
                                    "Error: Encountered a non-flip-3 action card not handled during a turn"
                                );
                                std::process::exit(-1);
                            }
                        },
                        _ => {
                            eprintln!("Error: Encountered a card not handled during a turn");
                            std::process::exit(-1);
                        }
                    }
                }
                None => (),
            }
            self.players.push_back(current_player);
        }
        self.print_scoreboard();
        false
    }

    pub fn round_over(&self) -> bool {
        let mut over = true;
        for player in &self.players {
            over &= player.out || player.stay;
        }
        over
    }

    pub fn round(&mut self, mut first_round: bool) {
        println!("\nPlay a round of Flip 7!");
        while !self.round_over() {
            if self.turn(first_round) {
                break;
            }
            first_round = false;
        }
        println!("\nRound is over!");
        let n_players = self.players.len();
        let mut cards_in_hands = 0;
        for _ in 0..n_players {
            let mut current_player = self
                .players
                .pop_front()
                .expect("Error: How can we have a game with no players?");
            debug_println!("{0}", current_player.to_string());
            cards_in_hands += current_player.hand.len();
            self.discard.append(&mut current_player.end_round());
            self.players.push_back(current_player);
        }
        debug_println!("Cards in hands (to be discarded): {}", cards_in_hands);
        debug_println!(
            "Cards discarded that were not in hands: {}",
            self.round_discard.len()
        );
        self.discard.append(&mut self.round_discard);
        self.print_scoreboard();
        debug_println!("Cards remaining in deck: {0}", self.deck.cards.len());
        debug_println!("Cards in discard: {0}", self.discard.len());
        debug_println!(
            "Total Cards in Play: {0}",
            self.deck.cards.len() + self.discard.len()
        );
        assert!(self.round_discard.len() == 0);
        assert!(self.deck.cards.len() + self.discard.len() == EXPECTED_NUM_CARDS);
    }

    pub fn game_winner(&self) -> Option<&Player> {
        let mut winners: Vec<&Player> = vec![];
        for player in &self.players {
            if player.score >= self.target_score {
                winners.push(player)
            }
        }
        if winners.len() == 0 {
            None
        } else {
            let mut winner = winners
                .pop()
                .expect("We just checked winners had contents...");
            while let Some(contender) = winners.pop() {
                if contender.score > winner.score {
                    winner = contender;
                }
            }
            Some(winner)
        }
    }

    pub fn is_game_over(&self) -> bool {
        self.game_winner().is_some()
    }

    pub fn play(&mut self) {
        self.round(true);
        while !self.is_game_over() {
            self.round(false);
        }
        println!("\nGame over!");
        let winner = self
            .game_winner()
            .expect("Game cannot be over with no winner...");
        println!("Winner: {0} with {1} points!", winner.name, winner.score);
        self.print_scoreboard();
    }
}

#[derive(Clone, PartialEq)]
enum Strategy {
    IgnoreModifiers,
    SelfFlipThree,
    SmartFlipThree,
    RandomFlipThree,
}

impl std::str::FromStr for Strategy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ignore-modifiers" => Ok(Strategy::IgnoreModifiers),
            "self-flip-three" => Ok(Strategy::SelfFlipThree),
            "smart-flip-three" => Ok(Strategy::SmartFlipThree),
            _ => Err(format!("invalid value: {}", s)),
        }
    }
}

impl std::fmt::Display for Strategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Strategy::IgnoreModifiers => String::from("IgnoreModifiers"),
            Strategy::SelfFlipThree => String::from("SelfFlipThree"),
            Strategy::SmartFlipThree => String::from("SmartFlipThree"),
            Strategy::RandomFlipThree => String::from("RandomFlipThree"),
        };
        write!(f, "{}", s)
    }
}

#[derive(Parser)]
enum ExperimentCommands {
    OptimizeStay { strategies: Vec<Strategy> },
}

#[derive(Parser)]
enum Commands {
    Experiment {
        target_score: i32,
        #[command(subcommand)]
        command: ExperimentCommands,
    },
    Simulation {
        number_of_players: i32,
        target_score: i32,
        stay_value: i32,
    },
}

#[derive(Parser)]
struct CliArgs {
    #[command(subcommand)]
    command: Commands,
}

fn optimize_stay_value(target_score: i32, strategies: &Vec<Strategy>) {
    debug_println!("Perform experiment to optimize stay value with target score {target_score}");
    debug_println!("Using strategies:");
    for strategy in strategies {
        debug_println!("\t{}", strategy);
    }

    /*
    let stay_value = 25;

    let mut game = Game::new(1, target_score, stay_value);

    game.play();
    */
}

fn simulation(number_of_players: i32, target_score: i32, stay_value: i32) {
    debug_println!("Do simulation with {number_of_players} players up to {target_score} points");

    let mut game = Game::new(number_of_players, target_score, stay_value);

    game.play();
}

fn main() {
    let args = CliArgs::parse();

    match &args.command {
        Commands::Simulation {
            number_of_players,
            target_score,
            stay_value,
        } => simulation(*number_of_players, *target_score, *stay_value),
        Commands::Experiment {
            target_score,
            command,
        } => match command {
            ExperimentCommands::OptimizeStay { strategies } => {
                optimize_stay_value(*target_score, strategies)
            }
        },
    }
}
