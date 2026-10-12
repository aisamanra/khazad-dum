# Khazad-Dum

Khazad-Dum is a project for fast directory-walking and hashing in a way that also respects `.gitignore` and other configurable ignores.

This does walking, hashing, and ignore-handling in parallel, powered by the `walkdir` crate, and can be configured to produce different hashes.

## Usage

```
# walk the current directory and print the hashes of each file to stdout
$ khazad-dum

# walk the directory /opt/foo and write only the sha256 sums for it
$ khazad-dum /opt/foo --compute-hashes=sha256
```
