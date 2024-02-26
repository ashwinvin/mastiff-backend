# Recipes

A recipe stores the configuration for a specific game/software and instructions for 
how the container for it must be built and maintained. Recipes are made in the panel
and panel transfers the data to the node.

Recipes do not support in place updates but are rather deleted and fully recreated
from the tar file. This is mainly to prevent unnecesary state management creeping 
into the backend. 

The docker images are built/pulled when the recipes are parsed, this prevents slowdowns
in container startup when it is attempted after a recipe upload. 

## Recipe File Structure
The recipes are stored in their own directory in the configured directory.

```
game_recipe
   recipe.yaml
   Dockerfile (optional)
   (any other data or scripts)
```

If you require bundling any files during the image creation, include it in the recipe.
The files except `recipe.yaml` will be available during the image building. 

<div class="warning">
Do not try to add a recipe directly to the directory, the recipe won't show up in the panel
as the node pulls recipes from the panel, not vice versa.
</div>

## Recipe Config

### `recipe.yaml` Structure

```toml
name = "blah" # Name of the recipe. Must be a valid directory name. 
version = "123" # Set automatically by the panel.
image = "Local" # The source of the image. Set to `Local` if a Dockerfile is provided.
process_started_indicator = "yes" # The string which the container logs after it has fully started. 
process_ended_indicator = "no" # The string which the container logs after it has fully exited. 
process_stop_cmd = "exit" # The command to send to the container to stop it. If not given, SIGTERM will be send
min_ports = 1 # The minimum number of port allocation(s) required for the container. 
```

### Dockerfile
There are few constraints on the container's parameter for it work smoothly with mastiff.

- It is recommended to use alpine as the base image due to its small size.
- The `WORKDIR` must be set to `/home`.
- The `USER` must be set as `container`.


> The implementation is in `/managers/recipe.rs`
