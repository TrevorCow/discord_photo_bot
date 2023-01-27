# Discord Bot and Web Server

This project is a Discord bot and web server built in Rust. It was created as a project for my photography class and as a
way to practice the Rust programming language.

## Features

- The bot allows users to interact with the web server by sending commands and receiving responses.
- The web server displays galleries of pictures from the class discord.

## Getting Started

To run this project, you will need to have Rust and Cargo installed on your machine. You will also need to create a
Discord bot and get its API token.

1. Clone the repository:
   ```git clone https://github.com/trevorcow/discord_photo_bot```

2. Set the following environment variables:
   ```DISCORD_TOKEN=YOUR_DISCORD_BOT_TOKEN```

3. Start the discord bot and server:
   ```cargo run```

4. Invite the bot to your Discord server and use ```/collectphotos```, it has to be set up like our photography discord server to work write.

## Built With

- [Rust](https://www.rust-lang.org/) - The programming language used
- [Cargo](https://doc.rust-lang.org/cargo/) - The package manager for Rust
- Main libraries (crates)
    - [serenity](https://docs.rs/serenity/) - For the discord bot
    - [tiny_http](https://docs.rs/tiny_http/) - For the webserver
    - [handlebars](https://docs.rs/handlebars/) - For dynamically building the webpages

## Contributing

If you are interested in contributing to this project, please feel free to open a pull request or issue.

## Acknowledgments

- Photography class for providing the inspiration for this project
- Rust community for providing helpful resources and support.
- ChatGPT for writing README.md file. Yes it wrote almost the whole thing, I just cleaned it up and fixed some links