# Recipes

A recipe stores the configuration for a specific game/software and instructions for 
how the container for it must be built and maintained. Recipes are made in the panel
and panel transfers the data to the node.

### Recipe File Structure
The recipes are stored individually as a tar archive in the configured directory.

```
game_recipe.tar
   |- recipe.yaml
   |- ContainerFile (optional)
   |- (any other data or scripts)
```

<div class="warning">
If you require bundling any files during the image creation, include it in the recipe.
The files except `recipe.yaml` will be available during the image building. 
</div>

### Containerfile
There are few constraints on the container's parameter for it work smoothly with mastiff.

- It is recommended to use alpine as the base image due to its small size.
- The `WORKDIR` must be set to `/home`.
- The `USER` must be set as `container`.