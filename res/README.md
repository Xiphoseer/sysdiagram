# Example Files

These examples were generated from <https://github.com/microsoft/sql-server-samples/blob/master/samples/databases/adventure-works/data-warehouse-install-script/sysdiagrams.csv> using the following shell command:

```sh
awk "-F|" "{ system(\"echo \" \$5 \" | xxd -r -p > \\\"\" \$1 \".sysdiagram\\\"\") }" sysdiagrams.csv
```
