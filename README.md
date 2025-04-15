# zero-to-production-in-rust 
Pointing newcomers to the [original repo](https://github.com/LukeMathWalker/zero-to-production) which contains more comprehensive solutions and documentation. This repo exists as a playground for me to follow along to the corresponding book (which is great, you should read it!)

# running the code locally
Essentially there are two components to this project; our databases (Postgres, Redis) and the backend which serves static HTML. 

Running the commands below requires that you have Docker installed on your system.

To get the databases up and running use the scripts in `scripts/`, e.g. 
```bash 
./scripts/init_db.sh
./scripts/init_redis.sh
```
The first script will ensure [`sqlx`](https://github.com/launchbadge/sqlx) is installed and run a Posgres docker container -- applying all the migrations found in /migrations. The second one just sets up a Redis server, also in a docker container. 

To setup the backend we can use a trusty 
```bash
cargo run
```

And there you go !
