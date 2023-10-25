# sysdiagram

### Useful shell commands

```sh
cat sysdiagrams.csv | awk "-F|" "{ system(\"echo \" \$5 \" | base64 -d > \\\"\" \$1 \".sysdiagram\\\"\") }"
```