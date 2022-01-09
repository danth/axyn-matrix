{ fetchurl }:

{
  name = "shakespeare";
  buildScript = ./build.py;
  input = fetchurl {
    name = "shakespeare-plays.json";
    url = "https://bridgesdata.herokuapp.com/api/shakespeare/plays";
    sha256 = "tt16c120Ty2KZ/3T/NCX3K/oOyY7YRMwmsBASiEbEJ8=";
  };
}
