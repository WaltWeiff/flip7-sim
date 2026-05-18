use clap::Parser;
use debug_print::debug_println;
use rand::seq::SliceRandom;
use std::collections::{HashMap, VecDeque};
use std::vec;

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
        println!("Adding number cards...");
        deck.add_numbers();
        println!("Adding modifier cards...");
        deck.add_modifiers();
        println!("Adding action cards...");
        deck.add_actions();
        debug_println!("Created deck of {0} cards", deck.cards.len());
        assert!(deck.cards.len() == 94);
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
    numbers: Vec<Card>,
    modifiers: Vec<Card>,
    stay_value: i32,
    score: i32,
    second_chance: bool,
    out: bool,
}

impl Player {
    pub fn new(name: String, stay_value: i32) -> Player {
        debug_println!("Create a new player named {name}");
        Player {
            name,
            numbers: vec![],
            modifiers: vec![],
            stay_value,
            score: 0,
            second_chance: false,
            out: false,
        }
    }

    pub fn draw(&mut self, card: Card) -> Option<Card> {
        match card {
            Card::Number(_) => {
                if self.numbers.contains(&card) {
                    // if the hand already has this card, don't score it
                    debug_println!(
                        "{0}'s hand already has \"{1}\" in it!",
                        self.name,
                        card.to_string()
                    );
                    Some(card)
                } else {
                    self.numbers.push(card);
                    None
                }
            }
            Card::Modifier {
                is_mult: _,
                value: _,
            } => {
                // add modifier card to hand
                self.modifiers.push(card);
                None
            }
            Card::Action(_) => Some(card),
        }
    }

    pub fn discard_hand(&mut self) -> Vec<Card> {
        debug_println!("{0} discards their hand!", self.name);
        let mut hand: Vec<Card> = vec![];
        hand.append(&mut self.numbers);
        hand.append(&mut self.modifiers);
        hand
    }

    pub fn get_live_score(&self) -> i32 {
        let mut live_score = self.score.clone();
        for card in &self.numbers {
            match card {
                Card::Number(value) => live_score += value,
                _ => println!("Error: should not have a non-number card in the number card list!"),
            }
        }
        let mut multipliers: VecDeque<&Card> = VecDeque::new();
        for card in &self.modifiers {
            match card {
                Card::Modifier { is_mult, value } => {
                    if *is_mult {
                        multipliers.push_back(card);
                    } else {
                        live_score += value;
                    }
                }
                _ => println!(
                    "Error: should not have a non-modifier card in the modifier card list!"
                ),
            }
        }
        for card in multipliers {
            match card {
                Card::Modifier { is_mult: _, value } => live_score *= value,
                _ => println!(
                    "Error: should not have a non-modifier card in the multiplier card list!"
                ),
            }
        }
        live_score
    }

    pub fn get_points_scored(&self) -> i32 {
        self.get_live_score() - self.score
    }

    pub fn take_turn(&mut self, card: Card) -> Option<Card> {
        debug_println!("{0} has drawn \"{1}\"", self.name, card.to_string());
        // Add the card to hand, if able
        let todo = self.draw(card);

        match todo {
            Some(ref card) => {
                match card {
                    Card::Number(_) => {
                        // bust
                        debug_println!("{0} busts!", self.name);
                        if self.second_chance {
                            debug_println!("But they had a second chance. Phew!");
                            self.second_chance = false;
                            None
                        } else {
                            debug_println!("{0} is out of the round!", self.name);
                            self.out = true;
                            todo
                        }
                    }
                    Card::Modifier {
                        is_mult: _,
                        value: _,
                    } => {
                        eprintln!("Error: Impossible state. Failed to draw a modifier card.");
                        std::process::exit(-1);
                    }
                    Card::Action(_) => todo,
                }
            }
            None => None,
        }
    }

    pub fn end_round(&mut self) -> Vec<Card> {
        println!("{0} scored {1} points this round!", self.name, self.get_points_scored());
        self.score = self.get_live_score();
        self.out = false;
        self.discard_hand()
    }

    pub fn to_string(&self) -> String {
        format!(
            "Player Summary:\n\tName: {0}\n\tScore: {1}\n\tLive Score: {2}\n\tNumber Cards: {3}\n\tModifier Cards: {4}\n\tSecond Chance: {5}\n\tOut: {6}",
            self.name,
            self.score,
            self.get_live_score(),
            self.numbers.len(),
            self.modifiers.len(),
            self.second_chance,
            self.out
        )
    }
}

struct Game {
    players: VecDeque<Player>,
    deck: Deck,
    discard: Vec<Card>,
    target_score: i32,
}

impl Game {
    pub fn new(num_players: i32, target_score: i32, stay_value: i32) -> Game {
        debug_println!("Create a new game with {num_players} players staying at {stay_value}");
        let mut game: Game = Game {
            players: VecDeque::new(),
            deck: Deck::new(),
            discard: vec![],
            target_score,
        };
        for i in 0..num_players {
            game.players
                .push_back(Player::new(format!("Player {i}"), stay_value));
        }
        game
    }

    pub fn player_turn(&mut self, player: &mut Player) {
        // out or not
        if !player.out {
            // hit or stay
            if player.get_live_score() - player.score > player.stay_value {
                println!("{0} chooses to stay!", player.name);
                player.out = true;
            } else {
                // take turn
                debug_println!("{0} is taking their turn", player.name);
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
                                // discard hand
                                self.discard.append(&mut player.discard_hand());
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
                            Card::Action(ref action_type) => match action_type {
                                ActionType::Freeze => {
                                    // todo: targeting
                                    debug_println!(
                                        "{0} drew freeze. Skipping for now...",
                                        player.name
                                    );
                                }
                                ActionType::Flip3 => {
                                    debug_println!(
                                        "{0} drew Flip Three. Can only target ourselves right now...",
                                        player.name
                                    );
                                    // todo: targeting
                                    for _ in 0..3 {
                                        self.player_turn(player);
                                    }
                                }
                                ActionType::SecondChance => {
                                    debug_println!("{0} drew a second chance!", player.name);
                                    player.second_chance = true;
                                }
                            },
                        };
                        self.discard.push(card);
                    }
                    None => (),
                }
            }
        } else {
            println!("{0} is out of the round!", player.name);
            println!("{0}", player.to_string());
        }
    }

    pub fn print_scoreboard(&self) {
        println!("Scoreboard:");
        for player in &self.players {
            println!("\t{0}: {1} points, {2} live points (+{3})", player.name, player.score, player.get_live_score(), player.get_points_scored());
        }
    }

    pub fn turn(&mut self) {
        debug_println!("\nPlay a round of Flip 7!");
        let n_players = self.players.len();
        for _ in 0..n_players {
            let mut current_player = self
                .players
                .pop_front()
                .expect("Error: How can we have a game with no players?");
            self.player_turn(&mut current_player);
            self.players.push_back(current_player);
        }
        self.print_scoreboard();
    }

    pub fn round_over(&self) -> bool {
        let mut over = true;
        for player in &self.players {
            over &= player.out;
        }
        over
    }

    pub fn round(&mut self) {
        while !self.round_over() {
            self.turn();
        }
        println!("Round is over!");
        let n_players = self.players.len();
        for _ in 0..n_players {
            let mut current_player = self
                .players
                .pop_front()
                .expect("Error: How can we have a game with no players?");
            self.discard.append(&mut current_player.end_round());
            self.players.push_back(current_player);
        }
        self.print_scoreboard();
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
            let mut winner = winners.pop().expect("We just checked winners had contents...");
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
        while !self.is_game_over() {
            self.round();
        }
        println!("Game over!");
        let winner = self.game_winner().expect("Game cannot be over with no winner...");
        println!("Winner: {0} with {1} points!", winner.name, winner.score);
        self.print_scoreboard();

    }
}

/*
enum Strategies {
    Default,
    IgnoreModifiers,
    SelfFlipThree,
    SmartFlipThree,
}
*/

#[derive(Parser)]
enum Commands {
    Experiment {
        target_score: i32,
        stay_value: i32,
    },
    Simulation {
        number_of_players: i32,
        target_score: i32,
        stay_value: i32
    },
}

#[derive(Parser)]
struct CliArgs {
    #[command(subcommand)]
    command: Commands,
}

fn experiment(target_score: i32, stay_value: i32) {
    debug_println!("Do experiment up to {target_score} points, staying at {stay_value}");

    let mut game = Game::new(1, target_score, stay_value);

    game.play();
}

fn simulation(number_of_players: i32, target_score: i32, stay_value: i32) {
    println!("Do simulation with {number_of_players} players up to {target_score} points");

    let mut game = Game::new(number_of_players, target_score, stay_value);

    game.play();
}

fn main() {
    let args = CliArgs::parse();

    match &args.command {
        Commands::Experiment {
            target_score,
            stay_value,
        } => experiment(*target_score, *stay_value),
        Commands::Simulation {
            number_of_players,
            target_score,
            stay_value,
        } => simulation(*number_of_players, *target_score, *stay_value),
    }
}
