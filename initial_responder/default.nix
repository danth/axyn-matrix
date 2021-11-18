{ callPackage, en-core-web-md, flipgenic, python3, runCommand, lib }:

let
  pythonPackages = python3.withPackages (ps: [ en-core-web-md flipgenic ]);

  # The first source of data is written to a new database; subsequent sources
  # are appended to the database produced by the previous source. Each source
  # is its own derivation with the previous state of the database as an input.
  # This allows for new sources to be added without rebuilding the entire
  # database, provided that the new source is placed at the end of the list.
  addSource = database: sourcePath:
    let source = callPackage sourcePath { };
    in runCommand "flipgenic-${source.name}" { } ''
      ${if isNull database then "" else
        "cp --no-preserve=mode,ownership -r ${database} $out"}
      ${pythonPackages}/bin/python ${source.buildScript} ${source.input} $out
    '';

in lib.foldl addSource null [
  ./cornell_movie_dialogs_corpus
  ./craigslist_bargains
]
