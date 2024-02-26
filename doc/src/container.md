# Containers

Containers are the instances of the game/software which are intialised from 
a recipe. Underneath, they are essentially a docker container.

The mastiff backend tries it best to not recreate containers on each startup
to make startup fast. For that a label `mastiff.container.meta-hash` is 
attached to the docker container. This label stores a SHA-256 of all the 
environment variables and the recipe version that was in the last startup. 
This label is compared during the startup and then recreates the container if 
they don't match.

The home directory `/container/home` is mount to the container's data directory
which is accessible via ftp, if configured.
