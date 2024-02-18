# quokka

Uma versão Rust bastante modesta da [rinha do backend 2ª edição](https://github.com/zanfranceschi/rinha-de-backend-2024-q1) 2024/Q1

```
    __                      __
 .-'  `'.._...-----..._..-'`  '-.
/                                \
|  ,   ,'                '.   ,  |
 \  '-/                    \-'  /
  '._|          _           |_.'
     |    /\   / \    /\    |
     |    \/   | |    \/    |
      \        \"/         /
       '.    =='^'==     .'
         `'------------'`

                                    88        88                    
                                    88        88                    
                                    88        88                    
 ,adPPYb,d8 88       88  ,adPPYba,  88   ,d8  88   ,d8  ,adPPYYba,  
a8"    `Y88 88       88 a8"     "8a 88 ,a8"   88 ,a8"   ""     `Y8  
8b       88 88       88 8b       d8 8888[     8888[     ,adPPPPP88  
"8a    ,d88 "8a,   ,a88 "8a,   ,a8" 88`"Yba,  88`"Yba,  88,    ,88  
 `"YbbdP'88  `"YbbdP'Y8  `"YbbdP"'  88   `Y8a 88   `Y8a `"8bbdP"Y8  
         88                                                         
         88  
```

## Stack

* 2 Rust apps
* 1 PostgreSQL
* 1 NGINX

<img width="1023" alt="Screenshot 2024-02-18 at 02 06 09" src="https://github.com/leandronsp/quokka/assets/385640/2ca00c45-bf31-4372-b7ce-9b239886c5f2">

## Usage

```bash
$ make help

Usage: make <target>
  help                       Prints available commands
  start.dev                  Start the rinha in Dev
  start.prod                 Start the rinha in Prod
  docker.stats               Show docker stats
  health.check               Check the stack is healthy
  stress.it                  Run stress tests
  docker.build               Build the docker image
  docker.push                Push the docker image
```

## Inicializando a aplicação

```bash
$ docker compose up -d nginx

# Ou então utilizando Make...
$ make start.dev
```

Testando a app:

```bash
$ curl -v http://localhost:9999/clientes/1/extrato

# Ou então utilizando Make...
$ make health.check
```

## Unleash the madness

Colocando Gatling no barulho:

```bash
$ make stress.it 
$ open stress-test/user-files/results/**/index.html
```

----

[ASCII art generator](http://www.network-science.de/ascii/)
