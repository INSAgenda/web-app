# web-app

Welcome to [INSAgenda](https://insagenda.fr/)'s open-source client repository.  
Feel free to check our code, suggest ideas and contribute by adding new features.  

## Running

You can compile this project and run it on your local machine.  
In order to enable you to test the program, we will provide you with a binary of the backend. This binary is an unoptimized build that is intented to be used for development purposes only. This publicly provided backend is compatible with this public repository.  
However, the production backend will refuse all requests sent by clients you compiled yourself. Only we can compile clients that will be compatible with the production backend. When you contribute to this repository and your changes are accepted, we will take care of deploying your code to [insagenda.fr](https://insagenda.fr).  
  
If you need help at any time feel free to come chat with us on [our discord server](https://discord.gg/TpdbUyfcbJ).  
Now that you know how we work, let's get started!

### Install the tools

Please install [Rust](https://www.rust-lang.org/), [SQLite3](https://www.sqlite.org/index.html) and [Trunk](https://trunkrs.dev/) on your machine.

Here are the commands to run on Ubuntu-based machines:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sudo sh # Install Rust
sudo apt install sqlite3 # Install SQLite3
cargo install trunk # Install Trunk
```

### Organize data

This repository only contains the web-app hosted at `/agenda`.
All other files are stored in [our frontend repository](https://github.com/INSAgenda/frontend), and the backend is closed-source. There is also [another public repository](https://github.com/INSAgenda/data-structures) containing common data structures used by both the web-app and the backend.  

In order to setup your development environment, you will thus need to clone three repositories and download a backend binary.

Here is the recommended file structure:

```text
insagenda/
├─ frontend/
├─ web-app/
├─ data-structures/
├─ backend/
```

Commands to run:

```bash
mkdir insagenda && cd insagenda
git clone https://github.com/INSAgenda/frontend
git clone https://github.com/INSAgenda/web-app
git clone https://github.com/INSAgenda/data-structures
mkdir backend && cd backend
wget https://insagenda.fr/development/backend # Download the backend binary
wget https://insagenda.fr/development/database # Download an empty database ready to be used by the backend
cd ..
```

### Run the code

```bash
# Copy frontend into the backend-served folder
rm -rf backend/files
cp -r frontend/. backend/files 
mkdir backend/files/agenda

# Run the backend on background
./backend/backend &

# Build the web-app
cd web-app
trunk build --dist ../backend/files/agenda --public-url=agenda
```

The web-app will be served at `http://localhost:8080/agenda`.  

You can also run `trunk watch` instead of `trunk build`.
This will continuously build the web-app as source files are modified.

_Tip: Make yourself a script with the code above to increase your productivity._

## Contributing

Pull requests and issues are welcome!
