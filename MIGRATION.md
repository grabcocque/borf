# Migrating from @Core to Direct Imports

## Overview

We've removed the `@Core` module in favor of direct imports from specific modules. This improves modularity, reduces load times, and makes dependencies more explicit.

## Migration Steps

1. **Identify usages**: Search your codebase for `@Core` or `Core.` references
2. **Determine the source module**: For each function/type/operator previously accessed via Core, identify its original module
3. **Update your imports**: Add explicit dependencies on the required modules
4. **Update your references**: Replace Core.X with ModuleName.X

## Common Functions and Their Modules

### From Prim Module
- `Prim.typeof`
- `Prim.is_valid`
- `Prim.pipe` (pipe operator)
- `Prim.fcomp` (function composition)
- `Prim.compose`
- `Prim.and`, `Prim.or`, `Prim.not` (logical operators)

### From Flp Module
- `Flp.!Success`, `Flp.!Failure` (result handling)
- `Flp.id` (identity function)
- `Flp.flip` (argument flipping)
- `Flp.curry`, `Flp.uncurry` (currying operations)
- `Flp.-o>` (linearity operator)

### From T Module
- `T.T`, `T.Sym`, `T.Any`, `T.Void` (type fundamentals)
- `T.teq` (type equality)
- `T.<::`, `T.~`, `T.->`, etc. (type operators)

### From R Module
- `R.veq`, `R.seq`, `R.omega` (relation operators)

### From Mod Module
- `Mod.TypeSym`, `Mod.OpSym`, `Mod.FnSym`, `Mod.ModSym` (symbol types)
- `Mod.tau`, `Mod.delta`, `Mod.vis` (module system essentials)

## Example Migration

Before:
```borf
deps: {
  (@MyModule, @Core, true)
}

-- Using Core exports
let result = Core.pipe(value, Core.compose(f, g))
```

After:
```borf
deps: {
  (@MyModule, @Prim, true),
  (@MyModule, @Flp, true)
}

-- Using direct imports
let result = Prim.pipe(value, Prim.compose(f, g))
```

## Benefits of Direct Imports

1. **Better Performance**: Your code loads only what it needs
2. **Clearer Dependencies**: Dependencies are explicitly declared
3. **Better Maintainability**: Easier to understand which modules your code depends on
4. **Improved Module Boundaries**: Each module has clear responsibilities
