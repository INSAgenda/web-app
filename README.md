<div align="center">
    <a href="https://insagenda.insa.lol/">
        <img src="https://insagenda.insa.lol/assets/logo/logo.svg" alt="INSAgenda's logo" width="80" height="80">
    </a>
    <h1 align="center">INSAgenda (web-app)</h1>
    <p align="center">
        INSAgenda is a free website allowing students to view their course schedule at INSA Rouen.<br/>
        <a href="https://insagenda.insa.lol/"><b>Explore our website »</b></a><br/><br/>
    </p>
</div>

## Table of contents

1. [Running](#running)
    - [Install the tools](#install-the-tools)
    - [Organize the data](#organize-data)
    - [Run the code](#run-the-code)
2. [Contributing](#contributing)
    - [Recommandations](#recommandations)
    - [License](#license)

## Running

You can compile this project and run it on your local machine.  
  
In order to enable you to test the program, we will provide you with a binary of the backend[^backend-binary]. This binary is an unoptimized build that is intented to be used for *development purposes only*. This publicly provided backend is compatible with this public repository.  
  
However, the *production* backend will refuse all requests sent by clients you compiled yourself. Only we can compile clients that will be compatible with the production backend. When you contribute to this repository and your changes are accepted, we will take care of deploying your code to [insagenda.insa.lol](https://insagenda.insa.lol).  
  
If you need help at any time feel free to come chat with us on [our discord server](https://discord.gg/TpdbUyfcbJ).  
Now that you know how we work, let's get started!

### Install the tools

Please install [Rust](https://www.rust-lang.org/), [SQLite3](https://www.sqlite.org/index.html) and [Trunk](https://trunkrs.dev/) on your machine.

Here are the commands to run on Ubuntu-based systems:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sudo sh # Install Rust
rustup target add wasm32-unknown-unknown # Install wasm target for Rust
sudo apt install sqlite3 # Install SQLite3
cargo install trunk # Install Trunk
```

### Organize data

This repository only contains the web app hosted at `/agenda`.
All other files are stored in [our frontend repository](https://github.com/INSAgenda/frontend), and the backend is closed-source. There is also [another public repository](https://github.com/INSAgenda/common) containing common data structures used by both the web app and the backend.  

In order to setup your development environment, you will thus need to clone three repositories and download a backend binary[^backend-binary].

Here is the **required** file structure:

```text
insagenda/
├─ frontend/
├─ web-app/
├─ common/
├─ backend/
```

Commands to run:

```bash
mkdir insagenda && cd insagenda
git clone https://github.com/INSAgenda/frontend
git clone https://github.com/INSAgenda/yew-template
git clone https://github.com/INSAgenda/web-app
git clone https://github.com/INSAgenda/common
mkdir backend && cd backend
wget https://insagenda.insa.lol/development/backend.tar.gz # Download the backend binary
tar -xvf backend.tar.gz
wget https://insagenda.insa.lol/development/database # Download an empty database ready to be used by the backend
cd ..
```

### Run the code

```bash
# Run the backend in a terminal
./backend/backend

# Build the web-app in another terminal
cd web-app
trunk build --public-url=agenda
```

The web-app will be served at `http://localhost:8088/agenda`. All files from the frontend folder will also be served.  

You can also run `trunk watch` instead of `trunk build`.
This will continuously build the web-app as source files are modified.

_Tip: Make yourself a script with the code above to increase your productivity._

## Contributing

### Recommendations

Unwraps are ok on many web_sys methods, but you have to be absolutely sure it will not panic on any modern browser.  
  
Errors are to be handled, but unhandled errors are to be displayed to the user using our custom alert function. We do not currently provide the list of errors that could occur on each endpoint, but that's definitely something we would like have.  

### License

This project is unlicensed. This means the source code is protected by copyright laws in the most restrictive way.  
You can read the code and contribute to it, but you mustn't use it for any other purpose.

[^backend-binary]: The backend binary can be downloaded [from our website](https://insagenda.insa.lol/development/backend.tar.gz). This is obviously amd64 Linux-only. You will also need to download [an empty database](https://insagenda.insa.lol/development/database).
