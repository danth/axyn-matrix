{ fetchurl }:

{
  name = "shakespeare";
  buildScript = ./build.py;
  input = fetchurl {
    name = "shakespeare-plays.json";
    url = "https://bridgesdata.herokuapp.com/api/shakespeare/plays";
    sha256 = "k6jxkOPeE3+qXVuPCPhweZMssrddvKnnjZKXCheurFo=";
  };
}
