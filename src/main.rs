use std::io::Read;
use std::io::*;
use std::process::*;

const ENGINE_COMMAND: &'static str = "stockfish";
const ENGINE_TIME_MS: u32 = 1000;

pub fn error(message: impl Into<String>) -> ! {
    println!("Error: {}.", message.into());
    std::process::exit(1)
}

fn get_eval(fen: &str) -> i32 {

    let mut process = Command::new(ENGINE_COMMAND)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .unwrap();

    let stdin = process.stdin.take().unwrap();
    let stdout = process.stdout.take().unwrap();


    let mut stdin_writer = BufWriter::new(stdin);
    let mut stdout_reader = BufReader::new(stdout);

    let _ = stdin_writer.write("uci\n".as_bytes());
    let _ = stdin_writer.write(format!("position fen {fen}\n").as_bytes());
    let _ = stdin_writer.write(format!("go movetime {ENGINE_TIME_MS}\n").as_bytes());
    stdin_writer.flush();

    let mut line = String::new();
    let mut current_eval = 0;

    while let Ok(_) = stdout_reader.read_line(&mut line) {

        if line.starts_with("bestmove") {
            break;
        }

        let mut split = line.split(' ');

        while let Some(tag) = split.next() {

            if tag != "score" {
                continue;
            }

            current_eval = match split.next().unwrap() {
                "mate" => i32::MAX,
                "cp" => split.next().unwrap().parse().unwrap(),
                _ => unreachable!()
            };

            break;

        }

        line = String::new();

    }

    current_eval

}

fn main() {

    let args = std::env::args().collect::<Vec<_>>();

    if args.len() == 1 {
        error("No PGN given");
    }

    let Ok(mut file) = std::fs::File::open(args[1].clone())
    else {
        error("PGN does not exist");
    };

    let mut pgn = String::new();
    let _ = file.read_to_string(&mut pgn);

    let game = chess::game::Game::from_pgn(pgn);
    let mut current_move = Some(game.get_root());

    let mut board = chess::game::Board::default();
    let mut evaluations = Vec::new();

    evaluations.push(get_eval(&board.get_fen()));

    while let Some(move_node) = current_move {

        board.make_move(&move_node.played_move);

        let mut eval = get_eval(&board.get_fen());

        if board.side_to_move == chess::game::Colour::Black {
            eval = -eval;
        }

        evaluations.push(eval);

        current_move = game.get_main_line(move_node);

    }

    // calculation from https://lichess.org/page/accuracy

    let win_percentages: Vec<_> = evaluations.iter().map(|eval| 
        50.0 + 50.0 * (2.0 / (1.0 + (-0.00368308 * (*eval as f32)).exp()) - 1.0)
    ).collect();

    let mut white_accuracies = Vec::new();
    let mut black_accuracies = Vec::new();

    for i in 1..evaluations.len() {


        if i % 2 == 1 {
            let accuracy = 103.1668 * (-0.04354 * (win_percentages[i - 1] - win_percentages[i])).exp() - 3.1669;
            white_accuracies.push(accuracy);
        }
        else {
            let accuracy = 103.1668 * (-0.04354 * (win_percentages[i] - win_percentages[i - 1])).exp() - 3.1669;
            black_accuracies.push(accuracy);
        }
    }

    println!("{win_percentages:?}");
    println!("{white_accuracies:?}");
    println!("{black_accuracies:?}");

    println!("White Accuracy: {}%", white_accuracies.iter().cloned().sum::<f32>() / (white_accuracies.len() as f32));
    println!("Black Accuracy: {}%", black_accuracies.iter().cloned().sum::<f32>() / (black_accuracies.len() as f32));

}
