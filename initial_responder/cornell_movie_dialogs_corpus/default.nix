{ fetchzip }:

{
  name = "cornell-movie-dialogs-corpus";
  buildScript = ./build.py;
  input = fetchzip {
    name = "cornell-movie-dialogs-corpus";
    url = "https://www.cs.cornell.edu/~cristian/data/cornell_movie_dialogs_corpus.zip";
    sha256 = "ZholC/oCvWl0CqyT5vSDMual0wQ6EwQN5VCJUH0/hb8=";
    stripRoot = false;
    extraPostFetch = ''
      mv $out/cornell\ movie-dialogs\ corpus/* $out
      rm -r $out/__MACOSX $out/cornell\ movie-dialogs\ corpus
    '';
  };
}
